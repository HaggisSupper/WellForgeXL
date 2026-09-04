# WellForge Source Migration Catalog

## Scope and method

This is a file-level catalog of the full supplied archive, focused on source
code and associated technical material. Installer payloads, personal records,
administrative material, and generic dependency/build output are intentionally
outside the migration intake.

The archive contains **381,387 files**. This does not represent 381,387 unique
implementation assets: it includes repeated branch/release snapshots, packaged
dependencies, binaries, compressed artifacts, and generated output.

## Archive classification

| Classification | Evidence | Migration treatment |
| --- | --- | --- |
| Primary application source | Main development line: 2,881 C# files, 135 project files, 629 data scripts | Authoritative starting point for code migration |
| Other source branches and releases | 49,176 C# files across the archive; many are copied lines and release snapshots | Preserve for change history; do not merge mechanically |
| Shared framework source | 1,488 C# files | Assess for reusable domain, units, document, and utility behavior |
| Catalog source and scripts | Modular catalog application, XML tooling, and relational scripts | Port its business rules and normalized data model into Rust and PostgreSQL |
| UI-specification workspace | WPF controls/views plus TypeScript prototypes and migration/design notes | Use as interaction and component reference, not as a source-of-truth application |
| XML/data fixtures | 1,978 XML files plus project/BHA samples and report artifacts | Build parser, round-trip, migration, and calculation fixtures |
| Engineering reference corpus | 507 PDFs, 260 DOCX files, 170 DOC files, presentations, help material | Convert internally authored requirements into testable acceptance criteria; retain external papers as reference only |
| Build/deployment artifacts | 8,098 DLLs, 432 executables, installer directories, package payloads | Exclude from migration input; retain only for historical deployment reference |
| Web/package output | 25,474 JavaScript, 14,997 TypeScript, 13,607 source maps, and very large compressed dependency content | Exclude vendored/generated trees; inspect only authored prototypes and configuration |
| Media and reports | Images, binary data, report output, scans | Keep as fixtures or visual references where provenance is clear |

## Canonical source baseline

The primary product tree contains 45,119 C# files and 2,026 project files, but
is dominated by historical copies. The selected main development line is the
canonical intake baseline:

| Area | C# files | Project files | Treatment |
| --- | ---: | ---: | --- |
| Desktop application | 2,684 | 123 | Primary migration input |
| Catalog application | 200 | 11 | Primary catalog/workflow input |
| Data-to-XML converter | 8 | 1 | Format-migration reference |
| Total selected baseline | 2,892 | 135 | Review, extract, and test |

Development, quality, production, solver, prototype, and release copies share
roughly 64–85% of their source filenames with the selected baseline. Retain
them as comparison evidence only. Source-control marker files are numerous and
there is no reliable modern history to establish recency by timestamp alone.

## Implementation intake tiers

### Tier 1 — port deliberately

These are the best candidates for a clean Rust implementation because they
carry product behavior rather than obsolete presentation infrastructure.

| Capability | Existing evidence | Target |
| --- | --- | --- |
| Engineering domain and units | Domain model, unit handling, validation, lookup contracts | Rust domain crates with explicit types and unit-safe APIs |
| XML project/BHA formats | Project files, BHA files, XML utilities, conversion tool | Rust parser/writer with lossless unknown-node retention |
| Trajectory and well structure | Model, services, views, test fixtures | Rust services plus TypeScript visualization |
| Calculation engines | BHA, hydraulics, torque-and-drag solver projects and reports | Rust calculation crates with golden-result tests |
| Catalog rules | Component/category/attribute/release and publishing modules | Rust service layer and PostgreSQL model |
| Reporting inputs | Report engine, report data services, representative PDFs | Typed report data contracts and modern export adapters |

The main-line code is particularly suitable for a staged Rust intake:

| Priority | Area | Evidence | Treatment |
| --- | --- | --- | --- |
| High | Domain model | 201 files, with only four UI-coupled | Typed entities, units, validation, and project state |
| High | Services/contracts | 102 files, without detected UI coupling | Rust application services and explicit APIs |
| High | XML formats | 42 relevant files | Fixture-driven read, validate, and write compatibility |
| High | Catalog data | 53 XML files and 203 data scripts | PostgreSQL migrations, release, approval, and audit domains |
| High | Report core | Seven non-UI engine files | Report composition, with rendering replaced separately |
| Medium | Utilities/import parsing | 28 files, eight UI-coupled | Extract pure parsing/conversion logic first |
| Medium | Engineering solvers | 84 shared, 69 BHA, 70 hydraulics, and 50 torque/drag files | Isolate numerical kernels behind golden-result tests |
| Low | Desktop screens and charts | Tightly coupled legacy UI dependencies | Rebuild from workflow specifications in Tauri |

Solver code is valuable but not yet a clean library: approximately 23–35 files
per solver group depend on views, commands, charts, or other UI concerns. Port
the numerical kernels after separation, not the surrounding UI structure.

### Tier 2 — extract behavior, then replace

| Material | Why it is useful | Modern treatment |
| --- | --- | --- |
| WPF views, controls, and view models | Records workflows, validation, grouping, filtering, dialogs, and chart interactions | Convert behavior into UI specifications and implement in TypeScript/Tauri |
| Legacy service and repository layers | Exposes data ownership and workflow boundaries | Replace with Rust APIs and PostgreSQL repositories |
| Data scripts | Defines prior catalog and reference data relationships | Map to migration/staging scripts; do not execute unchanged |
| Prototype TypeScript | May contain reusable interaction experiments | Retain authored code only after dependency and license review |

### Tier 3 — reference-only

- Repeated development branches and release copies.
- Generated code, build outputs, package directories, source maps, and binary
  dependencies.
- Installer assets and setup packages.
- External training manuals, published papers, patents, and vendor material.
- Personal, finance, travel, certificate, scan, and unrelated office files.

## Technical-reference queue

The associated engineering material contains high-value inputs for acceptance
criteria and fixture design:

1. solver architecture and static/vibration interpretation material;
2. torque-and-drag brief/full report examples and a drillstring feasibility
   study;
3. catalog administration, maintenance, publishing, and end-user manuals;
4. plotting, hydraulics, catalog-data, and system-integration requirements;
5. dynamic stiff-string torque-and-drag research and directional-drilling
   reference material;
6. project, plan, BHA, motor evaluation, and operational report samples.

Published research and external manuals must inform requirements and validation
only. They are not source material to copy into code or documentation.

### Golden-fixture intake

A dedicated test-data collection provides four full project files, ten
standalone BHA XML files, fifteen additional XML samples, trajectory/formation/
fluid imports, real-world reports, a well-data workbook, and solver input/output
families. A separate training corpus adds 22 BHA files and 12 project files.

Create a licensed, de-identified fixture repository from this material. Its
initial test suites should cover:

- XML parsing, validation, and no-op round trips;
- import validation and unit conversion;
- calculation outputs at agreed numerical tolerances;
- catalog import and reference-data validation; and
- report snapshots and source-data consistency.

Additional motor tables cover hierarchy, geometry, materials, loads,
performance, power curves, and hole-size constraints. They are a good input for
a catalog-import contract and data-quality tests.

## UI/prototype intake

The separate UI workspace is a design and prototype archive, not a buildable
legacy solution. It contains 106 XAML resources/screens, 74 C# code-behind
files, and approximately 1,598 bindings or events. Treat these as interaction
and visual specifications.

High-value workflows include saved docking layouts, module loading, unsaved
change handling, preferences, autocomplete, drag/drop assembly composition,
grid editing, trajectory import/plotting, analysis post-processing, and the
domain forms.

Authored prototypes provide a more direct implementation head start:

- Rust trajectory, minimum-curvature, dogleg/build/turn, and downsampling code
  with unit tests;
- Tauri command models and a visualization shell;
- typed streaming records/state and chart hooks; and
- docking, chart, contour, 3D-view, plug-in discovery, and CSV-export
  components.

There are two byte-identical portable-component copies: retain exactly one
canonical copy after license review. Exclude dependency directories, bundles,
source maps, generated graph output, and lockfile-derived material. The
migration/style documents are useful candidate decisions, but they contain
conflicting framework and chart choices and require reconciliation with the
approved WellForge architecture.

## Clean migration rules

1. Select one main source line before copying any code.
2. Copy no UI framework or persistence implementation wholesale.
3. Isolate domain rules from WPF, service-location, ORM, and database calls.
4. Translate each selected C# behavior into a small Rust module with fixtures
   and tests before building its interface.
5. Rebuild screens from workflow specifications, not from direct XAML
   conversion.
6. Map catalog data into PostgreSQL through staged, verified migrations.
7. Preserve every source fixture and its output expectation for numerical and
   XML compatibility testing.

## Recommended next queue

1. Create a deduplicated manifest for the chosen main line.
2. Extract XML format, units, and trajectory code into a reviewed intake set.
3. Build calculation fixture packs from the solver reports and sample projects.
4. Map catalog entity and release behavior to PostgreSQL tables and migrations.
5. Convert the UI-specification workspace into WellForge component/workflow
   specifications, retaining only authored prototypes that pass review.

## Scan limitation

State mapping was deliberately omitted: the supplied archive is too large for
the available mapping tool to process reliably. This catalog is based on a
complete file inventory and parallel, read-only classification of the source,
technical-reference, and UI/prototype blocks.
