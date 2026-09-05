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

use crate::model::{
    ArtifactInput, ArtifactRecord, ChunkInput, ChunkRecord, CitationInput, ConceptInput,
    ConceptRecord, CorpusStats, SearchHit,
};

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
                        i64::try_from(input.size_bytes)
                            .context("artifact size exceeds SQLite range")?,
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

    pub fn upsert_concept(&self, input: ConceptInput) -> Result<ConceptRecord> {
        validate_concept_path(&input.concept_path)?;
        if input.title.trim().is_empty() || input.concept_type.trim().is_empty() {
            bail!("concept title and type must not be empty");
        }
        let frontmatter = serde_json::to_string(&input.frontmatter)?;
        let now = Utc::now().to_rfc3339();
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let existing_id = transaction
            .query_row(
                "SELECT id FROM concepts WHERE concept_path = ?1",
                params![input.concept_path],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let id = existing_id
            .map(|value| Uuid::parse_str(&value).context("stored concept UUID is invalid"))
            .transpose()?
            .unwrap_or_else(Uuid::new_v4);

        transaction.execute(
            "INSERT INTO concepts (
                id, concept_path, concept_type, title, domain, body, frontmatter_json,
                provenance_state, trust_state, lifecycle_state, source_confidence,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)
             ON CONFLICT(concept_path) DO UPDATE SET
                concept_type = excluded.concept_type,
                title = excluded.title,
                domain = excluded.domain,
                body = excluded.body,
                frontmatter_json = excluded.frontmatter_json,
                provenance_state = excluded.provenance_state,
                trust_state = excluded.trust_state,
                lifecycle_state = excluded.lifecycle_state,
                source_confidence = excluded.source_confidence,
                updated_at = excluded.updated_at",
            params![
                id.to_string(),
                input.concept_path,
                input.concept_type,
                input.title,
                input.domain,
                input.body,
                frontmatter,
                input.provenance_state,
                input.trust_state,
                input.lifecycle_state,
                input.source_confidence,
                now,
            ],
        )?;
        transaction.execute(
            "DELETE FROM concepts_fts WHERE concept_id = ?1",
            params![id.to_string()],
        )?;
        transaction.execute(
            "INSERT INTO concepts_fts(concept_id, title, body, domain)
             SELECT id, title, body, domain FROM concepts WHERE id = ?1",
            params![id.to_string()],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_concept(id)?
            .with_context(|| format!("concept {id} disappeared after upsert"))
    }

    pub fn get_concept(&self, id: Uuid) -> Result<Option<ConceptRecord>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT id, concept_path, concept_type, title, domain, body,
                        frontmatter_json, provenance_state, trust_state,
                        lifecycle_state, source_confidence
                 FROM concepts WHERE id = ?1",
                params![id.to_string()],
                concept_from_row,
            )
            .optional()
            .context("cannot read concept")
    }

    pub fn list_concepts(&self) -> Result<Vec<ConceptRecord>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, concept_path, concept_type, title, domain, body,
                    frontmatter_json, provenance_state, trust_state,
                    lifecycle_state, source_confidence
             FROM concepts ORDER BY concept_path",
        )?;
        statement
            .query_map([], concept_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("cannot list concepts")
    }

    pub fn upsert_chunk(&self, input: ChunkInput) -> Result<ChunkRecord> {
        validate_sha256(&input.content_hash)?;
        let ordinal = i64::try_from(input.ordinal).context("chunk ordinal exceeds SQLite range")?;
        let token_estimate = i64::try_from(input.token_estimate)
            .context("chunk token estimate exceeds SQLite range")?;
        let now = Utc::now().to_rfc3339();
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let existing_id = transaction
            .query_row(
                "SELECT id FROM chunks
                 WHERE artifact_id = ?1 AND ordinal = ?2 AND content_hash = ?3",
                params![input.artifact_id.to_string(), ordinal, input.content_hash],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let id = existing_id
            .map(|value| Uuid::parse_str(&value).context("stored chunk UUID is invalid"))
            .transpose()?
            .unwrap_or_else(Uuid::new_v4);

        transaction.execute(
            "INSERT INTO chunks (
                id, concept_id, artifact_id, ordinal, section_locator, source_locator,
                text, content_hash, token_estimate, embedding_state, embedding_model,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)
             ON CONFLICT(artifact_id, ordinal, content_hash) DO UPDATE SET
                concept_id = excluded.concept_id,
                section_locator = excluded.section_locator,
                source_locator = excluded.source_locator,
                text = excluded.text,
                token_estimate = excluded.token_estimate,
                embedding_state = excluded.embedding_state,
                embedding_model = excluded.embedding_model,
                updated_at = excluded.updated_at",
            params![
                id.to_string(),
                input.concept_id.map(|value| value.to_string()),
                input.artifact_id.to_string(),
                ordinal,
                input.section_locator,
                input.source_locator,
                input.text,
                input.content_hash,
                token_estimate,
                input.embedding_state,
                input.embedding_model,
                now,
            ],
        )?;
        transaction.execute(
            "DELETE FROM chunks_fts WHERE chunk_id = ?1",
            params![id.to_string()],
        )?;
        transaction.execute(
            "INSERT INTO chunks_fts(chunk_id, text)
             SELECT id, text FROM chunks WHERE id = ?1",
            params![id.to_string()],
        )?;
        transaction.commit()?;
        drop(connection);
        self.get_chunk(id)?
            .with_context(|| format!("chunk {id} disappeared after upsert"))
    }

    pub fn get_chunk(&self, id: Uuid) -> Result<Option<ChunkRecord>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT id, concept_id, artifact_id, ordinal, section_locator,
                        source_locator, text, content_hash, token_estimate,
                        embedding_state, embedding_model
                 FROM chunks WHERE id = ?1",
                params![id.to_string()],
                chunk_from_row,
            )
            .optional()
            .context("cannot read chunk")
    }

    pub fn chunks_for_artifact(&self, artifact_id: Uuid) -> Result<Vec<ChunkRecord>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, concept_id, artifact_id, ordinal, section_locator,
                    source_locator, text, content_hash, token_estimate,
                    embedding_state, embedding_model
             FROM chunks WHERE artifact_id = ?1 ORDER BY ordinal",
        )?;
        statement
            .query_map(params![artifact_id.to_string()], chunk_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("cannot list artifact chunks")
    }

    pub fn add_citation(&self, input: CitationInput) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO citations (
                id, concept_id, chunk_id, artifact_id, locator_type, locator, label, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id.to_string(),
                input.concept_id.map(|value| value.to_string()),
                input.chunk_id.map(|value| value.to_string()),
                input.artifact_id.to_string(),
                input.locator_type,
                input.locator,
                input.label,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(id)
    }

    pub fn link_edge(
        &self,
        source: Uuid,
        target: Uuid,
        edge_type: &str,
        artifact_id: Option<Uuid>,
    ) -> Result<Uuid> {
        if edge_type.trim().is_empty() {
            bail!("concept edge type must not be empty");
        }
        let id = Uuid::new_v4();
        let connection = self.lock()?;
        connection.execute(
            "INSERT OR IGNORE INTO concept_edges (
                id, source_concept_id, target_concept_id, edge_type,
                provenance_json, source_artifact_id, created_at
             ) VALUES (?1, ?2, ?3, ?4, '{}', ?5, ?6)",
            params![
                id.to_string(),
                source.to_string(),
                target.to_string(),
                edge_type,
                artifact_id.map(|value| value.to_string()),
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(id)
    }

    pub fn related_concepts(&self, id: Uuid) -> Result<Vec<(String, ConceptRecord)>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT e.edge_type,
                    c.id, c.concept_path, c.concept_type, c.title, c.domain, c.body,
                    c.frontmatter_json, c.provenance_state, c.trust_state,
                    c.lifecycle_state, c.source_confidence
             FROM concept_edges e
             JOIN concepts c ON c.id = e.target_concept_id
             WHERE e.source_concept_id = ?1
             ORDER BY e.edge_type, c.concept_path",
        )?;
        statement
            .query_map(params![id.to_string()], |row| {
                let edge_type: String = row.get(0)?;
                let concept = concept_from_row_offset(row, 1)?;
                Ok((edge_type, concept))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("cannot read related concepts")
    }

    pub fn lexical_search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let match_query = fts_query(query);
        if match_query.is_empty() {
            return Ok(Vec::new());
        }
        let limit = i64::try_from(limit).context("search limit exceeds SQLite range")?;
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT f.chunk_id, ch.concept_id, ch.artifact_id, a.display_name,
                    c.title, c.domain, ch.source_locator, ch.text, bm25(chunks_fts)
             FROM chunks_fts f
             JOIN chunks ch ON ch.id = f.chunk_id
             JOIN artifacts a ON a.id = ch.artifact_id
             LEFT JOIN concepts c ON c.id = ch.concept_id
             WHERE chunks_fts MATCH ?1
             ORDER BY bm25(chunks_fts), ch.id
             LIMIT ?2",
        )?;
        statement
            .query_map(params![match_query, limit], |row| {
                let rank: f64 = row.get(8)?;
                Ok(SearchHit {
                    chunk_id: parse_uuid_sql(row.get::<_, String>(0)?, 0)?,
                    concept_id: row
                        .get::<_, Option<String>>(1)?
                        .map(|value| parse_uuid_sql(value, 1))
                        .transpose()?,
                    artifact_id: parse_uuid_sql(row.get::<_, String>(2)?, 2)?,
                    artifact_name: row.get(3)?,
                    concept_title: row.get(4)?,
                    domain: row.get(5)?,
                    source_locator: row.get(6)?,
                    text: row.get(7)?,
                    score: -rank,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("lexical search failed")
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

fn validate_concept_path(value: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('/')
        || trimmed.contains("..")
        || trimmed.contains('\\')
    {
        bail!("concept path must be a relative forward-slash path without parent traversal");
    }
    Ok(())
}

fn fts_query(value: &str) -> String {
    value
        .split(|character: char| !character.is_alphanumeric() && character != '_' && character != '-')
        .filter(|token| !token.is_empty())
        .take(20)
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ")
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

fn parse_uuid_sql(value: String, column: usize) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn artifact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArtifactRecord> {
    let size_bytes: i64 = row.get(5)?;
    if size_bytes < 0 {
        return Err(rusqlite::Error::IntegralValueOutOfRange(5, size_bytes));
    }
    Ok(ArtifactRecord {
        id: parse_uuid_sql(row.get(0)?, 0)?,
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

fn concept_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConceptRecord> {
    concept_from_row_offset(row, 0)
}

fn concept_from_row_offset(row: &rusqlite::Row<'_>, offset: usize) -> rusqlite::Result<ConceptRecord> {
    let frontmatter_text: String = row.get(offset + 6)?;
    let frontmatter = serde_json::from_str(&frontmatter_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            offset + 6,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })?;
    Ok(ConceptRecord {
        id: parse_uuid_sql(row.get(offset)?, offset)?,
        concept_path: row.get(offset + 1)?,
        concept_type: row.get(offset + 2)?,
        title: row.get(offset + 3)?,
        domain: row.get(offset + 4)?,
        body: row.get(offset + 5)?,
        frontmatter,
        provenance_state: row.get(offset + 7)?,
        trust_state: row.get(offset + 8)?,
        lifecycle_state: row.get(offset + 9)?,
        source_confidence: row.get(offset + 10)?,
    })
}

fn chunk_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChunkRecord> {
    let ordinal: i64 = row.get(3)?;
    let token_estimate: i64 = row.get(8)?;
    if ordinal < 0 || token_estimate < 0 {
        return Err(rusqlite::Error::IntegralValueOutOfRange(
            if ordinal < 0 { 3 } else { 8 },
            if ordinal < 0 { ordinal } else { token_estimate },
        ));
    }
    Ok(ChunkRecord {
        id: parse_uuid_sql(row.get(0)?, 0)?,
        concept_id: row
            .get::<_, Option<String>>(1)?
            .map(|value| parse_uuid_sql(value, 1))
            .transpose()?,
        artifact_id: parse_uuid_sql(row.get(2)?, 2)?,
        ordinal: ordinal as u64,
        section_locator: row.get(4)?,
        source_locator: row.get(5)?,
        text: row.get(6)?,
        content_hash: row.get(7)?,
        token_estimate: token_estimate as u64,
        embedding_state: row.get(9)?,
        embedding_model: row.get(10)?,
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
