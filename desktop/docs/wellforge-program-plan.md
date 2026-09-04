# WellForge Master Program Plan

## Mission

Build a field-sovereign, local-first drilling-engineering suite with one deterministic Rust engineering core and multiple presentation planes. A rig workstation must plan, ingest, calculate, alert, report, and preserve auditable history through a complete WAN outage. Central services federate, synchronize, analyze, and collaborate; they do not replace field engineering authority.

## Non-Negotiable Architecture

- Rust 2024 owns all engineering, unit, coordinate, uncertainty, and data-quality logic.
- Tauri 2 + React/TypeScript provides desktop and later mobile presentation only; TypeScript never performs engineering calculations.
- SI is canonical internally. UI units are validated boundary conversions.
- Every calculation emits an immutable receipt: algorithm/profile/version, input revisions and hashes, unit/CRS context, backend, warnings, actor, and output hash.
- Raw observations are immutable. Normalized observations retain source mnemonic, source unit, normalized unit, source/acquisition/ingest times, quality state, and lineage.
- AI is advisory and explanatory only. It may retrieve calculation receipts and call deterministic tools; it cannot author safety-relevant engineering results.
- GPU work accelerates qualified batches only. A deterministic CPU implementation remains the numerical authority; CUDA is preferred and Vulkan/WebGPU is the fallback.
- No Docker, Electron, cloud dependency, or silent fallback in a core operational path.

## Product Planes

| Plane | Responsibility | Boundary |
|---|---|---|
| Field workstation | Planning, surveys, engineering, realtime operations, reports, local journal | Operates without WAN |
| Federated hub | Sync, reconciliation, canonical assets, API, identity, audit, fleet analytics | Never invalidates local rig authority |
| Fleet web | Historical/live fleet visibility, replay, model-vs-actual, alerts | Reads typed hub contracts |
| Mobile observer | Cached/live observations, alerts, reports, approvals, annotations | No independent engineering implementation |
| Extension runtime | Rust SDK, Python bindings, signed permission-scoped apps, deterministic replay | Cannot bypass domain invariants |
| Local advisory plane | Rig-state explanations, anomaly summaries, report drafting through Mistral.rs | Cannot bypass deterministic safety policies |

## Capability Ownership

| Crate | Owns | Must not own |
|---|---|---|
| `wellforge-core` | Versioned contracts, units, identities, plot/calculation receipt contracts | Domain equations |
| `wellforge-3d` (3Dmk) | Versioned renderer-neutral scene contracts, geometry validation, bounds, visibility defaults, and scene provenance | Domain equations, raw survey/BHA/AC ownership, persistence, or UI state |
| `wellforge-survey` | Minimum curvature, survey corrections, coordinate transforms, trajectory primitives | UI or persistence |
| `wellforge-iscwsa` | Versioned uncertainty profiles, error-model propagation, covariance, EOU | Anti-collision policies or UI |
| `wellforge-ac` | Closest approach, SF/OSF/MASD, collision risk, scanning, policies | Survey mathematics or uncertainty-model definition |
| `wellforge-bha` | Component catalog, BHA analysis, contacts, beam response, vibration | T&D string mechanics or fluid behavior |
| `wellforge-tnd` | Soft/stiff string, operations, friction calibration, sinusoidal/helical buckling, API 7G | BHA contacts or hydraulics |
| `wellforge-hydra` | Fluid/rheology, pressure losses, ECD/ESD, surge/swab, hole cleaning | T&D or BHA mechanics |
| `wellforge-wits` | WITS/WITSML/ETP adapters, replay, channel QC, rig state | Solver implementation |
| `wellforge-storage` | App-data SQLite migrations, typed authority access, immutable revision events and receipts, replay, and synchronization protocol | Engineering calculations or frontend database access |

## Delivery Phases

### P0 — Mathematical and Data Authority

Deliver:

- Minimum-curvature geometry, trajectory interpolation, target/lease/closest-point operations, and CRS/unit invariants.
- ISCWSA Rev.5 profile engine with versioned toolcodes, covariance propagation, EOU, and published/reference validation fixtures.
- Standard anti-collision outputs: center distance, clearance, SF, OSF, MASD, scanning, look-ahead, and offset selection.
- Soft-string T&D, stiffness escalation path, loading/unloading, buckling, API 7G checks, steady hydraulics, ECD/ESD, surge/swab, and flow-tool geometry.
- WITS, WITSML, ETP, canonical typed events, replayable journal, identity and unit registry.
- Native-selected XML/artifact workspaces backed by app-data SQLite,
  deterministic replay, persisted calculation receipts, linear immutable
  revision-event history, and durable synchronization envelopes. The desktop
  runtime has no network database dependency.

Exit criteria:

- Reference/property/conformance suite passes for geometry, uncertainty, anti-collision, T&D, buckling, hydraulics, units, and data replay.
- A representative pad can be planned, scanned, replayed, and reported locally with the WAN unavailable.

### P1 — Field and Fleet Product

Deliver:

- Full Tauri planning/engineering workstation, live operation workspace, reports, wall plots, and audit viewer.
- Field-to-hub typed synchronization with explicit safety-object conflicts; no last-writer-wins for surveys, plans, casing, or collision policy.
- Fleet web plane, mobile observer, alert workflows, model-vs-actual overlays, historical replay, and controlled offline authorization leases.
- Deterministic hierarchical rig-state engine with source evidence, confidence, overrides, and state-transition history.
- Fluid/friction calibration workflows with residual distributions, validity interval, and confidence.

Exit criteria:

- A multi-day outage/reconnect exercise preserves event lineage, catches all safety-object conflicts, and reconciles deterministic state.
- Desktop, web, and mobile display the same calculation receipt for the same input revision.

### P2 — Engineering Differentiation and Extension Runtime

Deliver:

- Probabilistic collision risk alongside—not replacing—standard deterministic outputs.
- GPU-batched offset scans, uncertainty ensembles, trajectory candidates, and qualified high-fidelity mechanical workloads with CPU parity checks.
- Safe-operating trajectory envelopes, constrained multi-objective candidate generation, tortuosity diagnostics, and calibration-aware optimization.
- Transient hydraulics/hole-cleaning research profile, higher-fidelity BHA analysis escalation, and state-aware anomaly detection.
- Rust SDK, Python bindings, signed extension manifests, permission-scoped runtime, local/hub equivalent APIs, replay/backfill harness.

Exit criteria:

- Every accelerated path reports CPU-equivalent tolerances and backend provenance.
- A third-party extension can run against a local replay and central API without bypassing contracts or audit controls.

### P3 — Physics-First Intelligence

Deliver:

- Cross-well calibration priors, predictive dysfunction/failure models, and human-supervised trajectory recommendations.
- Standardized bit/BHA inspection ontology and later computer-vision workflows only after human-label agreement is measured.
- Local Mistral.rs engineering explanation/reporting tools bound to calculation receipts.

Exit criteria:

- Prospective, rig-held-out validation demonstrates value, calibrated confidence, deterministic fallback, and explicit human authorization.

## Data Model Rules

1. Separate observation, interpretation, and decision objects.
2. Safety-critical revisions use typed three-way reconciliation and named human resolution.
3. Comments, tags, and layouts may use convergent collaboration semantics; plans, surveys, casing, and risk policy may not silently converge.
4. Each boundary validates units, identifiers, quality state, provenance, and revision before conversion to concrete Rust types.
5. Warehouse sharing is an export sink, never the system of record.

## Verification Program

| System | Required proof |
|---|---|
| Geometry | Published/reference examples, singular/near-zero cases, property tests, round trips |
| Position uncertainty | Profile-specific fixtures, covariance symmetry/PSD checks, EOU orientation checks |
| Anti-collision | Analytic/dense-scan agreement, no missed synthetic intersections, policy fixtures |
| T&D and buckling | Limiting cases, independent benchmark comparisons, applicability warnings |
| Hydraulics | Controlled pressure-loss and ECD cases, measured-vs-modeled residuals |
| Realtime | Malformed/duplicate/out-of-order/reconnect replay and no logical record loss |
| Sync | 1 h, 24 h, and 7 day divergence; all safety-object conflicts surfaced |
| GPU | CPU/GPU tolerance parity, deterministic seeds, backend receipt metadata |
| Security | AuthZ negative cases, expired offline leases, audit tamper detection |

## Release Gates

1. Geometry authority: all coordinate/unit/trajectory invariants pass.
2. Positioning authority: uncertainty profile validation passes before safety use.
3. Collision authority: deterministic outputs pass before probabilistic outputs leave research mode.
4. Engineering authority: T&D, buckling, and hydraulics show benchmark agreement and model applicability.
5. Realtime authority: replay, outage, and reconnect tests pass without silent loss or corruption.
6. Fleet authority: a rig remains operational without the hub and reconciles deterministically on return.
7. Intelligence authority: prospective validation, confidence calibration, deterministic fallback, and human authorization are present.

## Clean-Room and Source Discipline

- Implement from licensed standards, public primary literature, customer-owned data, and independently derived models.
- Do not use proprietary source, binary inspection, copied assets, access-controlled scraping, or undocumented private schemas.
- Keep source provenance with each reference fixture and calculation profile.
- Name any non-standard behavior as a WellForge policy/profile; do not present an inference as an external fact.

## Research and Conformance Workstream

The program treats public literature and licensed standards as executable specifications. Each source produces: a versioned requirement, a mathematical or schema specification, independent reference fixtures, a conformance suite, and provenance metadata.

### Research Sequence

1. Positioning uncertainty and anti-collision.
2. Minimum-curvature trajectory/planning primitives and coordinate reference systems.
3. Torque-and-drag, stiffness, and buckling.
4. Non-Newtonian hydraulics, ECD, surge/swab, and transport.
5. BHA analysis, motor performance, and drill-ahead tendency.
6. WITS, WITSML, ETP, MQTT, ingress QC, replay, and rig-state inference.
7. State-aware analytics, asset lifecycle, reporting, mobile notifications, APIs, and later vision.

### Normative Source Families

| Domain | Controlled baseline | Resulting WellForge artifact |
|---|---|---|
| Trajectory | Minimum-curvature directional-calculation literature; ISO 19111 | Geometry oracle, CRS types, interpolation/target/closest-point fixtures |
| Position uncertainty | ISCWSA Rev.5.13 and maintained tool-model material | Profile registry, covariance/EOU engine, toolcode fixtures |
| Collision avoidance | Industry collision-management and separation-rule literature | SF/OSF/MASD/risk policy schema, scan and alert conformance suite |
| Survey integrity and geomagnetics | Survey-integrity literature; WMM, IGRF, licensed high-definition model data | Reference-field registry, correction/QC contracts, provenance rules |
| T&D and buckling | Johancsik soft-string; stiff-string and confined-tubular buckling literature | Independent soft-string oracle, stiffness escalation, transition fixtures |
| Hydraulics | API RP 13D; non-Newtonian and eccentric-annulus literature | Versioned rheology/pressure-loss profiles and controlled reference cases |
| BHA analysis and motor | Finite-element/tendency/dynamics literature; licensed OEM motor curves | BHA validation corpus and motor-curve data contracts |
| Realtime interoperability | WITS, WITSML 2.1, ETP 1.2, MQTT 5.0 | Adapter suite, replay fixtures, typed canonical event schema |
| Rig state and analytics | Process-state and drilling-efficiency literature | State ontology, deterministic baseline, calibration/evaluation contracts |
| Assets, API, and alerts | ISO 14224, ISO 55001, OpenAPI, Push/Notification standards | Asset-event schema, generated API contracts, notification delivery tests |
| Bit/BHA inspection | Current IADC dull-grading manual and public vision literature | Label ontology, image fixture protocol, later model evaluator |

### Independent Oracle Classes

| Oracle | Purpose |
|---|---|
| Standards oracle | Reproduces licensed/official definitions and sanctioned validation profiles. |
| Paper oracle | Reproduces published examples, limiting cases, and independently derived equations. |
| Behavioral oracle | Compares lawful, customer-owned exports or licensed observed outputs without inferring inaccessible implementation. |
| CPU oracle | Double-precision Rust numerical reference used to qualify CUDA and Vulkan/WebGPU acceleration. |

### Empirical Characterization Rules

- Treat externally observed behavior as a hypothesis, never a source of internal implementation fact.
- Sweep one variable at a time, retain every input/output capture, and validate hypotheses on blind cases.
- Use metamorphic tests: rigid coordinate rotation, coherent unit/length scaling, zero-friction limits, symmetric BHA cases, channel aliases, late/out-of-order telemetry, and future-data leakage tests.
- Distinguish standards-valid policy differences from defects. A non-standard observed behavior is evidence to document, not behavior to copy.
- Never bypass authentication, inspect private code, scrape access-controlled systems, or import proprietary assets/schemas.

### Required Research Outputs

1. `standards_registry` with source version, license/provenance, implementation profile, and applicability.
2. Source-to-feature traceability: `source → normative equation/schema → reference fixture → conformance test → release gate`.
3. Deterministic replay harness that can drive the field workstation, hub ingestion, state engine, alerts, and solver stack at stepwise, real-time, and accelerated playback.
4. Differential comparator that classifies results as reference match, standards-valid policy difference, WellForge defect, or unresolved hypothesis.
5. Per-domain evidence reports containing numeric tolerances, out-of-domain behavior, unresolved assumptions, and validation coverage.

### Vertical Slice Milestones

| Slice | Deliverable | Evidence gate |
|---|---|---|
| Positioning | Trajectory + uncertainty profile + anti-collision scan | Reproduces reference fixtures with frozen precision rules |
| Mechanics | Soft-string + buckling + BHA analysis comparison harness | Limiting cases and independent benchmark family pass |
| Fluids | Rheology + pipe/annulus/nozzle + ECD/ESD | Controlled pressure-loss and ECD fixtures pass |
| Realtime | One input corpus through WITS/WITSML/ETP/canonical replay | No logical loss; idempotent replay and unit/identity parity |
| Operations | Rig-state + QC + state-aware KPIs | Held-out replay and fault-injection evidence passes |
| Product | Desktop/web/mobile use identical calculation receipts | Same input revision produces identical displayed result/provenance |

### Program Estimate

The deep mathematical, data, and empirical-validation track is planned as approximately 180 person-days. It may be parallelized across positioning, mechanics, realtime/data, and validation workstreams, but release gates remain dependency-ordered: geometry before positioning, positioning before safety-use collision output, trustworthy ingest before realtime analytics, and deterministic authority before AI advisory features.
