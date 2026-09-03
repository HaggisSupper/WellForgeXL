# Engine Rust Migration Plan

## Overview
Migrate the remaining Windows-only engine layers from VBA/Office Scripts/Node wrappers into Rust CLIs and libraries, while preserving current workbook and file contracts. The goal is to keep Excel/workbook behavior stable while moving computation, validation, hashing, bridge generation, and build/test orchestration into Rust first.

## Scope Definition (CRITICAL)
### In Scope
- Inventory current non-Rust engine layers.
- Define a phased Rust migration order focused on highest-value engine layers first.
- Preserve existing workbook/file contracts during migration.
- Identify risk points and verification gates.
- Specify a minimal first milestone that can be completed and verified safely in this session.

### Out of Scope (DO NOT TOUCH)
- Rewriting workbook formulas or presentation logic.
- Changing engineering math contracts beyond the engine boundary.
- Packaging/releasing deliverables.
- Cross-platform support beyond Windows.
- Broad refactors unrelated to engine migration.

## Reference basis
Engineering math and industry conventions for the pending engines are grounded
in the external reference archive documented at [`docs/REFERENCE_ARCHIVE.md`](../../REFERENCE_ARCHIVE.md).
Torque & Drag draws from `G:\My Drive\Drilling Background\Torque and Drag\`
and `Pipe Handbooks\`; Hydraulics from `Hydraulics Models\`; BHA vibration
from `BHA Analysis\` and `Vibration Primer\`. The archive is read-only
reference material — the repo remains fully self-contained at build/test time.

## Current State Analysis
From the existing roadmap and workbook specs:
- Rust trajectory execution is already the pattern: core math stays pure, analysis composes results, and a CLI owns canonical JSON, hashes, diagnostics, and a tab-delimited bridge.
- Workbook-facing automation still includes VBA/Office Script JSON exchange, workbook mapping/state sheets, and Windows COM-based install/validation flows.
- Node-based build scripts still orchestrate workbook generation and validation.
- The directional workbook spec already treats JavaScript as workbook generation/verification glue, not the engine.
- Existing plan files show the repo’s methodology: strict contracts first, then pure Rust libraries, then a CLI, then workbook adapters.

## Implementation Phases
### Phase 1: Inventory and contract freeze
- **Goal**: Lock the current engine boundary and enumerate remaining non-Rust layers.
- **Steps**:
  1. [ ] Catalog every VBA, Office Script, Node wrapper, workbook integration point, and build/test script involved in engine execution.
  2. [ ] Classify each layer as compute, validation, bridge/serialization, workbook I/O, or orchestration.
  3. [ ] Capture the current contracts that must remain stable: request/result JSON, bridge grammar, workbook map/state sheets, CLI exit codes, and file naming.
- **Verification**: Static review against current specs/plans; no code behavior changes.

### Phase 2: Rust compute boundary first
- **Goal**: Move all deterministic engine calculations into Rust libraries before touching workbook adapters.
- **Steps**:
  1. [ ] Extract or finish Rust crates for analysis/math domains currently still implemented in VBA/JS/Node.
  2. [ ] Keep input/output structs strict and versioned; no workbook or file I/O in pure libraries.
  3. [ ] Preserve current request/result semantics so wrappers can swap backends without contract drift.
- **Verification**: Rust unit tests and contract tests for strict serialization and deterministic outputs.

### Phase 3: Rust CLI ownership of engine IO
- **Goal**: Move validation, hashing, bridge generation, diagnostics, and file orchestration into Rust CLIs.
- **Steps**:
  1. [ ] Implement Windows CLI entrypoints that replace Node wrapper responsibilities.
  2. [ ] Add atomic output, backup, schema validation, and result verification in Rust.
  3. [ ] Keep stdout bounded and preserve existing CLI/file contracts for workbook callers.
- **Verification**: CLI tests for exit codes, hash stability, bridge round-trips, and atomic file behavior.

### Phase 4: Workbook adapter slimming
- **Goal**: Reduce VBA/Office Scripts to thin clients that only export inputs, invoke Rust, and import validated outputs.
- **Steps**:
  1. [ ] Replace workbook-side compute paths with Rust-backed calls.
  2. [ ] Keep sheet mappings/state sheets intact to preserve workbook integration.
  3. [ ] Preserve last-good-value behavior and error surfaces.
- **Verification**: Workbook contract tests and Windows Excel smoke tests.

### Phase 5: Build/test script migration
- **Goal**: Move build/test orchestration to Rust-centric scripts with Windows-only execution paths.
- **Steps**:
  1. [ ] Replace Node wrappers used only for engine orchestration with Rust/PowerShell entrypoints where possible.
  2. [ ] Keep workbook generation scripts only where they still own workbook layout concerns.
  3. [ ] Standardize verification commands around Rust cargo tests and Windows workbook checks.
- **Verification**: Script-level tests plus end-to-end regression run.

## Recommended Order
1. Rust compute libraries
2. Rust CLI validation/hashing/bridge
3. Workbook adapters thin-out
4. Build/test script migration
5. Final removal of obsolete Node/VBA engine logic

## Risk Points
- Contract drift between old wrappers and new Rust types.
- Excel/VBA trust-center and COM automation dependencies.
- Hidden assumptions in workbook map/state sheets.
- Bridge grammar incompatibility or ordering changes.
- Hash canonicalization differences (especially numeric normalization).
- Windows path/quoting and atomic file replacement edge cases.

## Minimal First Milestone
- Freeze the inventory of remaining non-Rust engine layers.
- Add or update one Rust-side contract test around the highest-value engine boundary (likely trajectory/analysis CLI contract).
- Confirm the existing workbook/bridge contract remains unchanged.
- Verification: run the narrowest existing Rust contract test for the engine boundary and document the non-Rust layers still remaining.

## Initial Migration Slice Completed
- Rust owns the core trajectory and BHA engine CLIs and their strict request/result contracts.
- Added Windows-only `doctor` health checks to both engine CLIs.
- Added build metadata and lockfile assertions to the Rust engine health checks.
- Added structured validation failure output for the BHA CLI.
- Verified the engine CLI test surface with targeted Rust tests.

## Notes
This plan intentionally mirrors the repo’s existing methodology: strict contract tests first, pure Rust core next, CLI ownership third, workbook clients last. Wubba Lubba Dub Dub.
