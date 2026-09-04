use serde_json::json;
use wellforge_core::{
    ApiError, CalculationBackend, CalculationContext, CalculationReceipt, InputRevision, Metres,
    Radians, ReceiptError, SiValueError,
};

#[test]
fn no_project_error_serializes_to_structured_json() {
    let error = ApiError::no_open_project();
    let value = serde_json::to_value(error).expect("error must be JSON serializable");

    assert_eq!(value["code"], "NO_OPEN_PROJECT");
    assert_eq!(value["message"], "No project is currently open");
    assert!(value.get("details").is_none());
}

#[test]
fn finite_si_values_are_transparent_on_the_wire() {
    let metres = Metres::try_new(123.45).expect("finite length is valid");
    let radians = Radians::try_new(1.25).expect("finite angle is valid");

    assert_eq!(serde_json::to_string(&metres).unwrap(), "123.45");
    assert_eq!(serde_json::to_string(&radians).unwrap(), "1.25");
    assert_eq!(serde_json::from_str::<Metres>("123.45").unwrap(), metres);
    assert_eq!(serde_json::from_str::<Radians>("1.25").unwrap(), radians);
}

#[test]
fn finite_si_values_reject_non_finite_construction() {
    assert_eq!(
        Metres::try_new(f64::NAN),
        Err(SiValueError::NonFiniteMetres)
    );
    assert_eq!(
        Radians::try_new(f64::INFINITY),
        Err(SiValueError::NonFiniteRadians)
    );
}

#[test]
fn finite_si_values_reject_null_and_out_of_range_json_values() {
    assert!(serde_json::from_str::<Metres>("null").is_err());
    assert!(serde_json::from_str::<Radians>("null").is_err());
    assert!(serde_json::from_str::<Metres>("1e999").is_err());
    assert!(serde_json::from_str::<Radians>("-1e999").is_err());
}

#[test]
fn calculation_receipt_canonically_hashes_an_output_and_preserves_provenance() {
    let receipt = CalculationReceipt::create(
        "minimum-curvature",
        "2026.1",
        vec![InputRevision {
            kind: "project_revision".to_owned(),
            id: "project-001:rev-4".to_owned(),
            content_sha256: "a".repeat(64),
        }],
        CalculationContext {
            unit_system: "si".to_owned(),
            crs: "EPSG:4979".to_owned(),
            backend: CalculationBackend::Cpu,
            actor_id: "engineer-17".to_owned(),
            warnings: vec!["input uses synthetic fixture".to_owned()],
        },
        &json!({"northM": 12.0, "tvdM": 99.0}),
    )
    .expect("receipt is valid");

    assert_eq!(receipt.algorithm(), "minimum-curvature");
    assert_eq!(receipt.output_sha256().len(), 64);
    assert_eq!(receipt.input_revisions().len(), 1);
    assert_eq!(receipt.context().backend, CalculationBackend::Cpu);
    assert!(
        receipt
            .verifies_output(&json!({"northM": 12.0, "tvdM": 99.0}))
            .expect("output is serializable")
    );
    assert!(
        !receipt
            .verifies_output(&json!({"northM": 12.1, "tvdM": 99.0}))
            .expect("output is serializable")
    );
}

#[test]
fn calculation_receipt_hash_is_independent_of_json_object_key_order() {
    let input = vec![InputRevision {
        kind: "project_revision".to_owned(),
        id: "project-001:rev-4".to_owned(),
        content_sha256: "b".repeat(64),
    }];
    let context = CalculationContext {
        unit_system: "si".to_owned(),
        crs: "EPSG:4979".to_owned(),
        backend: CalculationBackend::Cpu,
        actor_id: "engineer-17".to_owned(),
        warnings: vec![],
    };

    let first = CalculationReceipt::create(
        "minimum-curvature",
        "2026.1",
        input.clone(),
        context.clone(),
        &json!({"northM": 12.0, "tvdM": 99.0}),
    )
    .expect("first receipt is valid");
    let second = CalculationReceipt::create(
        "minimum-curvature",
        "2026.1",
        input,
        context,
        &json!({"tvdM": 99.0, "northM": 12.0}),
    )
    .expect("second receipt is valid");

    assert_eq!(first.output_sha256(), second.output_sha256());
}

#[test]
fn calculation_receipt_rejects_blank_identifiers_and_invalid_hashes() {
    let result = CalculationReceipt::create(
        " ",
        "2026.1",
        vec![InputRevision {
            kind: "project_revision".to_owned(),
            id: "project-001:rev-4".to_owned(),
            content_sha256: "not-a-digest".to_owned(),
        }],
        CalculationContext {
            unit_system: "si".to_owned(),
            crs: "EPSG:4979".to_owned(),
            backend: CalculationBackend::Cpu,
            actor_id: "engineer-17".to_owned(),
            warnings: vec![],
        },
        &json!({"northM": 12.0}),
    );

    assert!(matches!(result, Err(ReceiptError::InvalidAlgorithm)));
}

#[test]
fn calculation_receipt_rejects_invalid_serialized_provenance() {
    let receipt = CalculationReceipt::create(
        "minimum-curvature",
        "2026.1",
        vec![InputRevision {
            kind: "portable_project_artifact".to_owned(),
            id: "artifact-1".to_owned(),
            content_sha256: "a".repeat(64),
        }],
        CalculationContext {
            unit_system: "si".to_owned(),
            crs: "EPSG:4979".to_owned(),
            backend: CalculationBackend::Cpu,
            actor_id: "local-workstation".to_owned(),
            warnings: vec![],
        },
        &json!({"northM": 12.0}),
    )
    .expect("fixture receipt is valid");
    let mut serialized = serde_json::to_value(receipt).expect("receipt serializes");
    serialized["outputSha256"] = json!("not-a-digest");

    assert!(serde_json::from_value::<CalculationReceipt>(serialized).is_err());
}
