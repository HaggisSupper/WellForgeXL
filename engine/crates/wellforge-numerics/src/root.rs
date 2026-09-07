//! Deterministic scalar root-solving primitives.

use crate::{ConvergenceState, SolverDiagnostics};

/// Controls for deterministic scalar root solving.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootOptions {
    /// Absolute residual tolerance.
    pub absolute_tolerance: f64,
    /// Relative residual tolerance measured from the initial residual.
    pub relative_tolerance: f64,
    /// Maximum solver iterations.
    pub max_iterations: usize,
}

impl Default for RootOptions {
    fn default() -> Self {
        Self {
            absolute_tolerance: 1.0e-12,
            relative_tolerance: 1.0e-12,
            max_iterations: 128,
        }
    }
}

/// Successful scalar root solution and convergence evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootSolution {
    /// Root estimate.
    pub root: f64,
    /// Residual evaluated at `root`.
    pub residual: f64,
    /// Common solver diagnostics.
    pub diagnostics: SolverDiagnostics,
}

/// Scalar root solver failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootError {
    /// Bounds, initial guess, residual, derivative, or tolerance was non-finite or inadmissible.
    InvalidInput,
    /// The supplied interval does not bracket a sign change.
    InvalidBracket,
    /// The iteration bound was exhausted before convergence.
    NotConverged,
}

fn residual_tolerance(options: RootOptions, initial_residual: f64) -> Result<f64, RootError> {
    if !options.absolute_tolerance.is_finite()
        || !options.relative_tolerance.is_finite()
        || options.absolute_tolerance <= 0.0
        || options.relative_tolerance < 0.0
        || options.max_iterations == 0
    {
        return Err(RootError::InvalidInput);
    }
    Ok(options
        .absolute_tolerance
        .max(options.relative_tolerance * initial_residual))
}

fn valid_bracket(lower: f64, upper: f64, f_lower: f64, f_upper: f64) -> bool {
    lower.is_finite()
        && upper.is_finite()
        && lower < upper
        && f_lower.is_finite()
        && f_upper.is_finite()
        && (f_lower.abs() <= f64::EPSILON
            || f_upper.abs() <= f64::EPSILON
            || f_lower.is_sign_negative() != f_upper.is_sign_negative())
}

fn diagnostics(
    state: ConvergenceState,
    iterations: usize,
    initial_residual: f64,
    final_residual: f64,
    options: RootOptions,
    fallback_used: bool,
) -> SolverDiagnostics {
    SolverDiagnostics {
        state,
        iterations,
        initial_residual_norm: initial_residual,
        final_residual_norm: final_residual,
        absolute_tolerance: options.absolute_tolerance,
        relative_tolerance: options.relative_tolerance,
        fallback_used,
    }
}

/// Solves a continuous scalar residual on a sign-changing bracket by deterministic bisection.
///
/// # Errors
/// Returns [`RootError::InvalidBracket`] when the interval does not bracket a root,
/// [`RootError::InvalidInput`] for non-finite/inadmissible controls, and
/// [`RootError::NotConverged`] when the iteration limit is exhausted.
pub fn solve_bracketed<F>(
    mut lower: f64,
    mut upper: f64,
    residual: F,
    options: RootOptions,
) -> Result<RootSolution, RootError>
where
    F: Fn(f64) -> f64,
{
    let mut f_lower = residual(lower);
    let f_upper = residual(upper);
    if !valid_bracket(lower, upper, f_lower, f_upper) {
        return Err(RootError::InvalidBracket);
    }

    let initial_residual = f_lower.abs().min(f_upper.abs());
    let tolerance = residual_tolerance(options, initial_residual)?;
    if f_lower.abs() <= tolerance {
        return Ok(RootSolution {
            root: lower,
            residual: f_lower,
            diagnostics: diagnostics(
                ConvergenceState::Converged,
                0,
                initial_residual,
                f_lower.abs(),
                options,
                false,
            ),
        });
    }
    if f_upper.abs() <= tolerance {
        return Ok(RootSolution {
            root: upper,
            residual: f_upper,
            diagnostics: diagnostics(
                ConvergenceState::Converged,
                0,
                initial_residual,
                f_upper.abs(),
                options,
                false,
            ),
        });
    }

    for iteration in 1..=options.max_iterations {
        let midpoint = lower.midpoint(upper);
        let f_mid = residual(midpoint);
        if !f_mid.is_finite() {
            return Err(RootError::InvalidInput);
        }
        if f_mid.abs() <= tolerance {
            return Ok(RootSolution {
                root: midpoint,
                residual: f_mid,
                diagnostics: diagnostics(
                    ConvergenceState::Converged,
                    iteration,
                    initial_residual,
                    f_mid.abs(),
                    options,
                    false,
                ),
            });
        }
        if f_lower.is_sign_negative() == f_mid.is_sign_negative() {
            lower = midpoint;
            f_lower = f_mid;
        } else {
            upper = midpoint;
        }
    }

    Err(RootError::NotConverged)
}

/// Solves a bracketed scalar residual with Newton steps safeguarded by bisection.
///
/// Newton steps that are non-finite, use a near-zero derivative, or leave the current
/// sign-changing bracket are replaced by the bracket midpoint.
///
/// # Errors
/// Returns [`RootError::InvalidBracket`] when the interval does not bracket a root,
/// [`RootError::InvalidInput`] for non-finite/inadmissible inputs, and
/// [`RootError::NotConverged`] when the iteration limit is exhausted.
pub fn solve_safeguarded_newton<F, D>(
    mut lower: f64,
    mut upper: f64,
    initial: f64,
    residual: F,
    derivative: D,
    options: RootOptions,
) -> Result<RootSolution, RootError>
where
    F: Fn(f64) -> f64,
    D: Fn(f64) -> f64,
{
    let mut f_lower = residual(lower);
    let f_upper = residual(upper);
    if !valid_bracket(lower, upper, f_lower, f_upper) {
        return Err(RootError::InvalidBracket);
    }
    if !initial.is_finite() {
        return Err(RootError::InvalidInput);
    }

    let mut fallback_used = !(initial > lower && initial < upper);
    let mut x = if fallback_used {
        lower.midpoint(upper)
    } else {
        initial
    };
    let mut f_x = residual(x);
    if !f_x.is_finite() {
        return Err(RootError::InvalidInput);
    }
    let initial_residual = f_x.abs();
    let tolerance = residual_tolerance(options, initial_residual)?;
    if f_x.abs() <= tolerance {
        return Ok(RootSolution {
            root: x,
            residual: f_x,
            diagnostics: diagnostics(
                ConvergenceState::Converged,
                0,
                initial_residual,
                f_x.abs(),
                options,
                fallback_used,
            ),
        });
    }

    for iteration in 1..=options.max_iterations {
        if f_lower.is_sign_negative() == f_x.is_sign_negative() {
            lower = x;
            f_lower = f_x;
        } else {
            upper = x;
        }

        let slope = derivative(x);
        let newton = if slope.is_finite() && slope.abs() > f64::EPSILON {
            x - f_x / slope
        } else {
            f64::NAN
        };
        let candidate = if newton.is_finite() && newton > lower && newton < upper {
            newton
        } else {
            fallback_used = true;
            lower.midpoint(upper)
        };

        x = candidate;
        f_x = residual(x);
        if !f_x.is_finite() {
            return Err(RootError::InvalidInput);
        }
        if f_x.abs() <= tolerance {
            return Ok(RootSolution {
                root: x,
                residual: f_x,
                diagnostics: diagnostics(
                    ConvergenceState::Converged,
                    iteration,
                    initial_residual,
                    f_x.abs(),
                    options,
                    fallback_used,
                ),
            });
        }
    }

    Err(RootError::NotConverged)
}
