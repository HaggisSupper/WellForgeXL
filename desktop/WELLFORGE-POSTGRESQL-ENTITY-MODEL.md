# WellForge PostgreSQL Entity Model

> Historical reference only. SQLite is the active local authority; this model
> is not part of the desktop runtime or active migration path.

## Historical modeling note

This document preserves a relational domain-model sketch for comparison only.
SQLite is the authoritative transactional store in the active desktop design;
DuckDB and Polars provide fast, non-authoritative analytics, profiling,
reconciliation, and batch transformation. Versioned XML remains the portable
engineering artifact.

The model replaces copied wide records for publishing, snapshots, and audit
with stable identities, immutable revisions, frozen release membership, and
append-only events.

## Domain map

```text
organization ──< membership >── user_account
      │
      ├──< project ──< project_revision ──< calculation_run ──< calculation_artifact
      │       └──< project_access
      │
      ├──< catalog_item ──< catalog_item_revision ──< catalog_item_attribute
      │          │                     │
      │          │                     └──< catalog_item_relation >── catalog_item_revision
      │          │
      │          └── manufacturer
      │
      └──< audit_event

catalog_family ──< catalog_type ──< catalog_category ──< catalog_item_revision
attribute_definition ──< catalog_item_attribute
dimension ──< unit
unit_system ──< unit_system_preference >── unit

catalog_release ──< catalog_release_item >── catalog_item_revision
change_request ──> catalog_item_revision

import_batch ──< source_record ──< source_identity_map
```

## Schemas and tables

| Schema | Tables | Purpose |
| --- | --- | --- |
| `iam` | `organization`, `user_account`, `membership`, `role`, `scope_grant` | Identity, tenancy, roles, and access scope |
| `ref` | `dimension`, `unit`, `unit_system`, `unit_system_preference`, `manufacturer`, `material`, `connection` | Controlled reference data and canonical units |
| `catalog` | `family`, `type`, `category`, `attribute_definition`, `item`, `item_revision`, `item_attribute`, `item_relation`, `change_request`, `release`, `release_item` | Equipment catalog, workflow, and immutable publication |
| `project` | `project`, `project_revision`, `project_access`, `calculation_run`, `calculation_artifact` | Project lineage, XML revisions, and reproducible results |
| `audit` | `event` | Append-only business/security events |
| `stage` | `import_batch`, `source_record`, `source_identity_map`, `validation_result` | Restricted raw import, provenance, and migration evidence |

## Core table form

| Table | Identity and relationships | Required behavior |
| --- | --- | --- |
| `catalog.item` | UUID identity; belongs to organization and optional manufacturer | Stable business identity; never overwritten by a later engineering revision |
| `catalog.item_revision` | UUID; `item_id`, monotonically increasing `revision_no`, category | Immutable typed record, source checksum, author, timestamp, lifecycle state; unique `(item_id, revision_no)` |
| `catalog.item_attribute` | Revision + attribute definition + ordinal | Typed value with exclusive numeric/text/boolean/time/JSON value columns; numeric values in canonical unit |
| `catalog.item_relation` | Source revision, target revision, relation kind, sibling position | Explicit compatible/assembly relationships; validate tree cycles and unique sibling order |
| `catalog.release` | UUID, code, status, approval/publication data | Immutable release record with explicit state transitions |
| `catalog.release_item` | Release + item revision | Frozen release snapshot; later changes require a new revision and release membership |
| `project.project_revision` | Project + revision number, content hash, parent revision | Immutable XML artifact reference; unique project revision and content hash |
| `project.calculation_run` | Project revision + algorithm ID/version + input checksum | Receipt for deterministic calculation results and provenance |
| `audit.event` | UUID, organization, actor, aggregate type/ID, time, correlation ID | Append-only event; retain hashes/minimal structured details instead of copied wide rows |
| `stage.source_record` | Import batch + source system/key/hash | Raw input and validation result; no automatic promotion |
| `stage.source_identity_map` | Source system/table/key → target UUID | Preserves migration provenance without retaining old identifiers as primary keys |

## Modernization rules

- Use UUID primary keys and retain original identifiers only in staging/provenance.
- Use `timestamptz`, explicit lifecycle checks, foreign keys, and business-key
  uniqueness. Do not infer a relationship from a display name.
- Store dimensional catalog values as `numeric(p,s)` in canonical units.
  Preserve source unit/value only as provenance when needed.
- Use `jsonb` only for genuinely sparse, schema-variable attributes; keep
  searchable/calculation fields typed.
- Make revisions, release membership, calculation runs, and audit events
  immutable. Corrections create a later revision/event.
- Validate component trees with recursive queries and constraints rather than
  procedural cursor walks.
- Approve a change, create its revision, update release membership, and emit
  the audit event in one PostgreSQL transaction.

## Performance plan

Start with ordinary B-tree indexes on known query paths:

```text
catalog.item                 UNIQUE (organization_id, manufacturer_id, part_number)
catalog.item_revision        UNIQUE (item_id, revision_no)
catalog.item_revision        (category_id, lifecycle_state, created_at DESC)
catalog.item_attribute       UNIQUE (item_revision_id, attribute_definition_id, ordinal)
catalog.item_attribute       (attribute_definition_id, value_number, item_revision_id)
catalog.release_item         PRIMARY KEY (release_id, item_revision_id)
project.project_revision     UNIQUE (project_id, revision_no), UNIQUE (project_id, content_hash)
project.calculation_run      (project_revision_id, algorithm_id, algorithm_version, created_at DESC)
audit.event                  (organization_id, occurred_at DESC)
audit.event                  (aggregate_type, aggregate_id, occurred_at DESC)
```

Partition only the append-only `audit.event` table by month, and only after
measured volume warrants it. Use a BRIN index on its timestamp for large,
time-ordered histories. Do not partition catalog/project/revision tables at
the outset: their integrity and lookup patterns favor simpler B-tree layouts.

## Security and access

- The Tauri client never receives database credentials or connects directly.
- Rust service/storage code uses encrypted connections, SCRAM authentication,
  managed secrets, and distinct migration/runtime/read-only roles.
- PostgreSQL row-level security enforces organization-scoped data access from
  transaction-local actor/organization context.
- Global reference data is read-only and separately permissioned.
- The staging schema is accessible only to import roles; rejected data remains
  quarantined until explicitly corrected and reprocessed.

## DuckDB and Polars boundary

Polars performs repeatable import cleansing, type/unit normalization,
reconciliation, and data-quality checks before promotion. DuckDB supports
read-only analysis, profiling, large reconciliation queries, and
report-oriented aggregates. Both must stamp outputs with source batch/release
identity and “as-of” time.

Neither tool may publish a catalog release, revise a project, authorize a
user, or write an audit decision. Those operations always occur through the
Rust-controlled PostgreSQL transaction boundary.

## Migration sequence

1. Checksum every source extract and map each source field to a target field.
2. Land raw rows/documents in `stage` without coercion.
3. Use Polars to produce accepted and rejected normalized sets, with a
   versioned transformation record.
4. Resolve identities through `source_identity_map`, then promote controlled
   values before catalog/project data.
5. Create immutable revisions rather than updating imported records.
6. Reconstruct historic releases through `release` and `release_item`.
7. Compare counts, hashes, relationships, units, and representative exports.
8. Enable read-only parallel operation before controlled write enablement.
