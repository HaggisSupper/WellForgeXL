//! Structural and parity acceptance for the BHA static linear-solve backend.

use nalgebra::DVector;
use wellforge_bha_contract::BhaAnalysisRequest;
use wellforge_bha_model::BhaModel;
use wellforge_bha_static::{STATIC_LINEAR_SOLVER_BACKEND, solve_static};

fn independent_reduced_transverse_load(model: &BhaModel, request: &BhaAnalysisRequest) -> Vec<f64> {
    let mut full_load = vec![0.0; model.nodes.len() * 2];
    for element in 0..model.nodes.len() - 1 {
        let first = &model.nodes[element];
        let second = &model.nodes[element + 1];
        let length = second.md_m - first.md_m;
        let od = first.od_m.midpoint(second.od_m);
        let id = first.id_m.midpoint(second.id_m);
        let density = first.density_kg_m3.midpoint(second.density_kg_m3);
        let area = std::f64::consts::PI * (od.powi(2) - id.powi(2)) / 4.0;
        let buoyed_mass_per_length =
            (density - request.operating.fluid_density_kg_m3).max(0.0) * area;
        let mid_md = first.md_m.midpoint(second.md_m);
        let inclination = request
            .trajectory
            .iter()
            .min_by(|left, right| {
                (left.md_m - mid_md)
                    .abs()
                    .total_cmp(&(right.md_m - mid_md).abs())
            })
            .map_or(0.0, |station| station.inclination_rad);
        let distributed_load = buoyed_mass_per_length * 9.80665 * inclination.sin().abs();
        full_load[2 * element] += distributed_load * length * 0.5;
        full_load[2 * element + 2] += distributed_load * length * 0.5;
    }
    full_load[2..].to_vec()
}

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

#[test]
fn foundational_sparse_displacement_matches_independent_dense_lu_oracle() {
    let request = wellforge_bha_fixtures::minimal_request();
    let model = wellforge_bha_model::assemble_model(&request).expect("valid minimal BHA model");
    let solution = solve_static(&model, &request).expect("static sparse solve");
    let rhs = DVector::from_vec(independent_reduced_transverse_load(&model, &request));
    let dense = solution
        .stiffness
        .clone()
        .lu()
        .solve(&rhs)
        .expect("dense LU oracle must solve the same reduced physical system");

    let max_error = solution
        .displacement
        .iter()
        .zip(dense.iter())
        .map(|(sparse, dense)| (sparse - dense).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        max_error < 1.0e-10,
        "sparse/dense displacement mismatch {max_error:e}"
    );
}
