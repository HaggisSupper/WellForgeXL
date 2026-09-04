# WellForge Phase 1 Intake

## Objective

Establish a small, testable Rust foundation from the selected application line
without copying legacy UI or persistence code into the new product. Phase 1
creates the compatibility harness first; later implementation work uses it as
the acceptance gate.

## Intake boundary

| Include | Exclude |
| --- | --- |
| Domain entities, units, validation rules, XML formats, calculation inputs/outputs, report-data rules, catalog workflow rules | Views, view models, UI frameworks, dependency injection wiring, ORM mappings, installers, binaries, package trees, generated code, and release copies |
| XML/BHA/project fixtures, import files, representative reports, solver-result baselines | Personal material, external training/published material, and patent-adjacent content as implementation input |
| Internally authored requirements, data maps, catalog manuals, and test evidence | Any external document as code to copy or as an unreviewed functional contract |

## Initial Rust workspace

```text
wellforge-domain       entities, value objects, units, validation contracts
wellforge-formats      XML read/validate/write and compatibility preservation
wellforge-fixtures     approved, de-identified fixture corpus and expectations
wellforge-trajectory   survey geometry, minimum curvature, and related measures
wellforge-catalog      catalog rules, release state, attributes, and units
wellforge-storage      PostgreSQL migrations, typed access, and audit records
wellforge-reports      report-data assembly and snapshot models
```

Calculation crates for BHA, hydraulics, and torque-and-drag are intentionally
deferred until the first five crates can read fixtures and express units and
project state reliably.

## Work packages

### 1. Fixture governance

- Select a minimal representative fixture pack: valid/invalid XML, standalone
  BHA, complete project, trajectory import, catalog export, calculation input,
  calculation output, and report snapshot.
- De-identify names, customer data, and operational identifiers before bringing
  fixtures into the new repository.
- Register source type, content hash, expected operation, and expected result
  for every fixture.
- Preserve untouched source copies outside the new repository for traceability.

### 2. Domain and unit contract

- Define typed identifiers, measured values, unit conversions, well/project
  structure, survey stations, BHA components, and component attributes.
- Translate validation behavior into explicit errors rather than UI messages.
- Add unit and property tests before any screen implementation.

### 3. XML compatibility contract

- Implement read/validate/write for the minimal fixture pack.
- Preserve unrecognized elements, attributes, ordering-sensitive content, and
  encoding details where fixtures demonstrate the need.
- Test no-op round trips semantically and byte-for-byte where feasible.
- Treat malformed legacy inputs as named compatibility cases, not silent errors.

### 4. Trajectory slice

- Implement survey-station models and minimum-curvature calculations.
- Compare measured depth, inclination, azimuth, dogleg, build, turn, and output
  coordinates against selected fixtures at agreed tolerances.
- Expose a small command/API suitable for the desktop trajectory view.

### 5. Catalog and storage slice

- Model category, component, attribute, unit, release, approval, and audit
  rules independently of previous tables.
- Create PostgreSQL migrations and staged import mappings.
- Validate catalog exports and role/release behavior against approved examples.

## Definition of done for Phase 1

1. Rust builds cleanly and all fixture tests pass.
2. The selected XML files validate and round-trip without semantic loss.
3. Unit conversion and trajectory calculations match approved expectations.
4. A PostgreSQL schema can load a small catalog fixture and record an audited
   release operation.
5. The desktop proof of concept can open a project, show hierarchy and
   trajectory, and report validation results using the Rust boundary.

## Conversion rules

- Re-implement small, verified behaviors; do not convert whole files
  mechanically.
- Keep a source-to-test trace for every behavior selected for migration.
- Do not introduce database access, rendering, or application state into
  numerical/domain crates.
- Compare historical copies only when the baseline behavior is unclear.
- Require engineering review before accepting numerical tolerance changes.

## Immediate next action

Create the de-identified fixture manifest and select the first XML/BHA/project
set. That makes the parser and domain implementation measurable from its first
commit.

## State mapping

State mapping is skipped for this documentation-only intake update because the
supplied archive exceeds the available tool's reliable file-volume limit.
