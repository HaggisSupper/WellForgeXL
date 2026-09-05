#![forbid(unsafe_code)]

mod config;
mod extract;
mod model;
mod service;
mod store;

pub use config::{
    ConceptConfig, EmbeddingConfig, IngestConfig, RagConfig, SearchConfig, ServerConfig,
    StorageConfig,
};
pub use extract::{
    ArtifactFamily, ColumnProfile, DataProfile, ExtractionEnvelope, ExtractionStatus, TextSection,
    extract_path,
};
pub use model::{
    ArtifactInput, ArtifactRecord, ChunkInput, ChunkRecord, CitationInput, ConceptInput,
    ConceptRecord, CorpusStats, IngestReport, OkfExportReport, SearchHit,
};
pub use service::RagService;
pub use store::SqliteStore;
