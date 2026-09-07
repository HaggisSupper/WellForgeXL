use std::fs;

use tempfile::tempdir;
use wellforge_rag_core::{ArtifactInput, RagConfig, SqliteStore};

#[test]
fn config_resolves_storage_paths_relative_to_config_file() {
    let root = tempdir().unwrap();
    let config_dir = root.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("rag.toml");
    fs::write(
        &config_path,
        r#"
[storage]
sqlite = "../data/corpus.sqlite3"
lancedb = "../data/lancedb"
okf = "../data/okf"

[server]
bind = "127.0.0.1:8765"
"#,
    )
    .unwrap();

    let config = RagConfig::load(&config_path).unwrap();
    assert_eq!(
        config.storage.sqlite,
        root.path().join("data/corpus.sqlite3")
    );
    assert_eq!(config.storage.lancedb, root.path().join("data/lancedb"));
    assert_eq!(config.storage.okf, root.path().join("data/okf"));
    assert_eq!(config.server.bind.to_string(), "127.0.0.1:8765");
}

#[test]
fn sqlite_store_initializes_schema_and_deduplicates_artifacts_by_sha256() {
    let root = tempdir().unwrap();
    let db = root.path().join("corpus.sqlite3");
    let store = SqliteStore::open(&db).unwrap();

    let first = store
        .upsert_artifact(ArtifactInput {
            sha256: "a".repeat(64),
            source_uri: "file:///a/manual.pdf".into(),
            display_name: "manual.pdf".into(),
            mime_type: "application/pdf".into(),
            family: "document".into(),
            size_bytes: 123,
            modified_at: None,
            extraction_backend: "test".into(),
        })
        .unwrap();
    let second = store
        .upsert_artifact(ArtifactInput {
            sha256: "a".repeat(64),
            source_uri: "file:///b/copy.pdf".into(),
            display_name: "copy.pdf".into(),
            mime_type: "application/pdf".into(),
            family: "document".into(),
            size_bytes: 123,
            modified_at: None,
            extraction_backend: "test".into(),
        })
        .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(store.stats().unwrap().artifacts, 1);
    assert_eq!(store.artifact_aliases(first.id).unwrap().len(), 2);
}
