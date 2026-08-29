# WellForge Analysis Workbook Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver four SI-native, formula-driven drilling-analysis workbooks with coherent inputs, unit conversion, checks, results and native charts.

**Architecture:** A shared JavaScript builder library provides visual tokens, `Unit Map` rows, validation layout, result-summary construction and safe native-chart helpers. Each workbook owns its engineering formulas and test vectors. Workbook calculations are written as Excel formulas; scripts generate files and test formula/structure contracts only.

**Tech Stack:** Node.js, `@oai/artifact-tool`, Excel formulas, native Excel charts, optional Office Script documentation.

**Spec:** `docs/superpowers/specs/2026-08-27-wellforge-analysis-workbook-suite-design.md`

## Global Constraints

- SI calculation/source values only; display conversions and labels are formula-driven from `Unit Map`.
- No VBA, external links or script-only engineering calculations.
- Visible tabs: Summary, Inputs, Survey, Results, Graphs, Unit Map, Checks.
- Calculation formulas remain visible and traceable in calculation/helper sheets.
- Deliver formula, structural and visual checks with every workbook.

---

### Task 1: Establish test vectors and shared workbook contract

**Files:**
- Create: `tests/unit_contract.test.mjs`
- Create: `tests/test_vectors.json`
- Create: `src/common.mjs`

**Interfaces:**
- Produces: `addUnitMap(workbook)`, `addStatusBanner(sheet)`, `addChecksSheet(workbook, checks)`, `formatInput(range)`, `formatOutput(range)`.
- Consumes: `tests/test_vectors.json` with SI inputs and expected dimensions.

- [ ] Write failing tests asserting the required worksheet names, SI selector validation, and formula-linked unit labels.
- [ ] Run `node --test tests/unit_contract.test.mjs`; verify the test fails before the builder exists.
- [ ] Implement the minimum shared builder helpers and unit table.
- [ ] Re-run the test and confirm it passes.

### Task 2: Build API 7G strength and torque workbook

**Files:**
- Create: `tests/api7g.test.mjs`
- Create: `src/build_api7g.mjs`
- Create: `outputs/API_7G_Drill_String_Strength_and_Torque_SI.xlsx`

**Interfaces:**
- Consumes: `src/common.mjs` and API 7G test vectors.
- Produces: `buildApi7gWorkbook()` and the specified workbook.

- [ ] Write a failing test for tension, torque, utilisation and unit-display formulas.
- [ ] Run `node --test tests/api7g.test.mjs`; verify expected failure.
- [ ] Build the workbook using visible section calculations, governing-result formulas and two native charts.
- [ ] Re-run the test, export the workbook, render visible sheets, and verify no formula errors.

### Task 3: Build hydraulics and nozzle optimization workbook

**Files:**
- Create: `tests/hydraulics.test.mjs`
- Create: `src/build_hydraulics.mjs`
- Create: `outputs/Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx`

**Interfaces:**
- Consumes: common builder, flow-path and nozzle test vectors.
- Produces: `buildHydraulicsWorkbook()` with formula-based tube-section and nozzle outputs.

- [ ] Write a failing test for flow-path section coverage, surface-pressure status and nozzle ranking.
- [ ] Run the test and confirm it fails.
- [ ] Implement tube-section calculations, rig/surface-pressure controls, formula-backed candidate table and native charts.
- [ ] Re-run tests, export and visually verify all visible sheets.

### Task 4: Build torque, drag and buckling workbook

**Files:**
- Create: `tests/torque_drag.test.mjs`
- Create: `src/build_torque_drag.mjs`
- Create: `outputs/Torque_Drag_and_Buckling_SI.xlsx`

**Interfaces:**
- Consumes: common builder, survey/string test vectors.
- Produces: `buildTorqueDragWorkbook()` with depth-profile operation cases.

- [ ] Write a failing test for calculated depth-profile formulas, operating cases and buckling flags.
- [ ] Run the test and confirm it fails.
- [ ] Implement survey, string/hole sections, formula helpers, summary, strip charts and checks.
- [ ] Re-run tests, export and visually verify all visible sheets.

### Task 5: Build BHA vibration, bending and tendency workbook

**Files:**
- Create: `tests/bha.test.mjs`
- Create: `src/build_bha.mjs`
- Create: `outputs/BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx`

**Interfaces:**
- Consumes: common builder and BHA/WOB/toolface test vectors.
- Produces: `buildBhaWorkbook()` with transparent multi-WOB XY series.

- [ ] Write a failing test for BHA calculation rows, WOB-case output columns and polar-coordinate formulas.
- [ ] Run the test and confirm it fails.
- [ ] Implement formula-backed vibration/bending/tendency outputs, strip charts and scatter-based rose plot.
- [ ] Re-run tests, export and visually verify all visible sheets.

### Task 6: Package, inspect and document

**Files:**
- Create: `tests/suite_acceptance.test.mjs`
- Create: `OfficeScripts/WellForgeWorkbookRefresh.ts`
- Create: `README.md`
- Create: `WellForge_Analysis_Workbook_Suite_SI.zip`

**Interfaces:**
- Consumes: four completed workbooks and test outputs.
- Produces: archived suite, refresh script and operational guide.

- [ ] Write a failing acceptance test for output count, required visible sheets, charts, no formula errors and no VBA payload.
- [ ] Run the test and confirm it fails before all outputs exist.
- [ ] Add the non-calculating Office Script, user guide, archive and final workbook inspection.
- [ ] Re-run the full test suite and review rendered workbook surfaces before delivery.

## Self-review

- Coverage: Tasks 2–5 implement each requested workbook; Task 1 enforces common SI/unit/UI rules; Task 6 verifies and packages the suite.
- Placeholder scan: no deferred implementation markers are used.
- Interface consistency: each builder consumes the common helpers and returns a workbook builder named in its task.

