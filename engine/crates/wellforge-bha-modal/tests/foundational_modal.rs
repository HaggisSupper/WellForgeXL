//! Structural acceptance for the BHA modal mass-normalization backend.

use wellforge_bha_modal::MODAL_MASS_TRANSFORM_BACKEND;

#[test]
fn modal_solver_reports_triangular_solve_backend() {
    assert_eq!(
        MODAL_MASS_TRANSFORM_BACKEND,
        "nalgebra/cholesky-triangular-solves"
    );
}
