# Directional Drilling Wellplan and Survey Workbook Design

## Purpose

Refactor `Directional_Drilling_Wellplan_Survey_Workbook.xlsx` into the fifth WellForge analysis workbook. The new workbook preserves the independently verified minimum-curvature geometry, replaces fixed-range and presentation shortcuts with traceable formulas, and makes positional error, survey quality, slide response, target status, and projection status immediately reviewable.

The deliverable is `outputs/Directional_Drilling_Wellplan_and_Survey_SI.xlsx`. It is an engineering planning and review workbook, not an operational authorization system and not an ISCWSA survey-uncertainty or anti-collision model.

## Reference audit incorporated into the design

The source workbook contains 13 sheets, 951 formulas, nine chart objects, no VBA payload, and no external workbook links. Its 60 planned and 60 actual survey stations were independently recalculated with minimum curvature. Positional discrepancies were below `2E-12 ft` and DLS discrepancies below `1E-14 deg/100 ft`, so that geometry is retained.

The refactor must correct these material weaknesses:

- The source exposes TVD and vertical-section differences but omits the governing final crossline miss. At the final sample station the source data produces approximately `1,408.48 ft` crossline error, `1,424.24 ft` horizontal error, and `1,436.38 ft` 3D error.
- Planned and actual ranges stop at 60 rows and are not governed by input-capacity checks.
- Target tolerance is not used in a consistent horizontal-plus-vertical envelope test.
- Slide calibration treats DLS as a scalar and does not use toolface as a vector direction.
- Formation-top sign semantics conflict with the displayed `High(+)/Low(-)` label.
- Plan-at-survey and formation-top positions use linear interpolation of coordinates instead of a partial minimum-curvature arc.
- A hard-coded `18 deg/100 ft` fatigue threshold is not supported by pipe geometry, axial load, RPM, corrosion, or an operator-controlled limit.
- Charts truncate measured depth, use smoothed curves that can overshoot, display dummy scale series, and rely on chart-only sheets that do not render consistently outside desktop Excel.
- Input validation, formula-integrity checks, range checks, and survey-sequence checks are too shallow.
- The workbook carries stale personal/file-path metadata and instructions for an unavailable VBA module.

## Non-negotiable design contracts

### Calculation and unit contract

- All engineering calculations use canonical SI values in `Calc`: metres, radians, and radians per metre.
- Raw plan and survey inputs may be pasted in metres or feet and degrees or radians. Explicit raw-input unit selectors convert the values to canonical SI before any geometry is calculated.
- `Unit Map!B5` changes display units only. Changing it must never reinterpret raw input values.
- Display modes are `SI`, `Imperial`, and `Mixed`; every displayed converted value and unit label references `Unit Map`.
- Add the `Angular gradient` unit domain with canonical `rad/m`, Imperial `deg/100 ft`, and Mixed `deg/30 m`.
- Correct the suite's existing Mixed stress conversion to `Pa -> MPa = 1E-6`, and make every Imperial/Mixed factor an explicit row property rather than a domain-name conditional.
- Invalid display-system values must report an invalid state; they must not silently fall through to Mixed.
- Formula constants are dimensionless mathematical constants or explicit conversion factors in `Unit Map`; engineering limits are blue user inputs with source notes.

### Formula and automation contract

- Engineering results are Excel formulas, not JavaScript, Office Script, VBA, or hidden constants.
- JavaScript builds and validates the workbook but does not replace workbook calculations.
- The workbook contains no VBA, external links, named formulas that hide calculations, or macros.
- The optional suite Office Script may recalculate, timestamp, and clear approved input blocks; it may not calculate trajectory or engineering results.
- Formulas use bounded ranges and explicit error guards. Whole-column rolling formulas from the source workbook are not retained.

### Decision contract

Within five seconds, `Summary` must answer:

1. Is the current survey/model state `READY`, `CAUTION`, or `STOP`?
2. What is the latest valid survey MD?
3. What are current crossline and 3D errors against plan?
4. Is current DLS within the user-controlled operating limit?
5. What is the next target status?
6. What action or data correction is required next?

## Workbook topology

The visible worksheet order is:

1. `Summary`
2. `Inputs`
3. `Plan`
4. `Survey`
5. `Targets`
6. `Slide Performance`
7. `Formation Tops`
8. `Results`
9. `Graphs`
10. `Unit Map`
11. `Checks`

`Calc` follows the visible sheets and contains traceable canonical-SI geometry, interpolation, projection, target, and chart-helper tables. It may be hidden in the delivered workbook only if the formulas remain inspectable by unhiding it.

Input blocks are native Excel Tables with a formula-supported capacity of 500 plan rows, 500 survey rows, 100 targets, 200 slide intervals, and 100 formation tops. Each block has a visible used-row count and a capacity check. Empty rows must not create chart points or false checks.

## Input model

### Inputs

`Inputs` contains:

- well name, well identifier, operator, field, datum, north-reference statement, and coordinate-reference statement;
- vertical-section azimuth;
- plan raw length unit (`m` or `ft`), plan raw angle unit (`deg` or `rad`);
- survey raw length unit (`m` or `ft`), survey raw angle unit (`deg` or `rad`);
- user DLS operating limit and the unit in which it is entered;
- projection-to-bit MD and project-ahead MD;
- projection build and effective-turn tendencies;
- target defaults and alert tolerances;
- low-inclination threshold, minimum slide length, and slide-yield outlier threshold;
- method and limitation notes.

Unit selectors are data-validated lists. Limits have positive/range validation and carry a source/reference field. The workbook explicitly distinguishes `input units` from `display units`.

### Plan and Survey raw input blocks

Both blocks accept station identifier, MD, inclination, azimuth, optional source/comment, and an active-row indicator derived from MD. The displayed calculated columns include TVD, North, East, vertical section, crossline, DLS, and row QC. `Survey` additionally displays plan-at-survey coordinates, delta North/East/TVD/VS, along-track error, crossline error, horizontal error, and 3D error.

The source workbook's nonblank plan and survey values seed the delivered workbook as a traceable example dataset. Blue cells remain editable.

## Trajectory mathematics

### Minimum curvature

For each active interval, with inclination `I`, azimuth `A`, and measured-depth increment `dMD`, the dogleg is clamped before inverse cosine:

```text
beta = ACOS(CLAMP(COS(I1)*COS(I2) + SIN(I1)*SIN(I2)*COS(A2-A1), -1, 1))
RF   = 1 + beta^2/12 + beta^4/120                         when |beta| < 1E-9
RF   = 2*TAN(beta/2)/beta                                 otherwise
dN   = dMD/2 * (SIN(I1)*COS(A1) + SIN(I2)*COS(A2)) * RF
dE   = dMD/2 * (SIN(I1)*SIN(A1) + SIN(I2)*SIN(A2)) * RF
dTVD = dMD/2 * (COS(I1) + COS(I2)) * RF
DLS  = beta/dMD
```

Azimuth differences used for diagnostics are wrapped to `[-pi, pi]`. First-station positions are zero unless the user provides a starting coordinate/datum offset.

Vertical section and crossline use the user vertical-section azimuth `VSA`:

```text
VS        = N*COS(VSA) + E*SIN(VSA)
Crossline = -N*SIN(VSA) + E*COS(VSA)
```

### Exact partial minimum-curvature interpolation

Plan-at-survey, actual-at-target, and actual-at-formation-top calculations use the enclosing survey interval and a partial minimum-curvature arc. They do not linearly interpolate calculated coordinates.

For interval fraction `f`, direction vectors are interpolated by spherical linear interpolation (SLERP). The partial dogleg is `f*beta`; the interpolated inclination/azimuth are reconstructed from the direction vector; and a minimum-curvature displacement is calculated from the lower station across `f*dMD`. Small-dogleg interpolation uses a normalized linear direction-vector limit. Inputs outside the available MD range return an explicit `BEFORE START` or `BEYOND TD` state rather than an extrapolated coordinate.

### Positional error

At each actual station covered by the plan:

```text
dN         = ActualN - PlanAtActualN
dE         = ActualE - PlanAtActualE
dTVD       = ActualTVD - PlanAtActualTVD
dVS        = ActualVS - PlanAtActualVS
Along      = dN*COS(VSA) + dE*SIN(VSA)
Crossline  = -dN*SIN(VSA) + dE*COS(VSA)
Horizontal = SQRT(dN^2 + dE^2)
Error3D    = SQRT(dN^2 + dE^2 + dTVD^2)
```

Signs and axis conventions are shown on `Inputs` and `Results`.

## Slide-performance vector model

Each active slide interval identifies start/end survey stations, slide length, commanded toolface, and optional rotary background build/effective-turn rates. The workbook calculates:

```text
Build component          = (I2-I1)/interval MD
Effective-turn component = WRAP(A2-A1)*SIN((I1+I2)/2)/interval MD
Residual build           = (Build - RotaryBuild)*interval MD/slide length
Residual effective turn  = (EffectiveTurn - RotaryTurn)*interval MD/slide length
Slide yield              = SQRT(ResidualBuild^2 + ResidualTurn^2)
Response toolface        = MOD(ATAN2(ResidualTurn, ResidualBuild), 2*pi)
Toolface error           = WRAP(ResponseToolface - CommandedToolface)
```

This follows the highside/effective-turn vector relationship documented by ISCWSA. Rows below the low-inclination threshold, below minimum slide length, outside survey coverage, or above the configured outlier limit are flagged and excluded from rolling calibration. Rolling build, turn, yield, and toolface statistics are weighted by slide length over a bounded trailing interval.

## Targets

Each target contains target ID, target MD, center North/East/TVD, envelope type, major/half-length, minor/half-width, rotation, and vertical tolerance.

Supported horizontal envelopes are:

- `Point`: radial tolerance equals the major value;
- `Circle`: radius equals the major value;
- `Ellipse`: rotated semi-major and semi-minor axes;
- `Box`: rotated half-length and half-width.

The current state at target MD is actual position when surveyed through that depth, otherwise the projection from the latest survey. Local rotated coordinates determine horizontal inclusion; vertical inclusion is `ABS(dTVD) <= vertical tolerance`. Overall `HIT` requires both. Status distinguishes `ACTUAL HIT/MISS`, `PROJECTED HIT/MISS`, `NOT REACHED`, and invalid geometry. Infeasible/zero envelopes are errors, not silently coerced shapes.

## Projection model

Projection begins at the latest valid actual survey and uses user-provided build and effective-turn tendencies. At low inclination, effective-turn conversion is guarded and reported as low-confidence. Final orientation and displacement use the same clamped minimum-curvature formulas as the trajectory tables. Projection results are explicitly labelled deterministic and exclude survey uncertainty.

## Formation tops

Each row contains formation name, prognosed MD/TVD, actual pick MD, and optional vertical tolerance. Actual TVD at pick MD uses exact partial minimum-curvature interpolation on the actual survey. Structural sense is:

```text
High(+)/Low(-) = Prognosed TVD - Actual TVD
```

A shallower actual top is therefore positive (`HIGH`). Coverage and tolerance states are explicit. Formation-top comparison is a vertical structural comparison and is not presented as a 3D target-envelope test.

## Results and canonical Survey Contract

`Results` presents current/latest state, maximum errors, plan coverage, target summary, slide calibration summary, formation summary, and projection outputs using formula-driven display values and labels.

It also publishes a bounded `Survey Contract` for suite interoperability with these canonical columns:

- `Station_ID`
- `MD_m`
- `Inc_rad`
- `Azi_rad`
- `TVD_m`
- `North_m`
- `East_m`
- `VS_m`
- `Crossline_m`
- `DLS_rad_per_m`
- `Source`
- `Row_Status`

These columns are the approved exchange surface for future Torque/Drag, Hydraulics, BHA, and MWD workbook import work. This refactor does not silently link the workbooks together.

## Checks and status logic

`Checks` includes at least:

- input-unit metadata valid;
- required reference metadata present;
- plan and survey station counts within capacity;
- MD numeric, nonnegative, and strictly increasing;
- duplicate MD and excessive gap screening;
- inclination in `[0, pi]` and azimuth canonicalized;
- plan coverage of actual survey, with beyond-TD status;
- DLS against the user-controlled limit;
- target geometry/tolerance validity;
- target actual/projected coverage state;
- slide station references, slide length, low-inclination, and outlier screening;
- formation-pick coverage;
- formula-error sentinel;
- no VBA or external links;
- method notice that no ISCWSA covariance/error model or anti-collision separation factor is included.

`STOP` is produced by missing/invalid unit metadata, invalid active trajectory rows, non-increasing MD, formula errors, or invalid target geometry used for a decision. `CAUTION` is produced by plan overrun, DLS-limit exceedance, projected target miss, low-confidence projection, or slide-quality warnings. Otherwise the summary state is `READY`.

## Charts

All charts are native Excel objects on `Graphs` and use bounded formula-backed helper ranges on `Calc`. Blank rows return `#N/A` to avoid false chart points. Curves are unsmoothed.

Required charts are:

- plan view: East versus North, planned and actual;
- vertical section: VS versus TVD, planned and actual, with depth increasing downward;
- survey strip chart: inclination and azimuth versus MD;
- DLS strip chart: plan DLS, actual DLS, and the user limit versus MD;
- positional-error strip chart: TVD, VS, along-track, and crossline error versus MD;
- horizontal/3D error strip chart versus MD;
- slide-yield strip chart with QC state;
- target-state comparison using actual/projected center offsets against envelope limits.

No dummy scale series, fixed depth truncation, smoothed curves, or chart-only worksheets are used.

## Presentation

The workbook follows the WellForge visual system: charcoal headers, teal ready/action state, amber caution, red stop/limit state, blue editable inputs, and light-grey calculated values. Gridlines are hidden. Freeze panes keep headers visible in long input blocks. Section widths, wrapping, and number formats are set explicitly. Helper calculations are absent from the decision-facing sheets.

## Verification and acceptance

The refactor is accepted only when all of the following pass:

1. Independent JavaScript test vectors reproduce the source workbook's 60-station minimum-curvature positions within `1E-8 m` and DLS within `1E-10 rad/m` after conversion.
2. The sample final-survey error exposes crossline, horizontal, and 3D magnitudes equivalent to the independently audited values within display-rounding tolerance.
3. Partial-MC interpolation tests cover exact-station, mid-station, near-zero-dogleg, before-start, and beyond-TD cases.
4. Formation structural-high sign and rotated circle/ellipse/box target cases pass explicit examples.
5. Slide-vector tests cover pure build, pure turn, mixed toolface, low inclination, and zero/short slide length.
6. SI/Imperial/Mixed display changes alter values and labels but do not alter canonical SI calculations or reinterpret raw inputs.
7. Workbook import finds no `#REF!`, `#DIV/0!`, `#VALUE!`, `#NAME?`, or unintended `#N/A` in decision ranges.
8. OOXML inspection finds no VBA project, external workbook link, stale source file path, or personal metadata.
9. Every visible sheet is rendered and visually reviewed for clipping, overlap, truncated charts, hidden depth ranges, and unreadable labels.
10. The suite acceptance test, package copy, README, verification script, and suite archive all include the fifth workbook.

## Explicit exclusions

- ISCWSA Rev5 covariance propagation, error ellipses, separation factor, and anti-collision scanning are not included. They require a separately validated extension with controlled tool-code and error-model data.
- The workbook does not claim API, ISCWSA, or operator certification.
- The workbook does not calculate pipe fatigue limits. A user-controlled DLS operating screen is not a fatigue analysis.
- Motor geometry is not inferred from disconnected OD/hole-size heuristics.
- Live MWD ingestion and cross-workbook links are not part of this refactor; the Survey Contract is the stable future interface.
