# WITSML Hybrid Multibody BHA Release 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a Windows-native Rust CLI that consumes a WITSML-aligned BHA request, calculates verified static equilibrium and linearized frequency response with established numerical libraries, and returns value-only results to the BHA `.xlsm` workbook.

**Architecture:** A Rust 2024 workspace separates units/WITSML contracts, BHA mechanics, static analysis, modal analysis and CLI orchestration. `nalgebra`, `faer`, `parry3d-f64` and an acceptance-tested nonlinear solver own generic numerical operations; WellForge owns only drilling-domain assembly and constitutive behavior. VBA creates a bounded request, invokes the colocated CLI and imports schema-validated result values.

**Tech Stack:** Rust 2024, Cargo, serde, schemars, jsonschema, quick-xml, uom, nalgebra, faer, parry3d-f64, petgraph, rayon, clap, tracing, sha2, tempfile, Node test oracle, VBA7 and PowerShell.

**Spec:** `docs/superpowers/specs/2026-08-28-witsml-hybrid-multibody-bha-engine-design.md`

## Global Constraints

- Windows-first native execution; no Docker and no runtime network dependency.
- Production physics lives in Rust; Excel is a value-only client and reporting surface.
- SI is canonical inside the solver; source and display units remain provenance.
- WITSML object identity uses UUID/URI references; human labels are never relationship keys.
- Use established libraries for generic numerical algorithms; do not implement matrix factorizations, eigensolvers, optimizers, integrators, collision engines, quaternion packages or FFTs.
- Release 1 does not claim nonlinear time-domain vibration, frictional impact dynamics or calibrated DAT.
- Every result records source references, request hash, engine version, convergence evidence and result hash.
- No fallback from the Rust engine to workbook screening formulas.
- Every production behavior is introduced by a failing test and verified green before the next behavior.

---

## File structure

```text
engine/
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  deny.toml
  crates/
    wellforge-numerics-acceptance/
    wellforge-units/
    wellforge-witsml/
    wellforge-bha-contract/
    wellforge-bha-model/
    wellforge-bha-static/
    wellforge-bha-modal/
    wellforge-bha-cli/
    wellforge-bha-fixtures/
  fixtures/
    witsml/
    analytical/
    requests/
    expected/
schema/
  wellforge-bha-analysis-request.schema.json
  wellforge-bha-analysis-result.schema.json
VBA/
  WellForgeBhaEngine.bas
tools/
  Build-WellForgeBhaEngine.ps1
  Test-WellForgeBhaEngine.ps1
tests/
  bha_rust_engine_contract.test.mjs
  bha_workbook_engine_contract.test.mjs
```

## Task 1: Rust toolchain and numerical-library acceptance gate

**Files:**
- Create: `engine/rust-toolchain.toml`
- Create: `engine/Cargo.toml`
- Create: `engine/deny.toml`
- Create: `engine/crates/wellforge-numerics-acceptance/Cargo.toml`
- Create: `engine/crates/wellforge-numerics-acceptance/src/lib.rs`
- Create: `engine/crates/wellforge-numerics-acceptance/tests/library_acceptance.rs`
- Create: `tools/Build-WellForgeBhaEngine.ps1`
- Create: `docs/NUMERICAL_DEPENDENCIES.md`

**Interfaces:**
- Consumes: verified `cargo`, `rustc` and MSVC linker identities.
- Produces: locked numerical dependencies and `NumericsAcceptanceReport` proving required library operations.

- [ ] **Step 1: Verify the Windows Rust identity**

Run in PowerShell:

```powershell
$ErrorActionPreference = 'Stop'
rustc --version --verbose
cargo --version --verbose
Get-Command rustc,cargo | Format-List Name,Source,Version
```

Expected: both commands resolve to the intended Windows Rust installation. Record their complete output in the build log. Stop if either identity is unavailable.

- [ ] **Step 2: Create the workspace test before its implementation**

Write `library_acceptance.rs` with tests that require:

```rust
#[test]
fn libraries_cover_release_one_numerics() {
    let report = wellforge_numerics_acceptance::run();
    assert!(report.quaternion_round_trip);
    assert!(report.sparse_linear_solve);
    assert!(report.symmetric_eigenpairs);
    assert!(report.contact_distance_query);
    assert!(report.nonlinear_root_solve);
    assert!(report.licenses_allowed);
}
```

Set `rust-toolchain.toml` to:

```toml
[toolchain]
channel = "stable"
profile = "minimal"
components = ["clippy", "rustfmt"]
targets = ["x86_64-pc-windows-msvc"]
```

After the acceptance gate passes, replace `stable` with the exact release reported by `rustc --version --verbose` and commit that value.

- [ ] **Step 3: Run the test and verify RED**

Run:

```powershell
cargo test -p wellforge-numerics-acceptance --test library_acceptance
```

Expected: compilation fails because `run` and `NumericsAcceptanceReport` do not exist.

- [ ] **Step 4: Add and pin candidate dependencies**

From `engine` run:

```powershell
cargo add -p wellforge-numerics-acceptance nalgebra faer parry3d-f64 diffsol newton_rootfinder
cargo add -p wellforge-numerics-acceptance serde --features derive
cargo generate-lockfile
```

Use the versions resolved into `Cargo.lock`. Do not manually broaden resolved version requirements.

- [ ] **Step 5: Implement the acceptance operations**

Implement `run()` using public library APIs to prove:

- a `nalgebra::UnitQuaternion<f64>` rotates and inverse-rotates a vector;
- a `faer` linear solve reproduces a known 3-by-3 solution;
- a symmetric generalized eigen fixture returns the known first modes using library decomposition paths;
- `parry3d-f64` returns the expected signed separation/contact for cylinder-compatible primitives;
- the selected nonlinear library solves a three-variable residual from two initial states;
- every production dependency license is allowed by `deny.toml`.

If `newton_rootfinder` cannot consume the selected matrix backend or convergence controls, replace it in this task with a maintained nonlinear solver that passes the same test. Record the replacement and reason in `docs/NUMERICAL_DEPENDENCIES.md`.

- [ ] **Step 6: Verify GREEN and dependency policy**

Run:

```powershell
cargo test -p wellforge-numerics-acceptance --test library_acceptance
$env:CARGO_HOME = (Resolve-Path '.\.cargo-tools')
cargo install cargo-deny --locked
& "$env:CARGO_HOME\bin\cargo-deny.exe" check licenses bans sources
```

Expected: tests pass and dependency policy exits zero.

- [ ] **Step 7: Commit**

```powershell
git add engine tools/Build-WellForgeBhaEngine.ps1 docs/NUMERICAL_DEPENDENCIES.md
git commit -m "build: establish BHA numerical library gate"
```

## Task 2: Energistics units and WITSML source identity

**Files:**
- Create: `engine/crates/wellforge-units/Cargo.toml`
- Create: `engine/crates/wellforge-units/src/lib.rs`
- Create: `engine/crates/wellforge-units/tests/energistics_units.rs`
- Create: `engine/crates/wellforge-witsml/Cargo.toml`
- Create: `engine/crates/wellforge-witsml/src/lib.rs`
- Create: `engine/crates/wellforge-witsml/src/identity.rs`
- Create: `engine/crates/wellforge-witsml/tests/source_identity.rs`
- Create: `engine/crates/wellforge-bha-fixtures/Cargo.toml`
- Create: `engine/crates/wellforge-bha-fixtures/src/lib.rs`
- Create: `engine/fixtures/witsml/README.md`

**Interfaces:**
- Consumes: WITSML 2.0 UUID/URI semantics and the checked-in unit registry.
- Produces: `Quantity`, `QuantityClass`, `SourceObjectRef`, `WitsmlObjectType`, strict reference validation and shared deterministic fixture builders.

- [ ] **Step 1: Write failing unit and identity tests**

Required tests:

```rust
#[test]
fn rejects_force_unit_for_pressure_quantity() {
    let result = Quantity::parse(10.0, "kN", QuantityClass::Pressure);
    assert!(matches!(result, Err(UnitError::WrongQuantityClass { .. })));
}

#[test]
fn witsml_reference_requires_uuid_and_supported_type() {
    let result = SourceObjectRef::new("not-a-uuid", WitsmlObjectType::Tubular, None);
    assert!(matches!(result, Err(IdentityError::InvalidUuid(_))));
}
```

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p wellforge-units -p wellforge-witsml
```

Expected: missing types and functions.

- [ ] **Step 3: Implement dimension-safe quantities**

Use `uom` for dimensions and SI conversion. Add an explicit Energistics symbol table that maps accepted wire symbols to a single quantity class and conversion. Preserve original symbol/value separately from canonical SI.

- [ ] **Step 4: Implement WITSML identity types**

Define:

```rust
pub struct SourceObjectRef {
    pub uuid: uuid::Uuid,
    pub uri: Option<String>,
    pub object_type: WitsmlObjectType,
    pub content_hash: String,
    pub citation_name: String,
    pub source_system: String,
}
```

Restrict `WitsmlObjectType` to `Well`, `Wellbore`, `Trajectory`, `WellboreGeometry`, `Tubular`, `BhaRun`, `Log`, `ChannelSet` and `Channel` for Release 1.

Implement `wellforge-bha-fixtures` with deterministic UUID constants and builders such as `minimal_source_set()` and `minimal_request_without(WitsmlObjectType)`. Test crates must import these builders rather than reproduce request construction.

- [ ] **Step 5: Verify GREEN**

Run:

```powershell
cargo test -p wellforge-units -p wellforge-witsml
```

- [ ] **Step 6: Commit**

```powershell
git add engine/crates/wellforge-units engine/crates/wellforge-witsml engine/crates/wellforge-bha-fixtures engine/fixtures/witsml
git commit -m "feat: add Energistics units and WITSML identity"
```

## Task 3: Versioned BHA request and result contracts

**Files:**
- Create: `engine/crates/wellforge-bha-contract/Cargo.toml`
- Create: `engine/crates/wellforge-bha-contract/src/lib.rs`
- Create: `engine/crates/wellforge-bha-contract/src/request.rs`
- Create: `engine/crates/wellforge-bha-contract/src/result.rs`
- Create: `engine/crates/wellforge-bha-contract/src/validation.rs`
- Create: `engine/crates/wellforge-bha-contract/tests/contract.rs`
- Generate: `schema/wellforge-bha-analysis-request.schema.json`
- Generate: `schema/wellforge-bha-analysis-result.schema.json`

**Interfaces:**
- Consumes: `Quantity` and `SourceObjectRef`.
- Produces: `BhaAnalysisRequest`, `BhaAnalysisResult`, `validate_request` and generated schemas.

- [ ] **Step 1: Write failing contract tests**

Tests must reject a missing Tubular reference, disconnected components, duplicate UUIDs, invalid increasing-depth order, impossible OD/ID geometry and an unsupported contract major version.

```rust
#[test]
fn request_requires_complete_witsml_source_set() {
    let request = fixtures::minimal_request_without(WitsmlObjectType::Tubular);
    let errors = validate_request(&request).unwrap_err();
    assert!(errors.iter().any(|e| e.code == "WF-BHA-CONTRACT-004"));
}
```

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p wellforge-bha-contract
```

- [ ] **Step 3: Implement immutable request/result types**

Use `serde` and `schemars`. Use tagged enums for component representation, boundary conditions, load types, excitation types and result states. Set `deny_unknown_fields` on calculation-authoritative structures; place vendor extensions only in a namespaced extension map.

- [ ] **Step 4: Generate and validate schemas**

Add a test that regenerates schemas to a temporary directory and byte-compares normalized output against the checked-in files. Validate fixture JSON with `jsonschema`.

- [ ] **Step 5: Verify GREEN**

Run:

```powershell
cargo test -p wellforge-bha-contract
```

- [ ] **Step 6: Commit**

```powershell
git add engine/crates/wellforge-bha-contract schema/wellforge-bha-analysis-*.schema.json
git commit -m "feat: define immutable BHA analysis contracts"
```

## Task 4: WITSML offline projection adapter

**Files:**
- Create: `engine/crates/wellforge-witsml/src/xml.rs`
- Create: `engine/crates/wellforge-witsml/src/projection.rs`
- Create: `engine/crates/wellforge-witsml/tests/projection.rs`
- Add: `engine/fixtures/witsml/*.xml`
- Create: `engine/fixtures/witsml/fixture-provenance.json`

**Interfaces:**
- Consumes: license-permitted WITSML 2.0 fixture objects.
- Produces: `project_analysis_sources(objects: &[WitsmlObject]) -> Result<AnalysisSources, ProjectionError>`.

- [ ] **Step 1: Add sanitized fixture provenance**

For each XML fixture record source, license, original hash, sanitization actions and resulting hash. Do not include a fixture whose redistribution rights are unclear.

- [ ] **Step 2: Write failing projection tests**

Tests require explicit parent Wellbore references, Tubular-to-BhaRun linkage, trajectory station ordering, WellboreGeometry section extraction and preservation of source UUIDs and units.

- [ ] **Step 3: Verify RED**

Run:

```powershell
cargo test -p wellforge-witsml --test projection
```

- [ ] **Step 4: Implement XML parsing and projection**

Use `quick-xml` and typed deserialization for the supported subset. Reject unsupported or ambiguous structures with stable diagnostics; do not use XPath-like string scraping.

- [ ] **Step 5: Verify GREEN and round trip**

Run:

```powershell
cargo test -p wellforge-witsml
```

Expected: projection and normalized round-trip tests pass with source UUIDs unchanged.

- [ ] **Step 6: Commit**

```powershell
git add engine/crates/wellforge-witsml engine/fixtures/witsml
git commit -m "feat: project WITSML BHA source objects"
```

## Task 5: Hybrid multibody component graph and contact geometry

**Files:**
- Create: `engine/crates/wellforge-bha-model/Cargo.toml`
- Create: `engine/crates/wellforge-bha-model/src/lib.rs`
- Create: `engine/crates/wellforge-bha-model/src/component.rs`
- Create: `engine/crates/wellforge-bha-model/src/frame.rs`
- Create: `engine/crates/wellforge-bha-model/src/graph.rs`
- Create: `engine/crates/wellforge-bha-model/src/contact.rs`
- Create: `engine/crates/wellforge-bha-model/tests/model_graph.rs`

**Interfaces:**
- Consumes: validated `BhaAnalysisRequest`.
- Produces: `BhaModel`, `DofMap`, `ContactCandidate` and `assemble_model`.

- [ ] **Step 1: Write failing graph and geometry tests**

Test component ordering, connection orientation, quaternion normalization, disconnected-graph rejection, mass conservation and cylinder clearance/contact queries.

Add the assertion dependency explicitly:

```powershell
cargo add -p wellforge-bha-model --dev approx
```

```rust
#[test]
fn cylinder_contact_uses_parry_distance_query() {
    let candidate = fixtures::offset_collar_in_hole(0.004);
    let query = candidate.query().unwrap();
    assert_relative_eq!(query.clearance_m, 0.004, epsilon = 1.0e-12);
}
```

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p wellforge-bha-model
```

- [ ] **Step 3: Implement frames and component representations**

Use `nalgebra::Isometry3<f64>`, `UnitQuaternion<f64>`, `Vector3<f64>` and `Matrix3<f64>`. Component representations are `Rigid`, `Beam`, and `ModalFlexible`; Release 1 accepts `ModalFlexible` metadata but rejects modal solve requests until a validated modal basis is supplied.

- [ ] **Step 4: Implement graph validation**

Use `petgraph` to prove a single ordered mechanical path from bit to top boundary, reject cycles and verify joint endpoints.

- [ ] **Step 5: Implement contact geometry queries**

Use `parry3d-f64` distance/contact queries for broad/narrow phase geometry. WellForge converts geometry results into drilling normal-contact residuals; it does not reimplement geometric intersection algorithms.

- [ ] **Step 6: Verify GREEN**

Run:

```powershell
cargo test -p wellforge-bha-model
```

- [ ] **Step 7: Commit**

```powershell
git add engine/crates/wellforge-bha-model
git commit -m "feat: assemble hybrid BHA component graph"
```

## Task 6: Flexible beam residual, mass and tangent assembly

**Files:**
- Create: `engine/crates/wellforge-bha-static/Cargo.toml`
- Create: `engine/crates/wellforge-bha-static/src/lib.rs`
- Create: `engine/crates/wellforge-bha-static/src/beam.rs`
- Create: `engine/crates/wellforge-bha-static/src/assembly.rs`
- Create: `engine/crates/wellforge-bha-static/tests/beam_analytics.rs`
- Add: `engine/fixtures/analytical/beam-cases.json`

**Interfaces:**
- Consumes: `BhaModel` and `DofMap`.
- Produces: `ElementContribution { residual, tangent, mass }` and `assemble_system` using `faer` matrices.

- [ ] **Step 1: Write failing analytical tests**

Add axial extension, pure torsion, cantilever tip displacement/rotation, simply supported bending and consistent-mass tests. Each fixture declares dimensions, load, analytical answer and tolerance.

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p wellforge-bha-static --test beam_analytics
```

- [ ] **Step 3: Implement the drilling-domain element formulation**

Implement the approved geometrically nonlinear beam residual and consistent tangent using `nalgebra` for local transforms and `faer` for dynamic matrices. All factorizations and linear solves call library APIs. Central finite differences may appear only in a test that independently checks the analytical tangent.

- [ ] **Step 4: Verify tangent consistency**

Add a test comparing directional derivatives of the residual against the assembled tangent over deterministic perturbations. Require relative error below the fixture tolerance.

- [ ] **Step 5: Verify GREEN and mesh convergence**

Run:

```powershell
cargo test -p wellforge-bha-static --test beam_analytics
```

- [ ] **Step 6: Commit**

```powershell
git add engine/crates/wellforge-bha-static engine/fixtures/analytical/beam-cases.json
git commit -m "feat: assemble flexible BHA beam mechanics"
```

## Task 7: Static equilibrium and active normal contact

**Files:**
- Create: `engine/crates/wellforge-bha-static/src/equilibrium.rs`
- Create: `engine/crates/wellforge-bha-static/src/boundary.rs`
- Create: `engine/crates/wellforge-bha-static/src/normal_contact.rs`
- Create: `engine/crates/wellforge-bha-static/tests/static_equilibrium.rs`
- Add: `engine/fixtures/analytical/static-contact-cases.json`

**Interfaces:**
- Consumes: assembled residual/tangent, boundaries and contact candidates.
- Produces: `solve_static(&BhaModel, &StaticCase) -> Result<StaticSolution, StaticSolveError>`.

- [ ] **Step 1: Write failing equilibrium tests**

Cover gravity sag, buoyed weight, Euler buckling load, load stepping, contact activation/deactivation, coordinate-frame invariance, singular-model rejection and deterministic non-convergence diagnostics.

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p wellforge-bha-static --test static_equilibrium
```

- [ ] **Step 3: Integrate the accepted nonlinear solver**

Adapt the BHA residual and tangent to the nonlinear library selected in Task 1. WellForge owns load stepping and active-contact state transitions; the library owns Newton/Broyden/LM iteration, convergence bookkeeping and linear-solver dispatch. Do not duplicate the library iteration algorithm.

- [ ] **Step 4: Implement frictionless normal contact**

Use `parry3d-f64` for closest features, normals and penetration. Add the normal penalty/constraint contribution with explicit stiffness and recovery tolerances. Report every active contact and complementarity residual.

- [ ] **Step 5: Verify GREEN and force balance**

Run:

```powershell
cargo test -p wellforge-bha-static
```

Expected: all static fixtures pass; normalized force residual is at or below `1e-8` unless a stricter fixture tolerance applies.

- [ ] **Step 6: Commit**

```powershell
git add engine/crates/wellforge-bha-static engine/fixtures/analytical/static-contact-cases.json
git commit -m "feat: solve BHA static equilibrium and contact"
```

## Task 8: Linearized modal and harmonic-response analysis

**Files:**
- Create: `engine/crates/wellforge-bha-modal/Cargo.toml`
- Create: `engine/crates/wellforge-bha-modal/src/lib.rs`
- Create: `engine/crates/wellforge-bha-modal/src/eigen.rs`
- Create: `engine/crates/wellforge-bha-modal/src/harmonic.rs`
- Create: `engine/crates/wellforge-bha-modal/src/campbell.rs`
- Create: `engine/crates/wellforge-bha-modal/tests/modal_analytics.rs`
- Add: `engine/fixtures/analytical/modal-cases.json`

**Interfaces:**
- Consumes: converged `StaticSolution`, tangent stiffness, mass, damping and typed excitations.
- Produces: `solve_modal`, `solve_harmonic_response`, `build_campbell_map`.

- [ ] **Step 1: Write failing modal tests**

Require analytical uniform-beam frequencies, modal mass normalization, orthogonality, pre-stress frequency shift, contact-stiffness shift and single-degree-of-freedom harmonic response.

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p wellforge-bha-modal
```

- [ ] **Step 3: Implement eigenanalysis with library decompositions**

Use `faer` decomposition/eigen APIs selected in Task 1. Filter constrained and rigid modes by declared residual and modal-mass criteria. Never add a handwritten Lanczos, QR or inverse-iteration routine.

- [ ] **Step 4: Implement complex harmonic response**

Use library complex matrices/factorization for `(-ω²M + iωC + K)x = F`. Excitations must carry origin, order, amplitude source and confidence.

- [ ] **Step 5: Implement deterministic scenario maps**

Use `rayon` for independent WOB/RPM cases only after serial/parallel byte-normalized results compare equal. Sort outputs by case UUID and operating coordinates before serialization.

- [ ] **Step 6: Verify GREEN**

Run:

```powershell
cargo test -p wellforge-bha-modal
```

- [ ] **Step 7: Commit**

```powershell
git add engine/crates/wellforge-bha-modal engine/fixtures/analytical/modal-cases.json
git commit -m "feat: add preloaded BHA modal and frequency analysis"
```

## Task 9: Deterministic CLI and calculation evidence

**Files:**
- Create: `engine/crates/wellforge-bha-cli/Cargo.toml`
- Create: `engine/crates/wellforge-bha-cli/src/main.rs`
- Create: `engine/crates/wellforge-bha-cli/src/commands.rs`
- Create: `engine/crates/wellforge-bha-cli/src/evidence.rs`
- Create: `engine/crates/wellforge-bha-cli/tests/cli.rs`
- Add: `engine/fixtures/requests/release-one-minimal.json`
- Add: `engine/fixtures/expected/release-one-minimal.result.json`

**Interfaces:**
- Consumes: validated request JSON.
- Produces: atomic result JSON, JSONL diagnostics, process exit codes and calculation evidence.

- [ ] **Step 1: Write failing CLI integration tests**

Use `assert_cmd` to require `validate`, `solve-static`, `solve-modal`, `run`, `schema` and `version --json`. Assert failure exit codes for invalid schema, non-convergence and hash mismatch.

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p wellforge-bha-cli
```

- [ ] **Step 3: Implement commands with clap**

Each command parses explicit paths, never shell fragments. Use `tempfile` for sibling output, `sha2` for hashes, `tracing` for structured diagnostics and atomic rename after complete validation.

- [ ] **Step 4: Implement normalized deterministic serialization**

Sort keyed collections and scenario arrays before `serde_json` serialization. Canonicalize negative zero and reject non-finite numbers. Record compiler, target, engine version, dependency lock hash and solver settings.

- [ ] **Step 5: Verify GREEN and byte repeatability**

Run:

```powershell
cargo test -p wellforge-bha-cli
cargo run -p wellforge-bha-cli -- run --input fixtures/requests/release-one-minimal.json --output target/run-1.json
cargo run -p wellforge-bha-cli -- run --input fixtures/requests/release-one-minimal.json --output target/run-2.json --no-backup
Compare-Object (Get-Content target/run-1.json) (Get-Content target/run-2.json)
```

Expected: no differences.

- [ ] **Step 6: Commit**

```powershell
git add engine/crates/wellforge-bha-cli engine/fixtures/requests engine/fixtures/expected
git commit -m "feat: expose deterministic BHA solver CLI"
```

## Task 10: VBA-to-Rust engine orchestration

**Files:**
- Create: `VBA/WellForgeBhaEngine.bas`
- Modify: `VBA/WellForgeBha.bas`
- Modify: `VBA/WellForgeCore.bas`
- Modify: `tools/Build-WellForgeVbaSuite.ps1`
- Create: `tests/bha_rust_engine_contract.test.mjs`
- Create: `tests/bha_workbook_engine_contract.test.mjs`

**Interfaces:**
- Consumes: `wellforge-bha.exe` and workbook-owned input ranges.
- Produces: bounded request JSON, imported result values, chart refresh and persistent diagnostics.

- [ ] **Step 1: Write failing source-contract tests**

Assert that VBA:

- resolves only the executable colocated with the suite;
- verifies its SHA-256 against the package manifest;
- passes input/output paths as quoted arguments without `cmd.exe` interpolation;
- enforces timeout and process exit status;
- validates request hash and engine version before writing cells;
- writes no worksheet formulas;
- never calls `WF_CalcBHA` screening calculations as fallback.

- [ ] **Step 2: Verify RED**

Run:

```powershell
node --test tests/bha_rust_engine_contract.test.mjs tests/bha_workbook_engine_contract.test.mjs
```

- [ ] **Step 3: Implement request export**

Create a typed, bounded JSON request from named workbook tables and WITSML source-reference cells. Reuse the existing safe JSON codec and atomic-file conventions.

- [ ] **Step 4: Implement process execution and result import**

Use `WScript.Shell.Exec` with a bounded polling loop, timeout and captured stdout/stderr. On success, validate the result contract and hashes, capture old output values, write in bounded batches and restore on any write failure.

- [ ] **Step 5: Implement failure states**

Display `ENGINE UNAVAILABLE`, `ENGINE HASH MISMATCH`, `INVALID REQUEST`, `NON-CONVERGED` or `INVALID RESULT`. Preserve the last accepted values, mark them stale and display the full JSONL log path.

- [ ] **Step 6: Verify GREEN**

Run:

```powershell
node --test tests/bha_rust_engine_contract.test.mjs tests/bha_workbook_engine_contract.test.mjs tests/vba_engine_contract.test.mjs
node tools/lint_vba.mjs
```

- [ ] **Step 7: Commit**

```powershell
git add VBA tools/Build-WellForgeVbaSuite.ps1 tests/bha_*_engine_contract.test.mjs
git commit -m "feat: connect BHA workbook to Rust engine"
```

## Task 11: BHA engineering decision surfaces

**Files:**
- Modify: `src/build_bha.mjs`
- Modify: `VBA/WellForgeBhaEngine.bas`
- Modify: `docs/DRILLING_CHART_STANDARD.md`
- Create: `tests/bha_rust_visualization_contract.test.mjs`

**Interfaces:**
- Consumes: static/modal value arrays from `BhaAnalysisResult`.
- Produces: deformed-shape, contact, stress, modal, Campbell, FRF and critical-speed visualizations.

- [ ] **Step 1: Write failing visualization tests**

Require chart-helper blocks for centreline/contact, bending/stress/utilization, bit side force/tilt, mode shapes, Campbell orders, FRF heatmap and WOB/RPM critical-speed map. Assert that every chart series is value-backed and unit-linked.

- [ ] **Step 2: Verify RED**

Run:

```powershell
node --test tests/bha_rust_visualization_contract.test.mjs
```

- [ ] **Step 3: Build the value-only decision surfaces**

Use consistent distance-from-bit orientation, selected-component screen readers, observed-versus-predicted semantics and stable risk colors. Never label fixture data as field measurements.

- [ ] **Step 4: Verify GREEN and render**

Run:

```powershell
node src/build_bha.mjs
node --test tests/bha_rust_visualization_contract.test.mjs tests/bha.test.mjs
```

Render every changed visible BHA sheet and inspect for formula errors, empty series, clipped axes and inconsistent units.

- [ ] **Step 5: Commit**

```powershell
git add src/build_bha.mjs VBA/WellForgeBhaEngine.bas docs/DRILLING_CHART_STANDARD.md tests/bha_rust_visualization_contract.test.mjs
git commit -m "feat: add BHA static and frequency decision surfaces"
```

## Task 12: Windows build, regression and release evidence

**Files:**
- Complete: `tools/Build-WellForgeBhaEngine.ps1`
- Create: `tools/Test-WellForgeBhaEngine.ps1`
- Modify: `README.md`
- Modify: `RELEASE_NOTES.md`
- Modify: `docs/VBA_ENGINE_GUIDE.md`
- Create: `docs/BHA_ENGINE_VALIDATION.md`

**Interfaces:**
- Consumes: complete Release 1 source and fixtures.
- Produces: signed-or-hashed Windows executable, `.xlsm` suite, validation report and clean ZIP.

- [ ] **Step 1: Complete the isolated Windows builder**

The builder must:

- verify `rustc`, `cargo`, Excel and PowerShell identities;
- run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, all Rust tests and dependency policy;
- build `wellforge-bha.exe` with `cargo build --release --locked`;
- hash the executable and place it beside the workbook outputs;
- compile the `.xlsm` files;
- execute the BHA engine self-test through Excel;
- pause on success or failure unless `-NoPause` is explicit;
- write JSONL logs and a concise terminal summary.

- [ ] **Step 2: Write the release test before final packaging**

`Test-WellForgeBhaEngine.ps1` must validate one successful static/modal fixture and one expected non-converged fixture through both the CLI and compiled workbook. It must compare request/result hashes and reject residual formulas.

- [ ] **Step 3: Run all deterministic gates**

Run:

```powershell
Set-Location engine
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
Set-Location ..
node --test tests/*.test.mjs
node tools/lint_vba.mjs
powershell -ExecutionPolicy Bypass -File tools/Test-WellForgeBhaEngine.ps1 -NoPause
```

Expected: every command exits zero and reports zero failed tests.

- [ ] **Step 4: Produce validation evidence**

Document fixture equations, sources, tolerances, solver results, mesh-convergence evidence, dependency versions, license results and known Release 1 applicability limits in `docs/BHA_ENGINE_VALIDATION.md`.

- [ ] **Step 5: Package and verify exact bytes**

Create a clean ZIP containing source, schemas, fixtures, locked dependencies, executable, five `.xlsm` outputs, tests and documentation. Exclude renders, logs, Git metadata, build caches and inspection dumps. Generate an internal SHA-256 manifest, extract to a new directory, verify every checksum and rerun the focused release tests against the extraction.

- [ ] **Step 6: Commit**

```powershell
git add tools README.md RELEASE_NOTES.md docs engine/Cargo.lock
git commit -m "release: deliver BHA static and frequency engine"
```

## Plan self-review

- Every Release 1 specification requirement maps to a task.
- Numerical libraries own general algebra, decompositions, root solving and geometry queries.
- Domain implementations are limited to BHA mechanics and contracts.
- Nonlinear time-domain and DAT crates are not scaffolded prematurely.
- WITSML/ETP transport is not misrepresented as complete.
- Windows and workbook failure paths are explicit.
- All production tasks contain a red-green verification cycle.
