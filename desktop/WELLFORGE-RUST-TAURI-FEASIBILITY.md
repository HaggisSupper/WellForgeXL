# WellForge Rust and Tauri feasibility

## Verdict

An incremental port is **feasible and recommended**. A one-shot translation is not. Tauri is a strong replacement for the legacy desktop shell, TypeScript is appropriate for the user interface, and Rust is a good long-term home for file formats, engineering domain logic, numerical solvers, managed project artifacts, and queued synchronization. The critical condition is a behavior-preserving test corpus before solver migration.

| Area | Feasibility | Rationale |
|---|---|---|
| Desktop shell and navigation | High | Tauri provides a Windows desktop shell with a WebView frontend and Rust commands. |
| Forms, grids, and configuration screens | High | Rebuild in TypeScript/React or another web UI framework; no need to port XAML. |
| XML project/BHA parsing and validation | High | Rust has mature XML/serialization options; existing files make strong fixtures. |
| Domain model and units | High | Pure data/domain logic ports cleanly when separated from UI. |
| Catalog workflow | High | CRUD/release/audit flows map naturally to an API + database model. |
| Reports | Medium | Rebuild templates/output rules; OpenXML behavior needs comparison fixtures. |
| Engineering solvers | Medium | Numerically suitable for Rust, but legacy code is coupled to UI libraries and needs parity testing. |
| Exact visual parity | Low | Existing docking/charting controls must be redesigned, not translated. |
| One-step full replacement | Low | Too many duplicated branches, undocumented behavioral paths, and integration dependencies. |

## Target architecture

```text
WellForge Tauri desktop app
├── TypeScript frontend
│   ├── project editor, grids, charts, forms, reports UI
│   └── typed client generated from command/API schemas
├── Tauri command boundary
│   ├── narrow, validated commands
│   └── explicit filesystem/database capabilities
└── Rust workspace
    ├── wellforge-domain       domain entities and units
    ├── wellforge-formats      project/BHA XML readers, writers, migrations
    ├── wellforge-calc         trajectory, BHA, hydraulics, torque/drag algorithms
    ├── wellforge-catalog      catalog records, audit and release behavior
    ├── wellforge-storage      PostgreSQL access, migrations, and audited persistence
    ├── wellforge-reports      report data and export adapters
    └── wellforge-desktop      Tauri integration only
```

Keep numerical/domain crates free of Tauri and frontend dependencies. This permits headless regression testing, a CLI, batch processing, and future server use.

## Migration strategy

1. **Freeze the behavioral baseline.** Select a main source line; hash/version all input XML; capture expected solver outputs, reports, and catalog exports.
2. **Create format compatibility first.** Implement read-only Rust parsers for BHA/project XML, preserving unknown/legacy XML nodes. Validate against every collected document, including malformed-file handling.
3. **Ship a read-only desktop explorer.** Build a Tauri/TypeScript app that opens, validates, and visualizes projects/BHAs without running calculations. This tests the desktop architecture early.
4. **Port non-solver business slices.** Well/project metadata, trajectory editing, wellbore, materials, fluids, and catalog lookup workflows.
5. **Port solvers independently.** Start with the most testable deterministic solver; compare every output field against the existing application at defined tolerances. Do not port UI with solver code.
6. **Add authoring and report generation.** Enable controlled XML writes, then reports and release workflows only after round-trip compatibility is established.
7. **Parallel-run before retirement.** Use both applications on the same fixtures/projects until material outputs agree and user acceptance testing passes.

## Key design decisions

- Use TypeScript only for presentation state, forms, charts, and command orchestration. Keep engineering calculations in Rust.
- Treat XML as a compatibility contract. Avoid converting immediately to a lossy “clean” schema; retain unknown nodes and original encoding where required.
- Migrate the legacy relational schema into PostgreSQL, retaining explicit provenance and compatibility mappings for imported catalog records.
- Replace UI-control behavior intentionally: use modern data grids/charts and a layout that supports engineering workflows, rather than cloning legacy docking behavior exactly.
- Expose a narrow command API and scope all file access. Tauri v2 capabilities and filesystem scopes are designed for this explicit permission model.

## Principal risks and mitigations

| Risk | Mitigation |
|---|---|
| Numerical drift | Golden fixtures, invariant/property tests, explicit unit conversions, acceptance tolerances reviewed by engineering. |
| Hidden UI business logic | Trace view models/event handlers before porting each workflow; move rules into Rust services. |
| XML version incompatibility | Round-trip tests, unknown-node retention, migration versioning, and corpus-based validation. |
| Catalog/data migration | Map and reconcile legacy relational records before enabling PostgreSQL writes. |
| Desktop UI expectations | Prototype BHA/trajectory/result views with users before building the full shell. |
| Native integration | Inventory printing, Office/Excel, licensing, install, and database dependencies as separate workstreams. |

## First proof of concept

Build a small WellForge Tauri app that:

1. opens a project or BHA XML file;
2. validates and displays its hierarchy, including LWD/motor/RSS variants;
3. plots trajectory survey stations;
4. preserves unsupported XML during a no-op round trip; and
5. runs one deterministic calculation with an expected-result fixture.

Success on this proof demonstrates the three hard boundaries—desktop UI, Rust format/domain code, and legacy compatibility—without committing to a full rewrite.

## Data-platform decision

PostgreSQL is the target transactional system of record. It is an open-source,
server-based fit for catalog publishing, project metadata, revision history,
permissions, and audit records. XML remains a portable engineering artifact;
the database indexes metadata, catalog data, lineage, and controlled revisions
without replacing the file format prematurely.

The initial design deliberately uses one operational database:

- PostgreSQL for catalog releases, project metadata, user access, and audit
  events;
- Rust `sqlx` migrations and typed queries at the storage boundary;
- encrypted database connections, SCRAM authentication, least-privilege roles,
  and row-level security where isolation requires it;
- normal relational indexes for project, component, release, and revision
  lookups; partition high-volume append-only audit tables only after measured
  evidence justifies it.

This avoids adopting a separate embedded database or file-oriented analytics
format as a second system of record. DuckDB and Polars consume read-only
PostgreSQL projections and approved artifacts for analytics, validation, and
report preparation without changing the authoritative model.
