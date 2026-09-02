# WellForge Rust/VBA Engineering Workbook Suite — 2026-08-29 v7

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
