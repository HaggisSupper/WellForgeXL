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
#[serde(rename_all = "snake_case")]
pub enum InferredCausalityActor {
    ManualConfirmed,
    ManualInferred,
    AutomationCommand,
    AutomationInferred,
    Unknown,
}

impl InferredCausalityActor {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ManualConfirmed => "manual_confirmed",
            Self::ManualInferred => "manual_inferred",
            Self::AutomationCommand => "automation_command",
            Self::AutomationInferred => "automation_inferred",
            Self::Unknown => "unknown",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "manual_confirmed" => Some(Self::ManualConfirmed),
            "manual_inferred" => Some(Self::ManualInferred),
            "automation_command" => Some(Self::AutomationCommand),
            "automation_inferred" => Some(Self::AutomationInferred),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferredCausalityClaimLevel {
    ObservedSequence,
    LikelyReaction,
    CausalClaim,
}

impl InferredCausalityClaimLevel {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ObservedSequence => "observed_sequence",
            Self::LikelyReaction => "likely_reaction",
            Self::CausalClaim => "causal_claim",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "observed_sequence" => Some(Self::ObservedSequence),
            "likely_reaction" => Some(Self::LikelyReaction),
            "causal_claim" => Some(Self::CausalClaim),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferredCausalityOutcome {
    Effective,
    PartiallyEffective,
    Ineffective,
    Adverse,
    Inconclusive,
}

impl InferredCausalityOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Effective => "effective",
            Self::PartiallyEffective => "partially_effective",
            Self::Ineffective => "ineffective",
            Self::Adverse => "adverse",
            Self::Inconclusive => "inconclusive",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "effective" => Some(Self::Effective),
            "partially_effective" => Some(Self::PartiallyEffective),
            "ineffective" => Some(Self::Ineffective),
            "adverse" => Some(Self::Adverse),
            "inconclusive" => Some(Self::Inconclusive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InferredCausalityInput {
    pub episode_key: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub actor: InferredCausalityActor,
    pub claim_level: InferredCausalityClaimLevel,
    pub situation: Value,
    pub intervention: Value,
    pub observed_response: Value,
    pub outcome: InferredCausalityOutcome,
    pub confidence: f64,
    pub inference_method: String,
    pub canonical_state_ref: Option<String>,
    pub source_artifact_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferredCausalityRecord {
    pub id: Uuid,
    pub episode_key: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub actor: InferredCausalityActor,
    pub claim_level: InferredCausalityClaimLevel,
    pub situation: Value,
    pub intervention: Value,
    pub observed_response: Value,
    pub outcome: InferredCausalityOutcome,
    pub confidence: f64,
    pub inference_method: String,
    pub canonical_state_ref: Option<String>,
    pub source_artifact_id: Option<Uuid>,
    pub authority: String,
    pub is_canonical: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
