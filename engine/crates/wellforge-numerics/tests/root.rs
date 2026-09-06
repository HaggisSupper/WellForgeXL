//! Contract tests for deterministic scalar root solvers.

use wellforge_numerics::{
    RootError, RootOptions, solve_bracketed, solve_safeguarded_newton,
};

#[test]
fn bracketed_solver_converges_on_square_root_two() {
    let solution = solve_bracketed(0.0, 2.0, |x| x * x - 2.0, RootOptions::default())
        .expect("valid sign-changing bracket");

    assert!((solution.root - 2.0_f64.sqrt()).abs() < 1.0e-10);
    assert!(solution.diagnostics.converged());
    assert!(solution.residual.abs() <= 1.0e-10);
}

#[test]
fn bracketed_solver_rejects_interval_without_sign_change() {
    let error = solve_bracketed(-1.0, 1.0, |x| x * x + 1.0, RootOptions::default())
        .expect_err("non-sign-changing bracket must fail");

    assert_eq!(error, RootError::InvalidBracket);
}

#[test]
fn safeguarded_newton_falls_back_when_newton_step_leaves_bracket() {
    let solution = solve_safeguarded_newton(
        0.0,
        2.0,
        0.1,
        |x| x * x * x - 2.0,
        |x| 3.0 * x * x,
        RootOptions::default(),
    )
    .expect("safeguarded solve");

    assert!((solution.root - 2.0_f64.cbrt()).abs() < 1.0e-10);
    assert!(solution.diagnostics.converged());
    assert!(solution.diagnostics.fallback_used);
}
