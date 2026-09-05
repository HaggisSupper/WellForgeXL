# WellForge Archive-Example Deep Dive

## Scope and handling

This review covered the drilling-engineering examples held in the designated
read-only archive. It excluded installers, binaries, dependency trees, build
output, caches, duplicated third-party content, and generated graph artifacts.
No archive source, schema, asset, or text was copied into the WellForge
workspace. The findings below are independently stated capability and testing
requirements.

## Evidence inventory

| Area | Read-only evidence | WellForge use |
| --- | --- | --- |
| Engineering help | 168 embedded help topics, with a mirrored branch | Workflow and capability map only |
| Technical material | 186 curated training files and 24 course files | Research pointers and candidate public-reference families |
| Structured examples | 201 inspectable XML, delimited text, spreadsheet, project, assembly, and solver-output artifacts | De-identified fixture-design input |
| Application source | 482 solution files; representative area included 47 C# projects, 1,154 C# files, 144 views, and 60 test-named files | Architecture and test-pattern input only |
| Broad document set | 1,531 PDF/Office/help files after obvious exclusions | Triage queue, not a direct implementation source |

## Capability model

The examples support the following independently authored WellForge hierarchy:

```text
workspace
  -> well
    -> plan and observed trajectory
      -> ordered intervals and survey stations
        -> formation, fluid, assembly, string, rig constraints
          -> scenario/configuration
            -> analysis job/run
              -> immutable result snapshot and report projection
```

Key rules:

- Planned trajectory and observed survey data remain distinct.
- Reusable configuration is versioned and copy-as-new creates new identities.
- Engineering work runs as a typed job with validated inputs, diagnostics, and
  immutable output attached to a run; it never mutates source configuration.
- Units are canonical internally, converted only at controlled display/import
  boundaries.
- Import is previewed and validated before a transactional commit; failed
  validation cannot partially persist data.
- Reports are projections from stable result snapshots, never a second source
  of truth.

## Required domain contracts

| Contract | Core fields |
| --- | --- |
| Workspace and well | Identity, label, status, unit preference, containment |
| Plan and trajectory | Ordered stations, measured depth, inclination, azimuth, derived coordinates, curvature metrics, provenance, validation state |
| Interval and operating context | Ordered interval, casing, formation, fluid, rig limits, applicable configuration |
| Assembly and string | Ordered component records, geometry, material properties, placement, configuration |
| Analysis job/run | Analysis family, canonical parameter set, input-artifact reference, lifecycle state, diagnostics, typed output groups, receipt |
| Import preview | Format/version, counts, warnings/errors, cancellation/progress, explicit commit decision |
| Result series/chart | Axis/unit metadata, points, rendering hints, deterministic downsampling contract |

## Engineering workstreams

1. Trajectory: minimum curvature, dogleg severity, build/turn, coordinate
   transforms, import validation, and 3D/grid views.
2. Hydraulics: hydrostatic and frictional pressure, rheology profiles,
   annular velocity, nozzle/bit optimization, and explicit applicability.
3. Mechanics: torque-and-drag, pipe wear, static stress, buckling, vibration,
   natural frequency, and critical-speed result families.
4. Operations: typed observations, replay, deterministic state classification,
   scenario/run evidence, and report snapshots.

Each family needs independently sourced mathematics, synthetic fixtures,
numerical tolerances, and conformance tests before operational use. Any
well-control or safety-limit workflow remains deferred until current standard
validation and subject-matter review are complete.

## Modern stack mapping

| Need | WellForge implementation |
| --- | --- |
| Field-authoritative workspace and audit history | SQLite local store with typed Rust boundaries |
| Engineering calculations and receipts | Rust 2024 capability crates |
| Desktop presentation | Tauri 2 with React/TypeScript; no engineering math in the UI |
| Large-result transforms and reporting preparation | Polars over approved projections |
| Analytical joins, aggregates, and export preparation | DuckDB, downstream and non-authoritative |
| Interchange | Independently authored, versioned portable package/artifact contracts |

Docker is excluded from this stack. No archived implementation is a source of
truth for WellForge behavior; archive examples only inform requirements,
fixture design, and research sequencing.

## Next implementation order

1. SQLite project hierarchy, revision, analysis-job, result, and import-preview
   persistence.
2. Typed interval, formation, fluid, assembly, string, and rig-limit models.
3. Transactional import preview/commit with synthetic valid and invalid
   fixtures.
4. Rust solver-adapter lifecycle and deterministic result retrieval.
5. Unit-aware trajectory/result views, controlled downsampling, and report
   projections.
6. Further hydraulics, mechanics, and vibration slices only after reference
   fixtures and scope gates are in place.
