//! Embedded SQLite storage contracts and the local authority boundary.
//!
//! SQLite connections and migration statements remain private. Callers can persist only typed
//! records, preventing frontend code from supplying arbitrary SQL or connection details.

use std::{path::Path, time::Duration};

use rusqlite::{Connection, ErrorCode, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use wellforge_core::CalculationReceipt;

const SQLITE_MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_local_authority",
        include_str!("../migrations/sqlite/0001_local_authority.sql"),
    ),
    (
        "0002_revision_event_heads",
        include_str!("../migrations/sqlite/0002_revision_event_heads.sql"),
    ),
];

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum StorageError {
    #[error("a duplicate authority record already exists")]
    Duplicate,
    #[error("an authority record relationship is invalid")]
    InvalidRelationship,
    #[error("SQLite is busy; retry the operation")]
    Busy,
    #[error("SQLite storage is unavailable")]
    Unavailable,
    #[error("SQLite runtime settings could not be applied")]
    Configuration,
    #[error("applied migration checksum does not match {version}")]
    MigrationChecksumMismatch { version: String },
    #[error("applied migrations are not in their expected order")]
    MigrationOrderMismatch,
    #[error("identifier must not be empty: {field}")]
    InvalidIdentifier { field: &'static str },
    #[error("project name must not be empty")]
    InvalidProjectName,
    #[error("content SHA-256 does not match the supplied bytes")]
    ContentHashMismatch,
    #[error("calculation engine version must not be empty")]
    InvalidEngineVersion,
    #[error("journal sequence must be positive")]
    InvalidJournalSequence,
    #[error("journal event type must not be empty")]
    InvalidJournalEventType,
    #[error("journal payload must be a JSON object")]
    InvalidJournalPayload,
    #[error("artifact hash must be sha256: followed by a 64-character hexadecimal digest")]
    InvalidArtifactHash,
    #[error("parent revision ID must not be empty when supplied")]
    InvalidParentRevisionId,
    #[error("idempotency key must not be empty")]
    InvalidIdempotencyKey,
    #[error("actor ID must not be empty")]
    InvalidActorId,
    #[error("occurred timestamp must be an RFC 3339 UTC timestamp")]
    InvalidOccurredAt,
    #[error("import batch ID must not be empty")]
    InvalidImportBatchId,
    #[error("source system must not be empty")]
    InvalidSourceSystem,
    #[error("source location must not be empty")]
    InvalidSourceLocation,
    #[error("source checksum must be sha256: followed by a 64-character hexadecimal digest")]
    InvalidSourceChecksum,
    #[error("operator ID must not be empty")]
    InvalidOperatorId,
    #[error("source record key must not be empty")]
    InvalidSourceRecordKey,
    #[error("source entity kind must not be empty")]
    InvalidSourceEntityKind,
    #[error("source entity ID must not be empty")]
    InvalidSourceEntityId,
    #[error("source payload must be a JSON object")]
    InvalidSourcePayload,
    #[error("validation rule-set version must not be empty")]
    InvalidValidationRuleSetVersion,
    #[error("validation timestamp must be an RFC 3339 UTC timestamp")]
    InvalidValidatedAt,
    #[error("validation finding code must not be empty")]
    InvalidValidationFindingCode,
    #[error("validation finding message must not be empty")]
    InvalidValidationFindingMessage,
    #[error("rejected or review-required records must include at least one validation finding")]
    MissingValidationFinding,
    #[error("calculation receipt output or project provenance is invalid")]
    InvalidCalculationReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProject {
    pub id: String,
    pub name: String,
    pub created_at: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRevisionInput {
    pub id: String,
    pub project_id: String,
    pub parent_revision_id: Option<String>,
    pub content: Vec<u8>,
    pub content_sha256: String,
    pub created_at: String,
    pub actor_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProjectRevision {
    pub id: String,
    pub project_id: String,
    pub parent_revision_id: Option<String>,
    pub content_sha256: String,
}
/// Metadata-only representation of an immutable project revision for audit display.
///
/// Project content stays inside the local authority boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProjectRevisionAudit {
    pub id: String,
    pub parent_revision_id: Option<String>,
    pub content_sha256: String,
    pub created_at: String,
    pub actor_id: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct CoreCalculationReceiptInput {
    pub id: String,
    pub project_revision_id: String,
    pub receipt: CalculationReceipt,
    pub output: Value,
    pub recorded_at: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredCoreCalculationReceipt {
    pub id: String,
    pub project_revision_id: String,
    pub receipt: CalculationReceipt,
    pub recorded_at: String,
}
/// Metadata-only representation of a persisted calculation receipt for audit display.
///
/// Receipt bytes, calculation output, and any local paths remain private to storage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredCalculationReceiptAudit {
    pub id: String,
    pub project_revision_id: String,
    pub project_revision_content_sha256: String,
    pub content_sha256: String,
    pub recorded_at: String,
    pub algorithm: String,
    pub algorithm_version: String,
    pub actor_id: String,
    pub output_sha256: String,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayEventInput {
    pub id: String,
    pub project_id: String,
    pub project_revision_id: Option<String>,
    pub sequence: i64,
    pub event_type: String,
    pub payload: Value,
    pub occurred_at: String,
    pub actor_id: String,
}

/// Typed, embedded local authority storage. The connection is deliberately private.
#[derive(Debug)]
pub struct LocalStore {
    connection: Connection,
}

impl LocalStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Self::from_connection(Connection::open(path).map_err(map_sqlite_error)?, true)
    }
    #[cfg(test)]
    fn open_in_memory_for_test() -> Result<Self, StorageError> {
        Self::from_connection(
            Connection::open_in_memory().map_err(map_sqlite_error)?,
            false,
        )
    }
    fn from_connection(
        mut connection: Connection,
        require_wal: bool,
    ) -> Result<Self, StorageError> {
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(map_sqlite_error)?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(map_sqlite_error)?;
        let foreign_keys: i64 = connection
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .map_err(map_sqlite_error)?;
        let journal_mode: String = connection
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .map_err(map_sqlite_error)?;
        if foreign_keys != 1 || (require_wal && !journal_mode.eq_ignore_ascii_case("wal")) {
            return Err(StorageError::Configuration);
        }
        migrate(&mut connection)?;
        Ok(Self { connection })
    }
    pub fn create_project(&mut self, project: NewProject) -> Result<(), StorageError> {
        require_identifier(&project.id, "project.id")?;
        if project.name.trim().is_empty() {
            return Err(StorageError::InvalidProjectName);
        }
        validate_utc_timestamp(&project.created_at, StorageError::InvalidOccurredAt)?;
        self.connection
            .execute(
                "INSERT INTO project (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![project.id, project.name, project.created_at],
            )
            .map_err(map_sqlite_error)?;
        Ok(())
    }
    pub fn append_project_revision(
        &mut self,
        revision: ProjectRevisionInput,
    ) -> Result<(), StorageError> {
        require_identifier(&revision.id, "project_revision.id")?;
        require_identifier(&revision.project_id, "project_revision.project_id")?;
        validate_optional_parent(&revision.parent_revision_id)?;
        require_identifier(&revision.actor_id, "project_revision.actor_id")?;
        validate_utc_timestamp(&revision.created_at, StorageError::InvalidOccurredAt)?;
        verify_content_hash(&revision.content, &revision.content_sha256)?;
        if revision.parent_revision_id.as_deref() == Some(&revision.id) {
            return Err(StorageError::InvalidRelationship);
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(map_sqlite_error)?;
        let project_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1)",
                [&revision.project_id],
                |row| row.get(0),
            )
            .map_err(map_sqlite_error)?;
        if !project_exists {
            return Err(StorageError::InvalidRelationship);
        }
        let current_head = transaction
            .query_row(
                "SELECT revision_id FROM project_head WHERE project_id = ?1",
                [&revision.project_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(map_sqlite_error)?;
        if revision.parent_revision_id.as_ref() != current_head.as_ref() {
            return Err(StorageError::InvalidRelationship);
        }
        transaction.execute("INSERT INTO project_revision (id, project_id, parent_revision_id, content, content_sha256, created_at, actor_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![revision.id, revision.project_id, revision.parent_revision_id, revision.content, revision.content_sha256, revision.created_at, revision.actor_id]).map_err(map_sqlite_error)?;
        transaction.commit().map_err(map_sqlite_error)
    }
    pub fn latest_project_revision(
        &self,
        project_id: &str,
    ) -> Result<Option<StoredProjectRevision>, StorageError> {
        require_identifier(project_id, "project_revision.project_id")?;
        self.connection
            .query_row(
                "SELECT revision.id, revision.project_id, revision.parent_revision_id, revision.content_sha256 FROM project_head AS head JOIN project_revision AS revision ON revision.id = head.revision_id WHERE head.project_id = ?1",
                [project_id],
                |row| {
                    Ok(StoredProjectRevision {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        parent_revision_id: row.get(2)?,
                        content_sha256: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(map_sqlite_error)
    }
    /// Lists immutable revision metadata for one project, newest first.
    pub fn list_project_revision_audits(
        &self,
        project_id: &str,
    ) -> Result<Vec<StoredProjectRevisionAudit>, StorageError> {
        require_identifier(project_id, "project_revision.project_id")?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, parent_revision_id, content_sha256, created_at, actor_id, rowid \
                 FROM project_revision WHERE project_id = ?1",
            )
            .map_err(map_sqlite_error)?;
        let mut audits = statement
            .query_map([project_id], |row| {
                Ok((
                    StoredProjectRevisionAudit {
                        id: row.get(0)?,
                        parent_revision_id: row.get(1)?,
                        content_sha256: row.get(2)?,
                        created_at: row.get(3)?,
                        actor_id: row.get(4)?,
                    },
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        let mut ordered = audits
            .drain(..)
            .map(|(audit, rowid)| {
                Ok((
                    parse_utc_timestamp(&audit.created_at, StorageError::InvalidOccurredAt)?,
                    rowid,
                    audit,
                ))
            })
            .collect::<Result<Vec<_>, StorageError>>()?;
        ordered.sort_by(|(left_at, left_rowid, _), (right_at, right_rowid, _)| {
            right_at
                .cmp(left_at)
                .then_with(|| right_rowid.cmp(left_rowid))
        });
        Ok(ordered.into_iter().map(|(_, _, audit)| audit).collect())
    }
    pub fn record_core_calculation_receipt(
        &mut self,
        input: CoreCalculationReceiptInput,
    ) -> Result<(), StorageError> {
        require_identifier(&input.id, "calculation_receipt.id")?;
        input
            .receipt
            .verifies_output(&input.output)
            .map_err(|_| StorageError::InvalidCalculationReceipt)?
            .then_some(())
            .ok_or(StorageError::InvalidCalculationReceipt)?;
        validate_core_receipt_binding(
            &self.connection,
            &input.project_revision_id,
            &input.receipt,
        )?;
        let content = serde_json::to_vec(&input.receipt)
            .map_err(|_| StorageError::InvalidCalculationReceipt)?;
        let hash = format!("sha256:{:x}", Sha256::digest(&content));
        validate_utc_timestamp(&input.recorded_at, StorageError::InvalidOccurredAt)?;
        self.connection.execute("INSERT INTO calculation_receipt (id, project_revision_id, engine_version, content, content_sha256, recorded_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![input.id, input.project_revision_id, input.receipt.algorithm_version(), content, hash, input.recorded_at]).map_err(map_sqlite_error)?;
        Ok(())
    }
    pub fn load_core_calculation_receipt(
        &self,
        id: &str,
    ) -> Result<Option<StoredCoreCalculationReceipt>, StorageError> {
        require_identifier(id, "calculation_receipt.id")?;
        let stored = self
            .connection
            .query_row(
                "SELECT project_revision_id, engine_version, content, content_sha256, recorded_at FROM calculation_receipt WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(map_sqlite_error)?;
        let Some((project_revision_id, engine_version, content, content_sha256, recorded_at)) =
            stored
        else {
            return Ok(None);
        };
        verify_content_hash(&content, &content_sha256)?;
        let receipt: CalculationReceipt = serde_json::from_slice(&content)
            .map_err(|_| StorageError::InvalidCalculationReceipt)?;
        if receipt.algorithm_version() != engine_version {
            return Err(StorageError::InvalidCalculationReceipt);
        }
        validate_core_receipt_binding(&self.connection, &project_revision_id, &receipt)?;
        Ok(Some(StoredCoreCalculationReceipt {
            id: id.to_owned(),
            project_revision_id,
            receipt,
            recorded_at,
        }))
    }
    pub fn latest_core_calculation_receipt_for_revision(
        &self,
        project_revision_id: &str,
    ) -> Result<Option<StoredCoreCalculationReceipt>, StorageError> {
        require_identifier(
            project_revision_id,
            "calculation_receipt.project_revision_id",
        )?;
        let id = self
            .connection
            .query_row(
                "SELECT id FROM calculation_receipt WHERE project_revision_id = ?1 ORDER BY rowid DESC LIMIT 1",
                [project_revision_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(map_sqlite_error)?;
        match id {
            Some(id) => self.load_core_calculation_receipt(&id),
            None => Ok(None),
        }
    }
    /// Lists verified, metadata-only receipt summaries for one project's revisions, newest first.
    pub fn list_calculation_receipt_audits(
        &self,
        project_id: &str,
    ) -> Result<Vec<StoredCalculationReceiptAudit>, StorageError> {
        require_identifier(project_id, "project_revision.project_id")?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT receipt.id, receipt.project_revision_id, revision.content_sha256, \
                        receipt.engine_version, receipt.content, receipt.content_sha256, receipt.recorded_at, receipt.rowid \
                 FROM calculation_receipt AS receipt \
                 JOIN project_revision AS revision ON revision.id = receipt.project_revision_id \
                 WHERE revision.project_id = ?1",
            )
            .map_err(map_sqlite_error)?;
        let stored = statement
            .query_map([project_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;

        let mut audits = stored
            .into_iter()
            .map(
                |(
                    id,
                    project_revision_id,
                    project_revision_content_sha256,
                    engine_version,
                    content,
                    content_sha256,
                    recorded_at,
                    rowid,
                )| {
                    verify_content_hash(&content, &content_sha256)?;
                    let receipt: CalculationReceipt = serde_json::from_slice(&content)
                        .map_err(|_| StorageError::InvalidCalculationReceipt)?;
                    if receipt.algorithm_version() != engine_version {
                        return Err(StorageError::InvalidCalculationReceipt);
                    }
                    validate_core_receipt_binding(
                        &self.connection,
                        &project_revision_id,
                        &receipt,
                    )?;
                    Ok((
                        StoredCalculationReceiptAudit {
                            id,
                            project_revision_id,
                            project_revision_content_sha256,
                            content_sha256,
                            recorded_at,
                            algorithm: receipt.algorithm().to_owned(),
                            algorithm_version: receipt.algorithm_version().to_owned(),
                            actor_id: receipt.context().actor_id.clone(),
                            output_sha256: receipt.output_sha256().to_owned(),
                            warnings: receipt.context().warnings.clone(),
                        },
                        rowid,
                    ))
                },
            )
            .collect::<Result<Vec<_>, StorageError>>()?;
        let mut ordered = audits
            .drain(..)
            .map(|(audit, rowid)| {
                Ok((
                    parse_utc_timestamp(&audit.recorded_at, StorageError::InvalidOccurredAt)?,
                    rowid,
                    audit,
                ))
            })
            .collect::<Result<Vec<_>, StorageError>>()?;
        ordered.sort_by(|(left_at, left_rowid, _), (right_at, right_rowid, _)| {
            right_at
                .cmp(left_at)
                .then_with(|| right_rowid.cmp(left_rowid))
        });
        Ok(ordered.into_iter().map(|(_, _, audit)| audit).collect())
    }
    pub fn append_replay_event(&mut self, event: ReplayEventInput) -> Result<(), StorageError> {
        require_identifier(&event.id, "replay_journal_event.id")?;
        require_identifier(&event.project_id, "replay_journal_event.project_id")?;
        if event
            .project_revision_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(StorageError::InvalidParentRevisionId);
        }
        if event.sequence <= 0 {
            return Err(StorageError::InvalidJournalSequence);
        }
        if event.event_type.trim().is_empty() {
            return Err(StorageError::InvalidJournalEventType);
        }
        if !event.payload.is_object() {
            return Err(StorageError::InvalidJournalPayload);
        }
        require_identifier(&event.actor_id, "replay_journal_event.actor_id")?;
        validate_utc_timestamp(&event.occurred_at, StorageError::InvalidOccurredAt)?;
        if let Some(revision_id) = &event.project_revision_id {
            let revision_project: String = self
                .connection
                .query_row(
                    "SELECT project_id FROM project_revision WHERE id = ?1",
                    [revision_id],
                    |row| row.get(0),
                )
                .map_err(|_| StorageError::InvalidRelationship)?;
            if revision_project != event.project_id {
                return Err(StorageError::InvalidRelationship);
            }
        }
        let payload = serde_json::to_string(&event.payload)
            .map_err(|_| StorageError::InvalidJournalPayload)?;
        self.connection.execute("INSERT INTO replay_journal_event (id, project_id, project_revision_id, sequence, event_type, payload_json, occurred_at, actor_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![event.id, event.project_id, event.project_revision_id, event.sequence, event.event_type, payload, event.occurred_at, event.actor_id]).map_err(map_sqlite_error)?;
        Ok(())
    }
    pub fn enqueue_sync_envelope(&mut self, envelope: SyncEnvelope) -> Result<(), StorageError> {
        envelope.validate()?;
        validate_authority_hash(&envelope.artifact_hash, StorageError::InvalidArtifactHash)?;
        self.connection.execute("INSERT INTO sync_envelope (idempotency_key, artifact_hash, parent_revision_id, actor_id, occurred_at, operation) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![envelope.idempotency_key, envelope.artifact_hash, envelope.parent_revision_id, envelope.actor_id, envelope.occurred_at, envelope.operation.as_str()]).map_err(map_sqlite_error)?;
        Ok(())
    }
    pub fn project_count(&self) -> Result<u64, StorageError> {
        self.count("project")
    }
    pub fn project_revision_count(&self) -> Result<u64, StorageError> {
        self.count("project_revision")
    }
    pub fn calculation_receipt_count(&self) -> Result<u64, StorageError> {
        self.count("calculation_receipt")
    }
    pub fn replay_event_count(&self) -> Result<u64, StorageError> {
        self.count("replay_journal_event")
    }
    pub fn sync_envelope_count(&self) -> Result<u64, StorageError> {
        self.count("sync_envelope")
    }
    fn count(&self, table: &'static str) -> Result<u64, StorageError> {
        self.connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .map_err(map_sqlite_error)
    }
}

fn migrate(connection: &mut Connection) -> Result<(), StorageError> {
    migrate_with(connection, SQLITE_MIGRATIONS)
}

fn migrate_with(
    connection: &mut Connection,
    migrations: &[(&str, &str)],
) -> Result<(), StorageError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_sqlite_error)?;
    transaction.execute_batch("CREATE TABLE IF NOT EXISTS schema_migration (ordinal INTEGER PRIMARY KEY NOT NULL, version TEXT NOT NULL UNIQUE, checksum TEXT NOT NULL, applied_at TEXT NOT NULL) STRICT;").map_err(map_sqlite_error)?;
    let applied = {
        let mut statement = transaction
            .prepare("SELECT ordinal, version, checksum FROM schema_migration ORDER BY ordinal")
            .map_err(map_sqlite_error)?;
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?
    };
    if applied.len() > migrations.len() {
        return Err(StorageError::MigrationOrderMismatch);
    }
    for (expected_ordinal, (stored_ordinal, stored_version, stored_checksum)) in
        applied.iter().enumerate()
    {
        let (expected_version, expected_statement) = migrations[expected_ordinal];
        if *stored_ordinal != expected_ordinal as i64 || stored_version != expected_version {
            return Err(StorageError::MigrationOrderMismatch);
        }
        if stored_checksum != &migration_checksum(expected_statement) {
            return Err(StorageError::MigrationChecksumMismatch {
                version: expected_version.to_owned(),
            });
        }
    }
    for (ordinal, (version, statement)) in migrations.iter().enumerate().skip(applied.len()) {
        transaction
            .execute_batch(statement)
            .map_err(map_sqlite_error)?;
        transaction.execute("INSERT INTO schema_migration (ordinal, version, checksum, applied_at) VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))", params![ordinal as i64, version, migration_checksum(statement)]).map_err(map_sqlite_error)?;
    }
    transaction.commit().map_err(map_sqlite_error)
}

fn migration_checksum(statement: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(statement.as_bytes()))
}
fn map_sqlite_error(error: rusqlite::Error) -> StorageError {
    if matches!(&error, rusqlite::Error::SqliteFailure(code, _) if matches!(code.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked))
    {
        StorageError::Busy
    } else if matches!(&error, rusqlite::Error::SqliteFailure(code, _) if code.code == ErrorCode::ConstraintViolation)
    {
        StorageError::Duplicate
    } else {
        StorageError::Unavailable
    }
}
fn validate_core_receipt_binding(
    connection: &Connection,
    project_revision_id: &str,
    receipt: &CalculationReceipt,
) -> Result<(), StorageError> {
    require_identifier(
        project_revision_id,
        "calculation_receipt.project_revision_id",
    )?;
    let revision_hash: String = connection
        .query_row(
            "SELECT content_sha256 FROM project_revision WHERE id = ?1",
            [project_revision_id],
            |row| row.get(0),
        )
        .map_err(|_| StorageError::InvalidRelationship)?;
    let revision_digest = revision_hash
        .strip_prefix("sha256:")
        .ok_or(StorageError::InvalidCalculationReceipt)?;
    if receipt.input_revisions().iter().any(|item| {
        item.kind == "project_revision"
            && item.id == project_revision_id
            && item.content_sha256 == revision_digest
    }) {
        Ok(())
    } else {
        Err(StorageError::InvalidCalculationReceipt)
    }
}
fn require_identifier(value: &str, field: &'static str) -> Result<(), StorageError> {
    if value.trim().is_empty() {
        Err(StorageError::InvalidIdentifier { field })
    } else {
        Ok(())
    }
}
fn verify_content_hash(content: &[u8], expected: &str) -> Result<(), StorageError> {
    validate_authority_hash(expected, StorageError::ContentHashMismatch)?;
    if format!("sha256:{:x}", Sha256::digest(content)) == expected {
        Ok(())
    } else {
        Err(StorageError::ContentHashMismatch)
    }
}
fn validate_authority_hash(value: &str, error: StorageError) -> Result<(), StorageError> {
    let valid = value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
    });
    if valid { Ok(()) } else { Err(error) }
}
fn validate_optional_parent(value: &Option<String>) -> Result<(), StorageError> {
    if value
        .as_deref()
        .is_some_and(|parent| parent.trim().is_empty())
    {
        Err(StorageError::InvalidParentRevisionId)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncOperation {
    Create,
    Upsert,
    Delete,
}
impl SyncOperation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Upsert => "upsert",
            Self::Delete => "delete",
        }
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEnvelope {
    pub artifact_hash: String,
    pub parent_revision_id: Option<String>,
    pub idempotency_key: String,
    pub actor_id: String,
    pub occurred_at: String,
    pub operation: SyncOperation,
}
impl SyncEnvelope {
    pub fn validate(&self) -> Result<(), StorageError> {
        validate_checksum(&self.artifact_hash, StorageError::InvalidArtifactHash)?;
        validate_optional_parent(&self.parent_revision_id)?;
        require_nonblank(&self.idempotency_key, StorageError::InvalidIdempotencyKey)?;
        require_nonblank(&self.actor_id, StorageError::InvalidActorId)?;
        validate_utc_timestamp(&self.occurred_at, StorageError::InvalidOccurredAt)
    }
}

/// Staged-import types remain validation-only and are not local authority tables.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBatchMetadata {
    pub batch_id: String,
    pub source_system: String,
    pub source_location: String,
    pub source_checksum: String,
    pub extracted_at: String,
    pub operator_id: String,
}
impl ImportBatchMetadata {
    pub fn validate(&self) -> Result<(), StorageError> {
        require_nonblank(&self.batch_id, StorageError::InvalidImportBatchId)?;
        require_nonblank(&self.source_system, StorageError::InvalidSourceSystem)?;
        require_nonblank(&self.source_location, StorageError::InvalidSourceLocation)?;
        validate_checksum(&self.source_checksum, StorageError::InvalidSourceChecksum)?;
        validate_utc_timestamp(&self.extracted_at, StorageError::InvalidOccurredAt)?;
        require_nonblank(&self.operator_id, StorageError::InvalidOperatorId)
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceIdentityMapping {
    pub source_system: String,
    pub source_entity_kind: String,
    pub source_entity_id: String,
    pub source_checksum: String,
}
impl SourceIdentityMapping {
    pub fn validate(&self) -> Result<(), StorageError> {
        require_nonblank(&self.source_system, StorageError::InvalidSourceSystem)?;
        require_nonblank(
            &self.source_entity_kind,
            StorageError::InvalidSourceEntityKind,
        )?;
        require_nonblank(&self.source_entity_id, StorageError::InvalidSourceEntityId)?;
        validate_checksum(&self.source_checksum, StorageError::InvalidSourceChecksum)
    }
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSourceRecord {
    pub batch_id: String,
    pub source_record_key: String,
    pub identity: SourceIdentityMapping,
    pub payload: Value,
}
impl RawSourceRecord {
    pub fn validate(&self) -> Result<(), StorageError> {
        require_nonblank(&self.batch_id, StorageError::InvalidImportBatchId)?;
        require_nonblank(
            &self.source_record_key,
            StorageError::InvalidSourceRecordKey,
        )?;
        self.identity.validate()?;
        if self.payload.is_object() {
            Ok(())
        } else {
            Err(StorageError::InvalidSourcePayload)
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationDisposition {
    Accepted,
    Rejected,
    NeedsReview,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationFinding {
    pub code: String,
    pub message: String,
}
impl ValidationFinding {
    fn validate(&self) -> Result<(), StorageError> {
        require_nonblank(&self.code, StorageError::InvalidValidationFindingCode)?;
        require_nonblank(&self.message, StorageError::InvalidValidationFindingMessage)
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StagedValidationResult {
    pub batch_id: String,
    pub source_record_key: String,
    pub disposition: ValidationDisposition,
    pub rule_set_version: String,
    pub validated_at: String,
    pub findings: Vec<ValidationFinding>,
}
impl StagedValidationResult {
    pub fn validate(&self) -> Result<(), StorageError> {
        require_nonblank(&self.batch_id, StorageError::InvalidImportBatchId)?;
        require_nonblank(
            &self.source_record_key,
            StorageError::InvalidSourceRecordKey,
        )?;
        require_nonblank(
            &self.rule_set_version,
            StorageError::InvalidValidationRuleSetVersion,
        )?;
        validate_utc_timestamp(&self.validated_at, StorageError::InvalidValidatedAt)?;
        self.findings
            .iter()
            .try_for_each(ValidationFinding::validate)?;
        if !matches!(self.disposition, ValidationDisposition::Accepted) && self.findings.is_empty()
        {
            Err(StorageError::MissingValidationFinding)
        } else {
            Ok(())
        }
    }
}
fn require_nonblank(value: &str, error: StorageError) -> Result<(), StorageError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}
fn validate_checksum(value: &str, error: StorageError) -> Result<(), StorageError> {
    let valid = value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    });
    if valid { Ok(()) } else { Err(error) }
}
fn parse_utc_timestamp(value: &str, error: StorageError) -> Result<OffsetDateTime, StorageError> {
    let timestamp = OffsetDateTime::parse(value, &Rfc3339).map_err(|_| error.clone())?;
    if timestamp.offset().is_utc() {
        Ok(timestamp)
    } else {
        Err(error)
    }
}
fn validate_utc_timestamp(value: &str, error: StorageError) -> Result<(), StorageError> {
    parse_utc_timestamp(value, error).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temporary_database_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("wellforge-{label}-{nonce}.sqlite3"))
    }

    fn remove_database_files(path: &Path) {
        for candidate in [
            path.to_path_buf(),
            PathBuf::from(format!("{}-wal", path.display())),
            PathBuf::from(format!("{}-shm", path.display())),
        ] {
            if candidate.exists() {
                fs::remove_file(candidate).expect("temporary database file removes");
            }
        }
    }

    #[test]
    fn each_file_connection_uses_wal_and_a_five_second_busy_timeout() {
        let path = temporary_database_path("connection-settings");
        let first = LocalStore::open(&path).expect("first file connection opens");
        let second = LocalStore::open(&path).expect("second file connection opens");

        for store in [&first, &second] {
            let journal_mode: String = store
                .connection
                .pragma_query_value(None, "journal_mode", |row| row.get(0))
                .expect("journal mode is readable");
            let busy_timeout_ms: i64 = store
                .connection
                .pragma_query_value(None, "busy_timeout", |row| row.get(0))
                .expect("busy timeout is readable");
            assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
            assert_eq!(busy_timeout_ms, 5_000);
        }

        drop(second);
        drop(first);
        remove_database_files(&path);
    }

    #[test]
    fn migration_rejects_checksum_and_ordinal_tampering() {
        let path = temporary_database_path("migration-tampering");
        drop(LocalStore::open(&path).expect("database migrates"));
        let connection = Connection::open(&path).expect("raw database opens");
        connection
            .execute(
                "UPDATE schema_migration SET checksum = 'sha256:tampered' WHERE version = '0001_local_authority'",
                [],
            )
            .expect("checksum is tampered");
        drop(connection);
        assert!(matches!(
            LocalStore::open(&path),
            Err(StorageError::MigrationChecksumMismatch { .. })
        ));

        let connection = Connection::open(&path).expect("raw database reopens");
        connection
            .execute(
                "UPDATE schema_migration SET checksum = ?1, ordinal = 9 WHERE version = '0001_local_authority'",
                [migration_checksum(SQLITE_MIGRATIONS[0].1)],
            )
            .expect("ordinal is tampered");
        drop(connection);
        assert_eq!(
            LocalStore::open(&path).expect_err("ordinal tampering is rejected"),
            StorageError::MigrationOrderMismatch
        );
        remove_database_files(&path);
    }

    #[test]
    fn migration_rejects_a_reordered_or_extended_applied_history() {
        let mut connection = Connection::open_in_memory().expect("database opens");
        let migrations = &[
            (
                "0001_first",
                "CREATE TABLE first_table (id INTEGER PRIMARY KEY) STRICT;",
            ),
            (
                "0002_second",
                "CREATE TABLE second_table (id INTEGER PRIMARY KEY) STRICT;",
            ),
        ];
        migrate_with(&mut connection, migrations).expect("ordered migrations apply");

        let reordered = [migrations[1], migrations[0]];
        assert_eq!(
            migrate_with(&mut connection, &reordered)
                .expect_err("reordered migrations are rejected"),
            StorageError::MigrationOrderMismatch
        );
        connection
            .execute(
                "INSERT INTO schema_migration (ordinal, version, checksum, applied_at) VALUES (2, '9999_unknown', 'sha256:unknown', '2026-08-25T00:00:00Z')",
                [],
            )
            .expect("unknown applied migration is inserted");
        assert_eq!(
            migrate_with(&mut connection, migrations)
                .expect_err("an extended applied history is rejected"),
            StorageError::MigrationOrderMismatch
        );
    }

    #[test]
    fn a_failed_migration_rolls_back_the_entire_batch() {
        let mut connection = Connection::open_in_memory().expect("database opens");
        let migrations = &[
            (
                "0001_first",
                "CREATE TABLE first_table (id INTEGER PRIMARY KEY) STRICT;",
            ),
            ("0002_broken", "THIS IS NOT VALID SQL;"),
        ];

        assert_eq!(
            migrate_with(&mut connection, migrations).expect_err("invalid migration fails"),
            StorageError::Unavailable
        );
        let first_table_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'first_table')",
                [],
                |row| row.get(0),
            )
            .expect("schema is readable");
        let migration_table_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_migration')",
                [],
                |row| row.get(0),
            )
            .expect("schema is readable");
        assert!(!first_table_exists);
        assert!(!migration_table_exists);
    }

    #[test]
    fn revision_event_migration_preserves_existing_data_and_seeds_the_head() {
        let mut connection = Connection::open_in_memory().expect("database opens");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("foreign keys enable");
        migrate_with(&mut connection, &SQLITE_MIGRATIONS[..1])
            .expect("foundation migration applies");
        connection
            .execute(
                "INSERT INTO project (id, name, created_at) VALUES ('project', 'Project', '2026-08-25T00:00:00Z')",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO project_revision (id, project_id, parent_revision_id, content, content_sha256, created_at, actor_id) VALUES ('legacy-revision', 'project', NULL, ?1, ?2, '2026-08-25T00:00:01Z', 'actor')",
                params![b"A", format!("sha256:{:x}", Sha256::digest(b"A"))],
            )
            .unwrap();

        migrate_with(&mut connection, SQLITE_MIGRATIONS).expect("head migration applies");
        let mut store = LocalStore { connection };
        let head = store
            .latest_project_revision("project")
            .unwrap()
            .expect("migrated head exists");
        assert_eq!(head.id, "legacy-revision");
        store
            .append_project_revision(ProjectRevisionInput {
                id: "revert-event".into(),
                project_id: "project".into(),
                parent_revision_id: Some("legacy-revision".into()),
                content: b"A".to_vec(),
                content_sha256: format!("sha256:{:x}", Sha256::digest(b"A")),
                created_at: "2026-08-25T00:00:02Z".into(),
                actor_id: "actor".into(),
            })
            .expect("repeat content appends as a new event after migration");
        assert_eq!(store.project_revision_count().unwrap(), 2);
    }

    #[test]
    fn sqlite_append_only_triggers_reject_updates_and_deletes_for_every_authority_record() {
        let mut store = LocalStore::open_in_memory_for_test().unwrap();
        store
            .create_project(NewProject {
                id: "project".into(),
                name: "Project".into(),
                created_at: "2026-08-25T00:00:00Z".into(),
            })
            .unwrap();
        let content = b"authority".to_vec();
        let hash = format!("sha256:{:x}", Sha256::digest(&content));
        let revision_digest = hash[7..].to_owned();
        store
            .append_project_revision(ProjectRevisionInput {
                id: "revision".into(),
                project_id: "project".into(),
                parent_revision_id: None,
                content,
                content_sha256: hash.clone(),
                created_at: "2026-08-25T00:00:01Z".into(),
                actor_id: "actor".into(),
            })
            .unwrap();
        let output = serde_json::json!({"result": 42});
        let receipt = CalculationReceipt::create(
            "calculation",
            "1.0.0",
            vec![wellforge_core::InputRevision {
                kind: "project_revision".into(),
                id: "revision".into(),
                content_sha256: revision_digest,
            }],
            wellforge_core::CalculationContext {
                unit_system: "si".into(),
                crs: "EPSG:4979".into(),
                backend: wellforge_core::CalculationBackend::Cpu,
                actor_id: "actor".into(),
                warnings: Vec::new(),
            },
            &output,
        )
        .unwrap();
        store
            .record_core_calculation_receipt(CoreCalculationReceiptInput {
                id: "receipt".into(),
                project_revision_id: "revision".into(),
                receipt,
                output,
                recorded_at: "2026-08-25T00:00:02Z".into(),
            })
            .unwrap();
        store
            .append_replay_event(ReplayEventInput {
                id: "event".into(),
                project_id: "project".into(),
                project_revision_id: Some("revision".into()),
                sequence: 1,
                event_type: "revision.created".into(),
                payload: serde_json::json!({"revisionId": "revision"}),
                occurred_at: "2026-08-25T00:00:03Z".into(),
                actor_id: "actor".into(),
            })
            .unwrap();
        store
            .enqueue_sync_envelope(SyncEnvelope {
                artifact_hash: hash,
                parent_revision_id: Some("revision".into()),
                idempotency_key: "sync".into(),
                actor_id: "actor".into(),
                occurred_at: "2026-08-25T00:00:04Z".into(),
                operation: SyncOperation::Upsert,
            })
            .unwrap();

        for statement in [
            "UPDATE project_revision SET actor_id = actor_id WHERE id = 'revision'",
            "DELETE FROM project_revision WHERE id = 'revision'",
            "UPDATE calculation_receipt SET recorded_at = recorded_at WHERE id = 'receipt'",
            "DELETE FROM calculation_receipt WHERE id = 'receipt'",
            "UPDATE replay_journal_event SET actor_id = actor_id WHERE id = 'event'",
            "DELETE FROM replay_journal_event WHERE id = 'event'",
            "UPDATE sync_envelope SET actor_id = actor_id WHERE idempotency_key = 'sync'",
            "DELETE FROM sync_envelope WHERE idempotency_key = 'sync'",
        ] {
            let error = store.connection.execute(statement, []).unwrap_err();
            assert!(
                error.to_string().contains("append-only"),
                "statement did not hit an append-only trigger: {statement}"
            );
        }
    }

    #[test]
    fn typed_receipt_load_rejects_a_database_row_rebound_to_another_revision() {
        let mut store = LocalStore::open_in_memory_for_test().unwrap();
        for project_id in ["project-a", "project-b"] {
            store
                .create_project(NewProject {
                    id: project_id.into(),
                    name: project_id.into(),
                    created_at: "2026-08-25T00:00:00Z".into(),
                })
                .unwrap();
        }
        for (revision_id, project_id, content) in [
            ("revision-a", "project-a", b"A".as_slice()),
            ("revision-b", "project-b", b"B".as_slice()),
        ] {
            store
                .append_project_revision(ProjectRevisionInput {
                    id: revision_id.into(),
                    project_id: project_id.into(),
                    parent_revision_id: None,
                    content: content.to_vec(),
                    content_sha256: format!("sha256:{:x}", Sha256::digest(content)),
                    created_at: "2026-08-25T00:00:01Z".into(),
                    actor_id: "actor".into(),
                })
                .unwrap();
        }
        let output = serde_json::json!({"result": 42});
        let receipt = CalculationReceipt::create(
            "calculation",
            "1.0.0",
            vec![wellforge_core::InputRevision {
                kind: "project_revision".into(),
                id: "revision-a".into(),
                content_sha256: format!("{:x}", Sha256::digest(b"A")),
            }],
            wellforge_core::CalculationContext {
                unit_system: "si".into(),
                crs: "EPSG:4979".into(),
                backend: wellforge_core::CalculationBackend::Cpu,
                actor_id: "actor".into(),
                warnings: Vec::new(),
            },
            &output,
        )
        .unwrap();
        store
            .record_core_calculation_receipt(CoreCalculationReceiptInput {
                id: "receipt-a".into(),
                project_revision_id: "revision-a".into(),
                receipt,
                output,
                recorded_at: "2026-08-25T00:00:02Z".into(),
            })
            .unwrap();
        store
            .connection
            .execute_batch(
                "DROP TRIGGER calculation_receipt_is_append_only_update;
                 UPDATE calculation_receipt SET project_revision_id = 'revision-b' WHERE id = 'receipt-a';",
            )
            .unwrap();

        assert_eq!(
            store
                .load_core_calculation_receipt("receipt-a")
                .expect_err("rebound receipt is rejected"),
            StorageError::InvalidCalculationReceipt
        );
    }

    #[test]
    fn audit_listings_are_newest_first_and_metadata_only() {
        let mut store = LocalStore::open_in_memory_for_test().unwrap();
        store
            .create_project(NewProject {
                id: "project".into(),
                name: "Project".into(),
                created_at: "2026-08-27T00:00:00Z".into(),
            })
            .unwrap();
        for (id, parent_revision_id, content, created_at, actor_id) in [
            (
                "revision-1",
                None,
                b"first".as_slice(),
                "2026-08-27T00:00:01Z",
                "author-a",
            ),
            (
                "revision-2",
                Some("revision-1"),
                b"second".as_slice(),
                "2026-08-27T00:00:02Z",
                "author-b",
            ),
        ] {
            store
                .append_project_revision(ProjectRevisionInput {
                    id: id.into(),
                    project_id: "project".into(),
                    parent_revision_id: parent_revision_id.map(str::to_owned),
                    content: content.to_vec(),
                    content_sha256: format!("sha256:{:x}", Sha256::digest(content)),
                    created_at: created_at.into(),
                    actor_id: actor_id.into(),
                })
                .unwrap();
        }
        let output = serde_json::json!({"minimumCurvature": 12.5});
        let receipt = CalculationReceipt::create(
            "minimum-curvature",
            "2.3.4",
            vec![wellforge_core::InputRevision {
                kind: "project_revision".into(),
                id: "revision-2".into(),
                content_sha256: format!("{:x}", Sha256::digest(b"second")),
            }],
            wellforge_core::CalculationContext {
                unit_system: "si".into(),
                crs: "EPSG:4979".into(),
                backend: wellforge_core::CalculationBackend::Cpu,
                actor_id: "engineer".into(),
                warnings: vec!["reviewed".into()],
            },
            &output,
        )
        .unwrap();
        store
            .record_core_calculation_receipt(CoreCalculationReceiptInput {
                id: "receipt-1".into(),
                project_revision_id: "revision-2".into(),
                receipt,
                output,
                recorded_at: "2026-08-27T00:00:03Z".into(),
            })
            .unwrap();

        let revisions = store.list_project_revision_audits("project").unwrap();
        assert_eq!(revisions.len(), 2);
        assert_eq!(revisions[0].id, "revision-2");
        assert_eq!(
            revisions[0].parent_revision_id.as_deref(),
            Some("revision-1")
        );
        assert_eq!(revisions[0].actor_id, "author-b");
        assert_eq!(revisions[0].created_at, "2026-08-27T00:00:02Z");
        assert_eq!(revisions[1].id, "revision-1");
        assert!(
            revisions
                .iter()
                .all(|item| item.content_sha256.starts_with("sha256:"))
        );

        let receipts = store.list_calculation_receipt_audits("project").unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].id, "receipt-1");
        assert_eq!(receipts[0].project_revision_id, "revision-2");
        assert_eq!(receipts[0].algorithm, "minimum-curvature");
        assert_eq!(receipts[0].algorithm_version, "2.3.4");
        assert_eq!(receipts[0].recorded_at, "2026-08-27T00:00:03Z");
        assert!(receipts[0].content_sha256.starts_with("sha256:"));

        for value in [
            serde_json::to_value(&revisions).unwrap(),
            serde_json::to_value(&receipts).unwrap(),
        ] {
            let encoded = value.to_string();
            assert!(!encoded.contains("first"));
            assert!(!encoded.contains("second"));
            assert!(!encoded.contains("content\""));
        }
    }

    #[test]
    fn audit_listings_order_fractional_timestamps_by_instant_then_insertion() {
        let mut store = LocalStore::open_in_memory_for_test().unwrap();
        store
            .create_project(NewProject {
                id: "project".into(),
                name: "Project".into(),
                created_at: "2026-08-27T00:00:00Z".into(),
            })
            .unwrap();
        for (id, parent_revision_id, content, created_at) in [
            (
                "revision-1",
                None,
                b"first".as_slice(),
                "2026-08-27T00:00:00Z",
            ),
            (
                "revision-2",
                Some("revision-1"),
                b"second".as_slice(),
                "2026-08-27T00:00:00.1Z",
            ),
            (
                "revision-3",
                Some("revision-2"),
                b"third".as_slice(),
                "2026-08-27T00:00:00.100Z",
            ),
        ] {
            store
                .append_project_revision(ProjectRevisionInput {
                    id: id.into(),
                    project_id: "project".into(),
                    parent_revision_id: parent_revision_id.map(str::to_owned),
                    content: content.to_vec(),
                    content_sha256: format!("sha256:{:x}", Sha256::digest(content)),
                    created_at: created_at.into(),
                    actor_id: "author".into(),
                })
                .unwrap();
        }
        let output = serde_json::json!({"value": 1});
        for (id, revision_id, revision_content, recorded_at) in [
            (
                "receipt-2",
                "revision-2",
                b"second".as_slice(),
                "2026-08-27T00:00:00.1Z",
            ),
            (
                "receipt-3",
                "revision-3",
                b"third".as_slice(),
                "2026-08-27T00:00:00.100Z",
            ),
        ] {
            let receipt = CalculationReceipt::create(
                "calculation",
                "1.0.0",
                vec![wellforge_core::InputRevision {
                    kind: "project_revision".into(),
                    id: revision_id.into(),
                    content_sha256: format!("{:x}", Sha256::digest(revision_content)),
                }],
                wellforge_core::CalculationContext {
                    unit_system: "si".into(),
                    crs: "EPSG:4979".into(),
                    backend: wellforge_core::CalculationBackend::Cpu,
                    actor_id: "engineer".into(),
                    warnings: Vec::new(),
                },
                &output,
            )
            .unwrap();
            store
                .record_core_calculation_receipt(CoreCalculationReceiptInput {
                    id: id.into(),
                    project_revision_id: revision_id.into(),
                    receipt,
                    output: output.clone(),
                    recorded_at: recorded_at.into(),
                })
                .unwrap();
        }

        assert_eq!(
            store
                .list_project_revision_audits("project")
                .unwrap()
                .into_iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            ["revision-3", "revision-2", "revision-1"]
        );
        assert_eq!(
            store
                .list_calculation_receipt_audits("project")
                .unwrap()
                .into_iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            ["receipt-3", "receipt-2"]
        );
    }
}
