# WITSML-Aligned Hybrid Multibody BHA Engine Design

**Status:** Approved architecture, implementation specification pending review  
**Date:** 2026-08-28  
**Product:** WellForge BHA Static Bending, Frequency/Vibration and Drill-Ahead Tendency

## Purpose

Replace the current workbook screening calculations with a deterministic Rust engineering engine that combines WITSML-aligned drilling data with hybrid rigid-flexible multibody mechanics. Excel remains an engineering input, review and reporting surface; it does not own the production physics.

The system must support progressively more expensive analyses without changing the source-data identity or result contract:

1. static BHA equilibrium and bending;
2. linearized modal and forced-frequency response;
3. nonlinear time-domain contact dynamics;
4. calibrated drill-ahead tendency and iterative trajectory prediction.

## Scope decomposition

The program is divided into independently releasable products because the numerical and validation requirements are materially different.

### Release 1: Contract, static equilibrium and linear frequency analysis

Release 1 delivers:

- a WITSML 2.0-aligned canonical BHA analysis contract;
- strict mapping for Well, Wellbore, Trajectory, WellboreGeometry, Tubular, BhaRun and drilling channels;
- hybrid rigid/flexible component definitions;
- deterministic three-dimensional static equilibrium;
- active wellbore-contact detection for static cases;
- linearization about the loaded equilibrium state;
- natural frequencies, mode shapes and forced-frequency response;
- critical-speed and WOB/RPM scenario maps;
- a native Windows Rust CLI;
- a VBA orchestration adapter that exchanges value-only JSON with the CLI;
- analytical, numerical and artifact-level regression evidence.

### Release 2: Nonlinear time-domain dynamics

Release 2 adds:

- changing unilateral contact and impact;
- axial, torsional and lateral coupling;
- forward and backward whirl classification;
- bit bounce, stick-slip and high-frequency torsional response;
- PDC and roller-cone bit-rock force interfaces;
- implicit time integration, checkpointing and deterministic replay;
- sensor-location acceleration, velocity, displacement, force and stress channels.

### Release 3: Calibrated drill-ahead tendency

Release 3 adds:

- bit side-force and tilt response laws;
- formation anisotropy and bedding-dip parameters;
- point-the-bit and push-the-bit actuator models;
- incremental borehole propagation;
- build, turn and walk predictions;
- calibration against actual survey and drilling-channel observations;
- parameter uncertainty and prediction envelopes.

### Release 4: Operational composition

Release 4 adds:

- WITSML/ETP ingestion and publication adapters;
- run-to-run calibration governance;
- Tauri decision surfaces and 3D visualization;
- fleet/scenario execution with optional CUDA acceleration;
- signed model packages and immutable calculation evidence.

No later-release capability may be represented as operational in an earlier release.

## Architectural principles

### Numerical infrastructure is library-owned

WellForge implements drilling-domain formulations, not general-purpose numerical algorithms. Production code must use reviewed numerical libraries for matrix storage and algebra, factorization, eigenproblems, nonlinear optimization, time integration, collision queries, transforms, FFTs, units and schema processing.

The planned library boundary is:

| Capability | Library | WellForge responsibility |
| --- | --- | --- |
| Spatial vectors, transforms and quaternions | `nalgebra` | Coordinate conventions, BHA frames and domain assembly |
| Medium/large dense and sparse algebra, decompositions and linear solves | `faer` | Residual, mass and tangent-matrix construction |
| Geometry and collision/contact queries | `parry3d-f64` | Wellbore/tool contact primitives and drilling contact laws |
| Stiff ODE and semi-explicit DAE integration for Release 2 | `diffsol` | Multibody residual, events, contact state and physical tolerances |
| Spectral transforms | `rustfft` | Windowing policy, channel selection and drilling interpretation |
| Nonlinear parameter optimization | a maintained Rust optimization crate selected by an acceptance spike | Objective functions, bounds and calibration evidence |
| Units and dimensional types | `uom` plus the Energistics mapping layer | Energistics symbols, quantity classes and wire conversion |
| XML/JSON serialization and schemas | `quick-xml`, `serde`, `serde_json`, `schemars`, `jsonschema` | WITSML projection rules and WellForge contract semantics |
| Component/reference graph | `petgraph` | BHA connectivity and WITSML reference invariants |
| Scenario parallelism | `rayon` | Deterministic scenario partitioning and result ordering |

Library versions are pinned in `Cargo.lock` after the Rust-capable build environment verifies their APIs and licenses. A library must pass a focused acceptance spike before it becomes calculation authority.

WellForge may implement the geometrically exact beam residual, BHA-specific joints, drilling contact constitutive laws, bit-rock interaction and DAT propagation because these are domain physics. It must not implement a replacement sparse factorization, eigensolver, generic optimizer, ODE/DAE integrator, collision engine, quaternion package or FFT.

If a selected library cannot meet a required numerical contract, the implementation plan records a library replacement decision. It must not silently introduce an unreviewed home-grown numerical routine.

### WITSML is the source-data substrate, not the solver result model

WITSML objects provide well identity, geometry, string configuration, run context and observed channels. WellForge analysis results live in a separately versioned extension object and reference the precise WITSML source-object UUIDs used by the calculation.

The engine must not distort WITSML by placing proprietary simulation structures into unrelated WITSML elements.

### Hybrid rigid-flexible multibody representation

The engine must not model the entire BHA as rigid bodies. Each component selects the least expensive representation that preserves its relevant mechanics:

| Representation | Intended use |
| --- | --- |
| Geometrically exact flexible beam | Drillpipe, HWDP, drill collars, shafts |
| Modal flexible body | Long complex tools with supplied or generated reduced modes |
| Rigid body | Bit bodies, short massive housings, sleeves |
| Reduced finite-element body | RSS, reamers and non-axisymmetric tools where contact geometry matters |
| Joint | Connections, bearings, flex joints, motor bends and steering actuators |
| Force element | Springs, damping, hydraulic forces and control forces |
| Contact primitive | Circle-cylinder, point-cylinder, blade-cylinder and bit-formation interaction |

Pure rigid-body analysis is prohibited for calculations that report bending stress, shaft flexibility, buckling or flexible natural modes.

### One model, progressive solvers

Static, modal, frequency and time-domain solvers consume the same assembled component graph. Modal analysis is a linearization of the converged static state, not a separate unloaded beam approximation.

### SI-canonical computation

All solver inputs and outputs use canonical SI values internally. Source WITSML units and workbook display units remain provenance and presentation concerns. Unit conversions use a versioned Energistics-aligned registry and must reject dimensionally incompatible quantities.

### Determinism and evidence

Every run records input hashes, source-object references, engine version, solver configuration, convergence history, warnings, result hash and execution environment. Repeated runs with the same inputs and deterministic settings must produce byte-stable normalized JSON results within the documented floating-point policy.

## Rust workspace

The implementation lives under `engine/` as a Rust 2024 workspace.

| Crate | Responsibility |
| --- | --- |
| `wellforge-units` | Canonical quantities, Energistics symbol mapping and dimension checks |
| `wellforge-witsml` | WITSML 2.0 object references, offline XML import/export and canonical projections |
| `wellforge-bha-contract` | Versioned request/result types and JSON Schema generation |
| `wellforge-bha-model` | Components, frames, joints, flexible bodies, loads and contact primitives |
| `wellforge-bha-static` | Assembly, active-set contact and nonlinear equilibrium |
| `wellforge-bha-modal` | Tangent matrices, eigenpairs, modal participation and harmonic response |
| `wellforge-bha-cli` | Headless validation and execution commands |
| `wellforge-bha-fixtures` | Analytical, published and sanitized engineering fixtures |

Release 2 adds `wellforge-bha-dynamics`; Release 3 adds `wellforge-bha-dat`. These crates must not exist as empty placeholders in Release 1.

The core solver must not depend on Excel, COM, Tauri, a database, a network service or an LLM.

## Canonical data model

### WITSML source references

`SourceObjectRef` contains:

- Energistics UUID;
- Energistics URI when available;
- object type;
- object version or immutable content hash;
- citation name;
- acquisition or creation timestamp;
- source system.

An analysis request requires references for:

- Well;
- Wellbore;
- Trajectory;
- WellboreGeometry;
- Tubular;
- BhaRun.

Observed data references identify Log, ChannelSet and Channel objects independently.

### Analysis request

`BhaAnalysisRequest` contains:

- contract version;
- case UUID and analysis UUID;
- WITSML source references;
- normalized trajectory stations;
- normalized wellbore-geometry sections;
- ordered tubular/BHA components;
- component mechanical representations;
- material and section properties;
- joints and steering actuators;
- static and dynamic contact primitives;
- drilling operating cases;
- fluid properties needed by the selected solver;
- sensor locations;
- solver configuration;
- extension metadata;
- provenance.

All records use UUID identity. Human-readable names are labels and never relationship keys.

### Analysis result

`BhaAnalysisResult` contains:

- request hash and result hash;
- engine and contract versions;
- calculation state: `complete`, `partial`, `failed` or `not_applicable`;
- convergence and quality evidence;
- static cases;
- modal cases;
- forced-response cases;
- warnings and applicability limits;
- source-object references;
- execution provenance.

Failed or non-converged analyses must not publish ordinary numeric results without an explicit partial-result state and diagnostic.

## WITSML mapping

| WellForge requirement | WITSML 2.0 source |
| --- | --- |
| Well identity and shared header | Well |
| Sidetrack/path identity | Wellbore |
| Actual directional survey | Trajectory and TrajectoryStation parts |
| Hole/casing geometry | WellboreGeometry and sections |
| Drillstring/BHA configuration | Tubular and tubular components/tools/bit record |
| Particular planned or executed run | BhaRun |
| WOB, torque, flow and run statistics | BhaRun DrillingParams |
| Time/depth observations | Log, ChannelSet and Channel |
| UUID/URI identity | Energistics Identifier Specification |
| Units | Energistics Unit of Measure Standard |

Plans, targets, proprietary solver settings and WellForge results remain explicit WellForge objects or extensions. They are not silently encoded as WITSML Trajectory measurements.

ETP transport is outside Release 1. Release 1 validates and transforms offline WITSML 2.0 object documents and uses the same UUID/DOR semantics required by a later ETP adapter.

## Mechanical formulation

### Coordinates and bodies

Rigid bodies use six spatial degrees of freedom with quaternion orientation and normalized-state enforcement. Flexible beam nodes use position and rotation variables suitable for arbitrary rigid motion and geometrically nonlinear bending. Modal bodies use a floating frame of reference plus bounded elastic modal coordinates.

Quaternion sign is canonicalized at serialization boundaries. Solver state never uses Euler angles as its authoritative orientation representation.

### Equations of motion

The assembled system is represented as:

\[
M(q)\ddot q + h(q,\dot q) + f_{int}(q,\dot q)
= f_g + f_f + f_b + f_c + f_a
\]

subject to joint constraints and unilateral contact conditions. Terms represent inertial/gyroscopic effects, internal elastic and damping forces, gravity/buoyancy, fluid loads, bit loads, contact and actuators.

### Static equilibrium

Static analysis sets velocity and acceleration to zero and solves the constrained nonlinear residual. The solver must:

- increment load from a stable initial state;
- update the active contact set;
- assemble material and geometric tangent stiffness;
- enforce bit and top-boundary conditions;
- stop on residual, displacement and contact-complementarity tolerances;
- report non-convergence without manufacturing values.

Release 1 supports frictionless normal static contact and explicitly bounded axial/torsional loads. Frictional migration and impacts belong to Release 2.

### Linearized modal analysis

Modal analysis uses the converged, preloaded static state:

\[
(K_t - \omega^2 M)\phi = 0
\]

where `K_t` includes material stiffness, geometric stiffness and the linearized active contact set. Constrained, rigid-body and numerically spurious modes are identified and excluded using declared tolerances.

### Harmonic response

Frequency response solves:

\[
(-\omega^2 M + i\omega C + K_t)x(\omega)=F(\omega)
\]

Excitations are explicit typed records: imbalance, bit blade pass, motor lobe pass, steering actuation or user-supplied harmonic load. Unknown excitation amplitudes must remain parameterized rather than silently assumed.

## Release 1 outputs

### Static

- deformed three-dimensional centreline;
- active contact locations and normal forces;
- axial force, shear, torque and bending moment;
- local curvature and neutral-axis orientation;
- component stress and utilization;
- bit side-force vector and bit tilt;
- steering reactions;
- convergence history and residual norms.

### Modal and frequency

- natural frequencies and normalized mode shapes;
- modal classification and participation;
- Campbell-diagram data;
- critical speed intersections and separation margins;
- component/sensor frequency-response functions;
- WOB/RPM scenario maps;
- damping and excitation assumptions.

## CLI contract

The Windows executable is `wellforge-bha.exe` and supports:

- `wellforge-bha validate --input <request.json>`
- `wellforge-bha solve-static --input <request.json> --output <result.json>`
- `wellforge-bha solve-modal --input <request.json> --output <result.json>`
- `wellforge-bha run --input <request.json> --output <result.json>`
- `wellforge-bha schema --output <directory>`
- `wellforge-bha version --json`

Commands return documented process exit codes and write structured JSONL diagnostics to stderr or a requested log file. Output replacement is atomic. Existing results are preserved as timestamped backups unless `--no-backup` is explicitly supplied.

No command invokes a network service. No Docker packaging is permitted.

## Excel/VBA boundary

The BHA workbook remains a value-only client after `.xlsm` compilation.

VBA must:

1. validate workbook-owned inputs;
2. construct a bounded request from declared ranges;
3. invoke the locally bundled Rust executable without shell interpolation;
4. enforce a configurable timeout;
5. verify process status, result schema, request hash and engine version;
6. write values only to result/helper ranges;
7. refresh charts and status surfaces;
8. preserve the complete diagnostic log path on failure.

The workbook must never silently fall back to its screening formulas when the production solver is unavailable. It reports `ENGINE UNAVAILABLE` or `ANALYSIS FAILED` and preserves the last accepted result as stale evidence.

## Visualization contract

The BHA workbook and later Tauri UI expose:

- 3D deformed BHA within the wellbore;
- contact force and clearance map;
- bending moment, stress and utilization versus distance from bit;
- bit side-force/tilt polar response;
- natural-frequency and mode-shape table;
- Campbell diagram with excitation-order overlays;
- forced-response heatmap by RPM and frequency;
- critical-speed map by WOB and RPM;
- scenario comparison and governing-condition summary.

Observed channels and predictions use different visual semantics. A predicted series must never be labelled as measured.

## Error handling

Blocking errors include:

- invalid or missing UUID relationships;
- unsupported WITSML object version;
- dimensionally incompatible units;
- non-increasing or geometrically invalid trajectory stations;
- impossible component geometry;
- disconnected component graph;
- initial penetration beyond configured recovery tolerance;
- singular unconstrained model;
- equilibrium non-convergence;
- invalid modal mass or non-finite eigenpairs;
- request/result hash mismatch.

Warnings include omitted optional fluid effects, estimated damping, incomplete excitation amplitudes and results outside calibrated applicability.

## Verification and validation

### Contract verification

- Validate official or license-permitted WITSML 2.0 fixtures against their XSDs.
- Verify UUID and Data Object Reference integrity.
- Verify every physical value against its quantity class.
- Round-trip canonical projections without losing source identity or units.

### Static solver verification

- axial bar extension;
- cantilever tip displacement and rotation;
- simply supported beam response;
- pure torsion;
- Euler buckling load;
- gravity sag with buoyancy;
- coordinate-frame invariance;
- contact activation/deactivation benchmark;
- mesh convergence;
- independent comparison with a trusted finite-element result.

### Modal verification

- free-free and constrained uniform-beam frequencies;
- modal mass normalization;
- orthogonality;
- pre-stress frequency shift;
- contact-stiffness frequency shift;
- harmonic single-degree-of-freedom response;
- mesh and retained-mode convergence.

### Acceptance tolerances

Each fixture declares its own physically justified tolerance. Default analytical relative tolerances are:

- static displacement and reaction: `1e-6`;
- first six natural frequencies: `1e-5`;
- normalized modal orthogonality residual: `1e-8`;
- force equilibrium residual divided by reference load: `1e-8`;
- deterministic normalized result comparison: `1e-12` where operation ordering is fixed.

Any relaxed tolerance requires a fixture-specific engineering justification in the test source.

## Performance policy

Release 1 prioritizes correctness and sparse CPU convergence. Parallel scenario execution is allowed only after single-case determinism is proven. CUDA may later accelerate contact batches, scenario ensembles and signal processing. GPU use is never accepted as evidence of correctness.

Vulkan/WebGPU is the visualization and portable-compute fallback where an algorithm has a validated implementation. It is not required for the first solver release.

## Security and deployment

- Windows-first native executable.
- Rust 2024 edition with locked dependencies and reproducible release build.
- No runtime network dependency.
- No Docker.
- Signed release artifacts when a signing identity is available.
- VBA launches only the executable colocated with the workbook suite and verifies its recorded hash.
- Input and output paths are explicit; command strings never contain untrusted shell fragments.

## Completion gates for Release 1

Release 1 is complete only when:

1. all specified crates contain production implementations with no placeholder paths;
2. WITSML fixtures and canonical projections validate;
3. every static and modal analytical fixture passes declared tolerances;
4. non-convergence and invalid-input paths are tested;
5. the CLI produces deterministic, schema-valid results and diagnostics;
6. the Windows VBA workflow invokes the compiled CLI and handles failure safely;
7. workbook results and charts are value-only and traceable to the request hash;
8. the existing workbook-suite regression tests remain green;
9. source, dependency, unit, contract and calculation evidence are packaged together;
10. no claim is made for nonlinear time-domain vibration or calibrated DAT.

## Explicit non-goals for Release 1

- ETP server or subscription implementation;
- nonlinear impact and frictional contact dynamics;
- production bit-rock cutting calibration;
- autonomous steering control;
- fleet or cloud services;
- ML-generated physics;
- real-time guarantees;
- certification as a WITSML product.

## Design confidence

- WITSML separation and object mapping: `0.97`
- hybrid rigid-flexible architecture: `0.97`
- Release 1 numerical scope: `0.91`
- workbook/CLI boundary: `0.95`
- later nonlinear and DAT scope: `0.84`, constrained by calibration and field validation data

Weighted design confidence: `0.93`.
