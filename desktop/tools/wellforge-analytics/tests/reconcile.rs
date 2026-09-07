use std::{
    fs,
    io::{Seek, SeekFrom, Write},
    path::Path,
    process::Command,
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use wellforge_analytics::{AnalyticsError, MAX_RECORD_BYTES, MAX_RECORDS, reconcile_files};

const DIGEST: &str = "sha256:77e753c3f0cbd7ef62c7eaab8d2b6fef77e753c3f0cbd7ef62c7eaab8d2b6fef";

fn write_fixture(root: &Path, records: &[&str]) -> (std::path::PathBuf, std::path::PathBuf) {
    let records_path = root.join("records.jsonl");
    let record_bytes = records.join("\n");
    fs::write(&records_path, &record_bytes).expect("record fixture writes");
    let manifest_path = write_manifest(root, &format!("{:x}", Sha256::digest(record_bytes)));
    (manifest_path, records_path)
}

fn write_manifest(root: &Path, records_sha256: &str) -> std::path::PathBuf {
    let manifest_path = root.join("manifest.json");
    fs::write(
        &manifest_path,
        format!(
            r#"{{"schemaVersion":"wellforge.analytics.approved-extract/v1","recordsSha256":"{records_sha256}","batch":{{"batchId":"batch-001","sourceSystem":"rig-feed","sourceLocation":"synthetic://rig-feed/batch-001","sourceChecksum":"{DIGEST}","extractedAt":"2026-08-25T12:00:00Z","operatorId":"fixture-runner"}},"ruleSetVersion":"2026.08"}}"#,
        ),
    )
    .expect("manifest fixture writes");
    manifest_path
}

fn change_manifest(manifest_path: &Path, change: impl FnOnce(&mut Value)) {
    let mut manifest: Value = serde_json::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
    change(&mut manifest);
    fs::write(manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
}

fn accepted_record(key: &str) -> String {
    format!(
        r#"{{"source":{{"batchId":"batch-001","sourceRecordKey":"{key}","identity":{{"sourceSystem":"rig-feed","sourceEntityKind":"survey_station","sourceEntityId":"entity-{key}","sourceChecksum":"{DIGEST}"}},"payload":{{"mdM":123.4}}}},"validation":{{"batchId":"batch-001","sourceRecordKey":"{key}","disposition":"accepted","ruleSetVersion":"2026.08","validatedAt":"2026-08-25T12:01:00Z","findings":[]}}}}"#,
    )
}

#[test]
fn reconciles_only_accepted_provenance_bound_records_deterministically() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let (manifest, records) = write_fixture(
        temporary.path(),
        &[
            &accepted_record("station-02"),
            &accepted_record("station-01"),
        ],
    );

    let first = reconcile_files(&manifest, &records).expect("fixture reconciles");
    let second = reconcile_files(&manifest, &records).expect("fixture reconciles consistently");

    assert_eq!(first, second);
    assert_eq!(first.accepted_record_count, 2);
    assert_eq!(first.rejected_record_count, 0);
    assert_eq!(first.distinct_entity_kind_count, 1);
    assert_eq!(first.engine, "duckdb-in-memory+polars");
    assert_eq!(first.batch_id, "batch-001");
    // Independently calculated from these exact fixture bytes using .NET SHA256.
    assert_eq!(
        first.input_sha256,
        "412ef5747c12d8d7a4b84894ba6d56e78a5a0bd0127c1fb90029a61e35d8ccf8"
    );
}

#[test]
fn rejects_records_that_are_not_accepted_or_not_bound_to_the_manifest() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let mut rejected = accepted_record("station-01");
    rejected = rejected.replace(
        "\"disposition\":\"accepted\"",
        "\"disposition\":\"rejected\"",
    );
    rejected = rejected.replace(
        "\"findings\":[]",
        "\"findings\":[{\"code\":\"required\",\"message\":\"missing value\"}]",
    );
    let (manifest, records) = write_fixture(temporary.path(), &[&rejected]);

    let error = reconcile_files(&manifest, &records).expect_err("non-accepted input is forbidden");

    assert!(matches!(error, AnalyticsError::NonAcceptedRecord { .. }));
}

#[test]
fn rejects_duplicate_source_record_keys() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let duplicate = accepted_record("station-01");
    let (manifest, records) = write_fixture(temporary.path(), &[&duplicate, &duplicate]);

    let error =
        reconcile_files(&manifest, &records).expect_err("duplicate record key is forbidden");

    assert!(matches!(
        error,
        AnalyticsError::DuplicateSourceRecordKey { .. }
    ));
}

#[test]
fn rejects_unsupported_manifest_schemas() {
    for schema in [
        "",
        " ",
        "unsupported/v1",
        "wellforge.analytics.approved-extract/v2",
        "wellforge.analytics.approved-extract/V1",
        " wellforge.analytics.approved-extract/v1",
        "wellforge.analytics.approved-extract/v1 ",
    ] {
        let temporary = tempfile::tempdir().unwrap();
        let (manifest, records) =
            write_fixture(temporary.path(), &[&accepted_record("station-01")]);
        change_manifest(&manifest, |value| value["schemaVersion"] = json!(schema));

        let error = reconcile_files(&manifest, &records).expect_err(schema);
        assert!(
            matches!(error, AnalyticsError::UnsupportedManifestSchema { schema_version } if schema_version == schema)
        );
    }
}

#[test]
fn required_manifest_fields_cannot_be_missing_or_have_invalid_types() {
    for field in ["schemaVersion", "recordsSha256", "batch", "ruleSetVersion"] {
        for value in [
            None,
            Some(Value::Null),
            Some(json!(42)),
            Some(json!(false)),
            Some(json!([])),
            Some(json!({})),
        ] {
            let temporary = tempfile::tempdir().unwrap();
            let (manifest, records) =
                write_fixture(temporary.path(), &[&accepted_record("station-01")]);
            change_manifest(&manifest, |manifest| match value {
                Some(value) => manifest[field] = value,
                None => {
                    manifest.as_object_mut().unwrap().remove(field);
                }
            });

            let error = reconcile_files(&manifest, &records).expect_err(field);
            assert!(matches!(error, AnalyticsError::Json { path, .. } if path == manifest));
        }
    }
}

#[test]
fn rejects_malformed_record_digests() {
    for digest in [
        String::new(),
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        "g".repeat(64),
        format!("sha256:{}", "a".repeat(64)),
        format!(" {}", "a".repeat(63)),
        format!("{}\n", "a".repeat(63)),
        "é".repeat(32),
    ] {
        let temporary = tempfile::tempdir().unwrap();
        let (manifest, records) =
            write_fixture(temporary.path(), &[&accepted_record("station-01")]);
        change_manifest(&manifest, |value| value["recordsSha256"] = json!(digest));

        let error = reconcile_files(&manifest, &records).expect_err(&digest);
        assert!(matches!(error, AnalyticsError::InvalidRecordsSha256));
    }
}

#[test]
fn digest_mismatch_is_rejected_before_invalid_records_are_decoded() {
    let temporary = tempfile::tempdir().unwrap();
    let (manifest, records) = write_fixture(temporary.path(), &["not-json"]);
    write_manifest(temporary.path(), &"0".repeat(64));

    let error = reconcile_files(&manifest, &records).unwrap_err();
    assert!(matches!(error, AnalyticsError::RecordsSha256Mismatch));
}

#[test]
fn matching_digest_still_requires_valid_record_json() {
    let temporary = tempfile::tempdir().unwrap();
    let (manifest, records) = write_fixture(temporary.path(), &["not-json"]);

    let error = reconcile_files(&manifest, &records).unwrap_err();
    assert!(matches!(error, AnalyticsError::Json { path, .. } if path == records));
}

#[test]
fn rejects_record_input_larger_than_the_byte_limit_before_digest_or_decoding() {
    let temporary = tempfile::tempdir().unwrap();
    let records_path = temporary.path().join("records.jsonl");
    let mut records = fs::File::create(&records_path).unwrap();
    records.seek(SeekFrom::Start(256 * 1024 * 1024)).unwrap();
    records.write_all(b"x").unwrap();
    drop(records);
    let manifest = write_manifest(temporary.path(), &"0".repeat(64));

    let error = reconcile_files(&manifest, &records_path).unwrap_err();
    assert!(matches!(
        error,
        AnalyticsError::RecordByteLimitExceeded {
            limit: MAX_RECORD_BYTES
        }
    ));
}

#[test]
fn rejects_too_many_nonblank_records_before_digest_or_decoding() {
    let temporary = tempfile::tempdir().unwrap();
    let (manifest, records) =
        write_fixture(temporary.path(), &[&format!("{}x", "x\n".repeat(500_000))]);
    write_manifest(temporary.path(), &"0".repeat(64));

    let error = reconcile_files(&manifest, &records).unwrap_err();
    assert!(matches!(
        error,
        AnalyticsError::RecordLimitExceeded { limit: MAX_RECORDS }
    ));
}

#[test]
fn exact_row_limit_excludes_blank_lines_and_reaches_record_decoding() {
    let temporary = tempfile::tempdir().unwrap();
    let input = format!("{}x", "x\n \t\r\x0c\n".repeat(499_999));
    let (manifest, records) = write_fixture(temporary.path(), &[&input]);

    let error = reconcile_files(&manifest, &records).unwrap_err();
    assert!(matches!(error, AnalyticsError::Json { path, .. } if path == records));
}

#[test]
fn handles_ascii_blank_lines_crlf_and_missing_final_newline_without_normalizing_hashes() {
    let temporary = tempfile::tempdir().unwrap();
    let first = accepted_record("station-01");
    let second = accepted_record("station-02");
    let mut hashes = std::collections::BTreeSet::new();
    for input in [
        format!("{first}\n{second}"),
        format!("{first}\n{second}\n"),
        format!("\t \r\x0c\n{first}\r\n \t\r\x0c\n{second}\r\n"),
    ] {
        let (manifest, records) = write_fixture(temporary.path(), &[&input]);
        let report = reconcile_files(&manifest, &records).unwrap();
        assert_eq!(report.accepted_record_count, 2);
        assert_eq!(report.distinct_entity_kind_count, 1);
        hashes.insert(report.input_sha256);
    }
    assert_eq!(hashes.len(), 3);
}

#[test]
fn a_whitespace_only_record_change_breaks_the_exact_digest_binding() {
    let temporary = tempfile::tempdir().unwrap();
    let record = accepted_record("station-01");
    let (manifest, records) = write_fixture(temporary.path(), &[&record]);
    fs::write(&records, format!("{record}\r\n")).unwrap();

    let error = reconcile_files(&manifest, &records).unwrap_err();
    assert!(matches!(error, AnalyticsError::RecordsSha256Mismatch));
}

#[test]
fn empty_or_blank_input_requires_a_matching_digest_then_fails_as_empty() {
    let temporary = tempfile::tempdir().unwrap();
    for input in ["", " \t\r\n\x0c\n"] {
        let (manifest, records) = write_fixture(temporary.path(), &[input]);
        assert!(matches!(
            reconcile_files(&manifest, &records),
            Err(AnalyticsError::EmptyInput)
        ));
        write_manifest(temporary.path(), &"0".repeat(64));
        let error = reconcile_files(&manifest, &records).unwrap_err();
        assert!(matches!(error, AnalyticsError::RecordsSha256Mismatch));
    }
}

#[test]
fn integrity_failure_does_not_create_or_overwrite_a_cli_report() {
    let temporary = tempfile::tempdir().unwrap();
    let (manifest, records) = write_fixture(temporary.path(), &[&accepted_record("station-01")]);
    write_manifest(temporary.path(), &"0".repeat(64));
    let report = temporary.path().join("report.json");
    for existing in [false, true] {
        if existing {
            fs::write(&report, b"existing report").unwrap();
        }
        let output = Command::new(env!("CARGO_BIN_EXE_wellforge-analytics"))
            .arg("--manifest")
            .arg(&manifest)
            .arg("--records")
            .arg(&records)
            .arg("--report")
            .arg(&report)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("records SHA-256 does not match manifest")
        );
        if existing {
            assert_eq!(fs::read(&report).unwrap(), b"existing report");
        } else {
            assert!(!report.exists());
        }
    }
}

#[test]
fn matching_digest_does_not_bypass_staged_contract_or_manifest_binding_checks() {
    for (pointer, value, expected) in [
        ("/source/sourceRecordKey", "", "contract"),
        ("/source/batchId", "other-batch", "batch"),
        ("/validation/batchId", "other-batch", "batch"),
        ("/validation/sourceRecordKey", "other-key", "batch"),
        ("/validation/ruleSetVersion", "other-rules", "rules"),
        (
            "/source/identity/sourceSystem",
            "other-system",
            "provenance",
        ),
        (
            "/source/identity/sourceChecksum",
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "provenance",
        ),
    ] {
        let temporary = tempfile::tempdir().unwrap();
        let mut record: Value = serde_json::from_str(&accepted_record("station-01")).unwrap();
        *record.pointer_mut(pointer).unwrap() = json!(value);
        let (manifest, records) = write_fixture(temporary.path(), &[&record.to_string()]);
        let error = reconcile_files(&manifest, &records).unwrap_err();
        assert!(
            match expected {
                "contract" => matches!(error, AnalyticsError::Contract(_)),
                "batch" => matches!(error, AnalyticsError::BatchMismatch { .. }),
                "rules" => matches!(error, AnalyticsError::RuleSetMismatch { .. }),
                "provenance" => matches!(error, AnalyticsError::ProvenanceMismatch { .. }),
                _ => unreachable!(),
            },
            "{pointer}: {error}"
        );
    }
}
