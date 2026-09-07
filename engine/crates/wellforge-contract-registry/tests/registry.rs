//! Acceptance tests for the centralized canonical contract registry and boundary version policies.

use wellforge_contract_registry::build_registry;

#[test]
fn registry_contains_all_current_canonical_contract_versions() {
    let registry = build_registry().expect("canonical registry must build");
    assert_eq!(registry.len(), 5);
    for descriptor in registry.iter() {
        assert_eq!(descriptor.request_schema_fingerprint.to_hex().len(), 64);
        assert_eq!(descriptor.result_schema_fingerprint.to_hex().len(), 64);
    }
}

#[test]
fn malformed_or_unsupported_versions_fail_before_domain_calculation() {
    let mut trajectory = wellforge_trajectory_fixtures::release_one_minimal_request();
    trajectory.contract_version = "1.bad.0".to_owned();
    let trajectory_errors = wellforge_trajectory_contract::validate_request(&trajectory)
        .expect_err("malformed trajectory SemVer must fail");
    assert_eq!(trajectory_errors[0].code, "WF-TRAJECTORY-CONTRACT-001");

    let mut bha = wellforge_bha_fixtures::minimal_request();
    bha.contract_version = "1.bad.0".to_owned();
    let bha_errors =
        wellforge_bha_contract::validate_request(&bha).expect_err("malformed BHA SemVer must fail");
    assert_eq!(bha_errors[0].code, "WF-BHA-CONTRACT-001");

    let mut tnd = wellforge_torque_drag_fixtures::canonical_pickup_case();
    tnd.contract_version = "0.2.0".to_owned();
    let tnd_errors = wellforge_torque_drag_contract::validate_request(&tnd)
        .expect_err("unsupported TnD version must fail");
    assert_eq!(tnd_errors[0].code, "WF-TND-REQ-001");

    let mut hydraulics = wellforge_hydraulics_fixtures::canonical_bingham_case();
    hydraulics.contract_version = "0.3.0".to_owned();
    let hydraulics_errors = wellforge_hydraulics_contract::validate_request(&hydraulics)
        .expect_err("unsupported hydraulics version must fail");
    assert_eq!(hydraulics_errors[0].code, "WF-HYD-REQ-001");
}

#[test]
fn stable_major_contracts_accept_valid_same_major_semver() {
    let mut trajectory = wellforge_trajectory_fixtures::release_one_minimal_request();
    trajectory.contract_version = "1.7.3".to_owned();
    assert!(wellforge_trajectory_contract::validate_request(&trajectory).is_ok());

    let mut bha = wellforge_bha_fixtures::minimal_request();
    bha.contract_version = "1.4.2".to_owned();
    assert!(wellforge_bha_contract::validate_request(&bha).is_ok());
}
