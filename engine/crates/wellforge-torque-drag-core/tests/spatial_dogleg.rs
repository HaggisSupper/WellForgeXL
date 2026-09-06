//! Spatial dogleg regression tests for the soft-string T&D solver.

use wellforge_torque_drag_core::solve_soft_string;
use wellforge_torque_drag_fixtures::canonical_pickup_case;

fn expected_spatial_curvature(i1: f64, a1: f64, i2: f64, a2: f64, dmd: f64) -> f64 {
    let cosine = i1.cos() * i2.cos() + i1.sin() * i2.sin() * (a2 - a1).cos();
    cosine.clamp(-1.0, 1.0).acos() / dmd
}

#[test]
fn pure_turn_generates_nonzero_spatial_curvature() {
    let mut request = canonical_pickup_case();
    assert!(request.trajectory.len() >= 2);
    let inclination = 60.0_f64.to_radians();
    request.trajectory[0].inclination_rad = inclination;
    request.trajectory[1].inclination_rad = inclination;
    request.trajectory[0].azimuth_rad = 10.0_f64.to_radians();
    request.trajectory[1].azimuth_rad = 30.0_f64.to_radians();

    let dmd = request.trajectory[1].md_m - request.trajectory[0].md_m;
    let expected = expected_spatial_curvature(
        request.trajectory[0].inclination_rad,
        request.trajectory[0].azimuth_rad,
        request.trajectory[1].inclination_rad,
        request.trajectory[1].azimuth_rad,
        dmd,
    );

    let result = solve_soft_string(&request).expect("solver must succeed");
    let actual = result.stations[0].dogleg_rad_m;
    assert!(expected > 0.0);
    assert!(
        (actual - expected).abs() <= 1.0e-12,
        "expected spatial curvature {expected:.15e}, got {actual:.15e}"
    );
}

#[test]
fn azimuth_wrap_uses_short_spatial_dogleg() {
    let mut request = canonical_pickup_case();
    assert!(request.trajectory.len() >= 2);
    let inclination = 70.0_f64.to_radians();
    request.trajectory[0].inclination_rad = inclination;
    request.trajectory[1].inclination_rad = inclination;
    request.trajectory[0].azimuth_rad = 359.0_f64.to_radians();
    request.trajectory[1].azimuth_rad = 1.0_f64.to_radians();

    let dmd = request.trajectory[1].md_m - request.trajectory[0].md_m;
    let expected = expected_spatial_curvature(
        request.trajectory[0].inclination_rad,
        request.trajectory[0].azimuth_rad,
        request.trajectory[1].inclination_rad,
        request.trajectory[1].azimuth_rad,
        dmd,
    );

    let result = solve_soft_string(&request).expect("solver must succeed");
    let actual = result.stations[0].dogleg_rad_m;
    assert!((actual - expected).abs() <= 1.0e-12);
    assert!(actual < 5.0_f64.to_radians() / dmd);
}
