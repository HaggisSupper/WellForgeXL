use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ArtifactInput {
    pub sha256: String,
    pub source_uri: String,
    pub display_name: String,
    pub mime_type: String,
    pub family: String,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub extraction_backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactRecord {
    pub id: Uuid,
    pub sha256: String,
    pub display_name: String,
    pub mime_type: String,
    pub family: String,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub extraction_backend: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ConceptInput {
    pub concept_path: String,
    pub concept_type: String,
    pub title: String,
    pub domain: String,
    pub body: String,
    pub frontmatter: Value,
    pub provenance_state: String,
    pub trust_state: String,
    pub lifecycle_state: String,
    pub source_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConceptRecord {
    pub id: Uuid,
    pub concept_path: String,
    pub concept_type: String,
    pub title: String,
    pub domain: String,
    pub body: String,
    pub frontmatter: Value,
    pub provenance_state: String,
    pub trust_state: String,
    pub lifecycle_state: String,
    pub source_confidence: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ChunkInput {
    pub concept_id: Option<Uuid>,
    pub artifact_id: Uuid,
    pub ordinal: u64,
    pub section_locator: String,
    pub source_locator: String,
    pub text: String,
    pub content_hash: String,
    pub token_estimate: u64,
    pub embedding_state: String,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkRecord {
    pub id: Uuid,
    pub concept_id: Option<Uuid>,
    pub artifact_id: Uuid,
    pub ordinal: u64,
    pub section_locator: String,
    pub source_locator: String,
    pub text: String,
    pub content_hash: String,
    pub token_estimate: u64,
    pub embedding_state: String,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CitationInput {
    pub concept_id: Option<Uuid>,
    pub chunk_id: Option<Uuid>,
    pub artifact_id: Uuid,
    pub locator_type: String,
    pub locator: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    pub chunk_id: Uuid,
    pub concept_id: Option<Uuid>,
    pub artifact_id: Uuid,
    pub artifact_name: String,
    pub concept_title: Option<String>,
    pub domain: Option<String>,
    pub source_locator: String,
    pub text: String,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorpusStats {
    pub artifacts: u64,
    pub concepts: u64,
    pub chunks: u64,
    pub citations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestReport {
    pub artifact_id: Uuid,
    pub concepts_written: u64,
    pub chunks_written: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OkfExportReport {
    pub files_written: u64,
    pub files: Vec<PathBuf>,
}
