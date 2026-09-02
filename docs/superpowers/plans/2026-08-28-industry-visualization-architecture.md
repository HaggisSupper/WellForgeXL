# WellForge Industry Visualization Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build coordinated industry-software-informed engineering dashboards and persistent chart controls across the five WellForge VBA workbooks.

**Architecture:** Artifact-tool builders create auditable formula reference templates and native chart objects. Discipline VBA engines recalculate canonical SI models, write value-only display/helper ranges, apply unit-aware chart titles and stable series styling, and populate selected-depth readers. Shared contracts validate dashboard topology, depth orientation, series composition, unit propagation, and Windows `.xlsm` compilation.

**Tech Stack:** JavaScript ESM, `@oai/artifact-tool`, Node test runner, OOXML inspection with JSZip, VBA 7.x, PowerShell Excel COM automation.

**Spec:** `docs/superpowers/specs/2026-08-28-industry-visualization-architecture-design.md`

## Global constraints

- Preserve all v4 calculation, exchange, unit, BHA polar, and packaging contracts.
- Do not copy proprietary code or assets.
- Do not introduce worksheet formulas into final `.xlsm` files.
- Keep all canonical calculations SI and all visible results unit-aware.
- Use no Docker and introduce no new dependencies.

## Task 1: Encode visualization contracts

- [ ] Add failing tests for required dashboard/configuration sheets.
- [ ] Add failing OOXML tests for T&D operation/limit/observed compositions.
- [ ] Add failing tests for hydraulics low/base/high flow families.
- [ ] Add failing source/VBA tests for selected-depth readers and chart-setting persistence.
- [ ] Run focused tests and confirm failure for the missing v5 behavior.

## Task 2: Add shared chart configuration primitives

- [ ] Add stable operation and semantic color constants.
- [ ] Add persisted chart-setting sheets and selected-depth controls.
- [ ] Extend the depth-chart builder to accept series styles without changing axis geometry.
- [ ] Add shared VBA chart styling, selected-depth lookup, and chart-title validation.
- [ ] Run shared and unit-display tests.

## Task 3: Build the torque-and-drag engineering dashboard

- [ ] Add observed/mock data inputs with explicit provenance.
- [ ] Add six-operation axial and three-operation torque helper blocks.
- [ ] Add tension, sinusoidal, helical, and torsional limit series.
- [ ] Add low/base/high friction sensitivity families.
- [ ] Add inclination and well-context helper blocks.
- [ ] Add selected-depth screen-reader table and governing callout.
- [ ] Update the VBA engine to populate all value-only blocks and style/refresh the dashboard.
- [ ] Run T&D workbook, OOXML, unit, and VBA tests.

## Task 4: Build the hydraulics engineering dashboard

- [ ] Add editable low/base/high flow cases.
- [ ] Calculate pressure, ECD, and velocity families independently.
- [ ] Add selected-depth screen-reader table and governing callout.
- [ ] Add flow-path/well-context annotations.
- [ ] Keep the nozzle envelope on conventional numeric axes.
- [ ] Update the VBA engine to populate all value-only blocks and style/refresh the dashboard.
- [ ] Run hydraulics workbook, OOXML, unit, and VBA tests.

## Task 5: Add suite-wide persisted settings and callouts

- [ ] Add `Chart Settings` to API 7G, BHA, and directional workbooks.
- [ ] Add selected-depth readers to directional and BHA depth-indexed surfaces.
- [ ] Add governing-state callouts to API 7G section results.
- [ ] Extend workbook-change handling and Windows self-tests.
- [ ] Run the complete deterministic test suite.

## Task 6: Visual QA and delivery

- [ ] Rebuild all five `.xlsx` reference templates.
- [ ] Inspect key ranges, formulas, chart series, and OOXML package integrity.
- [ ] Render every visible sheet and repair clipping, blank charts, inconsistent scales, and legend problems.
- [ ] Update documentation, manifest, release notes, and builder validation.
- [ ] Assemble a clean v5 ZIP, generate checksums, extract it fresh, and rerun release tests.
- [ ] Persist and present the verified v5 package.

