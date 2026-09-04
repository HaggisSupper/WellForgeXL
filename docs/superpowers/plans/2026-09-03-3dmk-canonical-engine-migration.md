# 3Dmk Canonical Engine Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the migrated 3Dmk desktop visualization to WellForgeXL's canonical Rust trajectory engine in small, reversible, tested increments.

**Architecture:** Keep `wellforge-3d` renderer-neutral and keep all trajectory mathematics in `engine/crates`. Add a narrow adapter at the desktop/native boundary that consumes the canonical serialized result contract, converts it into immutable 3Dmk scene documents, and avoids linking incompatible Cargo toolchains. Migrate one scene layer at a time, preserving the existing workbook and engine contracts throughout.

**Tech Stack:** Rust 2024, serde, Tauri 2, React 18, TypeScript, Zustand, native WebGL2, Cargo fixtures, Vitest.

**Spec:** `desktop/docs/superpowers/specs/2026-08-22-3dmk-visualization-core-design.md`

## Global Constraints

- `wellforge-3d` owns scene schema, validation, bounds, and provenance; it does not calculate engineering positions.
- `engine/crates/wellforge-trajectory-*` remains the calculation authority.
- Scene coordinates are finite SI metres in `north-east-tvd-m`.
- TypeScript renders validated scene values and manages view state only.
- No silent fallback from canonical trajectory results to duplicate desktop calculations.
- Every migration chunk must have focused tests, full affected-workspace verification, and a separate commit.
- Generated output, caches, and the legacy `work/` reference dump remain outside Git.

---

### Task 1: Adapt canonical trajectory results to 3Dmk

**Files:**
- Create: `desktop/src-tauri/src/trajectory_scene.rs`
- Modify: `desktop/src-tauri/src/main.rs`
- Modify: `desktop/src-tauri/Cargo.toml`
- Test: `desktop/src-tauri/src/trajectory_scene.rs`

**Interfaces:**
- Consumes: the canonical serialized `TrajectoryAnalysisResult` JSON contract produced by `engine`.
- Produces: `build_trajectory_scene(&TrajectoryAnalysisResult) -> Result<SceneDocumentV1, SceneError>`.

- [ ] Write a failing test proving plan and survey calculated stations become separate ordered scene layers, with NE-TVD coordinates and source provenance preserved.
- [ ] Run `cargo test -p wellforge-app trajectory_scene` and confirm failure because the adapter is absent.
- [ ] Define a strict local deserialization projection for the canonical result fields and implement the smallest adapter: plan path, survey path, station markers, Rust-side scene validation, and evidence-derived provenance.
- [ ] Run the focused test and then `cargo test --workspace` in `desktop`.
- [ ] Commit as `feat: adapt canonical trajectory results to 3dmk`.

### Task 2: Publish canonical trajectory scenes through Tauri

**Files:**
- Modify: `desktop/src-tauri/src/commands/mod.rs`
- Modify: `desktop/src-tauri/src/main.rs`
- Modify: `desktop/src/lib/ipc.ts`
- Test: `desktop/src-tauri/src/commands/mod.rs`

**Interfaces:**
- Consumes: validated trajectory result payload owned by Rust.
- Produces: typed `build_trajectory_scene` IPC response using `SceneDocumentV1`.

- [ ] Write a failing command test for a missing or malformed trajectory result.
- [ ] Implement explicit command input and typed error mapping without accepting raw UI geometry.
- [ ] Register the command once and validate the response at the TypeScript ingress.
- [ ] Run Rust command tests, frontend typecheck, and IPC tests.
- [ ] Commit as `feat: publish canonical trajectory scenes through tauri`.

### Task 3: Load real trajectory scenes in the desktop survey workspace

**Files:**
- Modify: `desktop/src/components/EmptyCanvas.tsx`
- Modify: `desktop/src/stores/scene.ts`
- Modify: `desktop/src/lib/ipc.ts`
- Test: `desktop/src/components/EmptyCanvas.test.tsx`

**Interfaces:**
- Consumes: the Tauri canonical trajectory scene command.
- Produces: a real scene state transition from selected project/result to 3Dmk viewport.

- [ ] Write a failing UI test that renders the empty state before load and the validated scene after load.
- [ ] Implement loading/error states and preserve the explicit empty state when no result exists.
- [ ] Ensure no trajectory positions are calculated in TypeScript.
- [ ] Run Vitest, typecheck, and production build.
- [ ] Commit as `feat: load canonical trajectory scenes in desktop workspace`.

### Task 4: Add wellbore visualization layers incrementally

**Files:**
- Modify: `desktop/crates/three_d/src/lib.rs`
- Modify: `desktop/src/lib/scene.ts`
- Modify: `desktop/src/components/ThreeDViewport.tsx`
- Test: corresponding Rust and Vitest contract tests

**Interfaces:**
- Consumes: validated scene layers from Rust adapters.
- Produces: selectable plan, survey, projection, target, and station layers without changing scene semantics.

- [ ] Add one primitive/layer capability at a time with a failing contract test first.
- [ ] Add layer metadata only when supplied by a canonical engine result.
- [ ] Keep camera, visibility, and selection state in the UI; keep coordinates and bounds in Rust.
- [ ] Run focused and full workspace tests after each layer.
- [ ] Commit each independently reviewable layer.

### Task 5: Converge duplicate desktop calculation crates

**Files:**
- Modify: `desktop/Cargo.toml`
- Modify: `desktop/src-tauri/Cargo.toml`
- Delete only after replacement tests pass: duplicate desktop `survey`, `bha`, `hydra`, `tnd`, and `wits` dependencies
- Modify: `docs/DESKTOP_MIGRATION.md`

**Interfaces:**
- Consumes: canonical `engine/crates` contracts and binaries.
- Produces: one calculation authority for desktop and workbook consumers.

- [ ] Add parity fixtures comparing each desktop-facing adapter with canonical engine output.
- [ ] Replace one duplicate dependency at a time and run both workspace suites.
- [ ] Remove duplicate crates only after no desktop code references them.
- [ ] Update migration documentation with the completed convergence map.
- [ ] Commit each completed discipline migration.

### Task 6: Release verification and GitHub integration

**Files:**
- Modify: `README.md`
- Modify: `docs/DESKTOP_MIGRATION.md`
- Modify: `docs/RUST_ENGINE_ROADMAP.md`

- [ ] Run desktop Rust tests and checks, canonical engine tests, frontend tests, typecheck, and build.
- [ ] Run bounded `graphify update C:\Development\WellForgeXL` after code/interface changes and keep generated graph output ignored.
- [ ] Review staged files and `git diff --check`.
- [ ] Commit the release documentation and push `main` only after all gates pass.
