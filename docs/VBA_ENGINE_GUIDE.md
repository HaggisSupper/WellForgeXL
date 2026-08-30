# WellForge Hybrid Rust/VBA Engine Guide

## Final architecture

The `.xlsx` files under `outputs` are regression references and styled templates. The final `.xlsm` workbooks are built by desktop Excel and contain:

- `WellForgeCore`: dispatch, unit selection, formula freezing, controls, status, chart refresh and automatic input-change handling.
- `WellForgeApi7G`: tubular strength, six load cases and combined-utilization envelopes.
- `WellForgeHydraulics`: all flow-path sections, Darcy pressure loss, ECD screening and nozzle optimization.
- `WellForgeTorqueDrag`: survey-derived drag, six operating states, torque and buckling screens.
- `WellForgeBhaEngine`: hash-verifies and invokes the colocated Rust BHA executable, validates request/result evidence, and writes value-only static, modal, FRF and Campbell outputs.
- `WellForgeBha`: retained prototype/presentation helpers; it is not the BHA calculation authority.
- `WellForgeTrajectoryEngine`: hash-verifies and invokes the colocated Rust trajectory executable, exports explicit workbook provenance and row identities, validates the complete fixed bridge in memory, and commits value-only plan, survey, target, slide, formation and projection outputs.
- `WellForgeDirectional`: retained legacy prototype plus presentation, unit-header and chart-refresh helpers; it is not the directional calculation authority and production dispatch never calls its VBA physics.
- `WellForgeJsonExchange`: unit-preserving JSON import/export and validation.

The API 7G, hydraulics and torque/drag engines calculate in VBA arrays. BHA and directional use fixed, colocated, SHA-256-verified Rust executables with no VBA calculation fallback. The trajectory client owns explicit request projection, SI/display conversion and presentation only; it does not parse result JSON or reproduce Rust geometry, interpolation, target, slide, formation or projection physics. Charts remain native Excel chart objects pointing at value-only ranges.

## Build

1. Extract the ZIP to a local Windows folder.
2. In Excel, enable **Trust access to the VBA project object model** under Trust Center > Macro Settings.
3. Double-click `tools/Build-WellForgeVbaSuite.cmd`.
4. The console remains open regardless of outcome. Read the concise error and the JSONL log path if the build fails.
5. Open the completed files under `outputs/vba-engine` and enable macros.

For CI or deliberate unattended use only:

```powershell
tools\Build-WellForgeVbaSuite.ps1 -NoPause
```

Use `-VisibleExcel` when diagnosing an Excel-side problem.

## Build gates

For each workbook the builder:

1. Preflights the source `.xlsx` OOXML manifest and rejects any declared package part that is missing before Excel starts.
2. Builds, tests and SHA-256 hashes `wellforge-bha.exe`, writing its checksum manifest before Excel starts.
3. Selects and verifies Rust 1.98.0, runs the trajectory CLI gates locked/offline, builds `wellforge-trajectory.exe`, and writes its colocated SHA-256 checksum manifest before Excel starts.
4. Saves a temporary `.xlsm` through Excel.
5. Imports all VBA/client modules and installs `ThisWorkbook` events.
6. Executes `WellForge_BuildInitialize`, which snapshots/removes POC formulas and runs the appropriate authority.
7. Executes `WellForge_UnitSwitchSelfTest`, which cycles SI, Imperial and a per-domain Custom selection and rejects unchanged calculated values or UOM labels.
8. Verifies that depth roadmaps retain response-X, reversed depth-Y geometry.
9. Rejects the workbook if any worksheet formula remains.
10. Confirms client/runtime version `2.0.0-vba` on `Summary`; BHA records evidence on `Rust Engine`, while directional records execution mode, state, paths, request/result hashes, engine version, executable hash and real accepted UTC on `Results!O5:P14`.
11. Saves the self-contained `.xlsm`, backing up any previous output.

## Runtime controls

The `Summary` sheet contains Calculate, Validate, Load JSON and Save JSON buttons. The workbook also recalculates when relevant inputs, survey data, unit selections or discipline tables change. Event recursion is suppressed while the engine writes outputs.

The calculation status block records engine state, version, timestamp and detail. Two-decimal worksheet display is retained; unit-map conversion factors retain their higher precision.

Each directional calculation creates a fresh `%TEMP%\WellForgeTrajectory\<run-id>` directory and invokes `validate`, `run` with diagnostics, `verify-result`, then `bridge` through a bounded process without `cmd.exe`. The client accepts only the fixed bridge grammar after checking version, request/result hashes, status, deterministic flag, counts, capacities, record ordering and exact ID parity. All records are staged before any result range is cleared or written. Commit snapshots are restored if a write or presentation refresh fails; if restoration itself is incomplete, the result state says so instead of claiming that prior values were preserved.

The seeded directional fixture uses Grid North consistently. Workbook North/East inputs are absolute local-grid coordinates, with the surface origin stored in `Inputs!B13:B14` and interpreted using the Plan length selector. Target center North/East use the Target length selector, are translated to surface-relative coordinates for Rust, and the surface origin is added back when station and projection results are presented. Projection inputs `Inputs!K5:K8` must be either fully populated or fully blank.

The bounded directional table rows are fixed identity slots backed by UUIDs on hidden `Calc!JA:JE`. Edit row values in place; sorting or reordering the bounded tables is not a supported workflow because identity follows the slot, not a moved row.

Linux source tests verify the adapter, workbook model and release-script wiring. They do not claim acceptance of a native Windows executable, Excel/COM automation, compiled VBA, macro execution, rendering or final workbook packages; those remain Windows release gates.

The manual `Windows Excel release verification` workflow targets only a self-hosted runner carrying the labels `Windows` and `wellforgexl-excel`. That runner must have desktop Excel, trusted VBA project access, and Rust 1.98.0 available. The workflow always uploads JSONL logs and `outputs/release-evidence.json`; missing executables, hash mismatches, missing workbooks, or missing logs produce `overall_status: failed` rather than an inferred pass.

## Scope

These workbooks remain transparent engineering planning and screening tools. Rust BHA Release 1 is a linear beam/static/modal model and explicitly rejects rigid or modal-flexible representations; contact force, nonlinear DAT calibration and six-degree rigid-body dynamics are future validation-gated releases. The trajectory result does not include ISCWSA covariance, anti-collision/separation-factor analysis or pipe-fatigue calculation. Execution technology does not remove the need for authoritative provenance, approved input data, method validation and qualified engineering review.
