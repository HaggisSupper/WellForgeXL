//! Deterministic robust weighted least-squares regression.

use crate::{ConvergenceState, SolverDiagnostics, SparseEntry, SparseLinearSystem};

/// Controls for robust iteratively reweighted least squares.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RobustRegressionOptions {
    /// Maximum robust reweighting iterations.
    pub max_iterations: usize,
    /// Absolute coefficient-change convergence tolerance.
    pub coefficient_tolerance: f64,
    /// Huber cutoff multiplier applied to the robust residual scale.
    pub huber_k: f64,
    /// Positive lower bound for the robust residual scale.
    pub minimum_scale: f64,
}

impl Default for RobustRegressionOptions {
    fn default() -> Self {
        Self {
            max_iterations: 64,
            coefficient_tolerance: 1.0e-10,
            huber_k: 1.345,
            minimum_scale: 1.0e-12,
        }
    }
}

/// Robust weighted regression result.
#[derive(Clone, Debug, PartialEq)]
pub struct RobustRegressionFit {
    /// Fitted linear coefficients in design-column order.
    pub coefficients: Vec<f64>,
    /// Iteration and objective evidence.
    pub diagnostics: SolverDiagnostics,
}

/// Robust regression validation or solve failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegressionError {
    /// Design, observation, or weight dimensions are incompatible.
    DimensionMismatch,
    /// Inputs or options contain non-finite values.
    NonFinite,
    /// A base weight is negative or no positive weight is available.
    InvalidWeight,
    /// The weighted normal equations are singular or could not be solved.
    LinearSolve,
    /// Robust reweighting did not stabilize within the iteration limit.
    NotConverged,
}

fn weighted_least_squares(
    design: &[Vec<f64>],
    observations: &[f64],
    weights: &[f64],
) -> Result<Vec<f64>, RegressionError> {
    let columns = design[0].len();
    let mut normal = vec![0.0; columns * columns];
    let mut rhs = vec![0.0; columns];
    for ((features, &observation), &weight) in design.iter().zip(observations).zip(weights) {
        if weight == 0.0 {
            continue;
        }
        for left in 0..columns {
            rhs[left] += weight * features[left] * observation;
            for right in 0..columns {
                normal[left * columns + right] += weight * features[left] * features[right];
            }
        }
    }
    let entries = normal
        .iter()
        .enumerate()
        .map(|(index, &value)| SparseEntry::new(index / columns, index % columns, value))
        .collect::<Vec<_>>();
    let system =
        SparseLinearSystem::new(columns, &entries).map_err(|_| RegressionError::LinearSolve)?;
    system
        .factor_and_solve(&entries, &rhs)
        .map(|solution| solution.values)
        .map_err(|_| RegressionError::LinearSolve)
}

fn residuals(design: &[Vec<f64>], observations: &[f64], coefficients: &[f64]) -> Vec<f64> {
    design
        .iter()
        .zip(observations)
        .map(|(features, &observation)| {
            observation
                - features
                    .iter()
                    .zip(coefficients)
                    .map(|(&feature, &coefficient)| feature * coefficient)
                    .sum::<f64>()
        })
        .collect()
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        values[middle - 1].midpoint(values[middle])
    } else {
        values[middle]
    }
}

fn median_absolute_deviation(values: &[f64]) -> f64 {
    let center = median(values.to_vec());
    median(values.iter().map(|value| (value - center).abs()).collect())
}

fn residual_score(values: &[f64]) -> f64 {
    median(values.iter().map(|value| value.abs()).collect())
}

fn residual_norm(values: &[f64], weights: &[f64]) -> f64 {
    values
        .iter()
        .zip(weights)
        .map(|(&residual, &weight)| weight * residual * residual)
        .sum::<f64>()
        .sqrt()
}

fn robust_seed(
    design: &[Vec<f64>],
    observations: &[f64],
    base_weights: &[f64],
) -> Result<Vec<f64>, RegressionError> {
    let mut best = weighted_least_squares(design, observations, base_weights)?;
    let mut best_score = residual_score(&residuals(design, observations, &best));

    if design.len() > design[0].len() {
        for excluded in 0..design.len() {
            let mut candidate_weights = base_weights.to_vec();
            candidate_weights[excluded] = 0.0;
            let Ok(candidate) = weighted_least_squares(design, observations, &candidate_weights)
            else {
                continue;
            };
            let score = residual_score(&residuals(design, observations, &candidate));
            if score < best_score {
                best = candidate;
                best_score = score;
            }
        }
    }
    Ok(best)
}

/// Fits a robust weighted linear model using a deterministic resistant seed followed by Huber IRLS.
///
/// The resistant seed evaluates the full weighted fit and each single-observation deletion, choosing
/// the candidate with the smallest median absolute residual. This protects the subsequent Huber
/// iteration against one high-leverage gross outlier while remaining deterministic.
///
/// # Errors
/// Returns [`RegressionError`] for malformed/non-finite inputs, invalid weights/options, singular
/// weighted systems, or failure of the robust coefficient iteration to stabilize.
pub fn robust_weighted_least_squares(
    design: &[Vec<f64>],
    observations: &[f64],
    weights: &[f64],
    options: RobustRegressionOptions,
) -> Result<RobustRegressionFit, RegressionError> {
    if design.is_empty()
        || observations.len() != design.len()
        || weights.len() != design.len()
        || design[0].is_empty()
        || design.iter().any(|row| row.len() != design[0].len())
    {
        return Err(RegressionError::DimensionMismatch);
    }
    if observations.iter().any(|value| !value.is_finite())
        || design.iter().flatten().any(|value| !value.is_finite())
        || weights.iter().any(|value| !value.is_finite())
        || !options.coefficient_tolerance.is_finite()
        || !options.huber_k.is_finite()
        || !options.minimum_scale.is_finite()
    {
        return Err(RegressionError::NonFinite);
    }
    if weights.iter().any(|&weight| weight < 0.0) || !weights.iter().any(|&weight| weight > 0.0) {
        return Err(RegressionError::InvalidWeight);
    }
    if options.max_iterations == 0
        || options.coefficient_tolerance <= 0.0
        || options.huber_k <= 0.0
        || options.minimum_scale <= 0.0
    {
        return Err(RegressionError::NotConverged);
    }

    let mut coefficients = robust_seed(design, observations, weights)?;
    let initial_residuals = residuals(design, observations, &coefficients);
    let initial_norm = residual_norm(&initial_residuals, weights);

    for iteration in 1..=options.max_iterations {
        let current_residuals = residuals(design, observations, &coefficients);
        let scale =
            (1.4826 * median_absolute_deviation(&current_residuals)).max(options.minimum_scale);
        let cutoff = options.huber_k * scale;
        let effective_weights = current_residuals
            .iter()
            .zip(weights)
            .map(|(&residual, &base_weight)| {
                let magnitude = residual.abs();
                let robust_weight = if magnitude <= cutoff || magnitude == 0.0 {
                    1.0
                } else {
                    cutoff / magnitude
                };
                base_weight * robust_weight
            })
            .collect::<Vec<_>>();
        let updated = weighted_least_squares(design, observations, &effective_weights)?;
        let max_change = coefficients
            .iter()
            .zip(&updated)
            .map(|(&old, &new)| (new - old).abs())
            .fold(0.0_f64, f64::max);
        coefficients = updated;
        let updated_residuals = residuals(design, observations, &coefficients);
        let final_norm = residual_norm(&updated_residuals, &effective_weights);
        if max_change <= options.coefficient_tolerance {
            return Ok(RobustRegressionFit {
                coefficients,
                diagnostics: SolverDiagnostics {
                    state: ConvergenceState::Converged,
                    iterations: iteration,
                    initial_residual_norm: initial_norm,
                    final_residual_norm: final_norm,
                    absolute_tolerance: options.coefficient_tolerance,
                    relative_tolerance: 0.0,
                    fallback_used: true,
                },
            });
        }
    }

    Err(RegressionError::NotConverged)
}
