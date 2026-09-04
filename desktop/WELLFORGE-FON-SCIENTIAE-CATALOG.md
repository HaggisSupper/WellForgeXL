# Fon Scientiae Geoscience Catalog

## Catalog status

Source location: `C:\Development\Fon Scientiae`

The first inventory captured only a shallow subset of the collection. Two
subsequent full read-only traversals matched at 381,384 files and
74,394,428,925 bytes; they replace those figures below. A full SHA-256 pass
then rechecked the same metadata fingerprint before and after hashing, so this
is a stable, content-addressed physical baseline.

No source files, scripts, formulas, schemas, or assets were copied into the
WellForge workspace. This catalog records independently stated information
value and implementation requirements only.

## Physical snapshot

Full traversal time: 2026-08-26 UTC

| Measure | Value |
| --- | ---: |
| Files | 381,384 |
| Total size | 69.29 GiB |
| UUID-named managed objects | 200,741 |
| Zero-byte operational records | 1,014 |
| Files at least 50 MiB | 366 |
| Compressed containers | 200,263 |
| Latest content timestamp observed | 2026-08-24 01:33:27 UTC |
| Stable content hashes | 83,390 |
| Duplicate-content groups | 55,208 |
| Files belonging to duplicate groups | 353,202 |

The source collection contains a large managed-history area alongside authored
engineering material. Managed objects are retained in physical counts but
suppressed from the default high-value authored-content view.

| Extension family | Files | Size | Default treatment |
| --- | ---: | ---: | --- |
| Compressed `.gz` containers | 200,123 | 34.70 GiB | Managed-history/default-hidden until indexed |
| Executables and libraries | 8,530 | 13.17 GiB | Runtime/dependency inventory |
| Domain `.lvr` files | 471 | 3.20 GiB | Engineering artifact triage |
| Backup `.bak` files | 53 | 2.89 GiB | Data-source triage, default read-only |
| SQL scripts | 5,365 | 2.78 GiB | Schema and data-model reference queue |
| XML documents | 1,978 | 1.68 GiB | Structured engineering/import queue |
| Domain `.dat` files | 219 | 1.53 GiB | Engineering artifact triage |
| Compiled-help references | 21 | 1.20 GiB | Documentation extraction queue |
| Office and PDF documents | 1,020 | 1.55 GiB | Technical-reference queue |
| C# source | 49,176 | 0.30 GiB | Portability and behavior-reference queue |
| JavaScript source | 25,474 | 0.43 GiB | Workflow/UI reference queue |
| Structured workbooks | 270 `.xlsx` plus legacy formats | 0.23 GiB+ | Engineering table/model queue |

## Durable physical-catalog fields

When the corpus stops changing, record each file using:

```text
relative path | parent shard | extension | byte size | UTC modified time |
classification | confidence | UUID flag | zero-byte flag | container flag |
SHA-256 | duplicate-group ID
```

Classification precedence:

1. `$tf` parent: managed history/cache.
2. UUID filename: managed object.
3. `.gz`, `.zip`, `.rar`: compressed container, content unknown until indexed.
4. Workbook extension: structured data.
5. Document extension: document.
6. Script extension: source/workflow reference.
7. Project/domain extension: application or engineering data.
8. Media or operational extension: respective low-priority class.

## High-value engineering content map

### Project and engineering artifacts

- Structured project material covers project/well context, trajectory stations,
  bore geometry, assembly/component hierarchy, static/vibration cases, load
  sweeps, and depth-indexed results.
- At least one project XML example is not strict-parseable, establishing a
  mandatory malformed-input diagnostic and recovery path for import.
- Domain artifact types include project, assembly, database, result, and
  application-data files; each needs a format-specific triage record before
  any future fixture conversion.

### Structured tables and result curves

- 45 legacy workbooks describe component identity, dimensions, material/mass,
  mechanical/load/torque capacity, flow behavior, and power curves.
- 24 newer/macro-enabled workbooks cover units, fluid properties, hydraulic
  scenarios, critical-speed cases, release/report analysis, and computed
  curves.
- One large curve workbook contains roughly 90,000 formula cells across force,
  moment, stress, displacement, clearance/contact, and kinetic-energy output
  families. It is output-shape and unit-validation evidence, not an equation
  source.

### Interchange and references

- A large well-information interchange reference includes 1,179 schemas, 114
  XML examples, 57 transformations, and generated documentation. It is a
  vocabulary and validation reference for independently authored interchange
  contracts.
- Engineering references cover trajectory, assembly/component behavior,
  drilling mechanics, and intervention operations. They are research leads
  requiring current authoritative-source and licensing review before
  implementation.
- Result/report examples support snapshot-report layouts and acceptance-test
  categories, never a source-of-truth data model.

## WellForge capability tags

`trajectory`, `bore-geometry`, `assembly-components`, `materials`,
`dimensional-data`, `load-capacity`, `fluid-properties`, `hydraulics`,
`power-performance`, `static-analysis`, `vibration-analysis`,
`critical-speed`, `curve-results`, `unit-normalization`,
`schema-interchange`, `requirements-traceability`, `report-fixtures`.

## Independently authored WellForge requirements

```text
project -> well -> trajectory/bore sections -> assembly/components
        -> load cases/analyses -> depth-indexed result curves -> report snapshot
```

- Typed quantities must keep canonical units, conversion, precision, and
  dimensional validation at the Rust boundary.
- Reference tables, scenario inputs, calculation outputs, and presentation
  artifacts remain separate.
- Parameter sweeps preserve input provenance and scenario identity.
- Import is schema-first, validates before commit, and has a clear malformed
  input diagnostic path.
- Result storage supports scalar/vector curves, units, comparison, controlled
  downsampling, plotting, and export.
- SQLite is the local authoritative project store; DuckDB and Polars only
  consume approved projections for downstream analytics and reporting.

## Synthetic-fixture candidates

1. A vertical/build/hold trajectory with a malformed counterpart.
2. Two bore segments and a three-component assembly with geometry, material,
   and capacity fields.
3. Four position-by-load static/dynamic scenarios.
4. A unit-conversion matrix and fluid-property table.
5. Small force, moment, stress, displacement, clearance/contact, and energy
   curve families.
6. A schema-valid interchange sample and one deliberately invalid sample.

## Stable physical catalog

The read-only manifest is
`work/fon-scientiae-catalog/catalog-20260826T233926Z.jsonl`, with a 106.5 MB
summary companion file. It covers all 381,384 files with relative path,
extension, byte size, modified timestamp, and SHA-256. The pre- and post-hash
metadata fingerprints are both
`9d553e109ca36f1aff37902265fc0ea2529fa6f4c4e832a9d8dc602bbaae2f33`, and the
run completed with zero failed reads.

`tools/catalog-fon-scientiae.ps1` remains the repeatable read-only catalog
tool. Container indexes are a separate follow-on step.
