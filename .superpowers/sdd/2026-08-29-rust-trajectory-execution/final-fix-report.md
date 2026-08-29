# Rust trajectory final-fix report

## Status and scope

All findings in the final-review list were addressed from reviewed base `fd53627d8d6ccba5768327183c82328e00222e7e`. The implementation preserves atomic output replacement and accepts both `complete` and `complete_with_warnings`. No bare-version, executable-replacement TOCTOU, incremental output draining, chart rollback, inherited test-title, packaging, or other accepted deferral was expanded.

No repository workbook, ZIP, package, output, or `.work` path was opened, rebuilt, changed, or staged. The mandated Node gate reported its own temporary workbook fixture under `/tmp`; it did not target a repository deliverable. Native Windows, desktop Excel, COM, VBA runtime, and rendering acceptance were not run or inferred.

Every Cargo command used:

```text
CARGO_HOME=/tmp/wellforge-rust/cargo
RUSTUP_HOME=/tmp/wellforge-rust/rustup
RUSTUP_TOOLCHAIN=1.98.0
CARGO_TARGET_DIR=/tmp/wellforge-rust/target
PATH=/tmp/wellforge-rust/cargo/bin:$PATH
```

Dependency-using Cargo commands were run with `--locked --offline`.

## Root-cause confirmation

1. **Universal CLI collision safety:** `run` alone called the three-path collision helper. `bridge` read and then could replace its verified result input; `schema` wrote its two destinations sequentially with no preflight. Existing nearest-ancestor canonicalization correctly resolved lexical and symlink-parent aliases, but equality was exact, so a nonexistent Windows suffix differing only by case remained distinct.
2. **Compiler provenance:** both result evidence and `version --json` used the literal `rustc 1.98.0 (pinned)`. No build step queried the compiler Cargo actually invoked.
3. **Public documentation:** README described one Rust lane/four VBA prototypes and a VBA-only directional calculation entry point; the roadmap claimed a delivered unit-preserving trajectory wire boundary and said Excel built the executable.
4. **Warning aggregation:** overall limitations inspected plan/target/slide/formation coverage only. Reached target `invalid_geometry`/`numerical_overflow` and covered slide `low_inclination`/`numerical_overflow` states could still produce overall `complete`.
5. **Target rotation:** validation included rotation in the finite-value set but did not enforce the canonical half-open `[0, 2π)` range used by periodic request angles.
6. **VBA Boolean numerics:** VBA considers Boolean values numeric for coercion purposes; required and optional helpers had no `vbBoolean` guard ahead of `IsNumeric`, `CStr`, helper delegation, or `CDbl`.
7. **Bridge capacities:** production used the correct strict `actual > maximum` checks for 500 plan, 500 survey, 100 target, 200 slide, and 100 formation records. The defect was missing executable generation/validation coverage at exact and first-over-limit boundaries, not incorrect production behavior.

## TDD evidence

### RED

- `cargo test -p wellforge-trajectory-cli --test cli --locked --offline` exited **101**: **13 passed / 5 failed**. The five reviewed-reason failures were bridge same/symlink alias exit `0` instead of `40`, schema identical-output exit `0` instead of `40`, missing build-injected compiler identity, and low-inclination output classified `Complete` instead of `CompleteWithWarnings`.
- `cargo test -p wellforge-trajectory-cli --bin wellforge-trajectory --locked --offline` exited **101**: **8 passed / 1 failed**. The warning test observed only the inherited plan/slide coverage limitations and none of the four typed evaluation limitations. In that same pre-fix run, all six newly added real bridge-capacity generation tests passed, confirming the finding was a regression-coverage gap and required no production capacity change.
- `cargo test -p wellforge-trajectory-contract --test contract target_rotation_uses_the_canonical_half_open_range --locked --offline` exited **101**: **0 passed / 1 failed** because a negative rotation returned `Ok(())`. An initially chosen `TAU - EPSILON` upper-neighbor literal rounded back to `TAU`; the test was corrected to the immediate representable predecessor before GREEN.
- `node --test tests/trajectory_rust_engine_contract.test.mjs` exited **1**: **8 passed / 1 failed** because the required numeric helper had no `vbBoolean` rejection.
- After universal all-pairs wiring was green, `cargo test -p wellforge-trajectory-cli --bin wellforge-trajectory windows_case_aliases_collide_for_run_bridge_and_schema_paths --locked --offline` exited **101**: **0 passed / 1 failed**. Forced Windows policy accepted the `RESULT.JSON`/`result.json` pair, proving the nonexistent-suffix case bug independently of the Linux filesystem.

### Minimal fixes

- Generalized collision preflight to resolve every named path, compare every pair, and run before command input reads or output writes for `run`, `bridge`, and `schema`. Native Windows comparison case-folds canonical nearest-existing-parent identities; Unix remains case-sensitive. Existing atomic writers were unchanged.
- Added a fallible Cargo build script that executes Cargo's actual `$RUSTC -Vv`, requires valid UTF-8 plus `rustc`, `commit-hash`, `host`, and `release` identity fields, and injects one normalized identity. Result evidence and `version --json` share that exact compile-time value. There is no fallback claim.
- Added target/slide typed-state limitations while preserving all existing coverage warnings and completed-warning acceptance.
- Enforced target rotation in `[0, 2π)` and retained finite validation. Existing fixture/workbook zero rotations remain valid.
- Rejected `vbBoolean` explicitly in required, optional, and nullable-optional trajectory numeric helpers before coercion/delegation.
- Added exact bridge output-count acceptance at all five limits plus exact first-over-limit errors for 501 plan, 501 survey, 101 target, 201 slide, and 101 formation records.
- Updated README, roadmap, and release notes for two Rust lanes/three VBA prototypes, canonical-SI trajectory fields without a unit-preserving Rust wire adapter, Cargo/PowerShell executable build ownership, Excel workbook-client ownership, and honest WITSML/ETP and Windows gates.

### Focused GREEN

- CLI integration: `cargo test -p wellforge-trajectory-cli --test cli --locked --offline` → exit **0**, **18 passed / 0 failed**.
- CLI unit/capacity/policy: `cargo test -p wellforge-trajectory-cli --bin wellforge-trajectory --locked --offline` → exit **0**, **10 passed / 0 failed**.
- Contract: `cargo test -p wellforge-trajectory-contract --test contract --locked --offline` → exit **0**, **54 passed / 0 failed**.
- VBA source contract: `node --test tests/trajectory_rust_engine_contract.test.mjs` → exit **0**, **9 passed / 0 failed**.
- Targeted warnings-denied CLI Clippy → exit **0**.
- Fresh fixture generation with locked/offline `cargo run` → exit **0**; `cmp fixtures/expected/trajectory-release-one-minimal.result.json /tmp/wellforge-rust/trajectory-final-fix-fixture.json` → exit **0**, empty output.

## Fresh full gates

All commands below were run after the source, test, fixture, VBA, and public-document changes.

| Gate | Exact command | Exit | Exact evidence |
|---|---|---:|---|
| Rust format | `cargo fmt --all -- --check` | 0 | Empty output; no formatting differences. |
| Rust lint | `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 | `Finished dev profile`; zero warnings accepted. |
| Rust tests | `cargo test --workspace --locked --offline` | 0 | **140 passed / 0 failed**; nonzero suites were `2+4+3+2+3+1+3+10+18+2+54+24+2+6+6 = 140`; all doc-test suites contained 0 tests. |
| Node contracts | `node --test tests/trajectory_rust_engine_contract.test.mjs tests/trajectory_workbook_engine_contract.test.mjs tests/directional.test.mjs tests/directional_workbook_values.test.mjs tests/vba_engine_contract.test.mjs` | 0 | **37 passed / 0 failed / 0 cancelled / 0 skipped / 0 todo**; reported duration `154690.723701 ms`. |
| VBA structural lint | `node tools/lint_vba.mjs` | 0 | `VBA structural lint passed for 9 modules.` |
| Whitespace | `git diff --check` | 0 | Empty output. |

Full test total: **177 passed / 0 failed** (140 Rust + 37 Node).

## Explicitly unrun platform gates and concerns

The following remain unrun and are not claimed: native Windows `wellforge-trajectory.exe` build/hash smoke, VBA compilation, Excel/COM success and failure flows, unit switching, chart refresh/rendering, rollback runtime, and packaged-deliverable acceptance. WITSML-aligned source identity is preserved; full WITSML 2.0 and ETP conformance are not claimed. No additional implementation concern remains from the reviewed findings.
