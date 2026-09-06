//! Shared deterministic numerical primitives for `WellForge` engineering engines.
//!
//! This crate owns solver mechanics and numerical evidence only. Domain physics
//! remains in the calculation engines that consume these primitives.

mod circular;
mod complementarity;
mod interpolation;
mod interval;
mod root;
mod volume;

pub use circular::{CircularAccumulator, CircularError, CircularSummary};
pub use complementarity::{fischer_burmeister, fischer_burmeister_gradient};
pub use interpolation::{InterpolationError, MonotoneCurve};
pub use interval::{Interval, IntervalError, partition_boundaries};
pub use root::{RootError, RootOptions, RootSolution, solve_bracketed, solve_safeguarded_newton};
pub use volume::{VolumeCoordinate, VolumeError, VolumeSegment};

/// Terminal state reported by an iterative numerical primitive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConvergenceState {
    /// The requested convergence criterion was satisfied.
    Converged,
    /// The solver stopped without satisfying its convergence criterion.
    NotConverged,
}

/// Common convergence evidence returned by iterative numerical primitives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverDiagnostics {
    /// Final convergence state.
    pub state: ConvergenceState,
    /// Number of solver iterations executed.
    pub iterations: usize,
    /// Residual norm before iteration.
    pub initial_residual_norm: f64,
    /// Residual norm at termination.
    pub final_residual_norm: f64,
    /// Absolute convergence tolerance.
    pub absolute_tolerance: f64,
    /// Relative convergence tolerance.
    pub relative_tolerance: f64,
    /// Whether a safeguarded fallback path was used.
    pub fallback_used: bool,
}

impl SolverDiagnostics {
    /// Returns true when the numerical primitive satisfied its convergence criterion.
    #[must_use]
    pub const fn converged(self) -> bool {
        matches!(self.state, ConvergenceState::Converged)
    }
}
