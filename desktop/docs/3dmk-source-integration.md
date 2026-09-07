# 3Dmk source integration

The standalone 3Dmk source tree has been reconciled with the canonical `desktop/` implementation. This is a file-and-feature integration: the source folder had no Git history to merge. It does not complete the separate remote-branch, Numerical Core, or Excel add-in release work.

## Source dispositions

The retained-file inventory contains 319 files (110,677,634 bytes). All 319 originals have an independently SHA-256-verified local-only copy. Generated build/dependency/cache directories were excluded from copying and remain in the original folder; no source folder was deleted.

| Source files | Count | Disposition |
| --- | ---: | --- |
| Byte-identical application, crate, tooling and documentation files | 112 | Already present in canonical desktop; the analytics README is additionally updated for the new contract |
| Analytics library and regression tests | 2 | Integrated schema/hash/byte/row guards, with immutable snapshot hardening |
| Workspace manifest/lockfile, toolchain, Tauri manifest/entrypoint/commands and TypeScript IPC | 7 | Kept newer canonical versions, including Rust 1.98 and canonical trajectory scene ingress |
| Formats crate manifest | 1 | Kept canonical version; only a trailing blank line differed |
| Alternative projection plan and design | 2 | Preserved locally as unfinished alternatives; canonical proposals clearly marked as not the implemented contract |
| Local conversation index and previous task scratch | 4 | Preserved locally; no conversation identifiers or task history published |
| Reference catalogs and imported help assets | 191 | Preserved as local-only reference; not added to runtime or Git history |

## Integrated behavior

The standalone analytics utility now requires the implemented source contract: `schemaVersion` equals `wellforge.analytics.approved-extract/v1`, and `recordsSha256` binds the exact JSONL bytes. Missing, malformed or mismatched integrity metadata fails closed. Producers of older unversioned manifests must supply these fields; this integration does not silently accept unsigned legacy extracts.

Record input is capped at 256 MiB and 500,000 nonblank lines. Limits and digest validation precede record decoding. A single bounded byte snapshot supplies both hashing and parsing, eliminating the source implementation's race between checking one pathname read and parsing another. This preserves the existing combined input hash and report shape. The tradeoff is up to 256 MiB of buffered raw input, plus decoded/columnar data, rather than a constant-memory streaming implementation.

SQLite and portable projects remain authoritative. DuckDB and Polars remain inside the offline, non-authoritative standalone utility; no new desktop runtime dependency, database output or network operation is introduced.

## Features gathered but not activated

- The alternate projection proposal would replace Polars with stream/DuckDB aggregate parity and introduce sorted entity-kind counts plus a projection digest. It may reduce dependencies and strengthen aggregate checks, but it changes the report contract and was not implemented in the source. Keep it as a separately testable design decision, not an assumed completed feature.
- Imported UI planning context covers frame modes, attention hierarchy, severity contrast and responsive/recovery fixtures. These can inform later desktop and add-in work, but the source contains no corresponding unique implementation to activate here.
- Imported engineering help and catalogs may aid research and requirements tracing. They do not establish calculation correctness or redistribution rights, and adding them to runtime or public Git history would introduce privacy, licensing and maintenance risk.
- Older toolchain/dependency manifests and scene entrypoints would remove newer canonical functionality. They are preserved for recovery, not used to downgrade the application.

## Recovery and verification

Local-only `ref/local-3dmk-20260907/` contains the source snapshot and a verified pre-integration Git bundle. Detailed per-file hash/disposition records are kept locally with the consolidation evidence. The original active source directory is left intact for task continuity.

Acceptance requires standalone analytics regression tests, strict formatting/lint checks for the changed crate, desktop type checking/tests/build, a desktop Rust workspace test run, and independent code/disposition review. Native Excel installation/COM acceptance is a separate release gate and is not claimed here.
