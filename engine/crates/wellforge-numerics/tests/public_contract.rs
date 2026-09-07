//! Public-module contract for the production numerical substrate.

use wellforge_numerics::{ConvergenceState, SolverDiagnostics};

#[test]
fn diagnostics_contract_is_domain_independent() {
    let diagnostics = SolverDiagnostics {
        state: ConvergenceState::Converged,
        iterations: 3,
        initial_residual_norm: 1.0,
        final_residual_norm: 1.0e-10,
        absolute_tolerance: 1.0e-9,
        relative_tolerance: 1.0e-9,
        fallback_used: false,
    };

    assert!(diagnostics.converged());
}
