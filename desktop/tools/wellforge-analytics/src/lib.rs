//! Deterministic, non-authoritative analytics over accepted staged extracts.
//!
//! This crate has no network client, connection-string input, promotion function, or durable
//! store. DuckDB is opened only with `:memory:` and Polars is used to materialize the accepted
//! rows for columnar reconciliation.

use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use duckdb::{Connection, params};
use polars::prelude::{DataFrame, IntoColumn, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use wellforge_storage::{
    ImportBatchMetadata, RawSourceRecord, StagedValidationResult, ValidationDisposition,
};

pub const MAX_RECORD_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_RECORDS: u64 = 500_000;

const APPROVED_EXTRACT_SCHEMA_VERSION: &str = "wellforge.analytics.approved-extract/v1";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovedExtractManifest {
    pub schema_version: String,
    pub records_sha256: String,
    pub batch: ImportBatchMetadata,
    pub rule_set_version: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExtractRecord {
    pub source: RawSourceRecord,
    pub validation: StagedValidationResult,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationReport {
    pub batch_id: String,
    pub accepted_record_count: u64,
    pub rejected_record_count: u64,
    pub distinct_entity_kind_count: u64,
    pub input_sha256: String,
    pub engine: String,
}

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error("unable to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid staged contract: {0}")]
    Contract(#[from] wellforge_storage::StorageError),
    #[error("manifest rule-set version must not be blank")]
    BlankRuleSetVersion,
    #[error("unsupported approved extract schema version: {schema_version}")]
    UnsupportedManifestSchema { schema_version: String },
    #[error("manifest records SHA-256 must be 64 lowercase hexadecimal characters")]
    InvalidRecordsSha256,
    #[error("records SHA-256 does not match manifest")]
    RecordsSha256Mismatch,
    #[error("record input exceeds byte limit of {limit} bytes")]
    RecordByteLimitExceeded { limit: u64 },
    #[error("record input exceeds record limit of {limit}")]
    RecordLimitExceeded { limit: u64 },
    #[error("record {source_record_key} does not match manifest batch {expected_batch_id}")]
    BatchMismatch {
        source_record_key: String,
        expected_batch_id: String,
    },
    #[error(
        "record {source_record_key} does not use manifest rule-set {expected_rule_set_version}"
    )]
    RuleSetMismatch {
        source_record_key: String,
        expected_rule_set_version: String,
    },
    #[error("record {source_record_key} is not accepted")]
    NonAcceptedRecord { source_record_key: String },
    #[error("record {source_record_key} is not provenance-bound to the manifest source")]
    ProvenanceMismatch { source_record_key: String },
    #[error("duplicate source record key: {source_record_key}")]
    DuplicateSourceRecordKey { source_record_key: String },
    #[error("record input contains no records")]
    EmptyInput,
    #[error("columnar reconciliation failed: {0}")]
    Polars(#[from] polars::error::PolarsError),
    #[error("in-memory reconciliation failed: {0}")]
    DuckDb(#[from] duckdb::Error),
}

pub fn reconcile_files(
    manifest_path: impl AsRef<Path>,
    records_path: impl AsRef<Path>,
) -> Result<ReconciliationReport, AnalyticsError> {
    let manifest_path = manifest_path.as_ref();
    let records_path = records_path.as_ref();
    let manifest_bytes = fs::read(manifest_path).map_err(|source| AnalyticsError::Read {
        path: manifest_path.to_owned(),
        source,
    })?;
    let manifest: ApprovedExtractManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|source| AnalyticsError::Json {
            path: manifest_path.to_owned(),
            source,
        })?;
    manifest.batch.validate()?;
    if manifest.rule_set_version.trim().is_empty() {
        return Err(AnalyticsError::BlankRuleSetVersion);
    }
    if manifest.schema_version != APPROVED_EXTRACT_SCHEMA_VERSION {
        return Err(AnalyticsError::UnsupportedManifestSchema {
            schema_version: manifest.schema_version,
        });
    }
    if !is_lowercase_sha256(&manifest.records_sha256) {
        return Err(AnalyticsError::InvalidRecordsSha256);
    }

    // Open once: limits, integrity validation, parsing and report hashing share these bytes.
    let record_bytes = fs::File::open(records_path)
        .and_then(|file| read_bounded(file, MAX_RECORD_BYTES))
        .map_err(|source| AnalyticsError::Read {
            path: records_path.to_owned(),
            source,
        })?;
    reconcile_snapshot(manifest, &manifest_bytes, records_path, &record_bytes)
}

// The internal limit is always below u64::MAX; retain one sentinel byte to detect overflow.
fn read_bounded(mut reader: impl Read, limit: u64) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.by_ref().take(limit + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn reconcile_snapshot(
    manifest: ApprovedExtractManifest,
    manifest_bytes: &[u8],
    records_path: &Path,
    record_bytes: &[u8],
) -> Result<ReconciliationReport, AnalyticsError> {
    if u64::try_from(record_bytes.len()).expect("record input size fits u64") > MAX_RECORD_BYTES {
        return Err(AnalyticsError::RecordByteLimitExceeded {
            limit: MAX_RECORD_BYTES,
        });
    }
    let lines = record_bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace));
    let mut record_count = 0_u64;
    for _ in lines.clone() {
        record_count += 1;
        if record_count > MAX_RECORDS {
            return Err(AnalyticsError::RecordLimitExceeded { limit: MAX_RECORDS });
        }
    }
    if format!("{:x}", Sha256::digest(record_bytes)) != manifest.records_sha256 {
        return Err(AnalyticsError::RecordsSha256Mismatch);
    }
    if record_count == 0 {
        return Err(AnalyticsError::EmptyInput);
    }

    let mut record_keys = Vec::new();
    let mut entity_kinds = Vec::new();
    let mut seen_keys = BTreeSet::new();
    for line in lines {
        let record: ExtractRecord =
            serde_json::from_slice(line).map_err(|source| AnalyticsError::Json {
                path: records_path.to_owned(),
                source,
            })?;
        validate_record(&manifest, &record)?;
        if !seen_keys.insert(record.source.source_record_key.clone()) {
            return Err(AnalyticsError::DuplicateSourceRecordKey {
                source_record_key: record.source.source_record_key,
            });
        }
        record_keys.push(record.source.source_record_key);
        entity_kinds.push(record.source.identity.source_entity_kind);
    }
    // Polars is limited to in-process columnar materialization. Nothing is written from it.
    let frame = DataFrame::new(vec![
        Series::new("source_record_key".into(), record_keys).into_column(),
        Series::new("source_entity_kind".into(), entity_kinds.clone()).into_column(),
    ])?;
    let accepted_record_count = u64::try_from(frame.height()).expect("usize fits in u64");
    let distinct_entity_kind_count =
        u64::try_from(entity_kinds.into_iter().collect::<BTreeSet<_>>().len())
            .expect("usize fits in u64");

    // DuckDB is deliberately ephemeral; `open_in_memory` forbids a database file from this path.
    let connection = Connection::open_in_memory()?;
    connection.execute(
        "CREATE TABLE reconciliation (accepted BIGINT NOT NULL, rejected BIGINT NOT NULL)",
        [],
    )?;
    connection.execute(
        "INSERT INTO reconciliation VALUES (?, ?)",
        params![
            i64::try_from(accepted_record_count).expect("count fits i64"),
            0_i64
        ],
    )?;
    let (accepted, rejected): (i64, i64) = connection.query_row(
        "SELECT sum(accepted), sum(rejected) FROM reconciliation",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    Ok(ReconciliationReport {
        batch_id: manifest.batch.batch_id,
        accepted_record_count: u64::try_from(accepted).expect("accepted count is non-negative"),
        rejected_record_count: u64::try_from(rejected).expect("rejected count is non-negative"),
        distinct_entity_kind_count,
        input_sha256: hex_digest(manifest_bytes, record_bytes),
        engine: "duckdb-in-memory+polars".to_owned(),
    })
}

fn validate_record(
    manifest: &ApprovedExtractManifest,
    record: &ExtractRecord,
) -> Result<(), AnalyticsError> {
    record.source.validate()?;
    record.validation.validate()?;
    let key = &record.source.source_record_key;
    if record.source.batch_id != manifest.batch.batch_id
        || record.validation.batch_id != manifest.batch.batch_id
    {
        return Err(AnalyticsError::BatchMismatch {
            source_record_key: key.clone(),
            expected_batch_id: manifest.batch.batch_id.clone(),
        });
    }
    if record.validation.source_record_key != *key {
        return Err(AnalyticsError::BatchMismatch {
            source_record_key: key.clone(),
            expected_batch_id: manifest.batch.batch_id.clone(),
        });
    }
    if record.validation.rule_set_version != manifest.rule_set_version {
        return Err(AnalyticsError::RuleSetMismatch {
            source_record_key: key.clone(),
            expected_rule_set_version: manifest.rule_set_version.clone(),
        });
    }
    if record.validation.disposition != ValidationDisposition::Accepted {
        return Err(AnalyticsError::NonAcceptedRecord {
            source_record_key: key.clone(),
        });
    }
    if record.source.identity.source_system != manifest.batch.source_system
        || record.source.identity.source_checksum != manifest.batch.source_checksum
    {
        return Err(AnalyticsError::ProvenanceMismatch {
            source_record_key: key.clone(),
        });
    }
    Ok(())
}

fn hex_digest(manifest_bytes: &[u8], record_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(manifest_bytes);
    hasher.update([0]);
    hasher.update(record_bytes);
    format!("{:x}", hasher.finalize())
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde_json::json;

    use super::*;

    #[test]
    fn bounded_reader_keeps_only_the_limit_and_one_sentinel_byte() {
        for (input, limit, expected) in [
            ("", 4, ""),
            ("abc", 4, "abc"),
            ("abcd", 4, "abcd"),
            ("abcde", 4, "abcde"),
            ("abcdefghi", 4, "abcde"),
            ("abc", 0, "a"),
        ] {
            let mut reader = Cursor::new(input.as_bytes());
            let bytes = read_bounded(&mut reader, limit).unwrap();
            assert_eq!(bytes, expected.as_bytes(), "input={input:?}, limit={limit}");
            assert_eq!(reader.position(), expected.len() as u64);
        }
    }

    #[test]
    fn captured_snapshot_survives_original_file_changes_and_removal() {
        let temporary = tempfile::tempdir().unwrap();
        let records_path = temporary.path().join("records.jsonl");
        let checksum = format!("sha256:{}", "0".repeat(64));
        let record_bytes = serde_json::to_vec(&json!({
            "source": {
                "batchId": "batch-snapshot", "sourceRecordKey": "original",
                "identity": {
                    "sourceSystem": "synthetic", "sourceEntityKind": "survey_station",
                    "sourceEntityId": "original", "sourceChecksum": checksum
                },
                "payload": {"mdM": 123.4}
            },
            "validation": {
                "batchId": "batch-snapshot", "sourceRecordKey": "original",
                "disposition": "accepted", "ruleSetVersion": "2026.08",
                "validatedAt": "2026-08-25T12:01:00Z", "findings": []
            }
        }))
        .unwrap();
        let manifest_bytes = serde_json::to_vec(&json!({
            "schemaVersion": "wellforge.analytics.approved-extract/v1",
            "recordsSha256": format!("{:x}", Sha256::digest(&record_bytes)),
            "batch": {
                "batchId": "batch-snapshot", "sourceSystem": "synthetic",
                "sourceLocation": "synthetic://snapshot", "sourceChecksum": checksum,
                "extractedAt": "2026-08-25T12:00:00Z", "operatorId": "fixture-runner"
            },
            "ruleSetVersion": "2026.08"
        }))
        .unwrap();
        let manifest: ApprovedExtractManifest = serde_json::from_slice(&manifest_bytes).unwrap();
        fs::write(&records_path, &record_bytes).unwrap();
        let snapshot =
            read_bounded(fs::File::open(&records_path).unwrap(), MAX_RECORD_BYTES).unwrap();
        let original =
            reconcile_snapshot(manifest.clone(), &manifest_bytes, &records_path, &snapshot)
                .unwrap();
        assert_eq!(original.accepted_record_count, 1);
        assert_eq!(original.batch_id, "batch-snapshot");

        fs::write(&records_path, b"not-json: changed after capture").unwrap();
        let changed =
            reconcile_snapshot(manifest.clone(), &manifest_bytes, &records_path, &snapshot)
                .unwrap();
        assert_eq!(changed, original);

        fs::remove_file(&records_path).unwrap();
        let removed =
            reconcile_snapshot(manifest, &manifest_bytes, &records_path, &snapshot).unwrap();
        assert_eq!(removed, original);
    }
}
