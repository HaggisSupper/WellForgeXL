# WellForgeRag Design

## Status

Approved for implementation by the user on 2026-09-05.

## Purpose

`WellForgeRag/` is the local-first retrieval and knowledge subsystem for WellForge. It is not a calculation engine and must never become an alternate source of engineering calculation authority.

The subsystem has three persistence roles:

1. **SQLite is canonical** for RAG structure, identities, provenance, concepts, relationships, chunks, citations, ingestion state, and lexical search.
2. **Open Knowledge Format (OKF) v0.2 is the concept representation** exported from and importable into the SQLite corpus. It gives agents a readable engineering concept graph with provenance/trust/lifecycle metadata. It is not the canonical database.
3. **LanceDB is the rebuildable semantic vector index**. Deleting LanceDB must not lose canonical RAG information; it can be regenerated from SQLite plus the configured embedding model.

Native engineering files remain authoritative for their own content. Deterministic Rust engineering engines remain calculation authority.

## Global constraints

- Windows-first and local-first.
- No Docker.
- Rust 2024 for the server/core/CLI/MCP implementation.
- Python is permitted only as an extraction sidecar; it must not own canonical state.
- SQLite must be sufficient for deterministic corpus recovery without LanceDB.
- LanceDB is an index, never a source of record.
- OKF is a semantic concept layer inside RAG, not a global WellForge interchange format.
- No macros or embedded document code are executed during ingestion.
- No arbitrary SQL is exposed through MCP or HTTP.
- No source is promoted to engineering authority merely because it is retrieved frequently or has high vector similarity.
- Avoid the term `golden`; use `reference fixture`, `expected-result fixture`, or `acceptance fixture` as appropriate.

## Repository boundary

The implementation lives in a top-level independent Cargo workspace:

```text
WellForgeRag/
  Cargo.toml
  rust-toolchain.toml
  README.md
  config/
  migrations/
  adapters/
  crates/
    wellforge-rag-core/
    wellforge-rag-server/
    wellforge-rag-mcp/
    wellforge-rag-cli/
  skills/
    drilling-engineering/
  scripts/
  tests/
```

It is intentionally **not** a member of `engine/Cargo.toml`. RAG dependency changes must not change deterministic engine lockfile identity or calculation evidence.

## Core architecture

```text
Native artifacts
      |
      v
Format router / extraction adapters
      |
      v
Normalized extraction envelope
      |
      +-----------------------+
      |                       |
      v                       v
SQLite canonical corpus   Concept generation/enrichment
      |                       |
      |<----------------------+
      |
      +--> OKF v0.2 materialization
      |
      +--> SQLite FTS5 lexical index
      |
      +--> Embedding client --> LanceDB semantic index
                               |
                               v
                       semantic candidates
                               |
                               v
                 SQLite hydration + citations
```

## SQLite canonical schema

### `artifacts`

One row per ingested native artifact.

Required fields:
- stable UUID
- SHA-256
- normalized source path/URI
- display name
- MIME type
- artifact family
- byte size
- modification time when known
- extraction backend/version
- ingestion status
- created/updated timestamps

Exact duplicate SHA-256 content resolves to one artifact identity with multiple source aliases.

### `artifact_aliases`

Tracks every known path/URI for a canonical artifact without duplicating content identity.

### `concepts`

Canonical concept records:
- concept UUID
- OKF-compatible concept path/slug
- type
- title
- engineering domain
- Markdown body
- frontmatter JSON
- provenance state
- trust state
- freshness/lifecycle state
- source confidence
- created/updated timestamps

### `concept_edges`

Typed directed relationships between concepts, including:
- `depends_on`
- `produced_by`
- `validated_by`
- `derived_from`
- `related_to`
- `supersedes`
- `applies_to`
- `measured_by`
- `implemented_by`

Every edge has provenance and optional source artifact/citation linkage.

### `chunks`

Retrieval units derived from concept/body/source text. Each chunk includes:
- chunk UUID
- concept UUID when applicable
- artifact UUID
- ordinal
- section locator
- page/sheet/record locator when applicable
- normalized text
- content hash
- token estimate
- embedding state/model identity

### `citations`

Structured source locators. Store source identity and locator, not fabricated quotations.

Examples:
- PDF page
- workbook sheet/range
- LAS section/curve
- XML object/path
- source line range
- database table/column

### `ingestion_runs`

Immutable run metadata: run UUID, start/end time, software version, config hash, success/failure counts, warnings and model identities.

### `index_state`

Records the SQLite corpus revision and the LanceDB/FTS index revision so stale vector indexes are detectable.

## SQLite FTS5

`concepts_fts` and `chunks_fts` provide lexical retrieval and a no-model fallback. Lexical search must work even when LM Studio or the embedding endpoint is unavailable.

## LanceDB vector schema

The LanceDB `chunks` table contains only rebuildable index material:
- `chunk_id`
- `concept_id`
- `artifact_id`
- `domain`
- `text`
- fixed-size `vector`
- `embedding_model`
- `content_hash`

Retrieval always hydrates final records from SQLite. LanceDB metadata cannot override SQLite metadata.

## Embeddings

Embeddings use a configurable OpenAI-compatible local endpoint, defaulting to LM Studio conventions. Configuration specifies:
- base URL
- embedding model
- vector dimension
- request timeout
- batch size

If embeddings are unavailable, ingestion succeeds with `embedding_state = pending` and the corpus remains queryable through SQLite FTS5. `reindex` can fill the vector index later.

## Concept generation

Concept enrichment uses a configurable OpenAI-compatible chat endpoint. The model proposes structured concepts and edges from the normalized extraction envelope.

The model is constrained to return validated JSON. Generated concepts are recorded with explicit provenance and trust state. LLM output can create `candidate` knowledge but cannot assert that an engineering equation, standard, or model is independently validated.

The deterministic fallback creates a source concept plus structured chunks even when no LLM is available.

## OKF v0.2

OKF documents are materialized under the configured corpus directory, default `data/okf/`.

Rules:
- SQLite concept UUID/path remains canonical.
- Each OKF file contains YAML frontmatter and Markdown body.
- OKF relationships use normal links plus typed relationship metadata where appropriate.
- Provenance/trust/freshness/lifecycle/attestation fields are emitted when available.
- Materialization is deterministic and can be regenerated from SQLite.
- External OKF bundles can be imported, but imports pass through the same validation/provenance path before becoming SQLite concepts.

## Ingestion modes

### Semantic document ingestion

Initial concrete support:
- PDF
- DOCX
- PPTX
- TXT/Markdown
- HTML
- PNG/JPEG/TIFF OCR when Tesseract is available
- XLSX/XLSM/XLSB value/table extraction through the Python sidecar

The Python adapter is a subprocess with JSON stdin/stdout contract. It never owns database state.

### Structured text/data ingestion

Native Rust support:
- JSON / JSONL
- YAML
- TOML
- CSV / TSV
- XML, including WITSML XML as structured XML evidence

Large tabular files are profiled rather than expanded into one text chunk per row.

### Drilling data ingestion

Initial native Rust support:
- LAS: version/well/curve metadata, curve units/descriptions and data-row count; curve sample arrays are not converted to prose chunks.
- WITSML XML: object/element structure and text evidence through XML ingestion.

DLIS remains a separately routed binary artifact until a verified parser is integrated; the system records the artifact and unsupported-parser status without pretending it extracted content.

### Analytical formats

Initial native Rust support:
- Parquet metadata/schema profiling.
- Arrow IPC schema profiling.
- SQLite database schema/table/view profiling.

DuckDB files are recorded as structured database artifacts but are not read through SQLite; support is enabled only when a verified DuckDB adapter is present.

## Search pipeline

`search(query)` performs:

1. FTS5 lexical retrieval from SQLite.
2. Semantic vector retrieval from LanceDB when an embedding endpoint/index is available.
3. Stable reciprocal-rank fusion of candidate chunk IDs.
4. SQLite hydration of concepts/artifacts/citations.
5. Domain and evidence filters.
6. Returned evidence includes source locator and trust/provenance state.

Search never calculates engineering outputs.

## HTTP server

`wellforge-rag-server` binds to loopback by default.

Endpoints:
- `GET /health`
- `GET /v1/stats`
- `POST /v1/search`
- `GET /v1/concepts/:id`
- `GET /v1/artifacts/:id`
- `POST /v1/ingest`
- `POST /v1/reindex`
- `POST /v1/okf/export`

Writes are local filesystem/database actions and return structured run IDs/status.

## CLI

`wellforge-rag` commands:
- `init`
- `ingest <path>`
- `search <query>`
- `concept <id>`
- `artifact <id>`
- `reindex`
- `export-okf`
- `stats`
- `doctor`

## MCP server

`wellforge-rag-mcp` uses the official Rust MCP SDK and stdio transport by default.

Tools:
- `rag_search`
- `rag_get_concept`
- `rag_get_artifact`
- `rag_ingest`
- `rag_related`
- `rag_export_okf`
- `rag_reindex`
- `rag_stats`

Resources:
- `wellforge-rag://concept/{id}`
- `wellforge-rag://artifact/{id}`

No arbitrary filesystem access or SQL execution is exposed.

## Drilling engineering skills

`WellForgeRag/skills/drilling-engineering/` teaches agents how to use retrieval evidence. The entry skill routes to focused skills for:
- evidence and source grounding
- directional/trajectory
- hydraulics and well control
- torque-drag/BHA mechanics
- drilling performance and rig state

Skill rules include:
- use RAG to retrieve evidence, not to replace deterministic calculation engines;
- distinguish parity/reference-fixture evidence from independent physical/model evidence;
- preserve units and pressure/temperature basis;
- prefer primary literature/standards when claiming model authority;
- cite the artifact locator returned by RAG;
- do not infer missing engineering inputs;
- surface contradictory sources rather than silently choosing one.

## Security and safety

- File size and extraction-output limits are enforced.
- Archive recursion is not enabled in the initial implementation.
- Office macros and scripts are never executed.
- Python extraction runs without shell interpolation and receives exact argument paths.
- HTTP binds to `127.0.0.1` by default.
- SQL parameters are bound; no MCP/HTTP endpoint accepts arbitrary SQL.
- Paths outside configured ingestion roots can be rejected by policy.
- Hashes are computed before extraction and used for deduplication.

## Testing

Required test layers:

1. SQLite migrations and canonical CRUD.
2. SHA-256 deduplication and alias behavior.
3. FTS retrieval.
4. deterministic chunking.
5. OKF deterministic export and re-import.
6. LAS parser reference fixtures.
7. CSV/JSON/XML profile fixtures.
8. search fusion with a fake semantic index.
9. HTTP endpoint tests.
10. MCP tool routing tests.
11. vector integration smoke test with temporary LanceDB and deterministic synthetic vectors.
12. `doctor` verifies SQLite, OKF directory, LanceDB path, and configured local model endpoints without mutating corpus state.

## Migration from `tools/doc-rag-mcp`

The existing Node document-RAG code remains available during migration. `WellForgeRag` becomes the preferred server/storage boundary. The existing Python extractor logic is moved/copied into the new sidecar contract and can be invoked by the Rust ingestion router. Existing Node entry points are not deleted in this change, preventing a hard cutover before the Rust path is verified.
