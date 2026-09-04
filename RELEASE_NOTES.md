# WellForge Rust/VBA Engineering Workbook Suite — 2026-08-29 v7

## Four standalone Rust engines — 2026-09-04

- Added a shared pinned Rust 1.98.0 Windows builder for `wellforge-bha.exe`, `wellforge-trajectory.exe`, `wellforge-torque-drag.exe`, and `wellforge-hydraulics.exe`.
- Added SHA-256 sidecars for the torque-drag and hydraulics executables and included all four engines in release archives and the Windows installer.
- Kept workbook authority explicit: BHA and directional dispatch to Rust; hydraulics and torque-drag remain VBA clients until their bridge migrations pass the Windows acceptance gates.

## Full release hardening — 2026-08-30

### PR #3 current-head acceptance hardening — 2026-08-31

- Replaced existence-based Windows evidence with six explicit, fail-closed gates bound to one workflow run, exact git SHA, and exact deterministic archive: native binaries, VBA compilation/Excel COM, unit switching, chart rendering, runtime rollback, and package acceptance.
- Added clean run directories, exact package allowlisting (including both declared license texts), SHA-256 manifests, deterministic ZIP creation, clean extraction, archive re-verification, executable-sidecar verification, and nonempty native Excel chart exports retained as PNG evidence.
- Added real injected-failure rollback self-tests for JSON exchange, the Rust BHA bridge, and the Rust trajectory bridge. Each test restores captured values and verifies equality before it can report success.
- Added a 75-minute parent watchdog that terminates the child process tree and every Excel process created after the dedicated runner's baseline while preserving partial gate evidence for the workflow's always-run evidence and artifact steps.
- Added explicit whole-project VBA compilation, a second clean extraction reopened in a fresh Excel process, and SHA-256 inventory of retained logs and rendered PNG evidence.
- Pinned action revisions, Node 24.19.0, Rust 1.98.0, cargo-deny 0.20.2, Ubuntu 24.04, and expanded dependency policy to include advisories.
- Linux release verification proves source and contract readiness only. Merge readiness requires the updated Linux checks to pass on this head. Release readiness remains explicitly withheld until the merged workflow runs on the qualified self-hosted Windows/Excel runner and `release-evidence.json` reports `overall_status: passed` for that exact commit.

- Added a deterministic repository verifier that hash-checks and materializes all five immutable source workbooks before running each Node test file in an isolated, timeout-bounded process, followed by VBA structural lint.
- Added a reproducible public-dependency release gate (`npm ci && npm run verify:release`) and pinned Linux CI for Node 24, Rust 1.98.0, rustfmt, warnings-denied Clippy, locked all-feature workspace tests, and cargo-deny 0.20.2.
- Split the private authoring-tool workbook contract from the public unit-contract surface so a fresh checkout no longer requires the internal `@oai/artifact-tool` package to execute the release gate.
- Added a manually dispatched Windows release workflow for a qualified desktop-Excel runner. It always retains JSON evidence, JSONL logs, executable hashes, and the five generated `.xlsm` files.
- The complete local authoring gate passed all 163 declared Node tests and deterministic structural lint for nine VBA modules. A clean public-dependency release run also passed.
- Rust compilation/policy CI and native Windows executable/hash, VBA compilation, Excel/COM, unit-switching, chart-refresh/rendering, rollback-runtime, and package-acceptance evidence remain platform gates; release readiness is not claimed until those workflows pass.

## Rust Trajectory Release 1 — source complete; Windows/Excel acceptance pending

- Added a pure Rust trajectory analysis lane for minimum-curvature plan/survey construction, exact partial-course interpolation, plan-versus-survey residuals, target envelopes, slide response, formation evaluation and optional tendency projection.
- Added strict request/result schemas, canonical hashes, actual build-captured compiler identity, atomic result/diagnostic/bridge outputs and collision preflight across every file-producing trajectory command.
- Added a bounded, versioned table bridge and a hash-verified, timeout-bounded directional VBA client with no VBA trajectory fallback or result-JSON parsing.
- Kept canonical trajectory calculation/result fields in SI. A unit-preserving trajectory wire adapter is not part of this release.
- Preserved WITSML-aligned Well, Wellbore, Trajectory and MD-datum identity/provenance without claiming full WITSML 2.0 or ETP conformance.
- Linux deterministic source, Rust and VBA-structure gates are covered. Native Windows executable/hash smoke, VBA compilation, Excel/COM behavior, unit switching, chart refresh/rendering, rollback runtime and package acceptance remain unrun platform gates.

## Rust BHA Release 1

- Added a Rust 2024 workspace with pinned `nalgebra`, `faer`, `parry3d-f64`, `levenberg-marquardt`, `uom`, `quick-xml`, `serde` and schema dependencies.
- Added strict WITSML 2.x-aligned source identities, trajectory stations, SI quantities, request/result schemas and deterministic fixtures.
- Added static lateral beam projection with buoyancy, inclination and compressive WOB geometric stiffness; OD/ID/hole crossing remains explicitly indication-only.
- Added natural modes, critical RPM, direct complex FRF and 1x/2x/3x Campbell margins.
- Added deterministic `validate`, `run`, `solve-static`, `solve-modal`, `verify-result`, `schema` and `version --json` CLI commands.
- Added request/result SHA-256 evidence, dependency-lock hash, compiler/target identity and atomic result replacement.
- Added a hash-verified, timeout-bounded VBA client with no BHA fallback to screening formulas.
- Moved BHA request/result JSON parsing, strict schema validation and evidence verification into Rust/`serde_json`; VBA consumes only a bounded, versioned tabular bridge.
- Added `Rust Engine Results`, a compact value-only decision surface backed by hidden `Rust Calc` arrays.
- Upgraded WITSML XML projection to `quick-xml 0.41.0` after the dependency gate identified `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` in the earlier parser.
- Added a Windows builder that compiles, tests, policy-checks, hashes and places `wellforge-bha.exe` beside the generated `.xlsm` files.

# Prior VBA visualization release — v6

## Delivered changes

- Added `BHA Geometry View`, a distance-from-bit XY decision surface that projects the hole wall, BHA OD, BHA ID and scaled estimated centreline together.
- Added projected radial-clearance and zero-clearance traces with explicit overlap indication; the workbook states that this is not solved contact, reaction force or nonlinear equilibrium.
- Added editable hole diameter, projection plane and deflection display-scale inputs, with display-unit-aware values and chart axes.
- Replaced categorical BHA bending views with connected distance-based bending moment, stress and screening-limit profiles.
- Added an operational low-frequency Campbell screen using 1x/3x/5x rotary orders and component first modes, while retaining high-frequency bit and near-bit values in the audit table.
- Extended `WellForgeBha.bas` to calculate and write all new geometry, clearance, bending-profile and vibration-chart arrays as values after `.xlsm` compilation.

- Standardized every MD/TVD profile as an XY drilling roadmap: calculated response on the horizontal axis, response axis at the top, depth on the vertical axis, and zero depth at the top.
- Rebuilt the hydraulics visuals as compatible multi-series pressure, ECD and annular-velocity roadmaps, plus a separate nozzle pressure envelope.
- Propagated SI, Imperial, Mixed and per-domain Custom unit selections into calculated values, table headers, chart source ranges, chart titles and axis labels.
- Corrected the directional plan and vertical-section plots so both plan and survey traces use display-unit helper data.
- Preserved BHA toolface units in JSON exchange and implemented overlaid WOB rose curves using the PolarPlotter-style radar/XY combination with transparent traces.
- Added a Windows VBA build self-test that exercises SI, Imperial and Custom displays, verifies depth-chart geometry, rejects residual worksheet formulas, and pauses with the full log path on success or failure.
- Standardized visible numeric precision to two decimal places and retained one shared mock case wherever inputs repeat across workbooks.
- Added a industry-software-informed torque-and-drag `Engineering Dashboard` with six operating modes, observed/mock hookload and torque, tension/torsional ratings, buckling limits, inclination, well context, friction sensitivity, and a selected-depth numerical reader.
- Added `Observed Data` with explicit mock provenance; production users must replace it with validated EDR or rig measurements.
- Added low/base/high hydraulic `Flow Cases` and a synchronized `Hydraulics Dashboard` for pressure, ECD, annular velocity, transport limits, and nozzle optimization.
- Added persisted `Chart Settings` to all five workbooks and a Windows visualization self-test that rejects missing dashboard series.
- Corrected hydraulic flow-path context boundaries and preserved useful nozzle-diameter precision on chart axes.
- Stabilized semantic chart colors for operations, observed data, sensitivity families, and engineering limits.

## Validation evidence

- Full Rust and workbook regression counts are recorded from the final packaged-byte verification rather than copied from an earlier suite revision.
- All five source workbooks rebuilt successfully.
- Visible-sheet formula-error scans matched zero entries.
- All declared OOXML parts and chart relationships passed integrity checks.
- VBA source passed deterministic structural lint.

The final `.xlsm` compilation is performed by desktop Excel on Windows through `tools/Build-WellForgeVbaSuite.cmd`.
