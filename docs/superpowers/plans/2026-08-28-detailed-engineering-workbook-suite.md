# Detailed Engineering Workbook Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver five detailed, formula-driven WellForge engineering workbooks with consistent units, mock data, charts, checks, and exchange contracts.

**Architecture:** Existing builders remain authoritative. Each discipline adds focused visible detail sheets and formula-backed result/chart helpers through the shared workbook factory. Existing JSON/VBA/Office Script exchange surfaces remain unchanged.

**Tech Stack:** JavaScript modules, `@oai/artifact-tool`, OOXML `.xlsx`, Excel formulas, VBA and Office Script exchange adapters.

**Spec:** `docs/superpowers/specs/2026-08-28-detailed-engineering-workbook-suite-design.md`

## Global Constraints

- SI is canonical for stored values and calculations.
- Display systems are SI, Imperial, Mixed, and Custom.
- Display precision is two decimals; conversion factors retain engineering precision.
- Shared mock records and exchange identifiers are reused across workbooks.
- No VBA is required for calculations.

### Task 1: Shared detailed topology

- [ ] Extend the workbook factory with discipline-specific visible sheets.
- [ ] Add contract tests for sheet topology and chart presence.

### Task 2: API 7G detailed application

- [ ] Add tubular catalog, load cases, section detail, and strength-profile charts.
- [ ] Validate governing utilisation and unit-linked outputs.

### Task 3: Hydraulics detailed application

- [ ] Add rheology, flow-path detail, nozzle cases, pressure/ECD profile, and optimisation charts.
- [ ] Validate section totals and surface-pressure limit screening.

### Task 4: BHA detailed application

- [ ] Add assembly, modes, bending, tendency matrix, and PolarPlotter-style helper layers.
- [ ] Validate formula-backed WOB overlays and severity outputs.

### Task 5: Directional detailed application

- [ ] Add dedicated engineering plot and audit surfaces without weakening the existing survey contract.
- [ ] Validate trajectory, QC, and unit mappings.

### Task 6: Suite acceptance

- [ ] Regenerate all workbooks.
- [ ] Run formula, structure, exchange, and render tests.
- [ ] Package the complete suite as a ZIP.
