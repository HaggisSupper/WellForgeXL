# Rust engine roadmap

## Architecture contract

Rust is the calculation, JSON parsing, validation and evidence authority for released Rust lanes. Excel/VBA is their input, value-only client and visualization plane; the three lanes not yet ported remain explicit prototypes. Each Rust engine emits value-only result arrays, immutable source identities, normalized request/result hashes, dependency-lock identity, calculation evidence and an explicit applicability statement. There is no silent fallback from Rust to workbook formulas or VBA screening calculations.

All engines reuse the locked numerical stack where applicable: `nalgebra` for spatial algebra and eigensystems, `faer` for matrix assembly/factorization, `parry3d-f64` for geometric distance/intersection queries, `levenberg-marquardt` for bounded nonlinear fitting/solution, `uom` for dimensional quantities, `serde_json` for strict JSON and `quick-xml` for the supported offline WITSML projection. WellForge supplies drilling-specific residuals, loads, constitutive choices and acceptance tests; it does not reproduce general numerical algorithms.

## Release order

### 1. BHA static and vibration — released foundation

- Linear small-deflection lateral beam/static preload.
- Modal frequencies and shapes, unit-force FRF, and 1x/2x/3x Campbell margins.
- Projected centerline, OD, ID and hole envelope for interference indication.
- No claimed contact reaction, frictional impact, whirl, six-degree rigid-body dynamics or DAT calibration.
- Rigid and modal-flexible component representations are rejected until their dedicated solvers exist.

### 2. Well-planning trajectory — released Rust lane

- WITSML-aligned Well, Wellbore, Trajectory and target identities with UUID/URI provenance.
- Minimum-curvature trajectory construction and exact partial-course interpolation.
- Build/turn/toolface control, target-envelope evaluation, formation intersections and plan-versus-survey residuals.
- Canonical calculation and result fields are SI. The unit-preserving trajectory wire adapter is not yet implemented or claimed.
- Deterministic station, target and projection fixtures cross-checked against an independent oracle.

### 3. Hydraulics

- The request names the governing standard profile and edition. The initial public baseline is API RP 13D, 7th Edition (2017, reaffirmed 2023); a later published edition is a new validated profile, not an automatic label change.
- Contract `0.2.0` selects neutral, physics-named pressure correlations and deterministic serial or multicore CPU execution while retaining byte-stable `0.1.0` compatibility behavior. The workbook defaults to serial screening until external generalized-correlation parity and workload benchmarks pass.
- Nozzle sweeps use a verified batch CLI envelope; homogeneous candidates reuse one prepared section-flow state while retaining per-candidate request and result hashes.
- Rheology fitting and evaluation for the models allowed by the selected profile, with temperature/pressure applicability metadata.
- Pipe and annulus regimes, effective hydraulic geometry, friction loss, hydrostatic pressure, ECD, velocity/transport indicators, bit/nozzle losses, TFA and pump operating envelopes.
- Results identify the standard/profile clauses implemented, excluded or requiring licensed validation data. “API compliant” is not reported until the selected edition’s acceptance matrix is complete and approved.
- Counter-current wellbore temperature exchange is a separate coupled thermal lane; its staging, conservation gates and acceleration policy are defined in `docs/HYDRAULICS_RUST_MIGRATION.md`.

### 4. Torque and drag — soft string

- Whole-well soft-string pass for pickup, slack-off, rotating, drilling, sliding and backreaming states.
- Axial load, torque, normal/contact-force indication, friction sensitivity and sinusoidal/helical buckling screens along measured depth.
- The soft pass owns broad interval discovery and always covers the complete modeled string.
- A deterministic severity classifier emits stable interval IDs, reasons, thresholds, peak locations and confidence. Candidate triggers include concentrated normal load, clearance/interference indication, curvature or geometry discontinuity, buckling proximity and high friction sensitivity.

### 5. Torque and drag — stiff string refinement

- The stiff-string solver runs only on severe-contact intervals selected by the soft-string classifier.
- Each refined interval includes an explicit transition buffer; boundary position, orientation, force and moment state are inherited from the accepted soft-string solution.
- Flexible beam/rod assembly uses the approved library matrix stack. Geometry queries use `parry3d-f64`; nonlinear equilibrium uses a library solver with reported residual and iteration evidence.
- Results retain the parent soft interval ID and report whether the refined interval converged, expanded, merged with a neighbor or requires engineering review.
- The engine never extrapolates stiff-string contact loads into unrefined intervals and never hides unclassified gaps.

## Validation gates for every engine

1. Strict versioned request/result schemas reject unknown fields and dimension errors.
2. WITSML source identity is preserved, but full WITSML 2.0/ETP conformance is not claimed without a dedicated conformance suite.
3. Closed-form, independent-oracle, limiting-case and deterministic regression fixtures pass.
4. `cargo fmt`, Clippy with warnings denied, all Rust tests and `cargo-deny` pass.
5. VBA performs only bounded process control and value writes for the Rust lane.
6. Workbook chart surfaces are rendered, inspected and scanned for visible formula errors.
7. Cargo/PowerShell builds and hashes the Windows executable; desktop Excel builds and hosts the macro-enabled workbook client. Native Windows executable, VBA/COM and rendered-workbook acceptance remain the final platform gates.
