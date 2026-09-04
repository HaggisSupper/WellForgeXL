# Local authority migrations

`sqlite/0001_local_authority.sql` and `sqlite/0002_revision_event_heads.sql` are
the active migrations for the embedded SQLite authority store. `LocalStore`
applies them inside an immediate transaction and records the result in
`schema_migration`; callers do not run migration SQL or provide database
connection settings. Every applied row is checked against its expected ordinal
and SHA-256 checksum. Reordered, unknown, or tampered history prevents startup,
and any failed migration batch is rolled back.

The store enables foreign keys, WAL mode, and a five-second busy timeout when it
opens. It contains project metadata; uniquely identified immutable revision
events with repeatable content SHA-256 and linear parent/head linkage;
immutable calculation receipts; immutable replay journal events; and durable
sync envelopes. SQLite triggers reject updates and deletes from every immutable
authority table. Hashes at this boundary use the canonical `sha256:` prefix
with 64 lower-case hexadecimal characters; revision and receipt hashes must
also match their supplied bytes.

Production desktop startup opens the store as a durable file in the operating
system application-data directory. No in-memory constructor is present in a
production build. Each file connection independently verifies WAL mode and
applies a five-second busy timeout.

DuckDB and Polars receive approved projections only. They remain downstream
analytics and transformation tools, never an authority for projects, revisions,
receipts, journal history, or synchronization state.

## Historical reference

The top-level `0001_foundation.sql` and `0002_stage_contract_alignment.sql`
files are inactive historical PostgreSQL design references. They are not read
or executed by `LocalStore`, and PostgreSQL is not a runtime dependency of the
desktop application.
