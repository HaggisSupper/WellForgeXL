# Drilling workbook calculation analysis

Analysis date: 2026-09-04

## Outcome

The 100 deduplicated research workbooks now have complete coverage of the
pipeline's declared static surfaces: sheets, formulas, defined names, unit
evidence, and VBA. The pipeline verified every source SHA-256, created one
private JSON capture per workbook, released that document before opening the
next workbook, and then streamed the captures into compact public inventories.

- 100 workbooks and 601 sheets were parsed with zero extraction errors and zero
  integrity failures.
- 486,312 formula cells were reduced to 5,117 structural formula families. This
  is a 98.95 percent reduction in inventory rows without losing occurrence
  counts or compressed cell ranges.
- 50,013 defined names, 2,352 grouped unit-evidence rows, 366 VBA modules, and
  988 VBA procedures were inventoried.
- 23 workbooks contain VBA. All 366 module sources were available for static
  review. No VBA, XLM macro, workbook event, link, or recalculation was executed.
- The sheet population is 580 worksheets, 14 chart sheets, and 7 embedded VBA
  module sheets. Visibility is 536 visible, 63 hidden, and 2 very hidden. No XLM
  macro sheets were found.

The private captures live under
`outputs/drilling-workbook-analysis/workbooks/<workbook-id>.json` and are ignored
by Git. They retain sheet names, representative formulas, compressed ranges,
defined-name expressions, and VBA source for local research. The tracked CSVs
use workbook IDs, sheet IDs, formula fingerprints, and aggregate metadata so raw
equations and source code are not published.

## Extraction coverage

| Static method | Workbooks | Notes |
| --- | ---: | --- |
| `openpyxl-read-only` | 58 | OOXML formulas, names, formats, and metadata |
| `calamine-static` | 40 | Rust parser for legacy `.xls`, `.xlsb`, and supported OOXML |
| `calamine-static-ooxml-fallback` | 1 | Recovered all 20 sheets and 26,922 formulas after malformed OOXML metadata stopped `openpyxl` |
| `calamine-static-after-standard-office-decryption` | 1 | Statically opened a compatibility-protected `.xls`; it contains 4 sheets, 4 names, and no formula records |

The Rust reader is pinned once at workspace level to Calamine `0.36.1`. That
version also recovered the previously malformed directional BIFF workbook: 10
sheets, 2,210 formulas, and 9,615 defined names.

## Domain inventory

| Catalog domain | Workbooks | Sheets | Formula cells | Families | VBA workbooks | Main calculation evidence |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Hydraulics | 41 | 185 | 384,239 | 1,059 | 4 | Rheology, Reynolds number, pipe/annulus loss, hydrostatics, ECD, nozzles, surge/swab, lag, hole cleaning, two-phase flow |
| BHA and tools | 24 | 193 | 56,344 | 1,047 | 4 | Geometry, motor performance, bending/loading, tool life, connection and stress capacity |
| Thermal | 3 | 47 | 29,371 | 1,981 | 1 | Geothermal gradient, material-property conversion, radial thermal resistance, elastomer expansion and life derating |
| Directional | 6 | 52 | 11,281 | 634 | 5 | Survey coordinates, dogleg severity, comparison and plotting tables |
| Torque and drag | 16 | 64 | 2,892 | 270 | 7 | Axial load, torque, contact indication, friction sensitivity, stress and connection capacity |
| Cementing and casing | 3 | 3 | 1,530 | 18 | 0 | Capacity, tally geometry, cement volume, limited strength checks |
| Uncategorized validation | 2 | 10 | 489 | 4 | 1 | Large validation tables and drilling-state/MSE VBA routines |
| General drilling | 2 | 34 | 147 | 85 | 1 | Nitrogen pressure/volume, casing/perforation validation, brine viscosity and fluid properties |
| Well control | 3 | 13 | 19 | 19 | 0 | Small worksheet surfaces; stronger kick-tolerance, MAASP, gas and kill-density evidence is embedded in workbook `2f31e7e9f41839d2` |

Calculation topics are multi-label evidence, so topic counts can overlap. The
exhaustive row-level answer is `CALCULATION_INVENTORY.csv`: one row per formula
family with workbook, sheet, occurrence count, layout, compressed cell ranges,
function set, external/volatile flags, topics, and detected units.

### Formula structure

| Layout | Families | Formula cells represented |
| --- | ---: | ---: |
| Rectangular table | 268 | 189,202 |
| Filled column | 1,159 | 174,214 |
| Multi-area family | 388 | 118,332 |
| Filled row | 474 | 1,736 |
| Single cell | 2,828 | 2,828 |

Three large hydraulics validation workbooks account for 330,096 formula cells,
or 67.9 percent of the complete collection, but only 14 formula families. They
are high-value parity datasets, not 330,096 independent algorithms:

| Workbook | Formula cells | Families | Disposition |
| --- | ---: | ---: | --- |
| `9d94e7c93c53521d` | 134,748 | 8 | Batch and power-law parity fixtures |
| `bcae2b0cfa613071` | 104,545 | 2 | Bingham large-sample parity fixtures |
| `30cdfd3f5cc38e9d` | 90,803 | 4 | Yield-power-law low-regime fixtures |
| `a20de360a6901fad` | 50,671 | 179 | Tool/connection data and lookup logic; review provenance before data migration |
| `b147eb91b2f2fd18` | 28,862 | 1,935 | Mixed motor/tool design system; separate physics from UI and databases |
| `2f31e7e9f41839d2` | 26,922 | 444 | Broad hydraulics, well-control, survey, and surge/swab comparison source |

The most common Excel functions by represented cell count are `ABS` (60,763),
`IF` (48,354), `PI` (16,329), `NA` (9,087), `COS` (8,120), `OR` (6,199),
`VLOOKUP` (3,974), `SIN` (3,551), `MIN` (3,318), `AND` (3,127), and `SQRT`
(3,064). This reinforces that lookup, presentation, and repeated table control
flow must not be mistaken for distinct engineering kernels.

### Calculation families by capability

| Capability | Represented formula cells | Interpretation |
| --- | ---: | --- |
| Reynolds number | 105,589 | Dominated by large hydraulics validation tables |
| Fluid properties | 64,798 | Rheology and property tables across hydraulics and tool workbooks |
| Connection capacity | 27,296 | Primarily BHA/tool specification tables, not only torque/drag workbooks |
| ECD | 12,962 | Broad workbook `2f31e7e9f41839d2` and hydraulics calculation surfaces |
| Pressure loss | 9,848 | Pipe, annulus, bit, and supporting pressure-budget logic |
| Directional coordinates | 8,525 | Survey and cross-domain TVD/geometry support |
| Dogleg severity | 8,226 | Directional sheets plus mixed tool-design calculations |
| Motor performance | 4,778 | Mostly mixed BHA/tool workbook evidence |
| Surge/swab | 4,121 | Strong candidate for a separate validated transient/quasi-steady lane |
| Hydrostatic pressure | 2,383 | Hydraulics and well-control support |
| Bit/nozzle | 1,811 | Nozzle geometry, drop, and motor/tool support |
| Axial load | 1,719 | Torque/drag and drilling-state routines |
| Bending/loading | 1,431 | BHA and mixed tool calculations |
| Lag | 1,384 | Circulation/transport timing and cuttings-lag logic |
| Rheology | 946 | Explicit models plus unit and input transformations |
| Contact force | 905 | Tool/BHA and torque/drag evidence |
| Two-phase flow | 737 | Limited, specialized workbook evidence |
| Hole cleaning | 597 | Transport and cuttings concentration indicators |
| Casing capacity | 456 | Geometry and volume emphasis; limited design-strength evidence |
| Friction factor | 420 | Explicitly labeled subset; many other loss families imply friction indirectly |
| Geothermal gradient | 358 | A simple linear profile and gradient table |
| Tool life | 324 | Temperature, pressure, flow, and wear derating logic |
| Cement volume | 172 | Small, table-shaped inventory |
| Thermal heat exchange | 10 | Radial resistance/overall-coefficient cells, not a full wellbore solver |

There are 219,092 hydraulics, 21,352 BHA, 10,303 directional, and 7,911
thermal formula occurrences whose local labels do not support a narrower topic.
They are retained under domain-specific `unresolved` labels rather than assigned
an unsupported interpretation.

## Unit inventory

The unit pass found 19,522 occurrences in 2,352 grouped evidence rows across 33
dimensions. Of those rows, 2,226 are high-confidence explicit cell labels and
126 are medium-confidence formula-context or number-format evidence.

| Dimension | Occurrences | Observed units | Canonical SI |
| --- | ---: | --- | --- |
| Length | 6,711 | `cm`, `ft`, `in`, `m`, `mm` | `m` |
| Pressure/stress | 2,235 | `bar`, `kPa`, `ksi`, `MPa`, `Pa`, `psi`, `psia`, `psig` | `Pa` |
| Fraction | 2,030 | `percent` | `1` |
| Time | 1,109 | `h`, `min`, `s` | `s` |
| Angle | 888 | `deg`, `rad` | `rad` |
| Volume | 874 | `bbl`, `ft^3`, `gal`, `in^3`, `m^3` | `m^3` |
| Area | 823 | `ft^2`, `in^2`, `m^2` | `m^2` |
| Volumetric flow | 637 | `bbl/min`, `gal/min`, `L/min`, `L/s`, `m^3/min`, `m^3/s` | `m^3/s` |
| Rotational speed | 605 | `rpm` | `rad/s` |
| Mass | 495 | `kg`, `lbm` | `kg` |
| Density | 474 | `g/cm^3`, `kg/m^3`, `lbm/ft^3`, `lbm/gal`, `ppg` | `kg/m^3` |
| Power | 432 | `hp`, `kW`, `W` | `W` |
| Absolute temperature | 370 | `degC`, `degF`, `K` | `K` |
| Velocity | 341 | `ft/min`, `ft/s`, `m/s` | `m/s` |
| Penetration rate | 321 | `ft/h`, `m/h` | `m/s` |
| Linear mass | 266 | `kg/m`, `lbm/ft` | `kg/m` |
| Force | 263 | `klbf`, `kN`, `lbf`, `N` | `N` |
| Energy | 131 | `Btu`, `J`, `kJ` | `J` |
| Dynamic viscosity | 131 | `cP` | `Pa*s` |
| Torque | 110 | `ft*lbf`, `kN*m`, `N*m` | `N*m` |
| Density ratio | 81 | `specific-gravity` | `1` |
| Pressure gradient | 36 | `kPa/m`, `MPa/m`, `psi/ft` | `Pa/m` |
| Yield stress | 30 | `lbf/(100*ft^2)` | `Pa` |
| Curvature | 30 | `deg/100ft`, `deg/30m` | `rad/m` |
| Pump rate | 22 | `strokes/min` | `1/s` |
| Frequency | 18 | `Hz` | `1/s` |
| Angular velocity | 17 | `rad/s` | `rad/s` |
| Thermal conductivity | 13 | `Btu/(h*ft*degF)` | `W/(m*K)` |
| Mass fraction | 11 | `ppm` | `1` |
| Heat-transfer coefficient | 7 | `Btu/(h*ft^2*degF)`, `W/(m^2*K)` | `W/(m^2*K)` |
| Temperature gradient | 7 | `degC/100m`, `degF/100ft` | `K/m` |
| Rheology consistency | 1 | `lbf*s^n/(100*ft^2)` | `Pa*s^n` |
| Specific heat capacity | 3 | `Btu/(lbm*degF)`, `J/(kg*K)`, `kJ/(kg*K)` | `J/(kg*K)` |

Required normalization controls are explicit in `UNIT_INVENTORY.csv`, including
the observed unit, SI multiplier/offset, pressure basis, temperature kind,
quantity kind, and reference-state requirement:

- Celsius and Fahrenheit absolute temperatures require affine conversion;
  temperature differences do not use the offset.
- Pressure requires gauge/absolute/reference metadata and must not be conflated
  with material stress merely because both normalize to pascals.
- `lbm/gal` may represent mass density, mud weight, or an equivalent pressure
  gradient depending on context.
- Fanning and Darcy friction factors differ by a factor of four and need an
  explicit convention in every contract and fixture.
- Power-law consistency conversion depends on the flow index exponent.
- `lbf/(100*ft^2)` requires a declared yield-point/stress convention.
- Volumes and flow rates need actual-versus-standard reference-state metadata
  when gas is involved.
- Heat-transfer coefficient requires an area basis; linear conductance and
  thermal conductivity are different dimensions.
- Curvature requires both an angle convention and interval basis.

The current `wellforge-units` wire registry covers only a subset of this matrix.
Thermal contracts must add temperature, temperature difference, conductivity,
specific heat, heat-transfer coefficient, energy, power, and mass-flow types
before accepting non-SI inputs. Solver internals remain metres, kilograms,
seconds, kelvin, pascals, watts, and joules.

## VBA analysis

The static VBA inventory contains 38,016 module source lines and 27,442
executable procedure lines:

- 366 modules: 238 class/document modules, 71 standard modules, 44 user forms,
  and 13 modules without a reliable suffix classification.
- 988 procedures: 899 subs, 77 functions, and 12 property getters.
- 924 procedures require an explicit call. Potential automatic entry points are
  49 worksheet events, 8 workbook events, 6 user-form events, and 1 auto-exec
  routine.
- The only auto-exec procedure is in workbook `65e07d7859867671`; the procedure itself
  has no static risk signal. It was not executed.

High-value VBA surfaces are:

| Workbook or family | Procedures | Static interpretation |
| --- | ---: | --- |
| `b147eb91b2f2fd18` | 502 | Mixed UI, charts, databases, motor/tool calculations, unit conversion, fitting, wear and life logic; 35 procedures carry review signals |
| `8c2e041f05eb97d0` | 92 | Nitrogen pressure/volume, tabular/casing validation, brine viscosity, volume properties, and extensive form control |
| `dbfddbefee5b196d` | 85 | Drilling-state detection, TVD, regression, MSE, rock strength, rig activity, hookload and bit-torque validation |
| `1d0b997e43f13c8f`, `659bcaadadb110f0`, `fd6bc4683cd208c1` | 38 each | Axial load/torque routines, unit conversion, chart/form control, and workbook events |
| `65e07d7859867671` | 38 | Lag/cuttings logic, rheology, regression, reporting and navigation |
| `2f31e7e9f41839d2` | 30 | Two large surge/swab routines, ECD, kick tolerance, surveys, charting, auto-depth, and 15 event callbacks |
| Five hash-distinct directional field workbooks | 15 each | Repeated survey calculation and workbook automation surface |

Static procedure classification found lookup logic in 83 procedures, motor
performance in 56, axial load in 53, dogleg severity in 39, unit conversion in
38, torque in 36, BHA geometry in 33, fluid properties in 24, and directional
coordinates in 23. These counts are semantic leads, not proof that each routine
is an independent or correct implementation.

Oletools reported 199 review indicators: 97 general review-required occurrences,
42 obfuscation-like string encodings, 32 auto-execution indicators, 15
external-process indicators, 10 IOC-formatted strings, 2 filesystem-write
indicators, and 1 environment-access indicator. Direct source scanning marked
procedure records containing filesystem writes (21), environment access (16),
COM automation (15), database access (5), and external-process calls (2), with
some procedures carrying multiple signals. The aggregation grain is one
procedure record per signal.

These are triage signals, not a malware verdict. The migration rule is still
strict: do not port workbook events, COM, shell/process launch, filesystem,
database, chart, form, or navigation behavior into calculation crates. Only
physics routines with independent tests and SI contracts are candidates.

## Thermal model conclusion

No complete counter-current wellbore temperature solver was identified in the
locally extracted formulas, defined names, VBA source, or p-code analysis.

| Workbook | Evidence | Reusable role |
| --- | --- | --- |
| `dc888cd53f297b84` | 152 formulas in 43 families; 10 cells build overall heat-transfer coefficients from serial cylindrical film/conduction resistances with logarithmic radius ratios. Property tables convert conductivity, heat capacity, density, and absolute temperature. | Independent property-conversion and radial-resistance fixtures after dimensional review |
| `348dd5eb51c7fac7` | 357 formulas in 3 filled-column families: a linear formation-temperature profile, first differences, and gradient over depth. | Formation-temperature and grid-gradient fixtures |
| `b147eb91b2f2fd18` | 28,862 formulas and 502 VBA procedures. Temperature logic concerns elastomer thermal expansion, stator/rotor fit, differential-pressure/flow sensitivity, wear, and life derating. | Temperature-dependent material/tool behavior only; not a wellbore heat exchanger |

The planned neutral Rust model remains `counter_current_wellbore_exchange`, not
a legacy workbook or vendor name. It should be a separate
`wellforge-wellbore-thermal` contract/core/fixtures/CLI lane with:

1. SI-only internal fields for mass flow (`kg/s`), temperature (`K`), heat
   capacity (`J/(kg*K)`), conductivity (`W/(m*K)`), linear conductance
   (`W/(m*K)`), heat rate (`W`), MD, TVD, pressure, and fluid state.
2. Explicit passage assignment and inlet boundaries for both conventional
   circulation (down string/up annulus) and physical reverse circulation (down
   annulus/up string), independent of numerical marching order.
3. Tubular-wall and annulus-to-formation resistance networks. The U-value
   workbook can seed fixtures, but it does not define the axial solver.
4. Formation temperature evaluated from TVD while transfer area is integrated
   over MD, including horizontal sections.
5. An implicit block-banded finite-volume reference solve with reported energy
   residual and grid-convergence evidence.
6. A bounded pressure-temperature outer iteration; neither hydraulics nor
   thermal code silently owns the other solver's convergence.
7. Parallel CPU execution first for independent scenarios and sensitivities.
   GPU support is enabled only above a benchmarked break-even size and must pass
   the same conservation and deterministic-tolerance fixtures.

## Migration disposition

| Disposition | Workbook evidence | Action |
| --- | --- | --- |
| Already represented in Rust | Steady pipe/annulus hydraulics, rheology families, nozzle loss, ECD, minimum-curvature trajectory, soft-string torque/drag, BHA static/modal foundation | Use workbook families only as adversarial parity fixtures; do not replace the current neutral APIs |
| Port as new physics kernels | Surge/swab, lag/transport, selected hole-cleaning and two-phase methods, radial thermal resistance, geothermal profile | Specify applicability and SI contracts first, then implement one bounded crate slice at a time |
| Import as data only | Reviewed material, motor, tool, connection, and thermal-property tables | Require provenance/license approval, schema validation, units, source hash, and versioning |
| Keep as test oracles | Large Bingham/power-law/yield-power-law tables and validation workbooks | Freeze representative cases, boundaries, and tolerances; avoid copying table control flow |
| Do not migrate | VBA events, UI/forms/charts, COM, filesystem, database, external-process code, workbook navigation, copied formatting formulas | Replace with native CLI contracts, application UI, and deterministic report generation |

Recommended bounded sequence:

1. Promote representative hydraulics families and boundary rows into SI parity
   fixtures, preserving workbook ID and source hash outside the public payload.
2. Close the current hydraulics pressure-budget and correlation parity matrix.
3. Add the missing thermal and flow dimensions to the shared wire-unit layer.
4. Implement and test material properties, geothermal profile, and radial
   resistance as independent thermal primitives.
5. Implement the steady counter-current finite-volume core and conservation
   suite before pressure/property coupling.
6. Add scenario-level multicore acceleration, benchmark it, and only then
   evaluate a GPU backend.
7. Treat surge/swab, lag, hole cleaning, two-phase flow, and well control as
   separate contracts with their own validity ranges and acceptance fixtures.

## Limits

- Static extraction inventories formulas and VBA source but does not calculate
  workbook outputs, resolve external links, or validate engineering correctness.
- Two workbooks contain 82 external-reference formula cells; external data must
  be replaced by versioned native inputs.
- Eleven workbooks contain 156 volatile-formula cells. Deterministic fixtures
  must replace workbook recalc-order behavior.
- Calamine does not reconstruct every shared or array-formula record in binary
  `.xls`/`.xlsb` files, so those formula totals are documented lower bounds.
- VBA source and p-code analysis are static only; no macro was executed.
- Formula-topic and unit inference is label-based. Ambiguous evidence remains
  medium confidence or `unresolved` rather than being silently guessed.
- The 50,013 defined names are not 50,013 algorithms: 49,815 are ranges, 155 are
  formula names, and 43 are external references. Five directional workbooks
  account for 48,075 range names, largely workbook/plot structure.
- Workbook equations and VBA remain untrusted research sources. Every migrated
  method needs independent standards review, dimensional checks, limiting cases,
  regression fixtures, and qualified engineering acceptance.

## Inventory map

| File | Grain and purpose |
| --- | --- |
| `ANALYSIS_SUMMARY.json` | Path-free merged counts by format, category, topic, and unit |
| `WORKBOOK_AUDIT.csv` | One row per workbook with coverage, totals, dependency flags, and macro totals |
| `SHEET_INVENTORY.csv` | One row per sheet with kind, visibility, used dimensions, formula counts, topics, and units |
| `CALCULATION_INVENTORY.csv` | One row per structural formula family with compressed ranges and semantic evidence |
| `UNIT_INVENTORY.csv` | One row per workbook/sheet/unit/evidence-source group with SI target and hazard |
| `DEFINED_NAME_INVENTORY.csv` | One row per name with hashed identity and reference classification |
| `VBA_INVENTORY.csv` | One row per module and procedure with hashed identity, trigger, topics, units, and risk signals |
| `MACRO_INDICATORS.csv` | Grouped static oletools indicator classes by workbook |
