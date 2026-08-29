//! BHA model assembly acceptance tests.

use wellforge_bha_fixtures::minimal_request;
use wellforge_bha_model::assemble_model;

#[test]
fn discretizes_ordered_components_and_conserves_mass() {
    let request = minimal_request();
    let model = assemble_model(&request).unwrap();
    assert_eq!(model.nodes.len(), 25);
    assert_eq!(model.component_graph.node_count(), 1);
    let expected_area = std::f64::consts::PI * (0.2032_f64.powi(2) - 0.0714_f64.powi(2)) / 4.0;
    let expected_mass = expected_area * 12.0 * 7850.0;
    assert!((model.total_mass_kg - expected_mass).abs() / expected_mass < 1.0e-12);
}

#[test]
fn reports_positive_centered_clearance() {
    let model = assemble_model(&minimal_request()).unwrap();
    assert!((model.nodes[0].radial_clearance_m - (0.31115 - 0.2032) / 2.0).abs() < 1.0e-12);
}
