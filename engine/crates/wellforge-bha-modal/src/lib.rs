//! Linearized modal analysis for BHA Release 1.

use nalgebra::{DMatrix, DVector, SymmetricEigen};
use num_complex::Complex64;
use thiserror::Error;
use wellforge_bha_contract::{
    BhaAnalysisRequest, CampbellPoint, FrequencyResponsePoint, ModeResult,
};
use wellforge_bha_model::BhaModel;
use wellforge_bha_static::StaticSolution;

/// Modal solve failure.
#[derive(Debug, Error)]
pub enum ModalSolveError {
    /// Consistent mass matrix is not positive definite.
    #[error("mass matrix is not positive definite")]
    NonPositiveMass,
    /// Dynamic stiffness could not be solved.
    #[error("singular dynamic stiffness at {0} Hz")]
    SingularDynamicStiffness(f64),
}

/// Computes direct complex receptance using library complex LU factorization.
///
/// # Errors
///
/// Returns [`ModalSolveError`] for a non-positive mass matrix or singular dynamic stiffness.
#[allow(clippy::cast_precision_loss)]
pub fn solve_frequency_response(
    static_solution: &StaticSolution,
    start_hz: f64,
    stop_hz: f64,
    points: usize,
) -> Result<Vec<FrequencyResponsePoint>, ModalSolveError> {
    let count = points.max(2);
    let cholesky = static_solution
        .mass
        .clone()
        .cholesky()
        .ok_or(ModalSolveError::NonPositiveMass)?;
    let l_inv = cholesky
        .l()
        .try_inverse()
        .ok_or(ModalSolveError::NonPositiveMass)?;
    let transformed = &l_inv * &static_solution.stiffness * l_inv.transpose();
    let eigen = SymmetricEigen::new((transformed.clone() + transformed.transpose()) * 0.5);
    let first_omega = eigen
        .eigenvalues
        .iter()
        .copied()
        .filter(|value| *value > 1.0e-9)
        .map(f64::sqrt)
        .min_by(f64::total_cmp)
        .ok_or(ModalSolveError::NonPositiveMass)?;
    let beta = 2.0 * 0.02 / first_omega;
    let size = static_solution.stiffness.nrows();
    let mut output = Vec::with_capacity(count);
    for index in 0..count {
        let frequency_hz = start_hz + (stop_hz - start_hz) * index as f64 / (count - 1) as f64;
        let omega = std::f64::consts::TAU * frequency_hz;
        let dynamic = DMatrix::<Complex64>::from_fn(size, size, |row, col| {
            Complex64::new(
                static_solution.stiffness[(row, col)]
                    - omega * omega * static_solution.mass[(row, col)],
                omega * beta * static_solution.stiffness[(row, col)],
            )
        });
        let mut force = DVector::<Complex64>::zeros(size);
        force[size - 2] = Complex64::new(1.0, 0.0);
        let solved = dynamic
            .lu()
            .solve(&force)
            .ok_or(ModalSolveError::SingularDynamicStiffness(frequency_hz))?;
        let response = solved[size - 2];
        output.push(FrequencyResponsePoint {
            frequency_hz,
            receptance_m_n: response.norm(),
            phase_deg: response.arg().to_degrees(),
        });
    }
    Ok(output)
}

/// Builds deterministic 1x, 2x and 3x order lines and nearest-mode margins.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn build_campbell_map(
    modes: &[ModeResult],
    max_rpm: f64,
    samples: usize,
) -> Vec<CampbellPoint> {
    let count = samples.max(2);
    let mut output = Vec::with_capacity(3 * count);
    for order in 1..=3 {
        for index in 0..count {
            let rpm = max_rpm * index as f64 / (count - 1) as f64;
            let excitation = order as f64 * rpm / 60.0;
            let margin = modes
                .iter()
                .map(|mode| (mode.natural_frequency_hz - excitation).abs())
                .min_by(f64::total_cmp)
                .unwrap_or(f64::NAN);
            output.push(CampbellPoint {
                order,
                rpm,
                excitation_frequency_hz: excitation,
                nearest_mode_margin_hz: margin,
            });
        }
    }
    output
}

/// Solves the generalized undamped eigenproblem using library Cholesky and symmetric eigendecomposition.
///
/// # Errors
///
/// Returns [`ModalSolveError`] when the mass matrix is not positive definite.
#[allow(clippy::redundant_closure_for_method_calls)]
pub fn solve_modes(
    model: &BhaModel,
    request: &BhaAnalysisRequest,
    static_solution: &StaticSolution,
) -> Result<Vec<ModeResult>, ModalSolveError> {
    let cholesky = static_solution
        .mass
        .clone()
        .cholesky()
        .ok_or(ModalSolveError::NonPositiveMass)?;
    let l = cholesky.l();
    let l_inv = l
        .clone()
        .try_inverse()
        .ok_or(ModalSolveError::NonPositiveMass)?;
    let transformed = &l_inv * &static_solution.stiffness * l_inv.transpose();
    let eigen = SymmetricEigen::new((transformed.clone() + transformed.transpose()) * 0.5);
    let mut pairs: Vec<(f64, DVector<f64>)> = eigen
        .eigenvalues
        .iter()
        .copied()
        .zip(
            eigen
                .eigenvectors
                .column_iter()
                .map(|column| column.into_owned()),
        )
        .filter(|(value, _)| *value > 1.0e-9)
        .collect();
    pairs.sort_by(|left, right| left.0.total_cmp(&right.0));
    let count = request.solver.requested_modes.min(pairs.len());
    Ok(pairs
        .into_iter()
        .take(count)
        .enumerate()
        .map(|(index, (lambda, vector))| {
            let physical = l_inv.transpose() * vector;
            let amplitudes: Vec<f64> = (0..model.nodes.len())
                .map(|node| {
                    if node == 0 {
                        0.0
                    } else {
                        physical[2 * node - 2]
                    }
                })
                .collect();
            let max = amplitudes
                .iter()
                .map(|value| value.abs())
                .fold(0.0_f64, f64::max)
                .max(f64::EPSILON);
            let frequency = lambda.sqrt() / std::f64::consts::TAU;
            ModeResult {
                mode_number: index + 1,
                natural_frequency_hz: frequency,
                critical_speed_rpm: frequency * 60.0,
                normalized_shape: amplitudes.into_iter().map(|value| value / max).collect(),
            }
        })
        .collect())
}
