//! Deterministic soft-string sanity test against the canonical fixture.

use wellforge_torque_drag_core::solve_soft_string;
use wellforge_torque_drag_fixtures::canonical_pickup_case;

#[test]
fn pickup_produces_monotonically_increasing_tension_up_hole() {
    let request = canonical_pickup_case();
    let result = solve_soft_string(&request).expect("solver must succeed");
    assert_eq!(result.stations.len(), request.trajectory.len());
    // On pickup, effective tension must increase from bit (deepest) to surface (shallowest).
    // Stations are surface-down order.
    let tensions: Vec<f64> = result
        .stations
        .iter()
        .map(|s| s.effective_tension_n)
        .collect();
    for window in tensions.windows(2) {
        assert!(
            window[0] >= window[1] - 1.0,
            "pickup: shallower tension must exceed or equal deeper tension: {window:?}"
        );
    }
    // API 7G utilization must be a valid ratio.
    let ratio = result.api7g.tensile_utilization;
    assert!(ratio.is_finite() && ratio >= 0.0);
}

#[test]
fn surface_station_buckling_values_are_finite_and_json_numeric() {
    let mut request = canonical_pickup_case();
    request.trajectory[0].inclination_rad = 0.0;
    let result = solve_soft_string(&request).expect("solver must succeed");
    let surface = &result.buckling[0];
    assert!(surface.sinusoidal_threshold_n.is_finite());
    assert!(surface.helical_threshold_n.is_finite());
}
