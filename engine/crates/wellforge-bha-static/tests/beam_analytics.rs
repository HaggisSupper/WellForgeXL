//! Analytical beam acceptance tests.

use wellforge_bha_static::cantilever_tip_deflection;

#[test]
fn cantilever_matches_closed_form_tip_deflection() {
    let calculated = cantilever_tip_deflection(2.0, 210.0e9, 8.0e-6, 1_000.0);
    let expected = 1_000.0 * 2.0_f64.powi(3) / (3.0 * 210.0e9 * 8.0e-6);
    assert!((calculated - expected).abs() / expected < 1.0e-12);
}

#[test]
fn static_solution_exposes_projected_od_hole_clearance() {
    let request = wellforge_bha_fixtures::minimal_request();
    let model = wellforge_bha_model::assemble_model(&request).unwrap();
    let solution = wellforge_bha_static::solve_static(&model, &request).unwrap();
    assert_eq!(solution.nodes.len(), model.nodes.len());
    assert!(
        solution
            .nodes
            .iter()
            .all(|node| node.projected_clearance_m.is_finite())
    );
}

#[test]
fn vertical_bha_has_no_transverse_gravity_sag() {
    let mut request = wellforge_bha_fixtures::minimal_request();
    for station in &mut request.trajectory {
        station.inclination_rad = 0.0;
    }
    let model = wellforge_bha_model::assemble_model(&request).unwrap();
    let solution = wellforge_bha_static::solve_static(&model, &request).unwrap();
    assert!(solution.nodes.iter().all(|node| node.x_m.abs() < 1.0e-12));
}
