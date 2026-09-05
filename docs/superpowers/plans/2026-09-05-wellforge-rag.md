# WellForgeRag Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local-first Rust RAG subsystem under `WellForgeRag/` with SQLite canonical storage, OKF v0.2 concept materialization, LanceDB semantic indexing, HTTP/CLI/MCP entry points, extraction adapters, and drilling-engineering retrieval skills.

**Architecture:** `wellforge-rag-core` owns config, SQLite, ingestion, chunking, OKF, embedding/chat clients, LanceDB indexing, and hybrid retrieval. Thin binaries expose the same core through CLI, loopback HTTP, and stdio MCP. The workspace remains independent from `engine/` so RAG dependency changes never mutate deterministic-engine evidence.

**Tech Stack:** Rust 2024, Rust 1.98, rusqlite 0.40.2, LanceDB 0.38.0, Arrow 58.x compatible with LanceDB, Axum 0.8, RMCP 3.2, Tokio, Reqwest, Serde, Python extraction sidecar.

**Spec:** `docs/superpowers/specs/2026-09-05-wellforge-rag-design.md`

## Global Constraints

- Windows-first and local-first.
- No Docker.
- SQLite is canonical; LanceDB is rebuildable.
- OKF is the concept representation inside RAG, not global WellForge authority.
- Python is extraction-only; Rust owns persistence and server boundaries.
- No macro/script execution during ingestion.
- No arbitrary SQL endpoint.
- Do not use the term `golden`.
- Do not add `WellForgeRag` to `engine/Cargo.toml`.

---

### Task 1: Independent workspace and configuration

**Files:**
- Create: `WellForgeRag/Cargo.toml`
- Create: `WellForgeRag/rust-toolchain.toml`
- Create: `WellForgeRag/.gitignore`
- Create: `WellForgeRag/config/wellforge-rag.toml`
- Create: `WellForgeRag/crates/wellforge-rag-core/Cargo.toml`
- Create: binary crate manifests for CLI/server/MCP

**Interfaces:**
- `RagConfig::load(path) -> Result<RagConfig>`
- Config resolves SQLite, LanceDB, OKF, ingestion-root and local model endpoints.

- [ ] Write config parsing tests for defaults and explicit paths.
- [ ] Add the workspace and manifests.
- [ ] Run `cargo test -p wellforge-rag-core config`.
- [ ] Commit.

### Task 2: SQLite canonical schema and repository

**Files:**
- Create: `WellForgeRag/migrations/0001_init.sql`
- Create: `WellForgeRag/crates/wellforge-rag-core/src/store.rs`
- Create: `WellForgeRag/crates/wellforge-rag-core/src/model.rs`

**Interfaces:**
- `SqliteStore::open(config) -> Result<SqliteStore>`
- `upsert_artifact`, `get_artifact`, `upsert_concept`, `get_concept`, `upsert_chunk`, `link_edge`, `add_citation`, `stats`, `lexical_search`

- [ ] Write migration/CRUD/dedup/FTS tests using temporary databases.
- [ ] Implement migrations and bound-parameter repository methods.
- [ ] Verify duplicate SHA-256 creates aliases rather than duplicate artifacts.
- [ ] Commit.

### Task 3: Deterministic extraction envelope and native parsers

**Files:**
- Create: `WellForgeRag/crates/wellforge-rag-core/src/extract/mod.rs`
- Create: `.../extract/text.rs`
- Create: `.../extract/tabular.rs`
- Create: `.../extract/xml.rs`
- Create: `.../extract/las.rs`
- Create: `.../extract/parquet.rs`
- Create: `.../extract/arrow.rs`
- Create: `.../extract/sqlite.rs`
- Create: `WellForgeRag/tests/fixtures/*`

**Interfaces:**
- `Extractor::extract(path) -> Result<ExtractionEnvelope>`
- Envelope includes artifact family, text sections, structured tables/profiles, locators and warnings.

- [ ] Write reference-fixture tests for TXT/JSON/CSV/XML/LAS/Parquet/Arrow/SQLite.
- [ ] Implement bounded parsers and profile large tabular sources instead of row-to-text expansion.
- [ ] Verify unsupported DLIS/DuckDB are recorded explicitly without fabricated extraction.
- [ ] Commit.

### Task 4: Python office/image extraction sidecar

**Files:**
- Create: `WellForgeRag/adapters/extract.py`
- Create: `WellForgeRag/adapters/requirements.txt`
- Create: `WellForgeRag/crates/wellforge-rag-core/src/extract/python.rs`

**Interfaces:**
- Rust invokes Python with exact path argument and parses one JSON result.
- Supported: PDF, DOCX, PPTX, XLSX, XLSM, XLSB, images/OCR, text/HTML.

- [ ] Port the existing non-executing extractor to the new JSON contract.
- [ ] Add subprocess timeout/output-size/error tests.
- [ ] Ensure extraction failures never mutate canonical state.
- [ ] Commit.

### Task 5: Concept generation, chunking, and OKF

**Files:**
- Create: `WellForgeRag/crates/wellforge-rag-core/src/chunk.rs`
- Create: `.../concept.rs`
- Create: `.../okf.rs`
- Create: `.../llm.rs`

**Interfaces:**
- deterministic `chunk_extraction`
- `ConceptGenerator` with local OpenAI-compatible implementation and deterministic fallback
- `OkfStore::export_all`, `import_bundle`

- [ ] Write deterministic chunking and OKF round-trip tests.
- [ ] Implement validated JSON concept proposals; LLM concepts enter as candidate/provenance-tagged records.
- [ ] Make OKF export byte-stable for unchanged SQLite content.
- [ ] Commit.

### Task 6: LanceDB and embeddings

**Files:**
- Create: `WellForgeRag/crates/wellforge-rag-core/src/embedding.rs`
- Create: `.../vector.rs`

**Interfaces:**
- `EmbeddingClient::embed(&[String]) -> Vec<Vec<f32>>`
- `VectorIndex::rebuild`, `upsert_chunks`, `search`
- `LanceVectorIndex` implementation using `lancedb` 0.38.

- [ ] Test OpenAI-compatible embedding response parsing with an in-process mock HTTP server.
- [ ] Test temporary LanceDB create/upsert/vector-search using deterministic synthetic vectors.
- [ ] Record index revision/model identity in SQLite.
- [ ] Commit.

### Task 7: Hybrid retrieval and ingestion service

**Files:**
- Create: `WellForgeRag/crates/wellforge-rag-core/src/ingest.rs`
- Create: `.../search.rs`
- Create: `.../lib.rs`

**Interfaces:**
- `RagService::init`
- `RagService::ingest_path`
- `RagService::search`
- `RagService::get_concept/get_artifact/related`
- `RagService::reindex/export_okf/stats/doctor`

- [ ] Write tests for transactional ingestion and reciprocal-rank fusion.
- [ ] Implement SHA-256 dedup, extraction, concept/chunk persistence, optional embeddings, and SQLite hydration.
- [ ] Verify semantic-index failure leaves SQLite ingestion successful and marked pending.
- [ ] Commit.

### Task 8: CLI and HTTP entry points

**Files:**
- Create: `WellForgeRag/crates/wellforge-rag-cli/src/main.rs`
- Create: `WellForgeRag/crates/wellforge-rag-server/src/main.rs`
- Create: `WellForgeRag/scripts/WellForgeRag.ps1`

**Interfaces:**
- CLI commands from spec.
- HTTP endpoints from spec, loopback by default.

- [ ] Add CLI argument tests and Axum router tests.
- [ ] Implement PowerShell launcher that locates release binaries beside the workspace and prints actionable errors.
- [ ] Commit.

### Task 9: MCP server

**Files:**
- Create: `WellForgeRag/crates/wellforge-rag-mcp/src/main.rs`
- Create: `WellForgeRag/crates/wellforge-rag-mcp/src/server.rs`

**Interfaces:**
- MCP tools: `rag_search`, `rag_get_concept`, `rag_get_artifact`, `rag_ingest`, `rag_related`, `rag_export_okf`, `rag_reindex`, `rag_stats`.
- MCP resources: `wellforge-rag://concept/{id}`, `wellforge-rag://artifact/{id}`.

- [ ] Write direct tool-handler tests against a temporary SQLite corpus.
- [ ] Implement stdio transport with RMCP 3.2.
- [ ] Verify no SQL or unrestricted filesystem tool is exposed.
- [ ] Commit.

### Task 10: Drilling engineering skills

**Files:**
- Create: `WellForgeRag/skills/drilling-engineering/SKILL.md`
- Create: `WellForgeRag/skills/drilling-engineering/evidence-grounding/SKILL.md`
- Create: `WellForgeRag/skills/drilling-engineering/directional/SKILL.md`
- Create: `WellForgeRag/skills/drilling-engineering/hydraulics-well-control/SKILL.md`
- Create: `WellForgeRag/skills/drilling-engineering/torque-drag-bha/SKILL.md`
- Create: `WellForgeRag/skills/drilling-engineering/drilling-performance/SKILL.md`

**Interfaces:**
- Skills reference the MCP tool names and evidence rules exactly.

- [ ] Write skill routing and evidence discipline.
- [ ] Add an automated text contract test that forbids calculation-authority claims and the prohibited terminology.
- [ ] Commit.

### Task 11: Documentation and CI

**Files:**
- Create: `WellForgeRag/README.md`
- Create: `WellForgeRag/docs/OPERATIONS.md`
- Create: `WellForgeRag/docs/SCHEMA.md`
- Create: `.github/workflows/wellforge-rag.yml`

**Interfaces:**
- CI runs only for `WellForgeRag/**` and its workflow.

- [ ] Document Windows setup, Python sidecar, LM Studio endpoints, storage recovery, entry points and migration from `tools/doc-rag-mcp`.
- [ ] CI: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` from `WellForgeRag`.
- [ ] Add Python `py_compile` check.
- [ ] Commit.

### Task 12: Verification and PR

- [ ] Run the full independent `WellForgeRag` CI.
- [ ] Confirm the PR diff does not modify `engine/Cargo.lock` or deterministic engine source.
- [ ] Confirm no generated SQLite/LanceDB data is committed.
- [ ] Confirm no `TODO`, placeholder, vendor/customer source names, or prohibited terminology remains in `WellForgeRag`.
- [ ] Open PR against `main` with explicit verification evidence and limitations.
