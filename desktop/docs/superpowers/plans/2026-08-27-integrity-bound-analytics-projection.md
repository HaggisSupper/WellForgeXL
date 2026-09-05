# Integrity-Bound Analytics Projection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bind standalone DuckDB/Polars analytics to bounded, approved JSONL extracts and publish a deterministic parity-checked projection report.

**Architecture:** `wellforge-analytics` remains a nested, non-desktop Cargo utility. Its manifest cryptographically binds the exact JSONL bytes; streaming validation limits resource use; Polars and in-memory DuckDB calculate the same per-kind aggregate independently before a report is returned.

**Tech Stack:** Rust 2024, SHA-256, serde, Polars 0.46, DuckDB 1.4.5, SQLite storage contracts.

**Spec:** `docs/superpowers/specs/2026-08-27-integrity-bound-analytics-projection-design.md`

## Global Constraints

- SQLite and portable project artifacts are the only authority sources.
- DuckDB uses `Connection::open_in_memory()` only; it receives no database path or URL.
- Polars and DuckDB stay outside the desktop Cargo workspace and Tauri runtime.
- No Docker, network operation, promotion, synchronization, or durable analytics database file.
- Record input is bounded at 256 MiB and 500,000 non-blank JSONL records; a limit failure never truncates input.
- Do not use legacy vendor/product naming or the phrase prohibited by the project language rule in new prose.
- Skip graph mapping because the user explicitly ruled it out for this corpus-scale task.

---

### Task 1: Bind and stream the approved input

**Files:**
- Modify: `tools/wellforge-analytics/src/lib.rs`
- Modify: `tools/wellforge-analytics/tests/reconcile.rs`

**Interfaces:**
- Consumes: `ApprovedExtractManifest`, a manifest JSON file, and a JSONL record file.
- Produces: a manifest contract with `schemaVersion` and `recordsSha256`; `reconcile_files` rejects mismatches and values over `MAX_RECORD_BYTES` or `MAX_RECORDS` before record decoding.

- [ ] **Step 1: Write the failing integration tests**

```rust
#[test]
fn rejects_a_record_digest_mismatch_before_decoding() {
    let (manifest, records) = write_fixture_with_manifest_digest("0".repeat(64), &["not json"]);
    assert!(matches!(reconcile_files(manifest, records), Err(AnalyticsError::RecordsDigestMismatch { .. })));
}

#[test]
fn rejects_inputs_over_the_explicit_limits() {
    assert!(matches!(reconcile_files(manifest, oversized_records), Err(AnalyticsError::InputByteLimitExceeded { .. })));
    assert!(matches!(reconcile_files(manifest, too_many_records), Err(AnalyticsError::RecordLimitExceeded { .. })));
}
```

- [ ] **Step 2: Run the focused tests and observe the missing contract failures**

Run: `cargo test --manifest-path tools/wellforge-analytics/Cargo.toml --test reconcile digest_mismatch -- --nocapture`

Expected: failure because the manifest has no bound records digest and the utility reads/decodes the full file first.

- [ ] **Step 3: Implement bounded two-pass streaming**

```rust
pub const MAX_RECORD_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_RECORDS: u64 = 500_000;

pub struct ApprovedExtractManifest {
    pub schema_version: String,
    pub records_sha256: String,
    pub batch: ImportBatchMetadata,
    pub rule_set_version: String,
}
```

Use `BufReader::read_until(b'\n', ...)` in a first pass that counts all bytes, counts non-blank lines, and computes SHA-256 of exact record bytes. Validate the manifest schema and lower-hex digest, compare before creating `ExtractRecord`, then perform the parsing pass with the same limits.

- [ ] **Step 4: Run the analytics reconciliation suite and formatter**

Run: `cargo test --manifest-path tools/wellforge-analytics/Cargo.toml --test reconcile`

Run: `cargo fmt --manifest-path tools/wellforge-analytics/Cargo.toml --all -- --check`

Expected: all focused tests pass.

### Task 2: Independently reconcile entity-kind aggregates

**Files:**
- Modify: `tools/wellforge-analytics/src/lib.rs`
- Modify: `tools/wellforge-analytics/tests/reconcile.rs`

**Interfaces:**
- Consumes: validated entity-kind rows from Task 1.
- Produces: `ReconciliationReport.entity_kind_counts: Vec<EntityKindCount>` sorted by kind and `projection_sha256: String`; `AnalyticsError::AggregateParity` on different Polars and DuckDB counts.

- [ ] **Step 1: Write the failing multi-kind projection test**

```rust
#[test]
fn reports_sorted_per_kind_counts_from_both_engines() {
    let report = reconcile_files(manifest, records_with_kinds(["bha_component", "survey_station", "survey_station"]))?;
    assert_eq!(report.entity_kind_counts, vec![
        EntityKindCount { source_entity_kind: "bha_component".into(), accepted_record_count: 1 },
        EntityKindCount { source_entity_kind: "survey_station".into(), accepted_record_count: 2 },
    ]);
    assert_eq!(report.projection_sha256.len(), 64);
}
```

- [ ] **Step 2: Run the test and observe the absent report projection fields**

Run: `cargo test --manifest-path tools/wellforge-analytics/Cargo.toml --test reconcile reports_sorted_per_kind_counts_from_both_engines -- --nocapture`

Expected: failure because the report exposes only total and distinct counts.

- [ ] **Step 3: Add independent engine aggregates and deterministic projection identity**

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EntityKindCount {
    pub source_entity_kind: String,
    pub accepted_record_count: u64,
}
```

Use Polars grouping over `source_entity_kind`, query DuckDB `GROUP BY source_entity_kind ORDER BY source_entity_kind`, convert both to `BTreeMap<String, u64>`, and fail with `AnalyticsError::AggregateParity` unless they match. Serialize the ordered report projection fields to bytes and SHA-256 them for `projection_sha256`.

- [ ] **Step 4: Run the focused suite and strict analytics lint**

Run: `cargo test --manifest-path tools/wellforge-analytics/Cargo.toml --test reconcile`

Run: `cargo clippy --manifest-path tools/wellforge-analytics/Cargo.toml --all-targets -- -D warnings`

Expected: all analytics tests and clippy pass.

### Task 3: Align local-authority and analytics documentation

**Files:**
- Modify: `tools/wellforge-analytics/README.md`
- Modify: `WELLFORGE-TAURI-PORT-ARCHITECTURE.md`
- Modify: `WELLFORGE-RUST-TAURI-FEASIBILITY.md`
- Modify: `WELLFORGE-SOURCE-MIGRATION-CATALOG.md`

**Interfaces:**
- Consumes: the completed Task 1 and Task 2 public contracts.
- Produces: documentation consistently naming SQLite as authority, DuckDB/Polars as non-authoritative downstream analytics, and historical PostgreSQL material as inactive reference.

- [ ] **Step 1: Write a documentation assertion checklist in the implementation record**

```text
SQLite: local authority
DuckDB/Polars: downstream, in-memory/non-authoritative analytics
Historical PostgreSQL DDL: reference only, not desktop runtime work
Docker: not part of the stack
```

- [ ] **Step 2: Search the named files for contradictory active-runtime claims**

Run: `rg -n -i "postgres|duckdb|polars|sqlite|docker" tools/wellforge-analytics/README.md WELLFORGE-TAURI-PORT-ARCHITECTURE.md WELLFORGE-RUST-TAURI-FEASIBILITY.md WELLFORGE-SOURCE-MIGRATION-CATALOG.md`

Expected: existing wording identifies locations requiring alignment.

- [ ] **Step 3: Amend each document with the approved operating model**

```text
SQLite is the local authority. DuckDB and Polars are downstream analytics-only tools.
Historical PostgreSQL DDL remains reference material and is not a desktop runtime dependency.
```

Retain historical context where useful, but label it unequivocally and do not create a migration/promotion plan for it.

- [ ] **Step 4: Re-run the checklist search and inspect the analytics README command contract**

Run: `rg -n -i "postgres.*(runtime|target|active)|docker" WELLFORGE-TAURI-PORT-ARCHITECTURE.md WELLFORGE-RUST-TAURI-FEASIBILITY.md WELLFORGE-SOURCE-MIGRATION-CATALOG.md`

Expected: no active runtime/database target claim and no Docker instruction.

### Task 4: Cross-boundary verification

- [ ] **Step 1: Run analytics formatting, tests, and lint**

Run: `cargo fmt --manifest-path tools/wellforge-analytics/Cargo.toml --all -- --check`

Run: `cargo test --manifest-path tools/wellforge-analytics/Cargo.toml`

Run: `cargo clippy --manifest-path tools/wellforge-analytics/Cargo.toml --all-targets -- -D warnings`

- [ ] **Step 2: Verify the desktop workspace remains isolated from analytics dependencies**

Run: `cargo metadata --no-deps --format-version 1 | rg 'wellforge-analytics|duckdb|polars'`

Expected: no output.

- [ ] **Step 3: Run the desktop workspace regression suite**

Run: `cargo test --workspace`

- [ ] **Step 4: Record verification outcomes and graph-mapping decision in this plan**

Document passed commands and state that graph mapping was skipped under the explicit corpus-scale constraint.
