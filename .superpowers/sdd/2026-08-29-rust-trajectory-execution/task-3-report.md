# Task 3 report: directional workbook as a value-only Rust client

## Status

The initial Task 3 implementation was committed as `c470a1ad311273bacd17379b9bbcb29a736c4f61` (`feat: connect trajectory workbook to Rust`). Post-commit review fix round 1 is complete and ready for a separate commit with subject:

`fix: harden trajectory workbook Rust boundary`

This report combines the inherited partial RED work with the completion work. The preserved predecessor edits were inspected and extended in place; they were not discarded or restarted. No Excel/COM process was invoked and no repository XLSX, XLSM, ZIP, output, or package artifact was written or rebuilt.

## Inherited partial work

The resumed worktree already contained:

- the two new Linux-verifiable Node contract tests;
- partial directional builder/model provenance, identity and evidence surfaces;
- Rust-only core dispatch, presentation refresh wiring and manifest changes;
- the trajectory build and smoke-test PowerShell scripts plus suite-builder wiring.

`VBA/WellForgeTrajectoryEngine.bas`, the Task 3 documentation update, this report and a Task 3 commit were not present. The inherited source was preserved and treated as the intended RED surface.

## TDD evidence

### Inherited partial RED

Command:

```text
node --test tests/trajectory_rust_engine_contract.test.mjs tests/trajectory_workbook_engine_contract.test.mjs
```

Observed exit: `1`.

Observed result: `10 tests; 5 passed; 5 failed`; duration approximately `80.745 s`.

The four adapter-source checks failed because the trajectory VBA module/procedures did not exist. The builder/model source check also failed because the exact `RUST REQUIRED — NO VBA FALLBACK` declaration was absent. The already-authored workbook surface and release-script checks passed. This RED belongs to the inherited partial implementation.

### Completion-owned second RED

After adding the adapter and strengthening the source tests for pinned/offline release gates, neutral authority wording and honest workbook metadata, the focused command was run again.

Observed exit: `1`.

Observed result: `10 tests; 8 passed; 2 failed`; duration approximately `64.476 s`.

The remaining failures were deliberate and specific:

1. the Windows build script did not yet select/verify Rust 1.98.0 or use locked/offline clippy, test and release build gates;
2. the directional builder did not yet publish the exact no-fallback declaration and related hardened metadata.

Those production gaps were then implemented. The focused GREEN run produced `10 passed; 0 failed` in approximately `65.743 s`.

### Regression found and fixed

The first full directional/VBA regression run produced `30 passed; 1 failed`. The existing safe-text regression required the exact phrase `no external links`, while the new note said `no external workbook links`. The workbook note was corrected without weakening the Rust requirement, and the focused failing test then passed `1/1`.

## Implementation

### Fixed, bounded Rust execution

- Added `WF_RunTrajectoryRustEngine` as the directional production entry point.
- Uses only the fixed colocated `wellforge-trajectory.exe` and `wellforge-trajectory.exe.sha256` artifacts.
- Verifies the manifest grammar and executable SHA-256 before request execution.
- Creates a fresh `%TEMP%\WellForgeTrajectory\<timestamp-attempt>` directory for every invocation.
- Executes `validate`, `run --diagnostics --no-backup`, `verify-result`, then `bridge` with explicit quoted paths, a bounded timeout and process termination, without `cmd.exe`.
- Preserves and guards the caller's prior `WF_Busy`, event and interaction states across the external-process sequence, then restores all three settings on success or failure. This makes both direct entry-point and Core-dispatched invocation non-reentrant.
- Reads only the canonical request hash from compact diagnostics JSONL; it never parses calculation result JSON in VBA.

### Explicit strict request export

- Reads authoritative, editable Well, Wellbore, plan trajectory and survey trajectory provenance from `Inputs!Q6:V9`.
- Reads explicit analysis/MD-datum/azimuth-reference/contract metadata from `Inputs!Q12:Q17`.
- Reads bounded row UUID identities from `Calc!JA7:JE506`; VBA does not synthesize UUIDs from labels, paths, environment data or randomness.
- Seeds honest SHA-256 digests of the sanitized fixture data instead of placeholder repeated-digit hashes.
- Uses `grid_north` consistently with the fixture's visible Grid North metadata.
- Requires every raw length, angle and angular-gradient unit selector and rejects unsupported/missing units before conversion.
- Requires transformation-critical controls and rejects Excel-error values even in optional numeric cells.
- Treats `Inputs!B13:B14` as absolute local-grid surface coordinates using the Plan length selector. Target coordinates use the Target selector, subtract the surface origin for Rust-local calculation and add the origin back for presentation.
- Uses the Plan angle selector for vertical-section azimuth in both request export and presentation.
- Requires projection `Inputs!K5:K8` to be either completely populated or completely blank.
- Rejects gaps and values beyond the fixed plan/survey 500, target 100, slide 200 and formation 100 row capacities.

The builder and guide explicitly state that bounded table rows are fixed identity slots: edit rows in place; sorting/reordering is not supported because UUID identity follows the slot.

### Complete in-memory bridge validation

- Requires exactly one first `H` record and exact tab-field counts for `H`, `P`, `S`, `R`, `T`, `L`, `F` and optional `X`.
- Validates bridge version, analysis UUID, request hash, result hash, engine version, accepted result status and deterministic flag.
- Validates strict record-kind phase order, exact request ID membership/order, duplicates, missing records and plan/survey/residual parity.
- Validates projection presence/count against the request.
- Uses a locale-independent finite-number grammar and rejects overflow/non-finite values.
- Enforces strict enums and status-dependent optional numeric/boolean field consistency, including empty invalid-target evaluation fields.
- Stages the entire bridge in arrays before any `ClearContents` or `.Value2` result mutation.

### Value-only commit and failure preservation

- Commits Plan, Survey, Targets, Slide Performance, Formation Tops, Results Survey Contract, Summary, Checks, canonical Calc blocks and chart-helper ranges as values.
- Clears obsolete formula-era helper blocks on each accepted run and repopulates every visible/result/check/summary/helper surface required after formula freezing.
- Retains presentation-only unit conversion, display-QC thresholds, headers and chart refresh in VBA; minimum-curvature, interpolation, target, slide-response, formation and projection physics remain Rust-authoritative.
- Preserves legacy presentation QC for short slides and slide-yield outliers without recomputing slide physics, and preserves the `NO ACTUAL PICKS` formation summary state.
- Adds the surface origin back to plan/survey/projection presentation values. Canonical target bridge positions remain exact surface-relative Rust values and are labeled accordingly.
- Writes accepted request/result/diagnostic paths, request/result hashes, Rust engine version, executable hash and a real Win32 UTC timestamp only after all value writes and presentation refresh succeed.
- Snapshots every mutated value range before commit. A commit failure attempts full reverse restoration; restoration success is observable. The client claims `LAST ACCEPTED VALUES PRESERVED` only when rollback succeeds and publishes `ROLLBACK INCOMPLETE` otherwise.
- Publishes the required `ENGINE UNAVAILABLE`, `ENGINE HASH MISMATCH`, `INVALID REQUEST`, `ANALYSIS FAILED` or `INVALID RESULT` state with an existing diagnostic path or an honest not-produced/intended-path message.

### Workbook and release integration

- Routes `WellForgeCore.WF_DispatchModel` directional production only to `WF_RunTrajectoryRustEngine`; there is no production VBA physics fallback.
- Keeps `WellForgeDirectional` as a legacy prototype plus presentation/header/chart helper and adds `WF_RefreshDirectionalPresentation`.
- Uses neutral `Calculation client / engine` Summary authority wording; the directional workbook does not claim VBA calculation authority.
- Seeds `NOT RUN` engine/hash evidence and a STOP result-integrity check so an unexecuted workbook cannot present READY or claim a verified hash.
- Declares the Rust directional authority and fixed artifacts in `data/wellforge-vba-engine-manifest.json`.
- Adds a pinned Rust 1.98.0 trajectory build script. It verifies the compiler identity and runs workspace formatting, warnings-denied workspace Clippy and workspace tests with `--locked --offline` before the package-scoped release build, copy and executable hash.
- Adds a standalone native trajectory release smoke-test script covering fixed artifact hash verification plus `validate`, `run`, `verify-result` and `bridge`.
- Wires the trajectory module and release build before Excel creation in the suite builder.
- Documents architecture, execution boundary, coordinate frame, fixed identity slots, evidence, rollback behavior, scope and Linux/Windows acceptance boundaries.

## Final verification

### Required combined Node gate

Command:

```text
node --test tests/trajectory_rust_engine_contract.test.mjs tests/trajectory_workbook_engine_contract.test.mjs tests/directional.test.mjs tests/directional_workbook_values.test.mjs tests/vba_engine_contract.test.mjs
```

Observed exit: `0`.

Observed result: `31 passed; 0 failed`; duration `145445.256429 ms`.

This is Linux source/model/regression evidence. Several inherited test titles mention Windows behavior, but their assertions inspect source declarations only; this run is not evidence of Windows compilation, COM, macro execution or rendering.

### VBA structural lint

Command:

```text
node tools/lint_vba.mjs
```

Observed exit: `0`.

Observed result: `VBA structural lint passed for 9 modules.`

This lint is not a VBA compiler.

### Required locked/offline trajectory CLI package gate

Command (from `engine`, with the repository's Rust 1.98.0 environment):

```text
cargo test -p wellforge-trajectory-cli --locked --offline
```

Observed exit: `0`.

Observed result: `2` focused unit tests and `13` real-process CLI integration tests passed; `0` failed.

### Diff validation

`git diff --check` exited `0` with no output.

## Acceptance boundary and concerns

- The standalone `tools/Test-WellForgeTrajectoryEngine.ps1` smoke-test script is declared and source-inspected, but it was not executed on Linux.
- A native Windows Rust 1.98.0 release build, native executable/hash smoke test, Excel/COM build, VBA compilation, macro execution, unit-switch runtime self-test, rendering and final XLSM/package acceptance remain Windows release gates and are not claimed here.
- Table sorting/reordering is intentionally unsupported because UUID identity follows fixed hidden Calc slots.
- `WF_CalcDirectional` remains as a legacy public prototype for existing regression compatibility, but production dispatch and the new adapter never call it as a fallback.
- The colocated SHA-256 manifest is an integrity checksum, not a digital signature or independent authenticity chain.

No remaining Task 3 source blocker was found.

## Files

- `VBA/WellForgeTrajectoryEngine.bas`
- `VBA/WellForgeCore.bas`
- `VBA/WellForgeDirectional.bas`
- `src/build_directional.mjs`
- `src/directional_workbook_model.mjs`
- `data/wellforge-vba-engine-manifest.json`
- `tools/Build-WellForgeTrajectoryEngine.ps1`
- `tools/Test-WellForgeTrajectoryEngine.ps1`
- `tools/Build-WellForgeVbaSuite.ps1`
- `tests/trajectory_rust_engine_contract.test.mjs`
- `tests/trajectory_workbook_engine_contract.test.mjs`
- `docs/VBA_ENGINE_GUIDE.md`
- `.superpowers/sdd/2026-08-29-rust-trajectory-execution/task-3-report.md`

## Post-commit review fix round 1

### Confirmed defects and focused RED

All tests below were added or tightened before production changes. Each command failed for the reviewed reason.

1. Canonical geometry boundary:

```text
node --test --test-name-pattern "adapter keeps presentation-only rotations out of canonical result blocks" tests/trajectory_rust_engine_contract.test.mjs
```

Observed exit: `1`. Observed result: `1 test; 0 passed; 1 failed`; duration `87.239329 ms`. The station helper lacked `PRESENTATION_ONLY_NOT_RUST_RESULT` and still accepted/wrote `verticalSection` and `crossline` values.

2. Canonical target helper schema:

```text
node --test --test-name-pattern "adapter maps the canonical target helper only from exact bridge fields or typed unavailable states" tests/trajectory_rust_engine_contract.test.mjs
```

Observed exit: `1`. Observed result: `1 test; 0 passed; 1 failed`; duration `143.10352 ms`. Target helper column 1 came from the visible label, unsupported columns were unset, and position columns added the surface origin instead of retaining exact Rust bridge values.

3. Pinned Windows release workspace gates:

```text
node --test --test-name-pattern "release scripts declare pinned workspace gates, hash, smoke-test wiring, and build before Excel" tests/trajectory_rust_engine_contract.test.mjs
```

Observed exit: `1`. Observed result: `1 test; 0 passed; 1 failed`; duration `124.060869 ms`. The script still declared package-scoped Clippy and tests instead of workspace gates.

4. Public-entry re-entrancy guards:

```text
node --test --test-name-pattern "public trajectory entry preserves caller runtime guards on success and failure" tests/trajectory_rust_engine_contract.test.mjs
```

Observed exit: `1`. Observed result: `1 test; 0 passed; 1 failed`; duration `94.361092 ms`. The entry point did not capture or guard `WF_Busy` and `Application.EnableEvents`.

The first directional regression then exposed one dependent formula-era defect:

```text
node --test tests/directional_workbook_values.test.mjs
```

Observed exit: `1`. Observed result: `8 tests; 7 passed; 1 failed`; duration `65083.975963 ms`. Exactly 59 active `Calc!BO7:BO65` formulas returned `#VALUE!` because the old dVS formula subtracted newly typed VS state columns.

A focused model contract pinned that dependency before its fix:

```text
node --test --test-name-pattern "in-memory canonical model marks VBA rotations as presentation-only state" tests/trajectory_workbook_engine_contract.test.mjs
```

Observed exit: `1`. Observed result: `1 test; 0 passed; 1 failed`; duration `22020.855645 ms`; `Calc!BO7` was `#VALUE!` rather than `NOT_RUN_RUST_REQUIRED`.

### Minimal fixes

- Canonical plan/survey, plan-at-survey and Survey Contract VS/crossline columns are now explicitly state-typed. VBA keeps rotations only in visible Plan/Survey and chart-helper presentation arrays. Rust bridge residuals remain the sole accepted source for canonical residual geometry.
- The formula-era source model uses `NOT_RUN_RUST_REQUIRED` for canonical dVS before execution and computes any initial dVS display/chart view only on the presentation surface, avoiding state arithmetic.
- The 19-column target helper now maps the Rust target UUID and bridge fields exactly. Legacy incidence/azimuth/dogleg/RF and north/east-difference slots use `UNAVAILABLE_NOT_IN_RUST_BRIDGE`; the vertical field is labeled and mapped as Rust `Vertical Difference`, not target dTVD.
- The Windows release builder now runs pinned formatting, warnings-denied workspace Clippy and locked/offline workspace tests before building the trajectory CLI release package.
- `WF_RunTrajectoryRustEngine` now preserves, sets and restores the caller's `WF_Busy`, `Application.EnableEvents` and `Application.Interactive` states on every success/failure path.

### Focused GREEN

Command:

```text
node --check src/directional_workbook_model.mjs && node --test --test-name-pattern "adapter keeps presentation-only rotations out of canonical result blocks|adapter maps the canonical target helper only from exact bridge fields or typed unavailable states|public trajectory entry preserves caller runtime guards on success and failure|release scripts declare pinned workspace gates, hash, smoke-test wiring, and build before Excel" tests/trajectory_rust_engine_contract.test.mjs
```

Observed exit: `0`. Observed result: `4 passed; 0 failed`; duration `124.239198 ms`.

Command:

```text
node --test --test-name-pattern "in-memory canonical model marks VBA rotations as presentation-only state|in-memory target helper declares exact Rust fields and typed unavailable legacy slots" tests/trajectory_workbook_engine_contract.test.mjs
```

Observed exit: `0`. Observed result: `2 passed; 0 failed`; duration `44565.706876 ms`.

After the dependent dVS fix, its focused model contract passed `1/1` in `22478.39437 ms`, and:

```text
node --test tests/directional_workbook_values.test.mjs
```

Observed exit: `0`. Observed result: `8 passed; 0 failed`; duration `61534.929657 ms`.

### Final GREEN verification for fix round 1

Combined Node gate:

```text
node --test tests/trajectory_rust_engine_contract.test.mjs tests/trajectory_workbook_engine_contract.test.mjs tests/directional.test.mjs tests/directional_workbook_values.test.mjs tests/vba_engine_contract.test.mjs
```

Observed exit: `0`. Observed result: `36 passed; 0 failed`; duration `160855.240295 ms`.

VBA structural lint:

```text
node tools/lint_vba.mjs
```

Observed exit: `0`. Exact output: `VBA structural lint passed for 9 modules.` This remains structural lint, not a VBA compiler.

Fresh isolated Rust verification used exactly this environment for every command:

```text
CARGO_HOME=/tmp/wellforge-rust/cargo
RUSTUP_HOME=/tmp/wellforge-rust/rustup
RUSTUP_TOOLCHAIN=1.98.0
CARGO_TARGET_DIR=/tmp/wellforge-rust/target
PATH=/tmp/wellforge-rust/cargo/bin:$PATH
```

Identity and formatting command:

```text
env CARGO_HOME=/tmp/wellforge-rust/cargo RUSTUP_HOME=/tmp/wellforge-rust/rustup RUSTUP_TOOLCHAIN=1.98.0 CARGO_TARGET_DIR=/tmp/wellforge-rust/target PATH=/tmp/wellforge-rust/cargo/bin:$PATH bash -c 'rustc --version --verbose && cargo --version --verbose && cargo fmt --all -- --check'
```

Observed exit: `0`. Identity output began `rustc 1.98.0 (88d9e12ae 2026-08-18)` and `cargo 1.98.0 (797e8a9bc 2026-08-05)`; formatting emitted no differences.

Workspace Clippy command:

```text
env CARGO_HOME=/tmp/wellforge-rust/cargo RUSTUP_HOME=/tmp/wellforge-rust/rustup RUSTUP_TOOLCHAIN=1.98.0 CARGO_TARGET_DIR=/tmp/wellforge-rust/target PATH=/tmp/wellforge-rust/cargo/bin:$PATH cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

Observed exit: `0`. Exact output: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.21s`.

Workspace test command:

```text
env CARGO_HOME=/tmp/wellforge-rust/cargo RUSTUP_HOME=/tmp/wellforge-rust/rustup RUSTUP_TOOLCHAIN=1.98.0 CARGO_TARGET_DIR=/tmp/wellforge-rust/target PATH=/tmp/wellforge-rust/cargo/bin:$PATH cargo test --workspace --locked --offline
```

Observed exit: `0`. All `126` workspace tests passed; `0` failed. This includes the `2` trajectory CLI unit tests and `13` real-process trajectory CLI integration tests.

`git diff --check` exited `0` with no output before the fix-round commit.

Fix commit: `fix: harden trajectory workbook Rust boundary` (this fix-round commit; its SHA is reported in the task handoff because a Git commit cannot contain its own identifier).

### Fix-round acceptance boundary

- The standalone trajectory smoke-test PowerShell script remains declared and source-inspected but was not executed on Linux.
- Windows PowerShell release execution, native Windows executable/hash smoke testing, Excel/COM, VBA compilation, macro execution, unit-switch runtime checks, rendering and workbook/package acceptance remain explicitly unrun.
- No repository workbook, package, ZIP, output artifact or `.work` path was modified or staged in this fix round.
