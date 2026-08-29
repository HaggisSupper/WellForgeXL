# WellForge Analysis Workbook Suite Design

## Purpose

Rebuild four independently usable drilling-engineering Excel workbooks using the clean operational topology of `T&D 4.002b.xlsm` while replacing its protected VBA calculation surface with visible, formula-driven calculations.

## Non-negotiable requirements

- SI is the sole calculation and stored-input system: m, kg/m3, Pa, N, N-m, m3/s, rad, and seconds.
- `Unit Map` is the only location that defines display units and conversion factors. Every displayed non-SI value is a formula referencing it.
- Inputs are editable blue cells; formula cells are charcoal/white and must not contain magic engineering constants.
- User-facing tabs appear in this order: `Summary`, `Inputs`, `Survey`, `Results`, `Graphs`, `Unit Map`, `Checks`.
- Calculation and chart-helper sheets may be hidden, but no engineering calculation may be encoded only in script or VBA.
- Workbooks contain no VBA, external links, or opaque macros. An optional Office Script may reset editable rows and force the workbook calculation/validation refresh, but must not calculate engineering results.
- All formula workbooks carry a planning/review status banner and a visible validity state. They do not provide operational authorization.

## Common worksheet contract

### Unit Map

The unit selector is `B3` (`SI`, `Imperial`, or `Mixed`).  Each unit domain is represented by a table row containing SI unit, selected display unit, and the multiplier/offset used by result-display formulas.  Source calculations always use SI values.  A representative result conversion is:

```excel
=IF('Unit Map'!$B$3="SI",CalcSI,CalcSI*'Unit Map'!$E$8)
```

For affine conversions (temperature), the conversion row includes multiplier and offset.  Labels reference the selected display-unit cell, never typed unit text.

### Inputs and checks

- Inputs: one value per concept, source-unit label beside it, input status formula, and optional list/range validation.
- `Survey`: MD, inclination and azimuth with SI source columns and converted display columns.
- `Checks`: named validity checks, expected state, formula result, and remediation wording. A Summary sheet must elevate failing checks.
- Calculations: helper tables contain explicit intermediate quantities and formulas so a reviewer can trace each headline output to inputs.

### Presentation and charts

The visual language follows the reference’s functional, table-first discipline but modernizes it: charcoal section bars, teal normal/action, amber attention, red limit exceedance, light-grey calculation fields, and no gridlines. Charts are native Excel charts, sourced from formula-backed helper tables. Every workbook includes a decision summary and depth-profile strip charts where a profile is meaningful.

## Workbook-specific requirements

### API 7G Drill String Strength and Torque

- Inputs: drill-string section geometry/material properties, connection torque/tension ratings, fluid density, block load and operating torque.
- Calculations: tubular metal area, internal area, buoyancy factor, axial load, tensile utilisation, torsional shear screening and combined utilisation.
- Outputs: section utilisation table, governing section, tension/torque bar chart, operating envelope chart.

### Steady-State Hydraulics and Nozzle Optimization

- Inputs: rig preset or a user surface-pressure limit, flow rate, mud density/rheology, all flow-path tube sections, bit/nozzle parameters.
- Calculations: section velocity, Reynolds number, friction factor, pipe and annular pressure loss, bit nozzle area/velocity/drop, ECD screening, surface-pressure reconciliation.
- Outputs: pressure-loss waterfall/table, depth strip chart, nozzle candidate ranking, pressure-limit status.

### Torque, Drag and Buckling

- Inputs: survey stations, hole sections, string elements, friction factors, fluid density, WOB, surface torque and operating condition.
- Calculations: minimum-curvature geometry, buoyed string weight, normal-force approximation, axial load/drag path, torque, sinusoidal and helical buckling screens.
- Outputs: POOH, RIH, slide, rotate and backream profiles; hookload/torque/drag/buckling strip charts; governing-depth table.

### BHA Vibration, Bending and Drill-Ahead Tendency

- Inputs: BHA components, station/support definition, RPM, flow, WOB cases and selected toolfaces.
- Calculations: section mass/stiffness screening, first-mode frequency screen, bending moment/stress screen, simplified tendency response by toolface and WOB.
- Outputs: frequency/bending/tendency strip charts and an Excel polar rose construction using XY scatter coordinates. Multiple WOB cases are independent series with transparent fills/lines.

## Traceability and scope

The legacy `T&D 4.002b.xlsm` is visual/workflow reference only. Its protected VBA source is not used or reproduced. Calculation methods used in the new workbooks must be named on a `Method` block and every output must trace through formulas to SI inputs.  No claim of full API certification is made unless a controlled, licensed standard source and a documented conformance test suite are supplied.

## Acceptance criteria

1. Every workbook opens without VBA, external-link or formula errors.
2. SI inputs are authoritative; changing the unit selector changes only formula-driven display values and labels.
3. Each visible summary identifies its governing constraint within five seconds.
4. All charts are native objects and trace to formula-backed source ranges.
5. A baseline and at least one invalid/limit-exceeded test case are calculated and checked.
6. A visual render of every visible sheet is reviewed for clipping, overlap, unreadable labels and broken charts.

