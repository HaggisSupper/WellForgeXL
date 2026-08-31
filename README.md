# WellForge Analysis Workbook Suite — canonical SI with configurable display units

This package contains five Excel engineering workbooks, two production Rust engine lanes, and three VBA prototype engines. The BHA and directional workbooks require their colocated, hash-verified `wellforge-bha.exe` and `wellforge-trajectory.exe`; neither falls back to VBA screening or trajectory calculations. The other three workbooks remain on the VBA prototype engines pending their Rust ports. `T&D 4.002b.xlsm` informed the workflow and visual topology only; its protected macros and catalog are not reproduced.

The workbooks are:

- `API_7G_Drill_String_Strength_and_Torque_SI.xlsx`
- `Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx`
- `Torque_Drag_and_Buckling_SI.xlsx`
- `BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx`
- `Directional_Drilling_Wellplan_and_Survey_SI.xlsx`

## Use

1. On Windows, double-click `tools/Build-WellForgeVbaSuite.cmd`, or run `tools/Build-WellForgeVbaSuite.ps1`. A fresh checkout uses the versioned source workbooks under `workbooks/source`; Node authoring dependencies are not required. The PowerShell window remains open on success or failure and prints the full JSONL log path.
2. Open the resulting files in `outputs/vba-engine`, enable macros, and enter or paste source inputs in the blue input cells. In the directional workbook, Plan, Survey, Target, Slide, and Formation source values retain their declared raw input units.
3. The three VBA prototype engines convert inputs to canonical SI in memory. The BHA and directional VBA clients invoke their colocated Rust engines. Rust/`serde_json` parses and validates each lane's JSON and emits a bounded versioned table bridge; the VBA clients do not parse result JSON. All lanes write value-only outputs and refresh charts.
4. Use `Unit Map!B5` to select SI, Imperial, Mixed or Custom display values. In Custom mode, choose SI, Imperial or Mixed independently for every physical domain from the dropdowns in column J. VBA updates unit labels and displayed values together.
5. Use the Summary buttons to Calculate, Validate, Load JSON or Save JSON. Relevant input changes also trigger VBA recalculation automatically.
6. Review `Checks`, then `Summary` for the governing constraint. Use `Results`, `Graphs`, and the discipline-specific detail tabs to investigate the depth/profile outcome.
7. For torque and drag, use `Engineering Dashboard` for synchronized operation, observed/mock, rating, buckling, inclination, and friction-sensitivity roadmaps. Replace `Observed Data` mock values with validated EDR/rig measurements before operational use.
8. For hydraulics, use `Flow Cases` to define low/base/high flow multipliers and `Hydraulics Dashboard` for synchronized pressure, ECD, annular-velocity, and nozzle-envelope review.
9. For BHA screening, use `BHA Geometry View` to compare the wellbore, projected OD/ID, scaled estimated centreline, radial clearance, bending moment and bending stress versus distance from bit. Use `Vibration Modes` for the operational low-frequency Campbell screen and full modal audit table.
10. For the Rust lanes, use the BHA `Rust Engine Results` surface or the directional Rust evidence/results surfaces for value-only decisions. Their engine sheets record source UUID/URI identities, request/result hashes and execution state; hidden calculation blocks remain chart data rather than calculation authority.
11. Use `Chart Settings` to persist selected MD, observed/limit visibility, sensitivity multipliers, well-context visibility, and report composition with the saved case.

Depth-based charts use the drilling-roadmap convention: calculated response runs horizontally across the top, MD/TVD runs vertically, and zero depth is at the top. Measured, modeled and limit series are overlaid only when their physical dimensions are compatible. See `docs/DRILLING_CHART_STANDARD.md`.

## Limits

These are transparent planning/review workbooks, not an approved operational decision system. BHA Rust Release 1 solves a linear lateral beam/static/modal model and direct unit-force FRF, but its negative-clearance output remains an interference indication: it does not solve contact reactions, frictional impact, full six-degree rigid-body dynamics, whirl or DAT calibration. See `docs/BHA_ENGINE_VALIDATION.md`. Before operational use, have the organization’s qualified drilling engineer validate model method, input data, tool specifications, applicable standard edition and sign-off process.

## JSON exchange

All five workbooks share the versioned contract in `schema/wellforge-analysis-exchange.schema.json` and the same mock case in `data/wellforge-mock-case.json`. Every physical value is stored as `{ "value": number, "unit": string }`; imports convert to canonical SI while exports preserve the original unit when the value has not changed. This workbook exchange is distinct from the Rust trajectory request/result boundary: its calculation fields are canonical SI, and a unit-preserving trajectory wire adapter is not yet implemented or claimed.

Each workbook contains:

- `Exchange Map`: the workbook-owned JSON-to-cell contract.
- `Exchange State`: hidden round-trip unit state.
- `Exchange Buffer`: JSON payload in `B5`, followed by action, status and diagnostics.

### Desktop Excel / VBA

Run `tools/Build-WellForgeVbaSuite.ps1` on Windows with desktop Excel to create macro-enabled copies under `outputs/vba-engine`. PowerShell invokes Cargo to build, test, and hash the Rust executables before Excel builds and hosts the macro-enabled workbook clients. Excel must allow Trust Center access to the VBA project object model. The source `.xlsx` files are never overwritten. Existing `.xlsm` outputs are timestamp-backed-up before replacement.

Each generated workbook exposes:

- `WellForge_CalculateAll` — run the discipline-specific calculation client/engine entry point and refresh outputs/charts.
- `WellForge_ValidateModel` — recalculate and surface model/check-sheet exceptions.

- `WellForge_LoadJson` — select and import a JSON file.
- `WellForge_SaveJson` — export through a temporary file with timestamped backup when replacing an existing file.
- `WellForge_ValidateExchange` — validate the current buffer/mapping without changing engineering inputs.

The builder verifies that the engine runs, exercises SI, Imperial and Custom unit switching, validates the required dashboard/series composition and depth-chart geometry, confirms the expected engine version, and rejects any final workbook that still contains worksheet formulas. See `docs/VBA_ENGINE_GUIDE.md`.

### Office Script

Add `OfficeScripts/WellForgeJsonExchange.ts` from Excel's Automate tab and call `main` with `Import`, `Export` or `Validate`. Office Scripts cannot open a local file picker, so paste JSON into `Exchange Buffer!B5` or pass it through the `jsonText` parameter; exported JSON is returned and written back to `B5`.

`OfficeScripts/WellForgeWorkbookRefresh.ts` remains available for recalculation and Checks-sheet stamping. It does not contain or replace engineering calculations.

See `docs/JSON_EXCHANGE_GUIDE.md` for the complete workflow and recovery notes.

The approved Rust port sequence and the soft-string-to-stiff-string interval contract are recorded in `docs/RUST_ENGINE_ROADMAP.md`.

## Source verification

Run `node tools/verify-node.mjs` after the repository's workbook-authoring runtime is available. The verifier checks and materializes all five immutable source workbooks before running every Node test file in an isolated, bounded child process, then runs the deterministic VBA structural lint. `--materialize-only` performs only the workbook hash and materialization gate.

The workbook authoring tests use the private `@oai/artifact-tool` development runtime and are therefore an extended development gate. The release CI gate validates checked-in workbook packages and all dependency-free contracts without claiming it can regenerate those packages. Native Rust executables and Excel/VBA/COM behavior remain separate Windows evidence gates.

The manual `Windows Excel release verification` workflow is the release-acceptance authority. It creates a run-scoped package from an exact git SHA, smoke-tests both native executables, verifies and cleanly extracts a deterministic archive, opens all five extracted `.xlsm` clients in desktop Excel, runs VBA compilation/COM, SI/Imperial/Custom switching, chart export to nonempty PNG files, injected rollback equality checks, and final package acceptance. `release-evidence.json` is fail-closed unless every named gate passes for the same workflow run, git SHA, and archive. A green Linux source workflow or the presence of generated files is not release evidence.
