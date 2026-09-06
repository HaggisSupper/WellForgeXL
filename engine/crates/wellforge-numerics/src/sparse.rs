//! Reusable sparse linear-system assembly and factorization boundary.

use std::collections::BTreeMap;

use faer::Col;
use faer::prelude::{Reborrow, Solve};
use faer::sparse::linalg::solvers::{Lu, SymbolicLu};
use faer::sparse::{Argsort, Pair, SparseColMat, SymbolicSparseColMat};

/// One numerical sparse-matrix entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SparseEntry {
    /// Zero-based row index.
    pub row: usize,
    /// Zero-based column index.
    pub col: usize,
    /// Numerical coefficient value.
    pub value: f64,
}

impl SparseEntry {
    /// Creates one sparse matrix entry.
    #[must_use]
    pub const fn new(row: usize, col: usize, value: f64) -> Self {
        Self { row, col, value }
    }
}

/// Sparse-system construction or solve failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SparseError {
    /// Matrix dimension or sparse entry set is empty.
    Empty,
    /// Right-hand-side length does not match the symbolic system dimension.
    DimensionMismatch,
    /// A sparse entry references a row or column outside the square matrix.
    IndexOutOfBounds,
    /// A numerical matrix entry, assembled coefficient, or right-hand-side value is NaN or infinite.
    NonFinite,
    /// Canonical numerical coordinates do not match the cached sparsity pattern.
    PatternMismatch,
    /// Symbolic or numerical factorization could not be constructed.
    Factorization,
    /// The factorized system did not produce a finite solution.
    Singular,
}

/// Solved sparse system and normalized residual evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseSolution {
    /// Solution vector.
    pub values: Vec<f64>,
    /// Euclidean residual divided by `max(||rhs||₂, 1)`.
    pub residual_norm: f64,
}

/// Reusable square sparse linear system with cached symbolic matrix and LU analysis.
pub struct SparseLinearSystem {
    dimension: usize,
    pattern: Vec<(usize, usize)>,
    symbolic_matrix: SymbolicSparseColMat<usize>,
    argsort: Argsort<usize>,
    symbolic_lu: SymbolicLu<usize>,
}

fn canonicalize_entries(
    dimension: usize,
    entries: &[SparseEntry],
) -> Result<Vec<SparseEntry>, SparseError> {
    let mut contributions = BTreeMap::<(usize, usize), Vec<f64>>::new();
    for entry in entries {
        if entry.row >= dimension || entry.col >= dimension {
            return Err(SparseError::IndexOutOfBounds);
        }
        if !entry.value.is_finite() {
            return Err(SparseError::NonFinite);
        }
        contributions
            .entry((entry.row, entry.col))
            .or_default()
            .push(entry.value);
    }

    contributions
        .into_iter()
        .map(|((row, col), mut values)| {
            values.sort_by(|left, right| {
                left.abs()
                    .total_cmp(&right.abs())
                    .then_with(|| left.total_cmp(right))
            });
            let value = values.into_iter().sum::<f64>();
            if value.is_finite() {
                Ok(SparseEntry::new(row, col, value))
            } else {
                Err(SparseError::NonFinite)
            }
        })
        .collect()
}

impl SparseLinearSystem {
    /// Builds and caches a square canonical sparsity pattern and symbolic LU analysis.
    ///
    /// Caller ordering is not part of the sparsity contract. Duplicate coordinates are assembled
    /// by deterministic summation while retaining coordinates whose assembled value is zero.
    ///
    /// # Errors
    /// Returns [`SparseError`] for empty, out-of-bounds, non-finite, or non-factorizable input.
    pub fn new(dimension: usize, entries: &[SparseEntry]) -> Result<Self, SparseError> {
        if dimension == 0 || entries.is_empty() {
            return Err(SparseError::Empty);
        }
        let canonical = canonicalize_entries(dimension, entries)?;
        let pattern: Vec<(usize, usize)> = canonical
            .iter()
            .map(|entry| (entry.row, entry.col))
            .collect();
        let pairs: Vec<Pair<usize, usize>> = pattern
            .iter()
            .map(|&(row, col)| Pair { row, col })
            .collect();
        let (symbolic_matrix, argsort) =
            SymbolicSparseColMat::try_new_from_indices(dimension, dimension, &pairs)
                .map_err(|_| SparseError::Factorization)?;
        let symbolic_lu =
            SymbolicLu::try_new(symbolic_matrix.rb()).map_err(|_| SparseError::Factorization)?;

        Ok(Self {
            dimension,
            pattern,
            symbolic_matrix,
            argsort,
            symbolic_lu,
        })
    }

    /// Refactorizes numerical values on the cached sparsity pattern and solves one right-hand side.
    ///
    /// Input entry order and duplicate representation may vary between solves provided the
    /// canonical coordinate set is unchanged.
    ///
    /// # Errors
    /// Returns [`SparseError::PatternMismatch`] if canonical coordinates differ from the cached
    /// pattern, [`SparseError::DimensionMismatch`] for an incompatible RHS length, and typed
    /// numerical errors for non-finite or unsolved systems.
    pub fn factor_and_solve(
        &self,
        entries: &[SparseEntry],
        rhs: &[f64],
    ) -> Result<SparseSolution, SparseError> {
        if rhs.len() != self.dimension {
            return Err(SparseError::DimensionMismatch);
        }
        if rhs.iter().any(|value| !value.is_finite()) {
            return Err(SparseError::NonFinite);
        }

        let canonical = canonicalize_entries(self.dimension, entries)?;
        if canonical.len() != self.pattern.len()
            || canonical
                .iter()
                .zip(&self.pattern)
                .any(|(entry, &(row, col))| entry.row != row || entry.col != col)
        {
            return Err(SparseError::PatternMismatch);
        }

        let values: Vec<f64> = canonical.iter().map(|entry| entry.value).collect();
        let matrix =
            SparseColMat::new_from_argsort(self.symbolic_matrix.clone(), &self.argsort, &values)
                .map_err(|_| SparseError::Factorization)?;
        let lu = Lu::try_new_with_symbolic(self.symbolic_lu.clone(), matrix.as_ref())
            .map_err(|_| SparseError::Factorization)?;
        let rhs_col = Col::<f64>::from_fn(self.dimension, |row| rhs[row]);
        let solution_col = lu.solve(&rhs_col);
        let solution: Vec<f64> = (0..self.dimension).map(|row| solution_col[row]).collect();
        if solution.iter().any(|value| !value.is_finite()) {
            return Err(SparseError::Singular);
        }

        let mut residual = vec![0.0; self.dimension];
        for entry in &canonical {
            residual[entry.row] += entry.value * solution[entry.col];
        }
        for (value, rhs_value) in residual.iter_mut().zip(rhs) {
            *value -= rhs_value;
        }
        let residual_l2 = residual
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        let rhs_l2 = rhs.iter().map(|value| value * value).sum::<f64>().sqrt();
        let residual_norm = residual_l2 / rhs_l2.max(1.0);

        Ok(SparseSolution {
            values: solution,
            residual_norm,
        })
    }
}
