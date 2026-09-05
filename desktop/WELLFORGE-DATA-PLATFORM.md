# WellForge Data Platform

## Decision

SQLite is the local transactional authority for the desktop application. It is
embedded through `rusqlite` with a bundled SQLite engine, so it requires no
service, container runtime, or network database. DuckDB and Polars remain the
downstream analytical and transformation layer. Portable XML documents remain
first-class engineering files.

## Responsibilities

| Concern | Authoritative location |
| --- | --- |
| Engineering project metadata | Local SQLite `project` table |
| Engineering project document | Versioned XML file plus immutable SQLite revision metadata |
| Project revision lineage | Local SQLite `project_revision` table |
| Calculated results | Immutable SQLite calculation receipt tied to a project revision |
| Replay/change history | Immutable SQLite replay journal events |
| Synchronization intent | Durable SQLite sync envelopes keyed by idempotency key |
| Analytical/reporting datasets | DuckDB queries and Polars transformations over approved projections |

## Local authority boundary

The Rust storage crate owns a private SQLite connection and exposes typed write
methods only. It enables foreign keys, WAL mode, and a five-second busy timeout
on open. Migrations execute inside an immediate transaction and are recorded in
`schema_migration`. Startup rejects checksum changes, ordinal changes, reordered
history, and unknown applied migrations; a failed migration batch rolls back in
full. Revisions and calculation receipts validate their content against a
canonical SHA-256 digest before persistence. Typed core receipts must identify
the exact stored project revision and matching content digest. Foreign keys,
unique constraints, and append-only triggers enforce lineage and prevent
updates or deletes for authority records.

The desktop opens `local-authority.sqlite3` beneath the operating system's
application-data directory during startup. Production builds expose no
in-memory store. `save_project` persists the selected file bytes as an
immutable revision event: an unchanged head save is idempotent, while changed
or reverted bytes append a new unique event whose parent must be the current
head. Head validation and advancement occur in one immediate transaction, so
stale or concurrent sibling appends cannot create branches.

Project metadata is intentionally separate from immutable authority records so
future metadata edits do not rewrite a revision, receipt, replay event, or sync
envelope. A correction is represented by a later immutable record.

## Analytics boundary

DuckDB and Polars consume approved, versioned projections from local authority
records and engineering files. Their outputs must be validated before a new
authority record is written; they cannot directly modify project history,
calculation receipts, replay events, or synchronization envelopes.

## Historical PostgreSQL material

The existing PostgreSQL entity-model document and the top-level SQL migrations
are inactive historical references. They are not part of the active migration
path, public storage API, or desktop runtime. No Docker or PostgreSQL runtime
dependency is used by the local authority slice.
