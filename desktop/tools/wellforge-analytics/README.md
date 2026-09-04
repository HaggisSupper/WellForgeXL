# WellForge analytics utility

This standalone utility reconciles **accepted**, provenance-bound staged extracts. It is deliberately outside the desktop Cargo workspace so its columnar dependencies do not become part of the desktop runtime.

It accepts a JSON manifest and JSONL records, then writes one deterministic JSON report:

```text
cargo run --manifest-path tools/wellforge-analytics/Cargo.toml -- \
  --manifest approved-extract.json \
  --records accepted-records.jsonl \
  --report reconciliation-report.json
```

## Operational boundary

- SQLite local authority records and portable project artifacts are the only approved projection sources.
- DuckDB is opened with `open_in_memory()` only; this utility has no database URL option and creates no DuckDB file.
- Polars is used only to materialize accepted rows in-process for columnar reconciliation.
- The utility cannot promote, persist, or release records. A typed application boundary must validate and persist any approved result through the SQLite local-authority API.
- Input must contain validated `accepted` records, with batch, rule-set, source-system, and source-checksum values matching the manifest. Duplicate source record keys, malformed staged contracts, rejected records, and provenance mismatches fail the entire run.
- Fixture data is synthetic. The report hashes the exact manifest and JSONL input bytes, in that order, separated by a zero byte.

## Dependency posture

- `duckdb 1.4.5` is compiled with the bundled engine and used only in memory.
- `polars 0.46.0` has default features disabled; no Parquet input/output feature is enabled.
- On Windows the build links the platform Restart Manager library required by DuckDB's bundled file-lock diagnostics.
- This tool is intentionally not a shared runtime dependency of the desktop application.
