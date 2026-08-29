# WellForge Industry Visualization Architecture Design

## Status

Approved by the user on 2026-08-28 after reviewing the WELLPLAN, DecisionSpace, Innova Engineering, and supplied TorqueDrag2013 visual benchmarks.

## Goal

Replace the v4 collection of mostly independent charts with coordinated, engineering-review workspaces that expose modeled operations, observed data, limits, well geometry, hydraulic sensitivities, and the numerical state at a selected calculation depth.

## Clean-room basis

The implementation derives behavior from public product pages, public manuals/training material, published drilling-engineering papers, and the user's supplied workbook. It must not copy proprietary code, formulas, artwork, macros, or report templates.

Primary references:

- Halliburton WELLPLAN product interface and module description: https://www.halliburton.com/en/products/engineers-desktop-suite/wellplan-software
- Innova Engineering documentation and quick-start material: https://docs.innova-drilling.com/introduction/innova-engineering-manual/innova-engineering/1.0-software-overview
- AADE torque-and-drag graphical output discussion: https://www.aade.org/download_file/2710/491
- Supplied TorqueDrag2013 workbook and PolarPlotter2010 add-in, used only as observable visual/behavioral references.

## Invariants

1. Canonical calculations and stored engineering inputs remain SI.
2. Final `.xlsm` workbooks contain no worksheet formulas; VBA is the calculation authority.
3. Every displayed value, table header, chart title, and chart axis follows SI, Imperial, Mixed, or Custom unit selection.
4. Every MD/TVD chart uses response on X, depth on Y, zero at the top, and depth increasing downward.
5. Actual/observed data is never silently fabricated in production use. The packaged demonstration case labels its deterministic observed series as mock data.
6. Unlike dimensions do not share an axis. Axial load, torque, density, pressure, velocity, inclination, and positional error use separate synchronized roadmaps.
7. Limits are plotted on the same physical scale as the response they govern.
8. Chart configuration is stored in workbook cells and survives save/load.

## Workbook behavior

### Torque, drag, and buckling

Add an `Engineering Dashboard`, `Observed Data`, and `Chart Settings` surface.

The dashboard contains synchronized roadmaps for:

- axial operations: PUW, SOW, BKR, SLD, ROT, and DRLG;
- tension rating, sinusoidal buckling, and helical buckling limits;
- observed/mock hookload against the governing modeled operation;
- rotating, drilling, and backream torque plus torsional rating and observed/mock torque;
- inclination versus the same MD interval;
- well context and component/casing boundary annotations.

A selected-MD control drives a screen-reader table containing the nearest station, modeled operation values, observed values, limits, utilization, and governing state.

Friction sensitivity is represented as explicit low/base/high modeled families, with factors stored in `Chart Settings`.

### Hydraulics and hole cleaning

Add a `Hydraulics Dashboard`, `Flow Cases`, and `Chart Settings` surface.

The dashboard contains:

- pressure components and total dynamic pressure against MD;
- ECD families for low/base/high flow rates against static density and the configured ECD limit;
- annular-velocity families against the minimum transport screen;
- nozzle pressure envelope on conventional numeric axes;
- selected-depth screen-reader values for pressure, ECD, annular velocity, flow case, margin, and state;
- flow-path boundaries and casing/open-hole context.

Flow cases are editable multipliers of the base flow rate and are recalculated independently in VBA.

### Directional, BHA, and API 7G

Retain their discipline-specific plots while adding persisted `Chart Settings`, selected-depth/state tables where depth applies, and governing-condition callouts. Directional depth plots share the selected MD and well-context annotations. BHA retains the PolarPlotter-style radar/XY construction. API 7G uses section/component context rather than pretending its categorical results are continuous depth profiles.

## Presentation rules

- Dark charcoal section bands and neutral grey body surfaces remain the suite visual language.
- Operation colors are stable across every surface: PUW, SOW, BKR, SLD, ROT, and DRLG must not change meaning between charts.
- Limits use restrained dashed lines and risk colors; observed data uses markers plus a solid dark line.
- Legends contain operation codes and full descriptions in nearby tables.
- Dashboard charts align vertically and share the same depth range.
- Helper data remains auditable but is placed below or to the left of the chart surface; it is not mixed with controls.

## VBA architecture

`WellForgeCore.bas` owns shared chart configuration, series styling, selected-depth lookup, and validation. Discipline engines own their calculations and write value-only chart helper ranges. Workbook events recalculate on relevant input, unit, flow-case, observed-data, and chart-setting changes.

The Windows builder rejects a workbook if:

- formulas remain after VBA initialization;
- required dashboard/configuration sheets are missing;
- a depth chart is not XY response-X/depth-Y with reversed depth;
- unit switching leaves a displayed chart title or result label unchanged;
- required model, observed, limit, or hydraulic sensitivity series is missing.

## Acceptance criteria

- T&D dashboard contains axial, torque, and inclination synchronized roadmaps plus a selected-depth table.
- T&D axial composition contains all six operations, buckling/tension limits, and observed/mock data.
- Hydraulics dashboard contains at least three flow-rate ECD families and three velocity families.
- Observed data, flow cases, chart settings, context markers, and governing callouts are visible and editable where appropriate.
- All chart source ranges are formula-backed in `.xlsx` reference templates and value-only after `.xlsm` compilation.
- SI/Imperial/Mixed/Custom switching updates data, labels, and axes.
- Source OOXML integrity, formula-error scans, VBA structural lint, rendered visual inspection, ZIP checksums, and clean-extraction release tests pass.

