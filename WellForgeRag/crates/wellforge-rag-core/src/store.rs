use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::model::{ArtifactInput, ArtifactRecord, CorpusStats};

const MIGRATION: &str = include_str!("../../../migrations/0001_init.sql");

#[derive(Clone)]
pub struct SqliteStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create SQLite directory {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("cannot open SQLite corpus {}", path.display()))?;
        connection.busy_timeout(Duration::from_secs(10))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection
            .execute_batch(MIGRATION)
            .context("cannot initialize SQLite RAG schema")?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn upsert_artifact(&self, input: ArtifactInput) -> Result<ArtifactRecord> {
        validate_sha256(&input.sha256)?;
        if input.source_uri.trim().is_empty() {
            bail!("artifact source URI must not be empty");
        }
        let now = Utc::now().to_rfc3339();
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let existing_id = transaction
            .query_row(
                "SELECT id FROM artifacts WHERE sha256 = ?1",
                params![input.sha256],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        let id = match existing_id {
            Some(value) => Uuid::parse_str(&value).context("stored artifact UUID is invalid")?,
            None => {
                let id = Uuid::new_v4();
                transaction.execute(
                    "INSERT INTO artifacts (
                        id, sha256, display_name, mime_type, family, size_bytes,
                        modified_at, extraction_backend, status, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'ready', ?9, ?9)",
                    params![
                        id.to_string(),
                        input.sha256,
                        input.display_name,
                        input.mime_type,
                        input.family,
                        i64::try_from(input.size_bytes).context("artifact size exceeds SQLite range")?,
                        input.modified_at.map(|value| value.to_rfc3339()),
                        input.extraction_backend,
                        now,
                    ],
                )?;
                id
            }
        };

        transaction.execute(
            "INSERT INTO artifact_aliases (artifact_id, source_uri, first_seen_at, last_seen_at)
             VALUES (?1, ?2, ?3, ?3)
             ON CONFLICT(artifact_id, source_uri)
             DO UPDATE SET last_seen_at = excluded.last_seen_at",
            params![id.to_string(), input.source_uri, now],
        )?;
        transaction.execute(
            "UPDATE artifacts SET updated_at = ?2 WHERE id = ?1",
            params![id.to_string(), now],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_artifact(id)?
            .with_context(|| format!("artifact {id} disappeared after upsert"))
    }

    pub fn get_artifact(&self, id: Uuid) -> Result<Option<ArtifactRecord>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT id, sha256, display_name, mime_type, family, size_bytes,
                        modified_at, extraction_backend, status
                 FROM artifacts WHERE id = ?1",
                params![id.to_string()],
                artifact_from_row,
            )
            .optional()
            .context("cannot read artifact")
    }

    pub fn artifact_aliases(&self, id: Uuid) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT source_uri FROM artifact_aliases
             WHERE artifact_id = ?1 ORDER BY source_uri",
        )?;
        let values = statement
            .query_map(params![id.to_string()], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(values)
    }

    pub fn stats(&self) -> Result<CorpusStats> {
        let connection = self.lock()?;
        Ok(CorpusStats {
            artifacts: table_count(&connection, "artifacts")?,
            concepts: table_count(&connection, "concepts")?,
            chunks: table_count(&connection, "chunks")?,
            citations: table_count(&connection, "citations")?,
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| anyhow::anyhow!("SQLite corpus mutex is poisoned"))
    }
}

fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("artifact SHA-256 must be exactly 64 hexadecimal characters");
    }
    Ok(())
}

fn parse_optional_datetime(value: Option<String>) -> rusqlite::Result<Option<DateTime<Utc>>> {
    value
        .map(|text| {
            DateTime::parse_from_rfc3339(&text)
                .map(|parsed| parsed.with_timezone(&Utc))
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
        })
        .transpose()
}

fn artifact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArtifactRecord> {
    let id: String = row.get(0)?;
    let size_bytes: i64 = row.get(5)?;
    if size_bytes < 0 {
        return Err(rusqlite::Error::IntegralValueOutOfRange(5, size_bytes));
    }
    Ok(ArtifactRecord {
        id: Uuid::parse_str(&id).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        sha256: row.get(1)?,
        display_name: row.get(2)?,
        mime_type: row.get(3)?,
        family: row.get(4)?,
        size_bytes: size_bytes as u64,
        modified_at: parse_optional_datetime(row.get(6)?)?,
        extraction_backend: row.get(7)?,
        status: row.get(8)?,
    })
}

fn table_count(connection: &Connection, table: &str) -> Result<u64> {
    let query = match table {
        "artifacts" => "SELECT COUNT(*) FROM artifacts",
        "concepts" => "SELECT COUNT(*) FROM concepts",
        "chunks" => "SELECT COUNT(*) FROM chunks",
        "citations" => "SELECT COUNT(*) FROM citations",
        _ => bail!("unsupported canonical table count {table}"),
    };
    let value: i64 = connection.query_row(query, [], |row| row.get(0))?;
    u64::try_from(value).context("negative SQLite table count")
}
