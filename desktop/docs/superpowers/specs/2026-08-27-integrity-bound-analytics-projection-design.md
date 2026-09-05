# Integrity-Bound Analytics Projection v1

## Purpose

Harden the approved local analytics boundary: SQLite and portable project artifacts remain authoritative; the standalone DuckDB and Polars utility consumes only a manifest that cryptographically binds a bounded JSONL extract. The utility remains offline, non-authoritative, and unable to write a database file or promote results.

## Decisions

- The manifest wire contract is versioned with `schemaVersion: "wellforge.analytics-projection.v1"`.
- `recordsSha256` is the lowercase 64-hex SHA-256 digest of the exact JSONL bytes. It is validated before any JSONL record is decoded.
- The utility performs two streaming passes over the record file: the first verifies byte and row limits while computing exact-input digests; the second parses and validates records. It never reads the full file into memory.
- Limits are fixed for v1: 256 MiB total input bytes and 500,000 non-blank JSONL records. Exceeding either limit is an explicit failure, never truncation.
- Polars and in-memory DuckDB independently aggregate accepted records by source entity kind. Their sorted counts must match exactly. DuckDB remains `open_in_memory()` only.
- The report retains exact-input identity and adds sorted `entityKindCounts` plus `projectionSha256`, which hashes the schema version, batch ID, rule-set version, bound record digest, and sorted aggregates.
- The desktop Cargo workspace and Tauri runtime do not depend on DuckDB or Polars. There is no database URL, network operation, promotion action, or durable analytics output.

## Error Handling

Malformed manifests, an unknown schema version, invalid record digest syntax, digest mismatch, oversized input, excess record count, duplicate keys, failed staged validation, and engine-count disagreement all fail the run. Neither rejected records nor partial reconciliation reports are emitted.

## Test Evidence

The feature requires red-to-green tests for a digest mismatch before JSON decoding, each capacity limit, a mixed-kind projection with exact sorted counts and parity, and preservation of rejected/duplicate-record rejection. The tool's focused integration suite, formatting, clippy, and the desktop workspace suite remain green.

## Documentation Boundary

Migration documentation must state that SQLite is the local authority. DuckDB and Polars are downstream analytics-only tools. Historical PostgreSQL DDL is retained as reference material, not runtime migration work.
