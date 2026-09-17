use std::fs;

use tempfile::tempdir;
use wellforge_rag_core::{RagConfig, RagService};

fn write_config(root: &std::path::Path) -> std::path::PathBuf {
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let config = config_dir.join("rag.toml");
    fs::write(
        &config,
        r#"
[storage]
sqlite = "../data/corpus.sqlite3"
lancedb = "../data/lancedb"
okf = "../data/okf"

[server]
bind = "127.0.0.1:8765"

[ingest]
roots = ["../sources"]
max_file_bytes = 1048576
max_extraction_bytes = 1048576
python = "python"
python_adapter = "../adapters/extract.py"

[embedding]
enabled = false
base_url = "http://127.0.0.1:1234/v1"
model = "test"
dimension = 4
batch_size = 8
timeout_seconds = 1

[concepts]
enabled = false
base_url = "http://127.0.0.1:1234/v1"
model = "test"
timeout_seconds = 1

[search]
lexical_weight = 1.0
semantic_weight = 1.0
candidate_limit = 20
"#,
    )
    .unwrap();
    config
}

#[test]
fn ingest_search_and_okf_export_round_trip_through_sqlite() {
    let root = tempdir().unwrap();
    let sources = root.path().join("sources");
    fs::create_dir_all(&sources).unwrap();
    let source = sources.join("hydraulics.md");
    fs::write(
        &source,
        "# Equivalent circulating density\n\nAnnular pressure loss contributes to ECD. Preserve TVD and fluid density basis.\n",
    )
    .unwrap();

    let config = RagConfig::load(write_config(root.path())).unwrap();
    let service = RagService::open(config).unwrap();
    let report = service.ingest_path(&source).unwrap();
    assert_eq!(service.stats().unwrap().artifacts, 1);
    assert!(report.concepts_written >= 1);
    assert!(report.chunks_written >= 1);

    let hits = service.search_lexical("annular pressure loss", 10).unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].artifact_id, report.artifact_id);
    assert!(!hits[0].source_locator.is_empty());

    let exported = service.export_okf().unwrap();
    assert!(exported.files_written >= 1);
    let okf_text = fs::read_to_string(exported.files[0].clone()).unwrap();
    assert!(okf_text.starts_with("---\n"));
    assert!(okf_text.contains("provenance_state:"));
    assert!(okf_text.contains("Equivalent circulating density"));
}

#[test]
fn ingestion_rejects_paths_outside_configured_roots() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("sources")).unwrap();
    let outside = root.path().join("outside.md");
    fs::write(&outside, "outside").unwrap();

    let config = RagConfig::load(write_config(root.path())).unwrap();
    let service = RagService::open(config).unwrap();
    let error = service.ingest_path(&outside).unwrap_err().to_string();
    assert!(error.contains("outside configured ingestion roots"));
}
