# WellForge analytics utility

This standalone utility reconciles **accepted**, provenance-bound staged extracts. It is deliberately outside the desktop Cargo workspace so its columnar dependencies do not become part of the desktop runtime.

It accepts a JSON manifest and JSONL records, then writes one deterministic JSON report:

```text
cargo run --manifest-path tools/wellforge-analytics/Cargo.toml -- \
  --manifest approved-extract.json \
  --records accepted-records.jsonl \
  --report reconciliation-report.json
```

## Approved-extract integrity contract

The manifest requires all four fields; missing fields or fields of the wrong type fail JSON deserialization:

- `schemaVersion`: exactly `wellforge.analytics.approved-extract/v1`.
- `recordsSha256`: exactly 64 lowercase hexadecimal characters containing `SHA256(record bytes)`, without a `sha256:` prefix. This binds the exact JSONL bytes, including blank lines, spaces, CRLF versus LF, and the presence or absence of a final newline.
- `batch`: valid staged import metadata, including batch ID, source system, source location, source checksum, extraction time, and operator ID.
- `ruleSetVersion`: a nonblank rule-set version matching every record's validation.

The public limits are `MAX_RECORD_BYTES = 256 * 1024 * 1024` (256 MiB) and `MAX_RECORDS = 500_000`, inclusive. Every input byte counts toward the byte limit. Records are newline-delimited nonblank lines; ASCII-whitespace-only lines are ignored when counting and decoding records. LF, CRLF, and a final record without a newline are supported.

After validating the manifest, the utility opens the record file once and captures at most `MAX_RECORD_BYTES + 1` bytes. It rejects an extra sentinel byte before scanning any records, without relying on file metadata or allocating from a reported file size. It then counts nonblank lines before comparing the records digest and before decoding record JSON. The failure order is byte limit, row limit, digest mismatch, then record decoding and staged validation; an empty input with a matching digest fails as empty. Unsupported schemas, malformed digests, limit violations, and digest mismatches have typed `AnalyticsError` variants. Every validation failure fails the entire run; the CLI neither creates nor overwrites a report in that case.

Counting, digest validation, decoding, and report hashing use the same immutable byte snapshot. Changes to the original file after capture cannot change the records being reconciled. The report's `inputSha256` remains the lowercase hexadecimal digest `SHA256(manifest bytes || [0] || record bytes)`, where `[0]` is one zero byte and both inputs retain their exact original bytes. Its existing fields and engine identifier are unchanged.

This deliberately trades bounded in-memory storage for integrity: the captured input can contain up to 256 MiB plus one sentinel byte, with additional memory for vector capacity, parsed records, and columnar reconciliation. This is not a streaming or constant-memory interface. No source digest or successful report grants authority to promote or persist data.

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
