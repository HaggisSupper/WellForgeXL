# Audit Viewer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose immutable local project revisions and calculation receipts in a read-only desktop audit workspace.

**Architecture:** The storage crate returns deliberately reduced audit summaries, Tauri serializes those summaries, and React validates and renders them. No local path, project-content bytes, SQL, or mutation capability crosses the boundary.

**Tech Stack:** Rust 2024, rusqlite, Tauri 2, React, TypeScript, Vitest.

**Spec:** User-approved audit-viewer design in this task.

## Global Constraints

- SQLite is the local authority; DuckDB and Polars remain analytics-only.
- Read-only API; immutable records and local paths remain protected.
- Run Rust formatting/tests/clippy and frontend tests/typecheck/build.
- Skip graph mapping: the user ruled it out for this corpus-scale task.

---

### Task 1: Typed storage audit summaries

**Files:** Modify `crates/db/src/lib.rs`; test `crates/db/src/lib.rs`.

- [x] Write a failing storage test that persists two project revisions and one receipt, then expects newest-first summaries with IDs, parent IDs, hashes, timestamps, actor, algorithm, and version but no content bytes.
- [x] Run `cargo test -p wellforge-storage audit` and confirm it fails because the listing API is absent.
- [x] Add `StoredProjectRevisionAudit` and `StoredCalculationReceiptAudit` plus read-only listing methods scoped by project ID.
- [x] Re-run the storage audit test and `cargo fmt --all -- --check`.

### Task 2: Typed Tauri audit command

**Files:** Modify `src-tauri/src/commands/mod.rs`, `src-tauri/src/main.rs`; test `src-tauri/src/commands/mod.rs`.

- [x] Write a failing command test for an active project returning only audit summaries.
- [x] Run the focused command test and confirm the active-project audit helper is absent.
- [x] Add `get_project_audit` and register it; map storage failures to structured API errors.
- [x] Re-run the command tests.

### Task 3: Audit workspace UI

**Files:** Modify `src/lib/ipc.ts`, `src/components/EmptyCanvas.tsx`, `src/stores/ui.ts`; create `src/components/AuditWorkspace.tsx` and tests.

- [x] Write failing UI/IPC tests for rejecting malformed audit responses and rendering hashes/provenance without a path.
- [x] Run focused Audit workspace tests and confirm failure.
- [x] Add strict client validation, an Audit navigation module, and a loading/error/empty/read-only audit display.
- [x] Replace free-form receipt warnings with `warningCount`, and render only fixed safe audit failure text.
- [x] Run `npm test`, `npm run typecheck`, and `npm run build`.

### Task 4: Full verification

- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `npm test` and `npm run build`.
- [x] Update the program plan with the delivered audit slice.

## Delivery Record — 2026-08-27

- Delivered a project-scoped, immutable Audit workspace for revision lineage and calculation receipt metadata.
- The desktop boundary permits identifiers, hashes, timestamps, actor/provenance, algorithm/version, and receipt `warningCount` only. It excludes project content, calculation output, receipt bytes, local paths, database details, and raw failure details.
- Timestamp ordering is chronological by parsed UTC instant; equal instants use insertion order deterministically.
- Verification: Rust workspace tests, Rust clippy with warnings denied, Rust formatting, frontend Vitest suite, TypeScript typecheck, and production build all passed.
