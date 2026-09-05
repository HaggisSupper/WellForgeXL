use std::{
    fs,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct RagConfig {
    pub storage: StorageConfig,
    pub server: ServerConfig,
    pub ingest: IngestConfig,
    pub embedding: EmbeddingConfig,
    pub concepts: ConceptConfig,
    pub search: SearchConfig,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub sqlite: PathBuf,
    pub lancedb: PathBuf,
    pub okf: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub roots: Vec<PathBuf>,
    pub max_file_bytes: u64,
    pub max_extraction_bytes: u64,
    pub python: String,
    pub python_adapter: PathBuf,
}

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct ConceptConfig {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub lexical_weight: f32,
    pub semantic_weight: f32,
    pub candidate_limit: usize,
}

#[derive(Debug, Deserialize)]
struct RawRagConfig {
    storage: RawStorageConfig,
    #[serde(default)]
    server: RawServerConfig,
    #[serde(default)]
    ingest: RawIngestConfig,
    #[serde(default)]
    embedding: RawEmbeddingConfig,
    #[serde(default)]
    concepts: RawConceptConfig,
    #[serde(default)]
    search: RawSearchConfig,
}

#[derive(Debug, Deserialize)]
struct RawStorageConfig {
    sqlite: PathBuf,
    lancedb: PathBuf,
    okf: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct RawServerConfig {
    bind: String,
}

impl Default for RawServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8765".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct RawIngestConfig {
    roots: Vec<PathBuf>,
    max_file_bytes: u64,
    max_extraction_bytes: u64,
    python: String,
    python_adapter: PathBuf,
}

impl Default for RawIngestConfig {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            max_file_bytes: 512 * 1024 * 1024,
            max_extraction_bytes: 64 * 1024 * 1024,
            python: "python".to_owned(),
            python_adapter: PathBuf::from("../adapters/extract.py"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct RawEmbeddingConfig {
    enabled: bool,
    base_url: String,
    model: String,
    dimension: usize,
    batch_size: usize,
    timeout_seconds: u64,
}

impl Default for RawEmbeddingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://127.0.0.1:1234/v1".to_owned(),
            model: "text-embedding-nomic-embed-text-v1.5".to_owned(),
            dimension: 768,
            batch_size: 32,
            timeout_seconds: 60,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct RawConceptConfig {
    enabled: bool,
    base_url: String,
    model: String,
    timeout_seconds: u64,
}

impl Default for RawConceptConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://127.0.0.1:1234/v1".to_owned(),
            model: "local".to_owned(),
            timeout_seconds: 90,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct RawSearchConfig {
    lexical_weight: f32,
    semantic_weight: f32,
    candidate_limit: usize,
}

impl Default for RawSearchConfig {
    fn default() -> Self {
        Self {
            lexical_weight: 1.0,
            semantic_weight: 1.0,
            candidate_limit: 40,
        }
    }
}

impl RagConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw_text = fs::read_to_string(path)
            .with_context(|| format!("cannot read RAG config {}", path.display()))?;
        let raw: RawRagConfig = toml::from_str(&raw_text)
            .with_context(|| format!("invalid RAG config {}", path.display()))?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let bind = raw
            .server
            .bind
            .parse::<SocketAddr>()
            .with_context(|| format!("invalid server bind address {}", raw.server.bind))?;

        if raw.embedding.enabled && raw.embedding.dimension == 0 {
            bail!("embedding dimension must be greater than zero when embeddings are enabled");
        }
        if raw.embedding.batch_size == 0 {
            bail!("embedding batch_size must be greater than zero");
        }
        if raw.search.candidate_limit == 0 {
            bail!("search candidate_limit must be greater than zero");
        }

        Ok(Self {
            storage: StorageConfig {
                sqlite: resolve_path(base, raw.storage.sqlite),
                lancedb: resolve_path(base, raw.storage.lancedb),
                okf: resolve_path(base, raw.storage.okf),
            },
            server: ServerConfig { bind },
            ingest: IngestConfig {
                roots: raw
                    .ingest
                    .roots
                    .into_iter()
                    .map(|value| resolve_path(base, value))
                    .collect(),
                max_file_bytes: raw.ingest.max_file_bytes,
                max_extraction_bytes: raw.ingest.max_extraction_bytes,
                python: raw.ingest.python,
                python_adapter: resolve_path(base, raw.ingest.python_adapter),
            },
            embedding: EmbeddingConfig {
                enabled: raw.embedding.enabled,
                base_url: raw.embedding.base_url.trim_end_matches('/').to_owned(),
                model: raw.embedding.model,
                dimension: raw.embedding.dimension,
                batch_size: raw.embedding.batch_size,
                timeout_seconds: raw.embedding.timeout_seconds,
            },
            concepts: ConceptConfig {
                enabled: raw.concepts.enabled,
                base_url: raw.concepts.base_url.trim_end_matches('/').to_owned(),
                model: raw.concepts.model,
                timeout_seconds: raw.concepts.timeout_seconds,
            },
            search: SearchConfig {
                lexical_weight: raw.search.lexical_weight,
                semantic_weight: raw.search.semantic_weight,
                candidate_limit: raw.search.candidate_limit,
            },
        })
    }
}

fn resolve_path(base: &Path, value: PathBuf) -> PathBuf {
    if value.is_absolute() {
        lexical_normalize(value)
    } else {
        lexical_normalize(base.join(value))
    }
}

fn lexical_normalize(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(value) => normalized.push(value),
        }
    }
    normalized
}
