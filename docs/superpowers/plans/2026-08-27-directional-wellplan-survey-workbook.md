# Directional Drilling Wellplan and Survey Workbook Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans`. Use `superpowers:test-driven-development` for each behavior change and `superpowers:verification-before-completion` before delivery.

**Goal:** Add a fifth, SI-canonical, formula-driven WellForge workbook that refactors the uploaded directional wellplan/survey workbook into an auditable trajectory, survey-error, target, slide-performance, formation-top, and projection model.

**Architecture:** `src/directional_contract.mjs` owns stable sheet/capacity/column constants, `src/directional_formulas.mjs` owns row-aware Excel formula factories, and `src/build_directional.mjs` owns workbook layout. `src/directional_reference_data.mjs` contains only the sanitized nonblank example inputs preserved from the uploaded reference. Shared workbook helpers accept a custom sheet topology, and `Unit Map` becomes fully data-driven. Canonical SI calculation tables and chart-helper tables live on `Calc`; visible sheets contain inputs, converted results, checks, and native Excel charts. Independent JavaScript math in tests calculates expected values but is never imported by production code.

**Tech stack:** `$CODEX_PRIMARY_RUNTIME_NODE`, `@oai/artifact-tool`, Node test runner, Excel formulas, native Excel charts, OOXML ZIP inspection, and the available `soffice` executable for a second calculation/open-save pass.

**Spec:** `docs/superpowers/specs/2026-08-27-directional-wellplan-survey-workbook-design.md`

**Reference input:** `../upload/01-Directional_Drilling_Wellplan_Survey_Workbook.xlsx`

**Primary output:** `outputs/Directional_Drilling_Wellplan_and_Survey_SI.xlsx`

The workspace is not initialized as a Git repository, so this plan uses test/verification checkpoints rather than inventing commit steps.

## Global execution rules

- Before the first workbook-authoring command, read the complete spreadsheet-skill references `style_guidelines.md`, `artifact_tool_docs/API_QUICK_START.md`, and `features/charts.md`, then run the required artifact-operation marker exactly once if it exists in the runtime.
- Use `apply_patch` for source, test, and documentation edits. Mechanical formatting, package copies, and archive construction may use deterministic CLI commands.
- Start every implementation task with a failing test or explicit failing inspection. Do not weaken a test to make the implementation pass.
- Keep engineering formulas in Excel. JavaScript may generate repetitive formula rows, create charts, and verify outputs.
- Preserve the four existing workbook builders and their output names.
- Never copy personal metadata, stale Windows paths, chart-only sheet artifacts, or dead VBA instructions from the reference workbook.

---

### Task 1: Freeze the sanitized reference dataset and independent math oracle

**Files:**

- Create: `src/directional_reference_data.mjs`
- Create: `tests/directional_math.test.mjs`
- Create: `tests/helpers/directional_math_oracle.mjs`

**Interfaces:**

- `src/directional_reference_data.mjs` exports `directionalReferenceData` with `metadata`, `inputs`, `plan`, `survey`, `targets`, `slideIntervals`, and `formationTops` arrays.
- `tests/helpers/directional_math_oracle.mjs` exports `minimumCurvature(stations)`, `interpolateMinimumCurvature(stations, md)`, `positionError(actual, planned, vsAzimuth)`, `slideVector(interval)`, and `targetEnvelopeStatus(target, position)`.
- Production code must not import the test oracle.

- [ ] Write `tests/directional_math.test.mjs` first so it fails because the fixture and oracle do not exist.
- [ ] Assert that the fixture contains exactly the nonblank source rows and no value containing a personal author name, `C:\\Users`, or `VBA_Macros.bas`.
- [ ] Use the uploaded workbook only as a read-only extraction source. Preserve displayed input values, target inputs, slide inputs, and formation picks; exclude formulas, authorship, comments, web extensions, and file paths.
- [ ] Implement the independent oracle with direction vectors and clamped minimum curvature.
- [ ] Assert the 60 plan and 60 survey stations reproduce the audited source positions within `1E-8 m` and DLS within `1E-10 rad/m` after unit conversion.
- [ ] Assert the final sample result is equivalent, within `0.05 m`, to crossline `1,408.48 ft`, horizontal error `1,424.24 ft`, and 3D error `1,436.38 ft`.
- [ ] Add exact-station, mid-interval, near-zero-dogleg, before-start, and beyond-TD partial-interpolation cases.
- [ ] Add pure-build, pure-turn, mixed-toolface, low-inclination, and zero-slide-length vector cases.
- [ ] Add point/circle/rotated-ellipse/rotated-box hit and miss cases, plus the corrected formation structural-high sign.
- [ ] Run:

  ```bash
  "$CODEX_PRIMARY_RUNTIME_NODE" --test tests/directional_math.test.mjs
  ```

  Confirm all independent numerical tests pass before workbook layout begins.

### Task 2: Generalize the shared workbook and unit contracts without regressing four existing books

**Files:**

- Modify: `src/common.mjs`
- Modify: `src/workbook.mjs`
- Modify: `tests/unit_contract.test.mjs`
- Modify: `tests/unit_display_contract.test.mjs`
- Create: `tests/directional_structure.test.mjs`

**Interfaces:**

- `createSuiteWorkbook(title, options = {})` accepts `options.sheetNames`; the current eight-sheet topology remains the default.
- `UNIT_ROWS` includes explicit `mixedMultiplier` values rather than deriving mixed factors from domain-name conditionals.
- Add `Angular gradient`: SI `rad/m`, Imperial `deg/100ft`, Mixed `deg/30m`, factors `1746.37535955875` and `1718.87338539247`.

- [ ] Add failing tests that request the directional topology and assert the exact visible order plus trailing `Calc`.
- [ ] Add a failing test that checks the new `Angular gradient` row and its selected unit/factor formulas.
- [ ] Add a regression test that `createSuiteWorkbook('x')` still returns `Summary`, `Inputs`, `Survey`, `Results`, `Graphs`, `Unit Map`, `Checks`, and `Calc` in the original order.
- [ ] Refactor `UNIT_ROWS` so every row owns both Imperial and Mixed factors. Preserve existing selected-factor row numbers by appending `Angular gradient` after `Speed`.
- [ ] Correct Mixed Stress from `0.001` to `0.000001`, fix `displayFormula()` so its default offset is a literal zero rather than the invalid reference `'Unit Map'!0`, and use `UNIT_SYSTEMS` as the single selector-validation source.
- [ ] Make an invalid/tampered display selector produce an explicit invalid state instead of falling through to Mixed.
- [ ] Refactor `addUnitMap` to consume the row data directly and add a visible note that display-unit changes do not reinterpret raw inputs.
- [ ] Refactor `createSuiteWorkbook` to accept the custom list without creating duplicate `Calc`, `Unit Map`, or `Checks` sheets.
- [ ] Run:

  ```bash
  "$CODEX_PRIMARY_RUNTIME_NODE" --test tests/unit_contract.test.mjs tests/directional_structure.test.mjs
  ```

### Task 3: Establish the directional formula contract and input surfaces

**Files:**

- Create: `src/directional_contract.mjs`
- Create: `src/directional_formulas.mjs`
- Create: `src/build_directional.mjs`
- Create: `tests/directional.test.mjs`

**Interfaces:**

- Export `directionalFormulaPlan()` with keys `doglegAngle`, `ratioFactor`, `deltaTvd`, `deltaNorth`, `deltaEast`, `doglegSeverity`, `slerpNorth`, `slerpEast`, `slerpVertical`, `partialPosition`, `crosslineError`, `error3d`, `effectiveTurn`, `responseToolface`, `targetEnvelope`, and `formationHighLow`.
- Export `buildDirectionalWorkbook()` returning an `@oai/artifact-tool` workbook.
- Directional topology is `Summary`, `Inputs`, `Plan`, `Survey`, `Targets`, `Slide Performance`, `Formation Tops`, `Results`, `Graphs`, `Unit Map`, `Checks`, `Calc`.
- `directionalFormulaPlan()` must call the same factories used to populate workbook cells; it may not be a disconnected surrogate formula set.

- [ ] Write failing formula-contract tests. Require `ACOS(MAX(-1,MIN(1,...)))`, a small-dogleg ratio-factor branch, wrapped azimuth deltas, direction-vector interpolation, crossline/3D formulas, vector effective turn/toolface, rotated target coordinates, and `Prognosed TVD - Actual TVD`.
- [ ] Write a failing workbook-structure test for exact sheets, capacity labels, raw-input unit selectors, input validation, visible method notes, and seeded reference rows.
- [ ] Implement `directionalFormulaPlan()` as auditable representative Excel formulas; it is a test/traceability surface and must match the formulas emitted into `Calc`.
- [ ] Build `Inputs` with metadata, raw-input units, display-unit explanation, user DLS limit, projection settings, target defaults, and slide-quality controls. Use blue editable cells and data validation.
- [ ] Build `Plan`, `Survey`, `Targets`, `Slide Performance`, and `Formation Tops` as native Excel Tables over 500/500/100/200/100-row formula-supported input regions. Freeze headers and keep raw inputs blue.
- [ ] Seed only the sanitized reference data into the blue input cells. Leave unused capacity blank.
- [ ] Add active-row, used-count, and capacity formulas; blank rows must stay visually blank and must not produce statuses.
- [ ] Run:

  ```bash
  "$CODEX_PRIMARY_RUNTIME_NODE" --test tests/directional.test.mjs tests/directional_structure.test.mjs
  ```

### Task 4: Implement canonical plan and survey minimum-curvature tables

**Files:**

- Modify: `src/build_directional.mjs`
- Modify: `tests/directional.test.mjs`
- Create: `tests/directional_workbook_values.test.mjs`

**Canonical `Calc` columns:**

- Plan: active, station, MD m, Inc rad, Azi rad, dMD m, dogleg rad, RF, dTVD m, dN m, dE m, TVD m, N m, E m, VS m, crossline m, DLS rad/m, row status.
- Survey: the same canonical geometry plus source and row status.

**Representative formulas:**

```excel
=ACOS(MAX(-1,MIN(1,COS(I_prev)*COS(I)+SIN(I_prev)*SIN(I)*COS(A-A_prev))))
=IF(ABS(beta)<1E-9,1+beta^2/12+beta^4/120,2*TAN(beta/2)/beta)
=dMD/2*(SIN(I_prev)*COS(A_prev)+SIN(I)*COS(A))*RF
=dMD/2*(SIN(I_prev)*SIN(A_prev)+SIN(I)*SIN(A))*RF
=dMD/2*(COS(I_prev)+COS(I))*RF
```

- [ ] Add failing tests that inspect the plan/survey canonical columns, clamping, small-angle branch, cumulative coordinates, and row guards.
- [ ] Add workbook-value tests that export the workbook, recalculate/open-save it with `soffice`, re-import it, and compare selected station outputs to the independent oracle.
- [ ] Convert raw input MD/angles to SI using explicit input-unit cells. Do not reference `Unit Map!B5` for canonical conversion.
- [ ] Implement 500 formula rows for plan and survey with error guards for blank rows, nonnumeric values, invalid inclination, and non-increasing MD.
- [ ] Populate visible calculated Plan/Survey columns by multiplying canonical SI cells by selected `Unit Map` factors. Unit labels must be formulas.
- [ ] Canonicalize azimuth to `[0, 2*pi)` while retaining a QC note when raw values required normalization.
- [ ] Run:

  ```bash
  "$CODEX_PRIMARY_RUNTIME_NODE" --test tests/directional.test.mjs tests/directional_workbook_values.test.mjs
  ```

### Task 5: Implement exact partial-MC interpolation and positional-error diagnostics

**Files:**

- Modify: `src/build_directional.mjs`
- Modify: `tests/directional_workbook_values.test.mjs`

**Interfaces:**

- `Calc` contains bounded plan-at-survey helper columns for lower interval, fraction, SLERP vector, interpolated orientation, partial RF/displacement, plan-at-MD coordinates, coverage state, and all position-error components.
- `Survey` exposes `dN`, `dE`, `dTVD`, `dVS`, along-track, crossline, horizontal, and 3D error.

- [ ] Add failing exported-workbook tests for exact plan stations, mid-station interpolation, near-zero-dogleg interpolation, and beyond-plan-TD behavior.
- [ ] Locate the bracketing interval with a bounded approximate match. Clamp exact-TD to the last valid interval with fraction one; return an explicit state outside the plan range.
- [ ] Build direction vectors `uN=SIN(I)*COS(A)`, `uE=SIN(I)*SIN(A)`, `uV=COS(I)` and SLERP helpers. Use normalized linear interpolation when total dogleg is below `1E-9`.
- [ ] Reconstruct partial `Inc=ACOS(CLAMP(uV))` and `Azi=MOD(ATAN2(...),2*PI())`, then calculate the partial minimum-curvature displacement from the lower plan station.
- [ ] Calculate signed component errors and absolute horizontal/3D magnitudes using the vertical-section axis contract.
- [ ] Assert the sample final station displays the audited crossline, horizontal, and 3D miss instead of only TVD/VS differences.
- [ ] Run the workbook-value test and inspect the final sample row formulas and values.

### Task 6: Implement slide vector calibration, projections, targets, and formation tops

**Files:**

- Modify: `src/build_directional.mjs`
- Modify: `tests/directional.test.mjs`
- Modify: `tests/directional_workbook_values.test.mjs`

**Interfaces:**

- Slide helpers emit build, effective-turn, residual build/turn, slide yield, response toolface, wrapped toolface error, length-weighted trailing calibration, and QC state.
- Projection helpers emit projected orientation/position at bit MD and project-ahead MD with confidence state.
- Target helpers emit local rotated offsets, envelope utilization, vertical utilization, actual/projected basis, and hit/miss state.
- Formation helpers emit exact actual TVD at pick MD, `High(+)/Low(-)`, structural sense, and coverage/tolerance state.

- [ ] Add failing formula and exported-value tests for pure build, pure turn, mixed response, low-inclination exclusion, short-slide exclusion, and length-weighted calibration.
- [ ] Implement effective turn as wrapped azimuth change times sine of average inclination. Subtract rotary build/effective-turn components before dividing interval response by slide length.
- [ ] Calculate response toolface from the residual build/turn vector and wrap error to `[-pi, pi]`.
- [ ] Implement deterministic projection from the latest valid survey using user build/effective-turn tendencies and the same minimum-curvature displacement formulas. Guard effective-turn conversion at low inclination.
- [ ] Add failing point/circle/rotated-ellipse/rotated-box target tests. Require both horizontal envelope and vertical tolerance for `HIT`.
- [ ] Use actual interpolated coordinates when surveyed through target MD; otherwise use projected coordinates and label the result `PROJECTED`.
- [ ] Add formation tests proving a shallower actual pick produces positive `HIGH`. Use actual partial-MC interpolation, not coordinate linear interpolation.
- [ ] Run:

  ```bash
  "$CODEX_PRIMARY_RUNTIME_NODE" --test tests/directional.test.mjs tests/directional_workbook_values.test.mjs
  ```

### Task 7: Build the decision surface, Survey Contract, and checks

**Files:**

- Modify: `src/build_directional.mjs`
- Modify: `tests/directional.test.mjs`
- Modify: `tests/directional_workbook_values.test.mjs`

**Interfaces:**

- `Summary` state formula returns `STOP`, `CAUTION`, or `READY` from `Checks` severity counts.
- `Results` publishes current state plus the canonical Survey Contract columns specified by the design.
- `Checks` has check name, measured result, status, severity, and required action.

- [ ] Add failing tests for summary state, current survey MD, current crossline/3D error, DLS vs user limit, next-target state, and required-action text.
- [ ] Implement checks for unit metadata, reference metadata, row capacity, numeric/range validity, strictly increasing MD, duplicate MD, gap warning, plan coverage, DLS limit, target validity, slide quality, formation coverage, and formula-error sentinels.
- [ ] Add visible `INFO` rows stating that the workbook has no ISCWSA covariance/error model, separation factor, anti-collision result, pipe-fatigue calculation, VBA, or external links.
- [ ] Implement `STOP` for invalid unit/trajectory/formula/decision-target states and `CAUTION` for plan overrun, DLS exceedance, projected miss, or low-confidence projection.
- [ ] Publish the 500-row canonical Survey Contract on `Results`, with blank unused rows and explicit source/status fields.
- [ ] Apply conditional formatting to summary and checks states without relying on color alone; status text must remain visible.
- [ ] Run all directional tests.

### Task 8: Create formula-backed native charts and perform visual QA

**Files:**

- Modify: `src/build_directional.mjs`
- Modify: `tests/render_contract.test.mjs`
- Modify: `.work/verify_suite.mjs`

**Required charts:** plan view; vertical section; Inc/Azi strips; DLS and limit; signed positional errors; horizontal/3D error; slide yield/QC; target-state comparison.

- [ ] Add a failing drawing-inspection test requiring at least eight native charts on `Graphs` and no chart-only worksheets.
- [ ] Build bounded helper tables on `Calc`; inactive rows return `NA()` only in chart-helper ranges.
- [ ] Use XY scatter for plan/vertical-section geometry and line/scatter charts for depth strips. Disable smoothing and remove dummy scale series.
- [ ] Bind DLS-limit and target-limit series to formula cells rather than literal chart values.
- [ ] Set chart titles, legends, axis titles, readable positions, and full-depth source ranges. Invert TVD/depth axes where the chart API supports it; otherwise reverse the formula-helper order and label the convention.
- [ ] Export and render every visible sheet with `.work/verify_suite.mjs`; include the directional workbook in its list and exclude only `Calc`.
- [ ] Review rendered PNGs at readable scale for clipping, overlap, blank charts, truncated MD, incorrect target legend, and hidden units. Correct all material issues and rerender.
- [ ] Open/save the output with `soffice`, re-import it with artifact-tool, and repeat formula-error and drawing inspections to detect interoperability damage.

### Task 9: Integrate the fifth workbook into suite build, tests, documentation, and archive

**Files:**

- Modify: `src/build_suite.mjs`
- Modify: `tests/suite_acceptance.test.mjs`
- Modify: `tests/unit_display_contract.test.mjs`
- Modify: `tests/render_contract.test.mjs`
- Modify: `.work/verify_suite.mjs`
- Modify: `README.md`
- Rebuild: `outputs/Directional_Drilling_Wellplan_and_Survey_SI.xlsx`
- Rebuild: `package/`
- Rebuild: `WellForge_Analysis_Workbook_Suite_SI.zip`

- [ ] Update the output list to five exact filenames and add `buildDirectionalWorkbook` to `src/build_suite.mjs`.
- [ ] While revising the runner, replace `new URL(...).pathname` plus string concatenation with `fileURLToPath()` and `path.join()` so encoded and Windows paths remain valid.
- [ ] Update acceptance wording and assertions from four to five VBA-free workbooks.
- [ ] Add a result-display cell from the directional workbook to `unit_display_contract.test.mjs` and prove it references `Unit Map`.
- [ ] Expand acceptance inspection to reject `vbaProject.bin`, `externalLinks/`, `C:\\Users`, a personal author name, and `VBA_Macros.bas` in the directional OOXML payload.
- [ ] Update README use guidance to distinguish raw-input units from display units and document the Survey Contract plus explicit uncertainty exclusions.
- [ ] Run the complete build and test sequence:

  ```bash
  "$CODEX_PRIMARY_RUNTIME_NODE" src/build_suite.mjs
  "$CODEX_PRIMARY_RUNTIME_NODE" --test tests/*.test.mjs
  "$CODEX_PRIMARY_RUNTIME_NODE" .work/verify_suite.mjs
  ```

- [ ] Recreate `package/` from the verified current source, tests, docs, Office Script, README, and five workbooks. Do not package `node_modules`, temporary renders, the uploaded reference, or audit scratch files.
- [ ] Recreate `WellForge_Analysis_Workbook_Suite_SI.zip` from the clean package directory and list the archive to verify exact contents.

### Task 10: Final independent verification and delivery

**Files:**

- Verify: all five files in `outputs/`
- Verify: `WellForge_Analysis_Workbook_Suite_SI.zip`

- [ ] Invoke `superpowers:requesting-code-review` with the spec, plan, changed-file list, and test evidence. Resolve material findings using `superpowers:receiving-code-review`.
- [ ] Invoke `superpowers:verification-before-completion` and rerun the final commands rather than relying on earlier output.
- [ ] Inspect the directional workbook OOXML for VBA, external links, personal metadata, stale file paths, formulas outside intended ranges, and missing chart objects.
- [ ] Verify all output files are nontrivial in size, importable, and present in the archive.
- [ ] Save the verified directional workbook and updated suite archive as durable deliverables.
- [ ] Report the implemented model scope, test evidence, important exclusions, and workbook/archive links. Do not claim operational certification.

## Final acceptance matrix

| Requirement | Evidence |
|---|---|
| Verified minimum-curvature core retained | Independent oracle and exported-workbook station comparisons |
| Exact partial-MC interpolation | Midpoint, zero-dogleg, exact-station, and range-state tests |
| Governing crossline/3D error visible | Final sample numerical assertion plus Summary/Survey inspection |
| SI canonical with safe unit switching | Canonical-cell invariance and display-formula tests |
| Vector slide calibration | Build/turn/toolface numerical cases and QC exclusions |
| Real target envelopes | Point/circle/ellipse/box actual/projected hit/miss tests |
| Correct formation sign | Shallower-top positive-HIGH test |
| Decision-ready dashboard | Summary state/action formula tests and visual review |
| Strip/trajectory charts | Native drawing inspection and rendered-sheet review |
| Portable/no VBA | OOXML payload inspection and second-engine open/save pass |
| Suite integration | Five-file acceptance, README, package, and archive listing |

## Plan self-review

- Scope coverage: every approved enhancement maps to a task and acceptance row.
- Formula ownership: Excel owns all engineering results; the JavaScript oracle is test-only.
- Unit safety: raw-input conversion and display conversion are separate contracts.
- Interoperability: artifact-tool export/import, OOXML inspection, rendering, and `soffice` open/save are all required.
- Placeholder scan: the plan contains no deferred implementation markers.
