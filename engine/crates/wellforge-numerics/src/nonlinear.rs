//! Bounded damped Newton solver for small domain-independent nonlinear systems.

use crate::{ConvergenceState, SolverDiagnostics, SparseEntry, SparseLinearSystem};

/// Residual callback for a square nonlinear system.
pub trait ResidualModel {
    /// Evaluates the residual vector at `x` into `out`.
    fn residual(&self, x: &[f64], out: &mut [f64]);
}

/// Jacobian callback for a square nonlinear system.
pub trait JacobianProvider {
    /// Evaluates the row-major Jacobian at `x` into `out`.
    fn jacobian(&self, x: &[f64], out: &mut [f64]);
}

/// Controls for the bounded damped Newton iteration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewtonOptions {
    /// Maximum accepted Newton iterations.
    pub max_iterations: usize,
    /// Absolute residual-norm tolerance.
    pub absolute_tolerance: f64,
    /// Relative residual-norm tolerance against the initial residual.
    pub relative_tolerance: f64,
    /// Smallest line-search damping factor permitted.
    pub minimum_damping: f64,
}

impl Default for NewtonOptions {
    fn default() -> Self {
        Self {
            max_iterations: 64,
            absolute_tolerance: 1.0e-10,
            relative_tolerance: 1.0e-10,
            minimum_damping: 1.0 / 65_536.0,
        }
    }
}

/// Successful bounded Newton result.
#[derive(Clone, Debug, PartialEq)]
pub struct NewtonSolution {
    /// Converged unknown vector.
    pub values: Vec<f64>,
    /// Iteration and residual evidence.
    pub diagnostics: SolverDiagnostics,
}

/// Bounded Newton construction or convergence failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NewtonError {
    /// Input vectors are empty or have incompatible dimensions.
    DimensionMismatch,
    /// Bounds are invalid or the initial state lies outside them.
    InvalidBounds,
    /// Input, residual, Jacobian, or trial state contains a non-finite value.
    NonFinite,
    /// The Newton linear system could not be solved.
    LinearSolve,
    /// No residual-reducing damped step was found or the iteration limit was reached.
    NotConverged,
}

fn l2(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn evaluate_residual<M: ResidualModel>(
    model: &M,
    x: &[f64],
    residual: &mut [f64],
) -> Result<f64, NewtonError> {
    model.residual(x, residual);
    if residual.iter().any(|value| !value.is_finite()) {
        return Err(NewtonError::NonFinite);
    }
    Ok(l2(residual))
}

fn converged(norm: f64, initial_norm: f64, options: NewtonOptions) -> bool {
    norm <= options.absolute_tolerance
        + options.relative_tolerance * initial_norm.max(options.absolute_tolerance)
}

/// Solves a bounded square nonlinear system with safeguarded Newton steps and backtracking.
///
/// Jacobian values are interpreted as a row-major `n × n` matrix. Every trial state is
/// projected onto the supplied component bounds before its residual is evaluated.
///
/// # Errors
/// Returns [`NewtonError`] for invalid dimensions/bounds, non-finite values, an unsolved
/// Jacobian system, or failure to find a converged residual-reducing sequence.
pub fn solve_newton_system<M>(
    model: &M,
    initial: &[f64],
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    options: NewtonOptions,
) -> Result<NewtonSolution, NewtonError>
where
    M: ResidualModel + JacobianProvider,
{
    let n = initial.len();
    if n == 0 || lower_bounds.len() != n || upper_bounds.len() != n {
        return Err(NewtonError::DimensionMismatch);
    }
    if options.max_iterations == 0
        || !options.absolute_tolerance.is_finite()
        || !options.relative_tolerance.is_finite()
        || !options.minimum_damping.is_finite()
        || options.absolute_tolerance <= 0.0
        || options.relative_tolerance < 0.0
        || !(0.0..=1.0).contains(&options.minimum_damping)
        || options.minimum_damping == 0.0
    {
        return Err(NewtonError::InvalidBounds);
    }
    if initial.iter().any(|value| !value.is_finite())
        || lower_bounds.iter().any(|value| !value.is_finite())
        || upper_bounds.iter().any(|value| !value.is_finite())
    {
        return Err(NewtonError::NonFinite);
    }
    if (0..n).any(|index| {
        lower_bounds[index] > upper_bounds[index]
            || initial[index] < lower_bounds[index]
            || initial[index] > upper_bounds[index]
    }) {
        return Err(NewtonError::InvalidBounds);
    }

    let mut x = initial.to_vec();
    let mut residual = vec![0.0; n];
    let initial_norm = evaluate_residual(model, &x, &mut residual)?;
    if converged(initial_norm, initial_norm, options) {
        return Ok(NewtonSolution {
            values: x,
            diagnostics: SolverDiagnostics {
                state: ConvergenceState::Converged,
                iterations: 0,
                initial_residual_norm: initial_norm,
                final_residual_norm: initial_norm,
                absolute_tolerance: options.absolute_tolerance,
                relative_tolerance: options.relative_tolerance,
                fallback_used: false,
            },
        });
    }

    let mut fallback_used = false;
    let mut final_norm = initial_norm;
    for iteration in 1..=options.max_iterations {
        let mut jacobian = vec![0.0; n * n];
        model.jacobian(&x, &mut jacobian);
        if jacobian.iter().any(|value| !value.is_finite()) {
            return Err(NewtonError::NonFinite);
        }
        let entries = jacobian
            .iter()
            .enumerate()
            .map(|(index, &value)| SparseEntry::new(index / n, index % n, value))
            .collect::<Vec<_>>();
        let system = SparseLinearSystem::new(n, &entries).map_err(|_| NewtonError::LinearSolve)?;
        let rhs = residual.iter().map(|value| -value).collect::<Vec<_>>();
        let step = system
            .factor_and_solve(&entries, &rhs)
            .map_err(|_| NewtonError::LinearSolve)?;

        let current_norm = final_norm;
        let mut damping = 1.0;
        let mut accepted = None;
        while damping >= options.minimum_damping {
            let trial = x
                .iter()
                .zip(&step.values)
                .enumerate()
                .map(|(index, (&value, &delta))| {
                    (value + damping * delta).clamp(lower_bounds[index], upper_bounds[index])
                })
                .collect::<Vec<_>>();
            if trial.iter().any(|value| !value.is_finite()) {
                return Err(NewtonError::NonFinite);
            }
            let mut trial_residual = vec![0.0; n];
            let trial_norm = evaluate_residual(model, &trial, &mut trial_residual)?;
            if trial_norm < current_norm {
                accepted = Some((trial, trial_residual, trial_norm));
                break;
            }
            damping *= 0.5;
            fallback_used = true;
        }

        let Some((trial, trial_residual, trial_norm)) = accepted else {
            return Err(NewtonError::NotConverged);
        };
        if damping < 1.0 {
            fallback_used = true;
        }
        x = trial;
        residual = trial_residual;
        final_norm = trial_norm;
        if converged(final_norm, initial_norm, options) {
            return Ok(NewtonSolution {
                values: x,
                diagnostics: SolverDiagnostics {
                    state: ConvergenceState::Converged,
                    iterations: iteration,
                    initial_residual_norm: initial_norm,
                    final_residual_norm: final_norm,
                    absolute_tolerance: options.absolute_tolerance,
                    relative_tolerance: options.relative_tolerance,
                    fallback_used,
                },
            });
        }
    }

    Err(NewtonError::NotConverged)
}
