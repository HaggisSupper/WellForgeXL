# WellForge Tauri Port Architecture

## Approved stack

The WellForge desktop port uses this stack throughout the programme:

| Layer | Approved technology | Responsibility |
| --- | --- | --- |
| Desktop host | Tauri 2 | Native application lifecycle, secure command boundary, packaging, scoped capabilities |
| Engineering core | Rust 2024 | Domain models, units, XML, calculations, validation, reporting inputs, catalog rules |
| Desktop interface | React 18 + TypeScript | Forms, workflow state, charts, grids, layout, and presentation only |
| Frontend build | Vite | Development and production web assets |
| Interface state | Zustand | Typed presentation state, selections, layout, and command status |
| Styling | Tailwind CSS | Shared visual tokens and responsive desktop layout |
| Shared system of record | PostgreSQL | Catalog, releases, project metadata, access policy, audit, and controlled revisions |
| Analytics/transformation | DuckDB + Polars | Fast read-only analysis, reporting datasets, batch validation, and columnar transformations |
| Portable project artifact | Versioned XML | Offline project interchange and engineering-document compatibility |

TypeScript must not calculate engineering results. It sends typed requests to
Rust and renders typed responses, including calculation provenance and errors.

## Boundary model

```text
React + Zustand
        |
        | typed Tauri commands
        v
Tauri 2 command boundary
        |
        +--> Rust domain / formats / survey / solver crates
        |
        +--> PostgreSQL storage and audit boundary
        |
        +--> versioned XML project artifacts
```

Every calculation request includes a validated project revision and returns a
deterministic receipt containing input identity, algorithm/profile version,
warnings, backend, and output hash.

DuckDB and Polars operate downstream of validated PostgreSQL projections and
approved project artifacts. They accelerate analytical queries, report-data
assembly, fixture analysis, and batch transformations. They cannot publish a
catalog release, change a project revision, or become the safety/operational
source of truth.

## Offline operation

Offline work is supported by controlled XML project artifacts and a local
append-only operation queue stored with the project workspace. The queue is an
interchange/audit artifact, not a second database or a second source of truth.
When connectivity returns, Rust submits queued operations to PostgreSQL through
explicit reconciliation rules. Safety-relevant conflicts require named human
resolution; they never silently overwrite a shared revision.

## Tauri security rules

- Define narrow command inputs and structured Rust errors.
- Scope filesystem permissions to the user-selected project workspace only.
- Keep database credentials in the deployment secret manager; never expose them
  to the frontend bundle.
- Validate paths, XML, identifiers, units, and revisions at the Rust boundary.
- Use capability files to deny unneeded plugin, filesystem, shell, and network
  access by default.

## Read-only document inspection

The initial document command inspects only the active project path; its caller
cannot supply a path. It accepts only `.drillproj`, `.bha`, and `.xml` files,
rejects non-regular and reparse-point paths, resolves the path, and reads from
one opened file handle with a fixed 16 MiB byte limit. It applies the Rust
format parser's structural limits and returns a typed summary plus validation
diagnostics. It does not write files, open a database connection, invoke a
solver, or provide generic file browsing. Generic XML is classified by its
supported root element; all other roots receive a structured error. Broader
import workflows remain a separate future command.

## Native project selection

Project selection is owned by a Rust command backed by the desktop-native
dialog plugin. The renderer sends no path string: it can only request a single
selection or receive a cancellation. Before state changes, Rust rejects
unsupported extensions, reparse points, and non-regular files, then stores the
canonical path. The renderer is not granted dialog-plugin capability, so it
cannot use the plugin to obtain a path and submit it through a separate route.
Inspection revalidates the retained path when it reads it, which protects the
read path if the selected file changes after selection.

## Porting rules

1. Re-implement domain behavior in Rust with fixtures and tests.
2. Use legacy views only as workflow and interaction specifications.
3. Replace old UI framework components with React components and Tauri commands.
4. Keep XML compatibility and numerical verification ahead of visual parity.
5. Migrate catalog and audit workflows through PostgreSQL staging and verified
   releases.
6. Make each desktop screen consume a stable Rust command/API contract.

## Delivery order

1. Rust contracts, units, XML fixtures, and survey geometry.
2. Read-only Tauri explorer: project hierarchy, validation, and trajectory.
3. Catalog lookup/release workflows backed by PostgreSQL.
4. BHA, torque-and-drag, hydraulics, and report slices with numerical fixtures.
5. Controlled project authoring, offline queue reconciliation, and reporting.
6. Parallel operating validation before retiring an established workflow.

## Non-goals

- No direct conversion of XAML/code-behind into frontend code.
- No engineering formula implementation in TypeScript.
- No embedded database or file-oriented analytics store as a second authoritative
  persistence system.
- No broad filesystem or shell access from the desktop UI.
