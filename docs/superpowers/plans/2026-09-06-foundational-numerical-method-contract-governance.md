# Phase 1 Canonical Contract Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete Phase 1 stabilization by enforcing versioned canonical contracts, deterministic schema fingerprints, and explicit compatibility policy across all implemented WellForge engine boundaries.

**Architecture:** Add a domain-independent `wellforge-contract-governance` crate for contract identity/version/fingerprint primitives and a separate `wellforge-contract-registry` aggregator that depends on the domain contract crates. Domain contract crates consume governance primitives but never the aggregator, preventing dependency cycles.

**Tech Stack:** Rust 2024 / Rust 1.98, `semver`, `schemars`, `serde_json`, `sha2`, existing engine contract crates.

**Spec:** `docs/superpowers/specs/2026-09-06-foundational-numerical-method-contract-governance-addendum.md`

## Global Constraints
- This work is part of Phase 1; new calculation-family work remains frozen.
- No physics equations or accepted calculation outputs change as part of governance migration.
- Every boundary version must be valid SemVer and resolved through explicit policy.
- Internal numerical primitives do not carry domain contract versions.
- Registry aggregation must not introduce dependency cycles into domain contract crates.
- Schema fingerprint generation must be deterministic.
- Test-first for every behavioral change.
- No unsafe Rust.

---

### Task G1: Contract-governance primitive crate

**Files:**
- Modify: `engine/Cargo.toml`
- Create: `engine/crates/wellforge-contract-governance/Cargo.toml`
- Create: `engine/crates/wellforge-contract-governance/src/lib.rs`
- Create: `engine/crates/wellforge-contract-governance/src/version.rs`
- Create: `engine/crates/wellforge-contract-governance/src/schema.rs`
- Test: `engine/crates/wellforge-contract-governance/tests/governance.rs`

**Interfaces:**
- Produces `ContractId` validated from canonical dotted identifiers.
- Produces `CompatibilityPolicy::{Exact, SameMajor, ExplicitSet}`.
- Produces `ContractVersionPolicy` and `validate_version(&Version) -> Result<(), GovernanceError>`.
- Produces `SchemaKind::{Request, Result}`.
- Produces `SchemaFingerprint` and `fingerprint_schema(&serde_json::Value)`.
- Produces deterministic recursive JSON normalization for schema hashing.

- [ ] **Step 1: Write RED tests** proving canonical IDs accept `wellforge.trajectory.analysis`, malformed IDs are rejected, malformed SemVer never reaches compatibility checking, Exact/SameMajor/ExplicitSet policies behave deterministically, and reordered JSON object keys produce identical schema fingerprints.
- [ ] **Step 2: Verify RED** with `cargo test -p wellforge-contract-governance` and confirm failure is due to the missing package/API.
- [ ] **Step 3: Implement the minimum production crate** using `semver`, `serde_json`, and `sha2`; do not depend on domain contract crates.
- [ ] **Step 4: Verify GREEN** with formatting, Clippy `-D warnings`, and package tests.
- [ ] **Step 5: Commit** `feat: add canonical contract governance primitives`.

### Task G2: Domain descriptors and real SemVer enforcement

**Files:**
- Modify: `engine/crates/wellforge-trajectory-contract/Cargo.toml`
- Modify: `engine/crates/wellforge-trajectory-contract/src/validation.rs`
- Modify: `engine/crates/wellforge-bha-contract/Cargo.toml`
- Modify: `engine/crates/wellforge-bha-contract/src/validation.rs`
- Modify: `engine/crates/wellforge-torque-drag-contract/Cargo.toml`
- Modify: `engine/crates/wellforge-torque-drag-contract/src/validation.rs`
- Modify: `engine/crates/wellforge-hydraulics-contract/Cargo.toml`
- Modify: `engine/crates/wellforge-hydraulics-contract/src/validation.rs`
- Create/modify one focused descriptor module in each contract crate.
- Test: each existing contract test surface plus focused malformed/unsupported-version tests.

**Interfaces:**
- Trajectory descriptor ID: `wellforge.trajectory.analysis`.
- BHA descriptor ID: `wellforge.bha.analysis`.
- T&D descriptor ID: `wellforge.torque_drag.analysis`.
- Hydraulics descriptor ID: `wellforge.hydraulics.analysis`.
- Each crate exposes request/result schema descriptor functions and one explicit version policy.

- [ ] **Step 1: Add failing tests** showing BHA rejects `1.bad.0`, T&D rejects empty/malformed/unregistered versions, hydraulics still accepts `0.1.0` and `0.2.0` but rejects `0.3.0`, and trajectory preserves major-1 support according to its declared policy.
- [ ] **Step 2: Verify RED** on the four contract crates.
- [ ] **Step 3: Add governance dependencies and route version validation through `ContractVersionPolicy`; preserve existing stable error-code families.**
- [ ] **Step 4: Add request/result schema descriptor functions generated from `schemars::schema_for!` and fingerprinted by governance utilities.**
- [ ] **Step 5: Verify GREEN** across all four contract crates and their CLI consumers.
- [ ] **Step 6: Commit** `refactor: enforce canonical engine contract versions`.

### Task G3: Canonical registry aggregator

**Files:**
- Modify: `engine/Cargo.toml`
- Create: `engine/crates/wellforge-contract-registry/Cargo.toml`
- Create: `engine/crates/wellforge-contract-registry/src/lib.rs`
- Test: `engine/crates/wellforge-contract-registry/tests/registry.rs`

**Interfaces:**
- Produces `ContractDescriptor { id, version, compatibility, request_fingerprint, result_fingerprint, title }`.
- Produces `canonical_descriptors() -> Result<Vec<ContractDescriptor>, RegistryError>`.
- Registry depends on contract crates; contract crates never depend on registry.

- [ ] **Step 1: Write RED tests** requiring all four engine families to be present, IDs unique, `(id,version)` pairs unique, fingerprints non-empty, and an intentionally duplicated/conflicting descriptor rejected by registry validation.
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Implement aggregation and invariant checking only; no runtime service or plugin layer.**
- [ ] **Step 4: Verify GREEN and deterministic descriptor ordering.**
- [ ] **Step 5: Commit** `feat: add canonical contract registry`.

### Task G4: Boundary enforcement acceptance

**Files:**
- Modify focused CLI tests for trajectory, BHA, T&D, and hydraulics.
- Modify Excel/VBA bridge tests or fixtures where the test harness already verifies request payloads.
- Modify CI workflow paths/gates if required.

**Interfaces:**
- Unsupported/malformed contract versions fail before engine calculations execute.
- Accepted result echoes the exact validated request contract version.

- [ ] **Step 1: Add failing CLI/bridge tests** for malformed and unsupported versions using existing request fixtures.
- [ ] **Step 2: Verify RED where entry points currently bypass strict enforcement.**
- [ ] **Step 3: Route entry points through the owning contract validation before solver invocation; do not duplicate compatibility logic in CLIs/VBA.**
- [ ] **Step 4: Verify GREEN** across contract, core, CLI, fixtures, and workbook bridge tests available on Linux.
- [ ] **Step 5: Commit** `test: enforce canonical contracts at engine boundaries`.

### Task G5: Phase 1 combined governance gate

**Files:**
- Modify Phase 1 evidence/docs only as required by verified results.

**Interfaces:**
- No new calculation physics.

- [ ] **Step 1:** `cargo fmt --all -- --check`.
- [ ] **Step 2:** `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] **Step 3:** `cargo test --workspace`.
- [ ] **Step 4:** run canonical registry tests and capture the four families plus registered versions/fingerprints.
- [ ] **Step 5:** confirm existing trajectory, BHA, T&D, and hydraulics reference fixtures remain behaviorally unchanged except for intentional rejection of previously invalid contract versions.
- [ ] **Step 6:** only when both the numerical-foundation plan and this governance plan are green may Phase 1 be declared stable and the calculation freeze lifted.
