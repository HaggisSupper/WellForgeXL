//! Acceptance contract for the remaining Phase 1 domain-independent numerical substrate.

use wellforge_numerics::{
    FlowEdge, JacobianProvider, NewtonOptions, ParcelQueue, ResidualModel, RobustRegressionOptions,
    continuity_residuals, robust_weighted_least_squares, solve_newton_system,
};

struct QuadraticSystem;

impl ResidualModel for QuadraticSystem {
    fn residual(&self, x: &[f64], out: &mut [f64]) {
        out[0] = x[0] * x[0] - 4.0;
        out[1] = x[1] - 3.0;
    }
}

impl JacobianProvider for QuadraticSystem {
    fn jacobian(&self, x: &[f64], out: &mut [f64]) {
        out.copy_from_slice(&[2.0 * x[0], 0.0, 0.0, 1.0]);
    }
}

#[test]
fn damped_newton_solves_small_bounded_system() {
    let solution = solve_newton_system(
        &QuadraticSystem,
        &[0.1, 8.0],
        &[0.0, -10.0],
        &[10.0, 10.0],
        NewtonOptions::default(),
    )
    .expect("bounded nonlinear solve");
    assert!((solution.values[0] - 2.0).abs() < 1.0e-9);
    assert!((solution.values[1] - 3.0).abs() < 1.0e-9);
    assert!(solution.diagnostics.converged());
}

#[test]
fn parcel_queue_conserves_volume_through_displacement() {
    let mut queue = ParcelQueue::new(10.0).expect("positive path capacity");
    queue.fill("mud_a", 10.0).expect("initial fill");
    let exited = queue
        .inject("pill", 3.0)
        .expect("conservative displacement");
    assert!((queue.total_volume() - 10.0).abs() < 1.0e-12);
    assert!((exited.iter().map(|parcel| parcel.volume).sum::<f64>() - 3.0).abs() < 1.0e-12);
    assert_eq!(
        queue.parcels().last().expect("injected parcel").label,
        "pill"
    );
}

#[test]
fn graph_continuity_residuals_balance_internal_node() {
    let residuals = continuity_residuals(
        3,
        &[
            FlowEdge::new(0, 1, 12.0),
            FlowEdge::new(1, 2, 7.0),
            FlowEdge::new(1, 2, 5.0),
        ],
    )
    .expect("valid graph");
    assert!(residuals[1].abs() < 1.0e-12);
    assert!((residuals[0] + 12.0).abs() < 1.0e-12);
    assert!((residuals[2] - 12.0).abs() < 1.0e-12);
}

#[test]
fn robust_regression_limits_single_outlier_influence() {
    let design = vec![
        vec![1.0, 0.0],
        vec![1.0, 1.0],
        vec![1.0, 2.0],
        vec![1.0, 3.0],
        vec![1.0, 4.0],
    ];
    let observations = [1.0, 3.0, 5.0, 7.0, 50.0];
    let weights = [1.0; 5];
    let fit = robust_weighted_least_squares(
        &design,
        &observations,
        &weights,
        RobustRegressionOptions::default(),
    )
    .expect("robust least squares");
    assert!((fit.coefficients[0] - 1.0).abs() < 0.5);
    assert!((fit.coefficients[1] - 2.0).abs() < 0.5);
    assert!(fit.diagnostics.converged());
}
