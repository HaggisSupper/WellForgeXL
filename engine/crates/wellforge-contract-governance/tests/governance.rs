//! Contract tests for domain-independent canonical contract governance.

use semver::Version;
use serde_json::json;
use wellforge_contract_governance::{
    CompatibilityPolicy, ContractId, ContractVersionPolicy, GovernanceError, fingerprint_schema,
    normalize_json,
};

#[test]
fn canonical_contract_ids_are_validated() {
    let id: ContractId = "wellforge.trajectory.analysis"
        .parse()
        .expect("canonical dotted identifier");
    assert_eq!(id.as_str(), "wellforge.trajectory.analysis");

    for invalid in [
        "",
        ".wellforge.trajectory.analysis",
        "wellforge..analysis",
        "WellForge.trajectory.analysis",
        "wellforge.trajectory-analysis",
        "wellforge.1trajectory.analysis",
    ] {
        assert!(invalid.parse::<ContractId>().is_err(), "accepted {invalid}");
    }

    let tnd: ContractId = "wellforge.torque_drag.analysis"
        .parse()
        .expect("underscore is canonical within a segment");
    assert_eq!(tnd.as_str(), "wellforge.torque_drag.analysis");
}

#[test]
fn malformed_semver_is_rejected_before_policy_matching() {
    let policy = ContractVersionPolicy::new(
        Version::parse("1.0.0").expect("registered version"),
        CompatibilityPolicy::SameMajor,
    );

    assert_eq!(
        policy.validate_str("1.bad.0"),
        Err(GovernanceError::InvalidSemanticVersion)
    );
}

#[test]
fn exact_same_major_and_explicit_set_are_deterministic() {
    let exact = ContractVersionPolicy::new(
        Version::parse("1.2.3").expect("registered version"),
        CompatibilityPolicy::Exact,
    );
    assert!(exact.validate_str("1.2.3").is_ok());
    assert_eq!(
        exact.validate_str("1.2.4"),
        Err(GovernanceError::UnsupportedVersion)
    );

    let same_major = ContractVersionPolicy::new(
        Version::parse("1.0.0").expect("registered version"),
        CompatibilityPolicy::SameMajor,
    );
    assert!(same_major.validate_str("1.99.7").is_ok());
    assert_eq!(
        same_major.validate_str("2.0.0"),
        Err(GovernanceError::UnsupportedVersion)
    );

    let explicit = ContractVersionPolicy::new(
        Version::parse("0.2.0").expect("registered version"),
        CompatibilityPolicy::ExplicitSet(vec![
            Version::parse("0.1.0").expect("explicit version"),
            Version::parse("0.2.0").expect("explicit version"),
        ]),
    );
    assert!(explicit.validate_str("0.1.0").is_ok());
    assert!(explicit.validate_str("0.2.0").is_ok());
    assert_eq!(
        explicit.validate_str("0.3.0"),
        Err(GovernanceError::UnsupportedVersion)
    );
}

#[test]
fn schema_normalization_and_fingerprinting_ignore_object_key_order() {
    let first = json!({
        "type": "object",
        "properties": {
            "alpha": {"type": "number"},
            "beta": {"type": "string"}
        },
        "required": ["alpha", "beta"]
    });
    let second = json!({
        "required": ["alpha", "beta"],
        "properties": {
            "beta": {"type": "string"},
            "alpha": {"type": "number"}
        },
        "type": "object"
    });
    let changed = json!({
        "type": "object",
        "properties": {
            "alpha": {"type": "integer"},
            "beta": {"type": "string"}
        },
        "required": ["alpha", "beta"]
    });

    assert_eq!(normalize_json(&first), normalize_json(&second));
    assert_eq!(fingerprint_schema(&first), fingerprint_schema(&second));
    assert_ne!(fingerprint_schema(&first), fingerprint_schema(&changed));
    assert_eq!(fingerprint_schema(&first).to_hex().len(), 64);
}
