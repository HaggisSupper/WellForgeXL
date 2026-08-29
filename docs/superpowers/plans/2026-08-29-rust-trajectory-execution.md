# Rust Trajectory Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the verified trajectory contracts and numerical functions into one deterministic Rust analysis, a strict file-only CLI/bridge, and a value-only Excel/VBA client.

**Architecture:** `wellforge-trajectory-core` remains the numerical geometry library. A new pure `wellforge-trajectory-analysis` crate composes plan, survey, target, slide, formation, and optional projection calculations without file or Excel dependencies. `wellforge-trajectory-cli` owns strict JSON, canonical hashes, schemas, diagnostics, atomic files, and the bounded bridge; VBA only exports inputs, invokes the colocated hash-verified executable, validates the bridge, and writes values.

**Tech Stack:** Rust 2024 / Rust 1.98.0, Cargo, serde, schemars, nalgebra, thiserror, clap, sha2, tempfile, Node source-contract tests, VBA7, PowerShell.

**Spec:** `docs/RUST_ENGINE_ROADMAP.md`

## Global Constraints

- Rust is the calculation, JSON parsing, validation, and evidence authority; Excel/VBA is an input, review, and visualization client.
- Preserve request order for all station, target, slide, and formation result arrays.
- Use established numerical libraries; do not add handwritten general vector, interpolation, nonlinear-solver, or JSON-parser implementations.
- Canonical calculation fields are SI. This plan does not claim the still-missing unit-preserving wire adapter.
- Preserve WITSML-aligned identity and provenance, but do not claim full WITSML 2.0 or ETP conformance.
- No network service, runtime download, Docker, formula/physics fallback, or LLM calculation path.
- Result JSON and bridge records contain only finite values; unavailable values use typed states and `Option` fields.
- Do not write or replace `.xlsx`, `.xlsm`, `.zip`, or other packaged deliverables during Linux implementation.
- Run Rust with `CARGO_HOME=/tmp/wellforge-rust/cargo`, `RUSTUP_HOME=/tmp/wellforge-rust/rustup`, `RUSTUP_TOOLCHAIN=1.98.0`, `CARGO_TARGET_DIR=/tmp/wellforge-rust/target`, and `/tmp/wellforge-rust/cargo/bin` first on `PATH`.
- All Cargo verification is `--locked --offline`; final gates are formatting, warnings-denied workspace Clippy, and the locked workspace test suite.

---

### Task 1: Complete the trajectory analysis boundary

**Files:**
- Modify: `engine/Cargo.toml`
- Modify: `engine/crates/wellforge-trajectory-contract/src/request.rs`
- Modify: `engine/crates/wellforge-trajectory-contract/src/result.rs`
- Modify: `engine/crates/wellforge-trajectory-contract/src/validation.rs`
- Modify: `engine/crates/wellforge-trajectory-contract/src/lib.rs`
- Modify: `engine/crates/wellforge-trajectory-contract/tests/contract.rs`
- Modify: `engine/crates/wellforge-trajectory-core/src/lib.rs`
- Modify: `engine/crates/wellforge-trajectory-core/tests/trajectory_core.rs`
- Create: `engine/crates/wellforge-trajectory-analysis/Cargo.toml`
- Create: `engine/crates/wellforge-trajectory-analysis/src/lib.rs`
- Create: `engine/crates/wellforge-trajectory-analysis/tests/analysis.rs`

**Interfaces:**
- `Target` gains `md_m: f64`; this is the exact survey/projection query depth for that envelope.
- Request `SlideInterval` retains identity, MD interval, slide length, commanded toolface, rotary baselines, and low-inclination threshold, but no caller-supplied endpoint inclination/azimuth.
- Core `ResolvedSlideInterval` carries the endpoint directions obtained by exact survey interpolation and is the sole input to `slide_response`.
- `ProjectionRequest { bit_md_m, ahead_m, build_tendency_rad_per_m, effective_turn_tendency_rad_per_m, low_inclination_threshold_rad }` is optional on `TrajectoryAnalysisRequest`.
- `project_tendency(survey, request) -> ProjectionAssessment` extends the final survey station to bit MD and then ahead using the same clamped minimum-curvature displacement and the workbook tendency convention.
- `analyze(request: &TrajectoryAnalysisRequest) -> Result<TrajectoryCalculation, TrajectoryAnalysisError>` calculates both courses once, matches each survey station to the plan by exact partial-course interpolation at survey MD, derives slide endpoints from survey interpolation, evaluates formations, and evaluates targets using actual survey first and the optional projection only beyond survey TD.
- `TrajectoryCalculation` contains ordered `plan`, `survey`, `plan_survey_residuals`, `targets`, `slides`, `formations`, and optional `projection` arrays/records.
- Strict aggregate types include `PlanSurveyResidual`, `TargetAssessment`, `TargetBasis { Actual, Projected, NotReached }`, `SlideAssessment`, `ProjectionAssessment`, `TrajectoryAnalysisStatus`, `ApplicabilityStatement`, `CalculationEvidence`, and `TrajectoryAnalysisResult`.
- The final result copies `analysis_id`, contract version, and immutable source references; evidence fields include engine/compiler/target/lock identities plus normalized request/result SHA-256 fields. Closed-form geometry reports deterministic completion/status, not invented iterative convergence.

- [ ] **Step 1: Write contract RED tests**

Add tests that deserialize a request containing target MD, simplified slide inputs, and optional projection. Require unknown-field rejection, finite nonnegative target/projection MD, `bit_md_m >= survey TD`, nonnegative ahead length, canonical low-inclination threshold, and unique IDs. Add result JSON round-trip tests proving all aggregate records deny unknown fields and never serialize non-finite placeholders.

- [ ] **Step 2: Run contract RED**

Run:

```bash
cargo test -p wellforge-trajectory-contract --test contract --locked --offline
```

Expected: FAIL because target MD, projection, simplified slide, and aggregate result types do not exist.

- [ ] **Step 3: Implement the strict request/result changes**

Use `#[serde(deny_unknown_fields)]`, `schemars::JsonSchema`, canonical SI suffixes, stable UUID linkage, and typed enum states. Do not place hashes or diagnostics in numerical component records.

- [ ] **Step 4: Write core projection and resolved-slide RED tests**

Cover a straight vertical projection, a pure build projection, effective-turn low-inclination guarding, bit MD behind survey TD, overflow, and survey-derived slide endpoints. Literal expected positions must be independently hand-derived or taken from the unchanged JavaScript oracle/workbook formulas.

- [ ] **Step 5: Run core RED**

Run:

```bash
cargo test -p wellforge-trajectory-core --test trajectory_core --locked --offline
```

Expected: FAIL because `ResolvedSlideInterval` and `project_tendency` do not exist.

- [ ] **Step 6: Implement core projection and slide resolution primitives**

For each projected leg, clamp inclination to `[0, pi]`, calculate effective azimuth change as `effective_turn * delta_md / max(sin(mean_inc), sin(low_threshold), 1e-9)`, wrap azimuth to `[0, 2*pi)`, and reuse minimum-curvature displacement. Mark the low-turn guard explicitly whenever nonzero effective turn uses the threshold denominator.

- [ ] **Step 7: Write analysis RED tests**

Require:

```rust
let result = analyze(&request).unwrap();
assert_eq!(result.plan.len(), request.plan.len());
assert_eq!(result.survey.len(), request.survey.len());
assert_eq!(result.plan_survey_residuals.len(), request.survey.len());
assert_eq!(result.targets[0].basis, TargetBasis::Actual);
assert_eq!(result.targets[1].basis, TargetBasis::Projected);
assert!(result.slides[0].response.is_some());
assert_eq!(result.formations[0].formation_uid, request.formations[0].uid);
```

Also require typed plan coverage beyond TD, target `NotReached`, slide endpoint coverage without response, projection guard propagation, source/order preservation, and rejection of ambiguous/overflowing courses.

- [ ] **Step 8: Run analysis RED**

Run:

```bash
cargo test -p wellforge-trajectory-analysis --test analysis --locked --offline
```

Expected: FAIL because the crate and `analyze` orchestration do not exist.

- [ ] **Step 9: Implement the minimal pure analysis layer**

Keep file I/O, hashing, backups, logging, and CLI parsing out of this crate. Coverage states remain in output rather than dropping unmatched records. Use MD matching only; never match by row index, display name, nearest depth, or unrelated UUID.

- [ ] **Step 10: Verify Task 1 and commit**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p wellforge-trajectory-contract -p wellforge-trajectory-core -p wellforge-trajectory-analysis --all-targets --locked --offline -- -D warnings
cargo test -p wellforge-trajectory-contract -p wellforge-trajectory-core -p wellforge-trajectory-analysis --locked --offline
cargo test --workspace --locked --offline
```

Commit only Task 1 files with subject `feat: compose trajectory analysis`.

---

### Task 2: Expose the deterministic trajectory CLI and bridge

**Files:**
- Modify: `engine/Cargo.toml`
- Modify: `engine/Cargo.lock`
- Create: `engine/crates/wellforge-trajectory-fixtures/Cargo.toml`
- Create: `engine/crates/wellforge-trajectory-fixtures/src/lib.rs`
- Create: `engine/crates/wellforge-trajectory-cli/Cargo.toml`
- Create: `engine/crates/wellforge-trajectory-cli/src/main.rs`
- Create: `engine/crates/wellforge-trajectory-cli/src/canonical.rs`
- Create: `engine/crates/wellforge-trajectory-cli/src/diagnostics.rs`
- Create: `engine/crates/wellforge-trajectory-cli/src/bridge.rs`
- Create: `engine/crates/wellforge-trajectory-cli/tests/cli.rs`
- Create: `engine/fixtures/requests/trajectory-release-one-minimal.json`
- Create: `engine/fixtures/expected/trajectory-release-one-minimal.result.json`

**Interfaces:**
- Executable: `wellforge-trajectory` (`wellforge-trajectory.exe` on Windows).
- Commands: `validate --input`, `run --input --output [--diagnostics] [--no-backup]`, `verify-result --input --request-hash`, `bridge --input --output --request-hash`, `schema --request --result`, and `version --json`.
- Explicit file paths only; no stdin calculation mode, shell fragment, network, or result JSON on stdout.
- Stable exits: `0` success, `2` CLI syntax, `10` invalid request, `20` calculation failure, `30` result/hash integrity failure, `40` file I/O failure.
- Canonical hashing serializes parsed strict structs, recursively changes numeric negative zero to positive zero, rejects non-finite values, and SHA-256 hashes compact JSON. Result hashing blanks only `evidence.result_hash`.
- `run` writes pretty JSON plus one newline through a same-directory temporary file. Existing output is preserved as a timestamped sibling backup unless `--no-backup` is supplied.
- Diagnostics are one JSON object per line with stable `level`, `event`, `code`, `analysis_id`, `request_hash`, and `message` fields. When `--diagnostics` is supplied, high-volume details go there and stdout remains bounded.
- Bridge version `1.0.0` is tab-delimited UTF-8. Header `H` carries bridge version, analysis UUID, request hash, result hash, engine version, status, and deterministic flag. Record kinds are `P` plan station, `S` survey station, `R` residual, `T` target, `L` slide, `F` formation, and `X` projection. Optional numeric fields are empty, strings cannot contain tabs/newlines, floats use 17-digit scientific notation, and bridge generation rejects plan/survey counts above 500, targets above 100, slides above 200, formations above 100, or more than one projection.

- [ ] **Step 1: Add deterministic fixture data**

Create fixed `Uuid::from_u128` source and record identities. The fixture must contain at least three plan and survey stations, one actual target, one projected target, one covered slide, one uncovered slide, one actual formation pick, and a deterministic projection.

- [ ] **Step 2: Write CLI RED tests**

Tests invoke `env!("CARGO_BIN_EXE_wellforge-trajectory")` through `std::process::Command`. Require strict unknown-field rejection, the stable exit table, two byte-identical runs, valid schemas, normalized request hash, tamper/request-hash rejection, atomic replacement/backup behavior, JSONL parseability, bounded stdout, bridge header/record counts, and strict result round-trip.

- [ ] **Step 3: Run CLI RED**

Run:

```bash
cargo test -p wellforge-trajectory-cli --test cli --locked --offline
```

Expected: FAIL because the fixture and CLI crates do not exist.

- [ ] **Step 4: Implement canonical JSON, evidence finalization, and typed command errors**

Keep canonicalization and result-hash verification in focused modules. Construct the aggregate result only after `validate_request` and `analyze` succeed. A typed calculation/coverage warning may produce `complete_with_warnings`; a contract error or failed plan/survey calculation never writes an accepted result.

- [ ] **Step 5: Implement atomic outputs, diagnostics, schemas, and bridge**

Validate the complete bridge in Rust before writing it. Preserve array order and emit exactly one header. `verify-result` must strict-deserialize, compare the supplied request hash, recompute the blanked result hash, and reject failed/incomplete results; both `complete` and `complete_with_warnings` are accepted completed states.

- [ ] **Step 6: Verify deterministic fixture execution**

Run:

```bash
cargo test -p wellforge-trajectory-cli --locked --offline
cargo run -p wellforge-trajectory-cli --locked --offline -- run --input fixtures/requests/trajectory-release-one-minimal.json --output /tmp/wellforge-rust/trajectory-run-1.json --no-backup
cargo run -p wellforge-trajectory-cli --locked --offline -- run --input fixtures/requests/trajectory-release-one-minimal.json --output /tmp/wellforge-rust/trajectory-run-2.json --no-backup
cmp /tmp/wellforge-rust/trajectory-run-1.json /tmp/wellforge-rust/trajectory-run-2.json
```

Expected: CLI tests pass and `cmp` exits `0`.

- [ ] **Step 7: Verify Task 2 and commit**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
```

Commit only Task 2 files with subject `feat: expose deterministic trajectory CLI`.

---

### Task 3: Connect the directional workbook as a value-only client

**Files:**
- Create: `VBA/WellForgeTrajectoryEngine.bas`
- Modify: `VBA/WellForgeCore.bas`
- Modify: `VBA/WellForgeDirectional.bas`
- Modify: `src/build_directional.mjs`
- Modify: `src/directional_workbook_model.mjs`
- Modify: `data/wellforge-vba-engine-manifest.json`
- Create: `tools/Build-WellForgeTrajectoryEngine.ps1`
- Create: `tools/Test-WellForgeTrajectoryEngine.ps1`
- Modify: `tools/Build-WellForgeVbaSuite.ps1`
- Create: `tests/trajectory_rust_engine_contract.test.mjs`
- Create: `tests/trajectory_workbook_engine_contract.test.mjs`
- Modify: `docs/VBA_ENGINE_GUIDE.md`

**Interfaces:**
- Workbook entry point: `WF_RunTrajectoryRustEngine`.
- Fixed colocated executable and manifest: `wellforge-trajectory.exe` and `wellforge-trajectory.exe.sha256`.
- The workbook exposes explicit provenance inputs for Well, Wellbore, plan trajectory, survey trajectory, and MD datum identities/source metadata. VBA must not synthesize required provenance from labels, paths, or random UUIDs.
- Each invocation uses a fresh `%TEMP%\WellForgeTrajectory\<run-id>` directory and explicit quoted paths. It runs validate, run with diagnostics, verify-result, then bridge under a bounded timeout without `cmd.exe` interpolation.
- The adapter stages and validates the complete bridge in memory before any `ClearContents` or `.Value2` write. It rejects wrong hashes/version/counts, duplicate/missing IDs, unknown record kinds, non-finite values, invalid enums, and capacities above plan/survey 500, targets 100, slides 200, formations 100.
- On success it writes canonical SI result/helper blocks as values and refreshes existing converted displays/charts. On failure it preserves the last accepted values and publishes `ENGINE UNAVAILABLE`, `ENGINE HASH MISMATCH`, `INVALID REQUEST`, `ANALYSIS FAILED`, or `INVALID RESULT` plus the diagnostic path.
- `WellForgeCore.WF_DispatchModel` routes directional production calculation only to the Rust entry point. `WF_CalcDirectional` is not called as a fallback.

- [ ] **Step 1: Write Linux-verifiable RED source tests**

Require fixed executable/hash names, no `cmd.exe`, bounded execution, explicit diagnostics, fresh run directory, validate/run/verify/bridge sequence, provenance fields, staged bridge validation before writes, last-result preservation, value-only writes, capacity checks, Rust-only dispatch, and manifest/build-script entries.

- [ ] **Step 2: Run source RED**

Run:

```bash
node --test tests/trajectory_rust_engine_contract.test.mjs tests/trajectory_workbook_engine_contract.test.mjs
```

Expected: FAIL because the trajectory adapter and wiring do not exist.

- [ ] **Step 3: Implement request export and bounded process control**

Reuse only narrow, proven JSON/file/process helpers. VBA owns unit conversion and presentation, not geometry, interpolation, target, slide, formation, projection, hashing, or result JSON parsing.

- [ ] **Step 4: Implement staged strict bridge import and value writes**

Do not call a general VBA JSON parser on result JSON. Parse the Rust bridge by fixed record grammar, validate all records first, then commit arrays in bounded `.Value2` writes. Update accepted evidence only after all writes succeed.

- [ ] **Step 5: Add builder provenance/evidence surfaces and release scripts**

Build scripts compile Rust and write SHA-256 before Excel automation. Linux tests prove source wiring only; they must not claim native Windows executable, COM, macro, or rendering acceptance.

- [ ] **Step 6: Verify Task 3 and commit**

Run:

```bash
node --test tests/trajectory_rust_engine_contract.test.mjs tests/trajectory_workbook_engine_contract.test.mjs tests/directional.test.mjs tests/directional_workbook_values.test.mjs tests/vba_engine_contract.test.mjs
node tools/lint_vba.mjs
cargo test -p wellforge-trajectory-cli --locked --offline
```

Commit only Task 3 source/test/document files with subject `feat: connect trajectory workbook to Rust`.

---

### Task 4: Final independent verification without packaging

**Files:**
- Modify only if an evidence defect is found in review.

**Interfaces:**
- Produces a verified source branch and explicit list of unrun Windows/Excel gates; it does not create or overwrite packaged deliverables.

- [ ] **Step 1: Run full deterministic gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
node --test tests/trajectory_rust_engine_contract.test.mjs tests/trajectory_workbook_engine_contract.test.mjs tests/directional.test.mjs tests/directional_workbook_values.test.mjs tests/vba_engine_contract.test.mjs
node tools/lint_vba.mjs
```

- [ ] **Step 2: Verify repository and package preservation**

Record `git diff --check`, relevant source hashes, and unchanged mtimes/hashes for existing `.xlsx`, `.xlsm`, and `.zip` deliverables. Do not rebuild them.

- [ ] **Step 3: Record the remaining platform gates**

The unrun gate is Windows desktop Excel: native `.exe` build, executable hash verification, VBA compilation, success/failure macro acceptance, unit switching, chart refresh/rendering, and last-accepted-value preservation. Do not label these passed from Linux source tests.
