# Detailed Engineering Workbook Suite Design

## Goal

Refactor the WellForge Excel suite from compact screening calculators into detailed, auditable engineering workbooks with the structural depth of the TorqueDrag2013 reference while retaining SI-canonical calculations, formula-driven unit conversion, two-decimal display precision, shared mock data, and JSON exchange.

## Common workbook contract

Every workbook exposes a visible workflow of Summary, Inputs, Survey/Geometry, Results, Graphs, and discipline-specific detail sheets. Unit Map, Checks, Calc, and exchange sheets remain shared infrastructure. Calculations remain formula-driven and SI-canonical; display units are selected through SI, Imperial, Mixed, or Custom dropdowns.

## Discipline depth

- API 7G: tubular catalog, load cases, section stress/capacity results, governing utilisation, and operation-specific strength charts.
- Hydraulics: fluid/rheology inputs, complete flow path, nozzle cases, pressure/ECD profile, hydraulic power and optimisation plots.
- Torque/drag: retain the approved detailed clean-room direction based on TorqueDrag2013 operation blocks.
- BHA: assembly detail, vibration modes, bending response, toolface/WOB tendency matrix, and PolarPlotter-style polar chart construction.
- Directional: retain its existing detailed planning/survey/QC topology and add dedicated engineering plot and audit surfaces where needed.

## Visual contract

Charts answer one engineering question each. Depth profiles use consistent depth ordering and units. Summary sheets expose governing condition, evidence, status, and required action. Helper ranges are formula-backed and kept off the primary decision surface.

## Acceptance

All generated workbooks must export, contain expected native chart objects, expose custom unit controls, preserve shared mock identifiers, and have no visible formula errors. Every visible sheet is rendered for layout review before packaging.
