# WellForgeXL Full Release Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every non-Excel gate reproducible in CI, make the native Windows Excel gate independently runnable with retained evidence, fix all failures exposed by those gates, and state release readiness without overstating unrun platform evidence.

**Architecture:** A repository-owned verification entry point materializes immutable workbook sources, runs deterministic Node contracts without test-order coupling, lints VBA, and invokes the pinned Rust workspace gates. GitHub Actions calls that entry point on Linux and runs a separate Windows workflow for native Rust executables and desktop Excel/COM when an eligible self-hosted runner is available. Generated evidence is uploaded as artifacts; source workbooks remain hash-verified inputs rather than mutable build outputs.

**Tech Stack:** Node.js built-in test runner, JavaScript ESM, PowerShell 5.1/7, Rust 1.98.0, Cargo/Clippy/rustfmt, cargo-deny 0.20.2, GitHub Actions, Microsoft Excel COM.

**Spec:** `docs/RUST_ENGINE_ROADMAP.md`, `docs/VBA_ENGINE_GUIDE.md`, and the release-gate statements in `README.md`.

## Global Constraints

- Windows-first release output; no Docker.
- Rust 2024 with pinned Rust 1.98.0 and `Cargo.lock`.
- Warnings-denied Clippy and dependency-policy checks remain mandatory.
- Source workbooks must be SHA-256 verified before use.
- No Node authoring dependency is required for a fresh-checkout release build.
- Linux/static evidence must never be represented as native Windows Excel/COM acceptance.
- Production result paths remain value-only and preserve last accepted values on failure.

---

### Task 1: Deterministic repository verification entry point

**Files:**
- Create: `tools/verify-node.mjs`
- Create: `tools/Verify-WellForgeSource.ps1`
- Modify: `tests/post_merge_review.test.mjs`
- Modify: `README.md`

**Interfaces:**
- Consumes: `workbooks/source/source-workbooks.sha256`, source workbook files, `tests/*.test.mjs`, and `tools/lint_vba.mjs`.
- Produces: one exit-code-bearing source gate that materializes compressed sources before consumers run and terminates cleanly.

- [ ] **Step 1: Write failing tests** asserting the verifier checks every manifest entry, materializes the gzip workbook, runs the Node suite only after materialization, and runs VBA lint.
- [ ] **Step 2: Run** `node --test tests/post_merge_review.test.mjs` and confirm the verifier tests fail because the entry point does not exist.
- [ ] **Step 3: Implement** the PowerShell materializer and Node coordinator using `spawnSync`, explicit timeouts, inherited output, and nonzero exit propagation.
- [ ] **Step 4: Run** the focused tests, then `node tools/verify-node.mjs`; confirm all tests finish and the process exits zero.
- [ ] **Step 5: Commit** with message `build: add deterministic source verification gate`.

### Task 2: Pinned Linux CI and dependency policy

**Files:**
- Create: `.github/workflows/source-verification.yml`
- Modify: `tests/post_merge_review.test.mjs`
- Modify: `README.md`

**Interfaces:**
- Consumes: Task 1 verifier, `engine/rust-toolchain.toml`, `engine/Cargo.lock`, and `engine/deny.toml`.
- Produces: required-capable Linux jobs for Node/VBA structure, Rust formatting, warnings-denied Clippy, locked workspace tests, and cargo-deny.

- [ ] **Step 1: Write failing source-contract tests** for checkout, pinned Node, pinned Rust, cache keys including `Cargo.lock`, `--locked`, `-D warnings`, and cargo-deny 0.20.2.
- [ ] **Step 2: Run** the focused contract test and confirm it fails because the workflow is absent.
- [ ] **Step 3: Implement** the workflow without containers and with least-privilege read-only contents permission.
- [x] **Step 4: Validate YAML parsing and rerun focused/full source verification.**
- [ ] **Step 5: Commit** with message `ci: enforce source and Rust release gates`.

### Task 3: Windows native and Excel/COM evidence workflow

**Files:**
- Create: `.github/workflows/windows-release-verification.yml`
- Create: `tools/Write-WellForgeReleaseEvidence.ps1`
- Modify: `tools/Build-WellForgeVbaSuite.ps1`
- Modify: `tests/post_merge_review.test.mjs`
- Modify: `docs/VBA_ENGINE_GUIDE.md`

**Interfaces:**
- Consumes: a labeled self-hosted Windows runner with desktop Excel, Rust 1.98.0, trusted VBA project access, and the existing suite builder.
- Produces: JSON release evidence, JSONL logs, executable hashes, and five `.xlsm` outputs uploaded as workflow artifacts.

- [ ] **Step 1: Write failing contract tests** for `self-hosted`, `Windows`, an explicit `wellforgexl-excel` label, workflow dispatch, timeouts, artifact upload on success/failure, and evidence fields for every release gate.
- [ ] **Step 2: Run** the focused test and confirm failure because the workflow/evidence writer is absent.
- [ ] **Step 3: Implement** the evidence writer and workflow; do not use GitHub-hosted Windows because desktop Excel is not guaranteed there.
- [x] **Step 4: Run static PowerShell parsing where available and source-contract tests everywhere.**
- [ ] **Step 5: Commit** with message `ci: add Windows Excel release evidence gate`.

### Task 4: Defect remediation and final readiness

**Files:**
- Modify only files implicated by reproduced failures.
- Test each production change in the closest existing test file or a new focused test.
- Modify: `RELEASE_NOTES.md`

**Interfaces:**
- Consumes: all gates from Tasks 1–3.
- Produces: a green source branch and an exact list of any unexecuted Windows evidence.

- [x] **Step 1: Run all locally available gates** and capture exact failures.
- [x] **Step 2: For each genuine defect, write a failing regression test, confirm red, implement the minimal fix, and confirm green.**
- [ ] **Step 3: Rerun Node/VBA structure, Rust format, Clippy, workspace tests, cargo-deny, and workflow contract checks from clean materialized inputs.
- [ ] **Step 4: Inspect Git diff, generated evidence contracts, unresolved review threads, and GitHub checks.
- [ ] **Step 5: Update release notes** with counts and platform caveats proven by the final run.
- [ ] **Step 6: Commit** with message `fix: close full release hardening findings` and publish a merge-readiness report.

## Self-Review

- Spec coverage: deterministic workbook materialization, Node/VBA source gates, pinned Rust gates, dependency policy, native Windows executables, Excel/COM, artifacts, and honest readiness reporting are each assigned.
- Placeholder scan: no deferred implementation markers are present; platform execution is explicitly routed to a qualified runner.
- Type consistency: Task 1 produces the verifier consumed by Task 2; Task 3 independently produces release evidence consumed by Task 4.
