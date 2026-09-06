//! Deterministic severe-interval classification tests for stiff-string refinement.

use wellforge_torque_drag_contract::{StiffIntervalReason, StiffStringMode};
use wellforge_torque_drag_core::{classify_stiff_intervals, solve_soft_string};
use wellforge_torque_drag_fixtures::canonical_pickup_case;

#[test]
fn contiguous_normal_load_triggers_merge_into_one_deterministic_interval() {
    let mut request = canonical_pickup_case();
    request.solver.severity_normal_load_n_m = 1.0;
    request.solver.severity_buckling_margin_n = 0.0;
    request.solver.transition_buffer_m = 25.0;

    let soft = solve_soft_string(&request).expect("soft solve");
    let first = classify_stiff_intervals(&request, &soft);
    let second = classify_stiff_intervals(&request, &soft);

    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    let interval = &first[0];
    assert_eq!(interval.start_md_m, 0.0);
    assert_eq!(interval.end_md_m, 3000.0);
    assert!(interval.reasons.contains(&StiffIntervalReason::NormalLoad));
}

#[test]
fn disabled_stiff_mode_emits_no_candidates() {
    let mut request = canonical_pickup_case();
    request.solver.stiff_string_mode = StiffStringMode::Disabled;
    request.solver.severity_normal_load_n_m = 1.0;
    let soft = solve_soft_string(&request).expect("soft solve");

    assert!(classify_stiff_intervals(&request, &soft).is_empty());
}
