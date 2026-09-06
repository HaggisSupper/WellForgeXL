# T&D Spatial Curvature and Stiff-String Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace inclination-only T&D curvature with full spatial dogleg curvature and add bounded stiff-string/contact refinement on severe intervals while preserving the whole-well soft-string authority.

**Architecture:** Keep `wellforge-torque-drag-core` as the whole-well soft solver and classifier. Extend the request/result contracts compatibly, then add a focused `wellforge-torque-drag-stiff` crate that consumes the request plus accepted soft result and returns optional interval refinements. The CLI/core orchestration attaches stiff results only when geometry exists and auto refinement selects intervals.

**Tech Stack:** Rust 2024, serde/schemars, uuid v5, nalgebra/faer, existing WellForge contract/evidence patterns.

**Spec:** `docs/superpowers/specs/2026-09-06-tnd-spatial-stiff-priorities-design.md`

## Global Constraints

- Preserve existing soft-string whole-well coverage and API 7G calculation authority.
- Existing request fixtures without hole geometry must continue to deserialize.
- Existing soft-only result serialization must remain unchanged when `stiff_string` is absent.
- Stiff refinement never extrapolates loads outside selected intervals.
- No contact values are fabricated when geometry is absent or the refinement fails.
- All dimensional values remain canonical SI.
- No Docker and no Python solver path.

---

### Task 1: Full spatial dogleg curvature

**Files:**
- Modify: `engine/crates/wellforge-torque-drag-core/src/lib.rs`
- Modify: `engine/crates/wellforge-torque-drag-core/tests/soft_string.rs`

**Interfaces:**
- Consumes: adjacent `TnDTrajectoryStation { inclination_rad, azimuth_rad, md_m }`.
- Produces: `spatial_dogleg_rad_per_m(upper, lower) -> f64` internally; `StationResult.dogleg_rad_m` carries this value.

- [ ] **Step 1: Write failing pure-turn and combined-dogleg tests**

Add tests constructing trajectories with constant inclination and changing azimuth, then assert `dogleg_rad_m > 0` and matches:

```rust
let beta = (i1.cos() * i2.cos() + i1.sin() * i2.sin() * (a2-a1).cos()).acos();
let expected = beta / delta_md;
```

Include azimuth wrap from 359° to 1° and assert the dot-product formula remains continuous.

- [ ] **Step 2: Run the T&D core test target and verify RED**

Run:

```bash
cd engine
cargo test -p wellforge-torque-drag-core --locked
```

Expected: new pure-turn test fails because current curvature uses only inclination difference.

- [ ] **Step 3: Implement spatial dogleg helper**

Use direction-cosine dot product with clamp to `[-1,1]`:

```rust
fn spatial_dogleg_rad_per_m(upper: &TnDTrajectoryStation, lower: &TnDTrajectoryStation) -> f64 {
    let dmd = lower.md_m - upper.md_m;
    if dmd <= 0.0 { return 0.0; }
    let cosine = upper.inclination_rad.cos() * lower.inclination_rad.cos()
        + upper.inclination_rad.sin() * lower.inclination_rad.sin()
            * (lower.azimuth_rad - upper.azimuth_rad).cos();
    cosine.clamp(-1.0, 1.0).acos() / dmd
}
```

Use this value in the existing normal-load calculation and result field.

- [ ] **Step 4: Run focused tests and formatter**

```bash
cargo fmt --all -- --check
cargo test -p wellforge-torque-drag-core --locked
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add engine/crates/wellforge-torque-drag-core
git commit -m "fix: use spatial dogleg in torque and drag"
```

### Task 2: Compatible hole geometry and solver options

**Files:**
- Modify: `engine/crates/wellforge-torque-drag-contract/src/request.rs`
- Modify: `engine/crates/wellforge-torque-drag-contract/src/validation.rs`
- Modify: `engine/crates/wellforge-torque-drag-contract/src/lib.rs`
- Test: `engine/crates/wellforge-torque-drag-contract/tests/stiff_contract.rs`

**Interfaces:**
- Produces: `TnDHoleSection`, `StiffStringMode`, `TnDSolverOptions`.
- Adds defaulted `hole: Vec<TnDHoleSection>` and `solver: TnDSolverOptions` to `TnDAnalysisRequest`.

- [ ] **Step 1: Write failing compatibility/validation tests**

Tests must prove:

```rust
// legacy JSON without hole/solver deserializes
assert!(serde_json::from_str::<TnDAnalysisRequest>(legacy_json).is_ok());

// invalid hole diameter is rejected
assert!(validate_request(&request_with_negative_hole).is_err());
```

Also validate positive max element length, transition buffer, contact penalty and severity normal-load threshold.

- [ ] **Step 2: Run contract tests and verify RED**

```bash
cargo test -p wellforge-torque-drag-contract --locked
```

Expected: types/fields do not exist.

- [ ] **Step 3: Add request types with defaults**

Use:

```rust
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StiffStringMode { Disabled, Auto }

impl Default for StiffStringMode { fn default() -> Self { Self::Auto } }
```

`TnDSolverOptions::default()` values:

```text
mode = auto
severity_normal_load_n_m = 25000
severity_buckling_margin_n = 0
transition_buffer_m = 30
max_element_length_m = 3
contact_penalty_n_m = 1e7
```

- [ ] **Step 4: Validate geometry/options**

Reject non-finite/non-positive diameter; overlapping hole sections; negative transition buffer; non-positive mesh length/contact penalty/severity normal threshold.

- [ ] **Step 5: Run contract tests**

```bash
cargo fmt --all -- --check
cargo test -p wellforge-torque-drag-contract --locked
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add engine/crates/wellforge-torque-drag-contract
git commit -m "feat: add T&D stiff-string geometry contract"
```

### Task 3: Severe-interval classifier and result contract

**Files:**
- Modify: `engine/crates/wellforge-torque-drag-contract/src/result.rs`
- Modify: `engine/crates/wellforge-torque-drag-core/src/lib.rs`
- Test: `engine/crates/wellforge-torque-drag-core/tests/severity.rs`

**Interfaces:**
- Produces: `StiffIntervalReason`, `StiffIntervalCandidate` and optional `TnDAnalysisResult.stiff_string` result envelope.
- Internal classifier: `classify_stiff_intervals(request, soft_result) -> Vec<StiffIntervalCandidate>`.

- [ ] **Step 1: Write RED tests for classifier**

Create deterministic synthetic soft results that trigger normal load and buckling independently. Assert adjacent triggers merge and buffer/clamp behavior is deterministic.

- [ ] **Step 2: Add optional result structures**

Add `StiffStringResult`, `StiffIntervalResult`, `StiffNodeResult`, `StiffConvergence` and `StiffIntervalReason`. Mark top-level optional field:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub stiff_string: Option<StiffStringResult>
```

- [ ] **Step 3: Implement deterministic classifier**

Merge adjacent triggered station spans, expand by transition buffer, and derive interval UUIDv5 from analysis ID + canonical decimal MD bounds.

- [ ] **Step 4: Verify soft-only serialization compatibility**

Existing fixture/result tests must remain byte-identical when `stiff_string == None`.

- [ ] **Step 5: Commit**

```bash
git add engine/crates/wellforge-torque-drag-contract engine/crates/wellforge-torque-drag-core
git commit -m "feat: classify T&D stiff refinement intervals"
```

### Task 4: Stiff-string/contact solver crate

**Files:**
- Create: `engine/crates/wellforge-torque-drag-stiff/Cargo.toml`
- Create: `engine/crates/wellforge-torque-drag-stiff/src/lib.rs`
- Create: `engine/crates/wellforge-torque-drag-stiff/tests/stiff_contact.rs`
- Modify: `engine/Cargo.toml`

**Interfaces:**
- Consumes: `&TnDAnalysisRequest`, `&TnDAnalysisResult`, `&StiffIntervalCandidate`.
- Produces: `Result<StiffIntervalResult, StiffSolveError>`.

- [ ] **Step 1: Write RED straight/penetration/contact tests**

Straight centered geometry: zero penetration/contact, converged.

Synthetic lateral forcing with clearance exceeded: positive contact force, finite bending stress, final penetration below tolerance.

Missing hole coverage: typed `StiffSolveError::MissingHoleGeometry`.

- [ ] **Step 2: Assemble local FE mesh**

Interpolate component properties at nodes and use:

```text
A = pi(OD^2-ID^2)/4
I = pi(OD^4-ID^4)/64
```

Euler-Bernoulli 2-DOF/node stiffness and geometric stiffness reuse the validated BHA matrix form.

- [ ] **Step 3: Add inherited distributed load**

Interpolate soft `normal_load_n_m` to the stiff mesh and combine with buoyed transverse gravity. Preserve soft interval boundary displacement at zero for Release 1 while inheriting axial compression through geometric stiffness.

- [ ] **Step 4: Add unilateral contact iteration**

At each translational node:

```rust
let penetration = displacement.abs() - radial_clearance;
let contact = if penetration > 0.0 {
    -displacement.signum() * contact_penalty_n_m * penetration
} else { 0.0 };
```

Iterate solve/contact-force update up to 50 iterations with normalized residual tolerance `1e-8`. If not converged, return a result with `StiffConvergence::NotConverged`, never fabricated convergence.

- [ ] **Step 5: Calculate nodal moment/stress**

Use discrete curvature, `M=EI*kappa`, `sigma=M*(OD/2)/I` and report clearance/contact.

- [ ] **Step 6: Run crate tests**

```bash
cargo fmt --all -- --check
cargo test -p wellforge-torque-drag-stiff --locked
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add engine/Cargo.toml engine/crates/wellforge-torque-drag-stiff
git commit -m "feat: add bounded T&D stiff contact refinement"
```

### Task 5: Integrate stiff refinement into the T&D calculation

**Files:**
- Modify: `engine/crates/wellforge-torque-drag-core/Cargo.toml` or orchestration crate as required to avoid a dependency cycle.
- If cycle would occur, create: `engine/crates/wellforge-torque-drag-analysis/Cargo.toml`
- Create/modify: `engine/crates/wellforge-torque-drag-analysis/src/lib.rs`
- Modify: `engine/crates/wellforge-torque-drag-cli/Cargo.toml`
- Modify: `engine/crates/wellforge-torque-drag-cli/src/main.rs`
- Modify: `engine/Cargo.toml`

**Interfaces:**
- `analyze_torque_drag(request) -> Result<TnDAnalysisResult, AnalysisError>` performs soft solve, classification, optional stiff refinement, then final status/warnings.

- [ ] **Step 1: Write integration RED test**

Request with hole geometry and severe soft interval must return `stiff_string = Some(...)`; legacy request without hole must return `None` with unchanged soft fields.

- [ ] **Step 2: Add orchestration without dependency cycles**

Prefer a new analysis crate if core→stiff and stiff→contract/core data dependencies would otherwise cycle. Stiff solver must not own the soft solver.

- [ ] **Step 3: Integrate typed warnings**

Missing geometry: soft result + `WF-TND-STIFF-001` warning.

Non-convergence: preserve stiff interval result, set overall warning, add `WF-TND-STIFF-002`.

- [ ] **Step 4: Update CLI to call the analysis layer**

Preserve strict JSON and deterministic evidence hashing behavior.

- [ ] **Step 5: Run focused T&D tests**

```bash
cargo test -p wellforge-torque-drag-contract --locked
cargo test -p wellforge-torque-drag-core --locked
cargo test -p wellforge-torque-drag-stiff --locked
cargo test -p wellforge-torque-drag-analysis --locked
cargo test -p wellforge-torque-drag-cli --locked
```

- [ ] **Step 6: Commit**

```bash
git add engine/crates/wellforge-torque-drag-* engine/Cargo.toml
git commit -m "feat: integrate T&D stiff-string refinement"
```

### Task 6: Full verification and documentation

**Files:**
- Modify: `docs/RUST_ENGINE_ROADMAP.md`
- Modify: relevant T&D docs if present.

**Interfaces:** none; evidence gate.

- [ ] **Step 1: Update roadmap claims precisely**

Mark spatial curvature implemented. Mark bounded planar stiff contact refinement implemented with explicit Release-1 exclusions. Do not claim full 3-D contact/impact/whirl.

- [ ] **Step 2: Run release gates**

```bash
cd engine
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo deny check
```

Expected: all pass, except any pre-existing base-repo fixture failure must be isolated and documented rather than hidden.

- [ ] **Step 3: Review diff for scope**

Confirm no RAG files and no unrelated engine families changed.

- [ ] **Step 4: Commit docs/evidence**

```bash
git add docs engine
git commit -m "docs: document T&D spatial and stiff refinement"
```
