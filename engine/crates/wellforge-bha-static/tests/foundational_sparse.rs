//! Structural and parity acceptance for the BHA static linear-solve backend.

use wellforge_bha_static::{STATIC_LINEAR_SOLVER_BACKEND, solve_static};

#[test]
fn static_solver_reports_foundational_sparse_backend() {
    assert_eq!(STATIC_LINEAR_SOLVER_BACKEND, "wellforge-numerics/sparse-lu");
}

#[test]
fn static_solver_does_not_recover_sparse_entries_by_scanning_dense_matrix() {
    let implementation = include_str!("../src/lib.rs");
    assert!(
        !implementation.contains("fn sparse_entries(matrix: &DMatrix"),
        "static FE solve must assemble sparse coefficients directly from element contributions"
    );
}

#[test]
fn foundational_sparse_path_preserves_static_solution_quality() {
    let request = wellforge_bha_fixtures::minimal_request();
    let model = wellforge_bha_model::assemble_model(&request).expect("valid minimal BHA model");
    let solution = solve_static(&model, &request).expect("static sparse solve");

    assert_eq!(solution.nodes.len(), model.nodes.len());
    assert_eq!(solution.displacement.len(), solution.stiffness.nrows());
    assert!(solution.residual_norm.is_finite());
    assert!(solution.residual_norm < 1.0e-10);
}
