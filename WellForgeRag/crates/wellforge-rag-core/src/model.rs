use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorpusStats {
    pub artifacts: u64,
    pub concepts: u64,
    pub chunks: u64,
    pub citations: u64,
}
