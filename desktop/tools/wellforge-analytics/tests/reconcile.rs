use std::{fs, path::Path};

use wellforge_analytics::{AnalyticsError, reconcile_files};

const DIGEST: &str = "sha256:77e753c3f0cbd7ef62c7eaab8d2b6fef77e753c3f0cbd7ef62c7eaab8d2b6fef";

fn write_fixture(root: &Path, records: &[&str]) -> (std::path::PathBuf, std::path::PathBuf) {
    let manifest_path = root.join("manifest.json");
    let records_path = root.join("records.jsonl");
    fs::write(
        &manifest_path,
        format!(
            r#"{{"batch":{{"batchId":"batch-001","sourceSystem":"rig-feed","sourceLocation":"synthetic://rig-feed/batch-001","sourceChecksum":"{DIGEST}","extractedAt":"2026-08-25T12:00:00Z","operatorId":"fixture-runner"}},"ruleSetVersion":"2026.08"}}"#,
        ),
    )
    .expect("manifest fixture writes");
    fs::write(&records_path, records.join("\n")).expect("record fixture writes");
    (manifest_path, records_path)
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
    assert_eq!(first.input_sha256.len(), 64);
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
