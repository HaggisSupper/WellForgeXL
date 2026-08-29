//! Numerical dependency acceptance test.

#[test]
fn libraries_cover_release_one_numerics() {
    let report = wellforge_numerics_acceptance::run();
    assert!(report.quaternion_round_trip);
    assert!(report.linear_solve);
    assert!(report.symmetric_eigenpairs);
    assert!(report.contact_distance_query);
    assert!(report.nonlinear_root_solve);
    assert!(report.licenses_allowed);
}
