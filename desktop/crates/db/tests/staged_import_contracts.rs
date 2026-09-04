use serde_json::json;
use wellforge_storage::{
    ImportBatchMetadata, RawSourceRecord, SourceIdentityMapping, StagedValidationResult,
    ValidationDisposition, ValidationFinding,
};

const CHECKSUM: &str = "sha256:77e753c3f0cbd7ef62c7eaab8d2b6fef77e753c3f0cbd7ef62c7eaab8d2b6fef";

fn valid_batch() -> ImportBatchMetadata {
    ImportBatchMetadata {
        batch_id: "batch-20260825-001".to_owned(),
        source_system: "catalog-export".to_owned(),
        source_location: "imports/catalog-20260825.json".to_owned(),
        source_checksum: CHECKSUM.to_owned(),
        extracted_at: "2026-08-25T14:30:00Z".to_owned(),
        operator_id: "import-service".to_owned(),
    }
}

fn valid_identity() -> SourceIdentityMapping {
    SourceIdentityMapping {
        source_system: "catalog-export".to_owned(),
        source_entity_kind: "component".to_owned(),
        source_entity_id: "component-0042".to_owned(),
        source_checksum: CHECKSUM.to_owned(),
    }
}

fn valid_record() -> RawSourceRecord {
    RawSourceRecord {
        batch_id: valid_batch().batch_id,
        source_record_key: "row-000042".to_owned(),
        identity: valid_identity(),
        payload: json!({"name": "Example component", "outerDiameterMm": 203.2}),
    }
}

fn valid_result() -> StagedValidationResult {
    StagedValidationResult {
        batch_id: valid_batch().batch_id,
        source_record_key: "row-000042".to_owned(),
        disposition: ValidationDisposition::Accepted,
        rule_set_version: "catalog-rules-1.0.0".to_owned(),
        validated_at: "2026-08-25T14:31:00Z".to_owned(),
        findings: vec![ValidationFinding {
            code: "normalization-applied".to_owned(),
            message: "Source text was normalized.".to_owned(),
        }],
    }
}

#[test]
fn staged_contracts_validate_and_round_trip_through_json() {
    let batch = valid_batch();
    let record = valid_record();
    let result = valid_result();

    batch.validate().expect("batch fixture must be valid");
    record.validate().expect("record fixture must be valid");
    result.validate().expect("result fixture must be valid");

    for value in [
        serde_json::to_value(&batch).expect("batch serializes"),
        serde_json::to_value(&record).expect("record serializes"),
        serde_json::to_value(&result).expect("result serializes"),
    ] {
        assert!(value.is_object());
    }

    let restored: RawSourceRecord =
        serde_json::from_str(&serde_json::to_string(&record).expect("record serializes"))
            .expect("record deserializes");
    assert_eq!(restored, record);
}

#[test]
fn import_batch_rejects_blank_or_unverifiable_provenance() {
    let invalid_batches = [
        ImportBatchMetadata {
            batch_id: " ".to_owned(),
            ..valid_batch()
        },
        ImportBatchMetadata {
            source_system: " ".to_owned(),
            ..valid_batch()
        },
        ImportBatchMetadata {
            source_location: " ".to_owned(),
            ..valid_batch()
        },
        ImportBatchMetadata {
            source_checksum: "sha256:short".to_owned(),
            ..valid_batch()
        },
        ImportBatchMetadata {
            extracted_at: "2026-08-25T14:30:00+01:00".to_owned(),
            ..valid_batch()
        },
        ImportBatchMetadata {
            operator_id: " ".to_owned(),
            ..valid_batch()
        },
    ];

    for batch in invalid_batches {
        assert!(batch.validate().is_err());
    }
}

#[test]
fn source_record_rejects_blank_identity_or_non_object_payloads() {
    let invalid_records = [
        RawSourceRecord {
            source_record_key: " ".to_owned(),
            ..valid_record()
        },
        RawSourceRecord {
            identity: SourceIdentityMapping {
                source_system: " ".to_owned(),
                ..valid_identity()
            },
            ..valid_record()
        },
        RawSourceRecord {
            identity: SourceIdentityMapping {
                source_entity_kind: " ".to_owned(),
                ..valid_identity()
            },
            ..valid_record()
        },
        RawSourceRecord {
            identity: SourceIdentityMapping {
                source_entity_id: " ".to_owned(),
                ..valid_identity()
            },
            ..valid_record()
        },
        RawSourceRecord {
            payload: json!("not-an-object"),
            ..valid_record()
        },
    ];

    for record in invalid_records {
        assert!(record.validate().is_err());
    }
}

#[test]
fn validation_result_requires_provenance_and_complete_findings() {
    let invalid_results = [
        StagedValidationResult {
            rule_set_version: " ".to_owned(),
            ..valid_result()
        },
        StagedValidationResult {
            validated_at: "not-a-timestamp".to_owned(),
            ..valid_result()
        },
        StagedValidationResult {
            disposition: ValidationDisposition::NeedsReview,
            findings: Vec::new(),
            ..valid_result()
        },
        StagedValidationResult {
            findings: vec![ValidationFinding {
                code: " ".to_owned(),
                message: "Useful detail.".to_owned(),
            }],
            ..valid_result()
        },
        StagedValidationResult {
            findings: vec![ValidationFinding {
                code: "bad-input".to_owned(),
                message: " ".to_owned(),
            }],
            ..valid_result()
        },
    ];

    for result in invalid_results {
        assert!(result.validate().is_err());
    }
}

#[test]
fn malformed_json_is_rejected_before_validation() {
    assert!(serde_json::from_str::<ImportBatchMetadata>("{not json}").is_err());
    assert!(serde_json::from_str::<RawSourceRecord>("[]").is_err());
    assert!(serde_json::from_str::<StagedValidationResult>("null").is_err());
}
