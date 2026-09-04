# WellForge codebase audit

## Scope

This is a read-only architecture audit of the full selected source archive. It deliberately separates executable source from historical copies, release snapshots, training data, installation artifacts, third-party code, and unrelated office/personal files.

## Executive summary

WellForge is a Windows engineering desktop platform for well trajectory, wellbore/interval, fluids, formations, bottom-hole assemblies, static/vibration/hydraulics/torque-and-drag analysis, reporting, and component catalog management. It is primarily C#/.NET Framework 4.0 WPF, with legacy relational persistence, XML project formats, and a large source-control archive containing multiple branches and copied release lines.

The full archive contains approximately 50,000 C# files, but the count is dominated by repeated development, QA, production, solver, and release snapshots. The principal maintained application candidate is the `Main-branch` desktop solution; the other copies should be treated as comparison/reference sources until a version-baseline decision is made.

## Archive classification

| Area | Classification | Role |
|---|---|---|
| WellForge application branches | Primary engineering source | Desktop UI, domain model, services, solvers, reports, installers, tests, and prototypes. |
| Shared framework | Shared source | Shell, utility modules, unit conversion, data transforms, documents, service helpers, WITS/WITSML tooling, and test UI. |
| Catalog application | Supporting product | Modular component catalog, legacy relational schema, XML export, publishing/release workflow. |
| Comanche and M2E | Related products | Separate release-line applications and shared/vendor dependencies. |
| Third party | Vendored dependencies | UI controls, logging, and older package source. Do not port directly. |
| Documents, training, reference projects, installs, backups | Evidence/assets | Requirements, test data, archived projects, help, installers, spreadsheets, reports, and research material. |

## Canonical WellForge desktop architecture

```text
WellForge Desktop (WPF / Prism / MVVM)
├── WellForge Model                 project, well, BHA, materials, and analysis entities
├── WellForge Services              persistence, project IO, lookup/repository services
├── WellForge UI Shell              composition, docking, navigation, shared contracts/resources
├── Domain UI modules
│   ├── Well information
│   ├── Trajectory
│   ├── Plan interval / wellbore
│   ├── Fluids
│   ├── Formation
│   ├── Bottom-hole assembly
│   ├── Analysis
│   └── Deliverables / reporting
├── Solver framework
│   ├── BHA analysis
│   ├── Torque and drag analysis
│   └── Hydraulics analysis
├── Reporting                     OpenXML-based output and report data services
└── Support                       units, lookup data, repository, licensing, resources, tests
```

### Main solution modules

The main desktop solution contains a domain model; service and service-contract projects; WPF shell/UI/resource projects; UI modules for well information, trajectory, plan interval, fluid, formation, BHA, analysis, and deliverables; solver frameworks; report generation/data services; lookup/repository projects; units/utilities; test projects; and mock/test helpers.

Its solver modules are not fully isolated: their projects reference WPF, Prism, charting, docking, and plotting libraries as well as domain code. That coupling is the main technical constraint on a port.

## Data and format boundaries

| Boundary | Role |
|---|---|
| Project XML | Serialized complete engineering projects: general information, survey/trajectory, wellbore, materials/fluids, BHA cases, solver settings, experiments, results, and report settings. |
| BHA XML | Reusable component-string documents with generic geometry plus specialized motor, RSS, stabilizer, hydraulic, tubular, and LWD data. |
| Legacy relational store | Lookups, application data, catalog data, conversion scripts, reporting/publishing data, and installation databases. |
| Spreadsheets/PDF-derived inputs | Catalog source data and engineering reference data; some imports require manual cleanup. |
| Office documents | Requirements, design, validation, release, and operational evidence. |

## Catalog subsystem

The catalog is a separate .NET Framework modular application with a browser/service tier and legacy relational data model. It maintains business product lines, categories, component metadata, attributes, units, user authority, audit history, snapshots, publishing, and releases.

Its export flow is:

```text
workbook/reference data → SQL staging/catalog tables → common or specialized XML conversion → component bulk load
```

The common master table represents pipe-like tools. Specialized families include LWD, MTR, MWD, motor, RSS, stabilizer, bit, and other equipment categories. Special tools may be emitted from their dedicated tables rather than the common master table.

## Shared framework and related product lines

The shared framework provides application shell infrastructure, utility modules, database/script activity, document tooling, transformation/units/interpolation processors, WITS and WITSML integration utilities, and test applications. Related product folders contain release archives and product variants; treat them as dependency/provenance candidates rather than part of the canonical desktop build until they are compared against the chosen baseline.

## Technology baseline

- C# and .NET Framework 3.5/4.0.
- WPF, Prism 4, Unity/MEF-era composition, MVVM-style view models and XAML.
- Entity Framework 4.1, legacy relational persistence, XML serialization, OpenXML report generation.
- Older plotting/docking/grid libraries, including proprietary WPF control suites.
- Visual Studio solution/project files and legacy source-control metadata; no modern Git repository was detected in the archive.

## Branches, copies, and release evidence

The main product directory includes development, QA, production, solver, feature-validation, fluid, release, and prototype copies. Release folders alone account for a very large share of the source files. Do not use line count as a measure of active complexity; first select one baseline and make all other copies read-only comparison corpora.

## High-value test/evidence assets

- Standalone and embedded BHA/project XML samples for parser compatibility.
- Hydraulics validation utilities and result comparisons.
- Model, UI, service, trajectory, formation, interval, and utility test projects.
- Component-catalog SQL schema and export scripts.
- Offline help/reference content describing BHA, motor, RSS, input validation, static analysis, and vibration analysis.

## Main risks

1. Legacy UI/control stack and old .NET runtime.
2. Duplicate branches/releases obscure the authoritative behavior.
3. Solver logic is coupled to WPF/charting dependencies.
4. XML format versions and malformed historical documents require tolerant readers plus strict writers.
5. SQL scripts include destructive rebuild patterns; use isolated databases only.
6. Config/security/LDAP-related code requires a credential and deployment review before any build or migration.

## Recommended baseline for future work

1. Use the main branch desktop solution as the initial implementation reference.
2. Use the main branch catalog as the catalog reference.
3. Preserve release/QA/production copies as regression evidence, not active source.
4. Build a fixture corpus from BHA/project XML, catalog records, expected calculations, and report outputs before rewriting behavior.
