# Foundational Numerical Method Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stabilize WellForge around a reusable Rust numerical substrate before any additional calculation-family development.

**Architecture:** Add a production `wellforge-numerics` crate beneath the engineering engines while retaining `wellforge-numerics-acceptance` as an independent acceptance boundary. Implement small deterministic primitives first, then migrate BHA static/modal as real acceptance consumers while preserving existing trajectory, T&D soft-string, and hydraulics physics.

**Tech Stack:** Rust 2024 / Rust 1.98, `faer 0.24.4`, `nalgebra 0.34.2`, existing workspace lint/test infrastructure.

**Spec:** `docs/superpowers/specs/2026-09-06-foundational-numerical-method-design.md`

## Global Constraints
- Freeze new calculation-family work until Phase 1 exit criteria pass.
- Preserve deterministic engine authority and immutable evidence gates.
- No new BHA dynamic/6-DOF capability.
- No cloud or Python calculation authority.
- No unsafe Rust.
- Test-first for every behavior change.
- Do not rewrite stable closed-form trajectory or steady hydraulics equations solely for abstraction consistency.

---

### Task 1: Production numerical crate boundary

**Files:**
- Modify: `engine/Cargo.toml`
- Create: `engine/crates/wellforge-numerics/Cargo.toml`
- Create: `engine/crates/wellforge-numerics/src/lib.rs`
- Test: `engine/crates/wellforge-numerics/tests/public_contract.rs`

**Interfaces:**
- Produces modules: `diagnostics`, `root`, `interval`, `volume`, `interpolation`, `circular`, `sparse`, `complementarity`.
- The initial crate compiles with no domain-engine dependency.

- [ ] **Step 1: Write the failing public-contract test**

```rust
//! Public-module contract for the production numerical substrate.

use wellforge_numerics::{ConvergenceState, SolverDiagnostics};

#[test]
fn diagnostics_contract_is_domain_independent() {
    let diagnostics = SolverDiagnostics {
        state: ConvergenceState::Converged,
        iterations: 3,
        initial_residual_norm: 1.0,
        final_residual_norm: 1.0e-10,
        absolute_tolerance: 1.0e-9,
        relative_tolerance: 1.0e-9,
        fallback_used: false,
    };
    assert!(diagnostics.converged());
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run from `engine/`:
```bash
cargo test -p wellforge-numerics --test public_contract
```
Expected: package/type resolution failure because `wellforge-numerics` does not exist.

- [ ] **Step 3: Add the workspace member and minimal diagnostics implementation**

Create the crate with workspace package/lint settings, `thiserror`, `faer`, and `nalgebra` as workspace dependencies. Define `ConvergenceState` and `SolverDiagnostics`; re-export them from `lib.rs`.

- [ ] **Step 4: Verify GREEN**

```bash
cargo test -p wellforge-numerics --test public_contract
cargo clippy -p wellforge-numerics --all-targets -- -D warnings
```
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add engine/Cargo.toml engine/crates/wellforge-numerics
git commit -m "feat: add foundational numerics crate"
```

### Task 2: Bracketed and safeguarded scalar roots

**Files:**
- Create: `engine/crates/wellforge-numerics/src/root.rs`
- Modify: `engine/crates/wellforge-numerics/src/lib.rs`
- Test: `engine/crates/wellforge-numerics/tests/root.rs`

**Interfaces:**
- Produces `RootOptions`, `RootSolution`, `RootError`.
- Produces `solve_bracketed<F>(lower, upper, residual, options)`.
- Produces `solve_safeguarded_newton<F,D>(lower, upper, initial, residual, derivative, options)`.

- [ ] **Step 1: Write RED tests** covering `x^2-2`, invalid non-sign-changing bracket, and Newton fallback when the proposed step exits the bracket.
- [ ] **Step 2: Verify RED** with `cargo test -p wellforge-numerics --test root`.
- [ ] **Step 3: Implement deterministic bisection/Brent-style bracket preservation and safeguarded Newton using the common diagnostics contract.**
- [ ] **Step 4: Verify GREEN and Clippy.**
- [ ] **Step 5: Commit** `feat: add safeguarded root solvers`.

### Task 3: Piecewise interval partition and cumulative volume coordinate

**Files:**
- Create: `engine/crates/wellforge-numerics/src/interval.rs`
- Create: `engine/crates/wellforge-numerics/src/volume.rs`
- Modify: `engine/crates/wellforge-numerics/src/lib.rs`
- Test: `engine/crates/wellforge-numerics/tests/interval_volume.rs`

**Interfaces:**
- Produces `Interval { start, end }`.
- Produces `partition_boundaries(&[&[f64]]) -> Result<Vec<Interval>, IntervalError>`.
- Produces `VolumeSegment { interval, area }` and `VolumeCoordinate::new`, `volume_at(x)`, `position_at(volume)`, `total_volume()`.

- [ ] **Step 1: Write RED tests** for intersecting hole/string boundaries `[0,100,200]` + `[0,50,150,200]`, zero-length suppression, tapered-area volume conservation, and inverse round-trip.
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Implement sorted/deduplicated partitioning and piecewise cumulative volume with binary-search lookup.**
- [ ] **Step 4: Verify GREEN and property-style round trips over representative points.**
- [ ] **Step 5: Commit** `feat: add interval and volume coordinates`.

### Task 4: Monotone interpolation and circular/vector statistics

**Files:**
- Create: `engine/crates/wellforge-numerics/src/interpolation.rs`
- Create: `engine/crates/wellforge-numerics/src/circular.rs`
- Modify: `engine/crates/wellforge-numerics/src/lib.rs`
- Test: `engine/crates/wellforge-numerics/tests/calibration_math.rs`

**Interfaces:**
- Produces `MonotoneCurve::new(points)` and `evaluate(x)` using a monotonicity-preserving piecewise cubic or linear-safe implementation.
- Produces `CircularAccumulator` and `CircularSummary { x, y, resultant, total_magnitude, coherence }`.

- [ ] **Step 1: Write RED tests** proving interpolation cannot overshoot local monotone bounds and repeated x is rejected; prove opposing TF vectors cancel and aligned vectors yield coherence 1.
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Implement the minimal monotone interpolation and vector accumulator.**
- [ ] **Step 4: Verify GREEN and Clippy.**
- [ ] **Step 5: Commit** `feat: add calibration numerical primitives`.

### Task 5: Complementarity primitive

**Files:**
- Create: `engine/crates/wellforge-numerics/src/complementarity.rs`
- Modify: `engine/crates/wellforge-numerics/src/lib.rs`
- Test: `engine/crates/wellforge-numerics/tests/complementarity.rs`

**Interfaces:**
- Produces `fischer_burmeister(gap, reaction) -> f64`.
- Produces `fischer_burmeister_gradient(gap, reaction, smoothing) -> (f64, f64)` with a defined origin treatment.

- [ ] **Step 1: Write RED tests** for open gap `(g>0, lambda=0)`, contact `(g=0, lambda>0)`, invalid penetration/reaction pair producing nonzero residual, and finite-difference gradient agreement away from the origin.
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Implement the Fischer-Burmeister residual and smoothed/defined gradient.**
- [ ] **Step 4: Verify GREEN and Clippy.**
- [ ] **Step 5: Commit** `feat: add complementarity primitives`.

### Task 6: Sparse solve boundary

**Files:**
- Create: `engine/crates/wellforge-numerics/src/sparse.rs`
- Modify: `engine/crates/wellforge-numerics/src/lib.rs`
- Test: `engine/crates/wellforge-numerics/tests/sparse.rs`

**Interfaces:**
- Produces a small canonical triplet/CSR assembly boundary and `SparseSolveReport` including residual norm.
- Must use `faer` sparse facilities; no home-grown factorization.

- [ ] **Step 1: Write RED test** for a tridiagonal beam-like system with known solution and residual below `1e-12`.
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Implement sparse assembly and factor/solve wrapper.**
- [ ] **Step 4: Verify GREEN, Clippy, and deterministic repeated solve.**
- [ ] **Step 5: Commit** `feat: add sparse solve boundary`.

### Task 7: Extend independent numerical acceptance

**Files:**
- Modify: `engine/crates/wellforge-numerics-acceptance/Cargo.toml`
- Modify: `engine/crates/wellforge-numerics-acceptance/src/lib.rs`
- Test: existing acceptance crate tests / CLI surfaces.

**Interfaces:**
- Acceptance crate depends on `wellforge-numerics` and verifies production root, volume, circular, complementarity, and sparse primitives against independent cases.

- [ ] **Step 1: Add failing acceptance assertions for the production crate.**
- [ ] **Step 2: Verify RED before adding the dependency/wiring.**
- [ ] **Step 3: Wire the production crate and extend `NumericsAcceptanceReport` with explicit foundation booleans.**
- [ ] **Step 4: Verify GREEN.**
- [ ] **Step 5: Commit** `test: exercise foundational numerical substrate`.

### Task 8: BHA static acceptance migration

**Files:**
- Modify: `engine/crates/wellforge-bha-static/Cargo.toml`
- Modify: `engine/crates/wellforge-bha-static/src/lib.rs`
- Test: existing BHA static tests plus a new sparse-parity test if required.

**Interfaces:**
- BHA static consumes `wellforge_numerics::sparse` for the reduced FE linear solve.
- Public `StaticSolution` contract and analytical cantilever oracle remain unchanged.

- [ ] **Step 1: Write a failing test/acceptance assertion that requires the common sparse solver path while preserving current result tolerance.**
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Replace dense solve-only conversion with sparse assembly/solve using the common boundary; preserve existing matrices only where modal analysis contract currently requires them.**
- [ ] **Step 4: Run BHA static, modal, CLI and fixture tests; compare analytical oracle and existing expected-result fixtures.**
- [ ] **Step 5: Commit** `refactor: use foundational sparse solver in BHA static`.

### Task 9: BHA modal numerical stabilization

**Files:**
- Modify: `engine/crates/wellforge-bha-modal/src/lib.rs`
- Test: existing mode/FRF tests plus parity coverage.

**Interfaces:**
- Preserve `solve_modes`, `solve_frequency_response`, and `build_campbell_map` public behavior.
- Eliminate explicit `try_inverse()` of the Cholesky factor by factored triangular solves.
- Assess modal-superposition FRF only if parity can be demonstrated against current direct complex-LU outputs within an explicit tolerance; otherwise retain direct FRF for Phase 1 and document the result.

- [ ] **Step 1: Write a failing structural/parity test that would detect explicit-inverse path removal without changing numerical outputs.**
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Refactor generalized eigen transformation using triangular solves.**
- [ ] **Step 4: Run existing modes/FRF/Campbell tests and an independent small generalized-eigen oracle.**
- [ ] **Step 5: Commit** `refactor: stabilize BHA modal factor solves`.

### Task 10: Full stabilization gate

**Files:**
- Modify docs only if evidence/results require clarification.
- Remove any temporary focused CI workflow added solely for development if redundant with the normal engine gate.

**Interfaces:**
- No new calculation physics.

- [ ] **Step 1: Run formatting:** `cargo fmt --all -- --check`.
- [ ] **Step 2: Run workspace Clippy:** `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] **Step 3: Run workspace tests:** `cargo test --workspace`.
- [ ] **Step 4: Confirm T&D spatial-dogleg tests remain green and trajectory/hydraulics outputs have no unintended changes.**
- [ ] **Step 5: Record Phase 1 evidence and only then declare the calculation freeze lifted.**
