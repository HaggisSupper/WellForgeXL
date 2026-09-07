//! Hybrid soft/stiff torque-and-drag result blending tests.

use wellforge_torque_drag_contract::{
    StiffConvergence, StiffIntervalReason, StiffIntervalResult, StiffNodeResult, StiffPointKind,
    StiffStringResult,
};
use wellforge_torque_drag_core::{blend_stiff_refinement, solve_soft_string};
use wellforge_torque_drag_fixtures::canonical_pickup_case;

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-9,
        "expected {expected:.12e}, got {actual:.12e}"
    );
}

#[test]
fn stiff_values_substitute_soft_stations_and_only_contact_points_are_inserted() {
    let request = canonical_pickup_case();
    let soft = solve_soft_string(&request).expect("soft solve");
    let original_len = soft.stations.len();
    let unchanged = soft
        .stations
        .iter()
        .find(|station| (station.md_m - 300.0).abs() <= 1.0e-9)
        .expect("300 m soft station")
        .clone();

    let interval_id =
        wellforge_torque_drag_contract::derive_stiff_interval_id(request.analysis_id, 0.0, 200.0);
    let stiff = StiffStringResult {
        intervals: vec![StiffIntervalResult {
            interval_id,
            start_md_m: 0.0,
            end_md_m: 200.0,
            reasons: vec![StiffIntervalReason::NormalLoad],
            convergence: StiffConvergence::Converged,
            residual_norm: 1.0e-10,
            iterations: 4,
            peak_contact_force_n: 12_000.0,
            peak_bending_stress_pa: 1.2e8,
            nodes: vec![
                StiffNodeResult {
                    md_m: 50.0,
                    effective_tension_n: 510_000.0,
                    torque_nm: 11_000.0,
                    normal_load_n_m: 9_500.0,
                    dogleg_rad_m: 1.0e-3,
                    displacement_m: 0.012,
                    radial_clearance_m: 0.0,
                    contact_force_n: 12_000.0,
                    bending_moment_nm: 22_000.0,
                    bending_stress_pa: 1.2e8,
                },
                StiffNodeResult {
                    md_m: 100.0,
                    effective_tension_n: 505_000.0,
                    torque_nm: 10_500.0,
                    normal_load_n_m: 8_750.0,
                    dogleg_rad_m: 8.0e-4,
                    displacement_m: 0.009,
                    radial_clearance_m: 0.002,
                    contact_force_n: 0.0,
                    bending_moment_nm: 18_000.0,
                    bending_stress_pa: 9.0e7,
                },
                StiffNodeResult {
                    md_m: 150.0,
                    effective_tension_n: 500_000.0,
                    torque_nm: 10_250.0,
                    normal_load_n_m: 8_000.0,
                    dogleg_rad_m: 7.0e-4,
                    displacement_m: 0.007,
                    radial_clearance_m: 0.004,
                    contact_force_n: 0.0,
                    bending_moment_nm: 15_000.0,
                    bending_stress_pa: 7.0e7,
                },
            ],
        }],
    };

    let blended = blend_stiff_refinement(soft, &stiff);

    assert_eq!(blended.stations.len(), original_len + 1);
    assert!(
        blended
            .stations
            .windows(2)
            .all(|pair| pair[1].md_m > pair[0].md_m)
    );

    let inserted = blended
        .stations
        .iter()
        .find(|station| (station.md_m - 50.0).abs() <= 1.0e-9)
        .expect("inserted contact point");
    near(inserted.effective_tension_n, 510_000.0);
    near(inserted.contact_force_n(), 12_000.0);
    assert_eq!(
        inserted.refinement.as_ref().expect("refinement").kind,
        StiffPointKind::InsertedContact
    );

    let substituted = blended
        .stations
        .iter()
        .find(|station| (station.md_m - 100.0).abs() <= 1.0e-9)
        .expect("substituted soft station");
    near(substituted.effective_tension_n, 505_000.0);
    near(substituted.torque_nm, 10_500.0);
    assert_eq!(
        substituted.refinement.as_ref().expect("refinement").kind,
        StiffPointKind::Substituted
    );

    assert!(
        blended
            .stations
            .iter()
            .all(|station| (station.md_m - 150.0).abs() > 1.0e-9)
    );

    let after = blended
        .stations
        .iter()
        .find(|station| (station.md_m - 300.0).abs() <= 1.0e-9)
        .expect("unchanged soft station");
    near(after.effective_tension_n, unchanged.effective_tension_n);
    near(after.torque_nm, unchanged.torque_nm);
    assert!(after.refinement.is_none());
}
