//! Contract tests for reusable sparse linear systems.

use wellforge_numerics::{SparseEntry, SparseError, SparseLinearSystem};

fn tridiagonal(diagonal: f64) -> Vec<SparseEntry> {
    vec![
        SparseEntry::new(0, 0, diagonal),
        SparseEntry::new(1, 0, -1.0),
        SparseEntry::new(0, 1, -1.0),
        SparseEntry::new(1, 1, diagonal),
        SparseEntry::new(2, 1, -1.0),
        SparseEntry::new(1, 2, -1.0),
        SparseEntry::new(2, 2, diagonal),
    ]
}

fn manufactured_rhs(dimension: usize, entries: &[SparseEntry], solution: &[f64]) -> Vec<f64> {
    let mut rhs = vec![0.0; dimension];
    for entry in entries {
        rhs[entry.row] += entry.value * solution[entry.col];
    }
    rhs
}

#[test]
fn sparse_system_solves_known_tridiagonal_case() {
    let entries = tridiagonal(2.0);
    let system = SparseLinearSystem::new(3, &entries).expect("valid sparse pattern");
    let solution = system
        .factor_and_solve(&entries, &[0.0, 0.0, 4.0])
        .expect("nonsingular solve");

    for (actual, expected) in solution.values.iter().zip([1.0, 2.0, 3.0]) {
        assert!((*actual - expected).abs() < 1.0e-12);
    }
    assert!(solution.residual_norm < 1.0e-12);
}

#[test]
fn sparse_system_reuses_symbolic_pattern_with_new_numeric_values() {
    let initial = tridiagonal(2.0);
    let system = SparseLinearSystem::new(3, &initial).expect("valid sparse pattern");
    let updated = tridiagonal(3.0);

    let solution = system
        .factor_and_solve(&updated, &[1.0, 2.0, 7.0])
        .expect("same pattern numeric refactorization");

    for (actual, expected) in solution.values.iter().zip([1.0, 2.0, 3.0]) {
        assert!((*actual - expected).abs() < 1.0e-12);
    }
}

#[test]
fn sparse_system_accepts_reordered_entries_for_same_coordinate_pattern() {
    let initial = tridiagonal(3.0);
    let system = SparseLinearSystem::new(3, &initial).expect("valid sparse pattern");
    let mut reordered = initial.clone();
    reordered.reverse();
    let expected = [1.0, 2.0, 3.0];
    let rhs = manufactured_rhs(3, &reordered, &expected);

    let solution = system
        .factor_and_solve(&reordered, &rhs)
        .expect("coordinate-equivalent pattern must not depend on caller ordering");

    for (actual, expected) in solution.values.iter().zip(expected) {
        assert!((*actual - expected).abs() < 1.0e-12);
    }
}

#[test]
fn sparse_system_sums_duplicate_element_contributions() {
    let entries = vec![
        SparseEntry::new(0, 0, 1.0),
        SparseEntry::new(0, 0, 3.0),
        SparseEntry::new(0, 1, -1.0),
        SparseEntry::new(1, 0, -1.0),
        SparseEntry::new(1, 1, 1.0),
        SparseEntry::new(1, 1, 2.0),
    ];
    let system =
        SparseLinearSystem::new(2, &entries).expect("duplicate FE contributions are valid");
    let expected = [2.0, 3.0];
    let rhs = manufactured_rhs(2, &entries, &expected);

    let solution = system
        .factor_and_solve(&entries, &rhs)
        .expect("duplicates must assemble by summation");

    for (actual, expected) in solution.values.iter().zip(expected) {
        assert!((*actual - expected).abs() < 1.0e-12);
    }
    assert!(solution.residual_norm < 1.0e-12);
}

#[test]
fn sparse_system_handles_required_row_pivoting() {
    let entries = vec![
        SparseEntry::new(0, 1, 2.0),
        SparseEntry::new(1, 0, 1.0),
        SparseEntry::new(1, 2, 3.0),
        SparseEntry::new(2, 0, 4.0),
        SparseEntry::new(2, 1, 5.0),
        SparseEntry::new(2, 2, 6.0),
    ];
    let system = SparseLinearSystem::new(3, &entries).expect("invertible pivoting case");
    let expected = [1.0, 2.0, 3.0];
    let rhs = manufactured_rhs(3, &entries, &expected);

    let solution = system
        .factor_and_solve(&entries, &rhs)
        .expect("sparse LU must pivot rather than reject a zero leading diagonal");

    for (actual, expected) in solution.values.iter().zip(expected) {
        assert!((*actual - expected).abs() < 1.0e-11);
    }
    assert!(solution.residual_norm < 1.0e-12);
}

#[test]
fn sparse_system_retains_symbolic_coordinate_when_numeric_value_becomes_zero() {
    let initial = tridiagonal(4.0);
    let system = SparseLinearSystem::new(3, &initial).expect("valid sparse pattern");
    let mut updated = initial.clone();
    updated[2].value = 0.0;
    let expected = [1.0, 2.0, 3.0];
    let rhs = manufactured_rhs(3, &updated, &expected);

    let solution = system
        .factor_and_solve(&updated, &rhs)
        .expect("numeric zero must not remove a cached symbolic coordinate");

    for (actual, expected) in solution.values.iter().zip(expected) {
        assert!((*actual - expected).abs() < 1.0e-11);
    }
}

#[test]
fn sparse_system_rejects_exactly_singular_matrix() {
    let entries = vec![
        SparseEntry::new(0, 0, 1.0),
        SparseEntry::new(0, 1, 2.0),
        SparseEntry::new(1, 0, 2.0),
        SparseEntry::new(1, 1, 4.0),
    ];
    let system = SparseLinearSystem::new(2, &entries).expect("valid singular sparsity pattern");

    let error = system
        .factor_and_solve(&entries, &[3.0, 6.0])
        .expect_err("exact singularity must return a typed error");

    assert!(matches!(
        error,
        SparseError::Factorization | SparseError::Singular
    ));
}

#[test]
fn sparse_system_recovers_manufactured_solution_on_512_dof_banded_system() {
    let dimension = 512;
    let mut entries = Vec::with_capacity(3 * dimension - 2);
    for row in 0..dimension {
        if row > 0 {
            entries.push(SparseEntry::new(row, row - 1, -1.0));
        }
        let row_value = f64::from(u16::try_from(row).expect("512-DOF row fits u16"));
        entries.push(SparseEntry::new(row, row, 4.0 + row_value * 1.0e-6));
        if row + 1 < dimension {
            entries.push(SparseEntry::new(row, row + 1, -1.0));
        }
    }
    let expected: Vec<f64> = (0..dimension)
        .map(|index| {
            let index_value = f64::from(u16::try_from(index).expect("512-DOF index fits u16"));
            (index_value * 0.013).sin() + 0.25
        })
        .collect();
    let rhs = manufactured_rhs(dimension, &entries, &expected);
    let system = SparseLinearSystem::new(dimension, &entries).expect("banded FE-size pattern");

    let solution = system
        .factor_and_solve(&entries, &rhs)
        .expect("manufactured sparse system must solve");

    let max_error = solution
        .values
        .iter()
        .zip(&expected)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0_f64, f64::max);
    assert!(max_error < 1.0e-10, "max solution error {max_error:e}");
    assert!(solution.residual_norm < 1.0e-12);
}

#[test]
fn sparse_system_preserves_accuracy_across_uniform_coefficient_scaling() {
    let expected = [1.0, -2.0, 3.0];
    for scale in [1.0e-9, 1.0, 1.0e9] {
        let entries = tridiagonal(4.0)
            .into_iter()
            .map(|entry| SparseEntry::new(entry.row, entry.col, entry.value * scale))
            .collect::<Vec<_>>();
        let rhs = manufactured_rhs(3, &entries, &expected);
        let system = SparseLinearSystem::new(3, &entries).expect("scaled sparse pattern");
        let solution = system
            .factor_and_solve(&entries, &rhs)
            .expect("uniform scaling must remain solvable");

        for (actual, expected) in solution.values.iter().zip(expected) {
            assert!((*actual - expected).abs() < 1.0e-10);
        }
        assert!(solution.residual_norm < 1.0e-11);
    }
}

#[test]
fn sparse_system_rejects_changed_pattern() {
    let entries = tridiagonal(2.0);
    let system = SparseLinearSystem::new(3, &entries).expect("valid sparse pattern");
    let mut changed = entries.clone();
    changed[0] = SparseEntry::new(0, 2, 2.0);

    let error = system
        .factor_and_solve(&changed, &[0.0, 0.0, 4.0])
        .expect_err("changed sparsity must be rejected");

    assert_eq!(error, SparseError::PatternMismatch);
}
