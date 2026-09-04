-- Active local authority schema for the embedded SQLite store.
-- This file is applied by LocalStore inside one BEGIN IMMEDIATE transaction.

CREATE TABLE project (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE project_revision (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES project(id),
    parent_revision_id TEXT REFERENCES project_revision(id),
    content BLOB NOT NULL,
    content_sha256 TEXT NOT NULL,
    created_at TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    CHECK (
        length(content_sha256) = 71
        AND substr(content_sha256, 1, 7) = 'sha256:'
        AND substr(content_sha256, 8) NOT GLOB '*[^0-9a-f]*'
    ),
    UNIQUE (project_id, content_sha256)
) STRICT;

CREATE INDEX project_revision_project_created_idx
    ON project_revision (project_id, created_at);

CREATE TRIGGER project_revision_parent_must_match_project
BEFORE INSERT ON project_revision
WHEN NEW.parent_revision_id IS NOT NULL
BEGIN
    SELECT CASE WHEN NEW.parent_revision_id = NEW.id THEN RAISE(ABORT, 'self parent revision') END;
    SELECT CASE WHEN (SELECT project_id FROM project_revision WHERE id = NEW.parent_revision_id) != NEW.project_id THEN RAISE(ABORT, 'parent project mismatch') END;
END;

CREATE TABLE calculation_receipt (
    id TEXT PRIMARY KEY NOT NULL,
    project_revision_id TEXT NOT NULL REFERENCES project_revision(id),
    engine_version TEXT NOT NULL,
    content BLOB NOT NULL,
    content_sha256 TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    CHECK (
        length(content_sha256) = 71
        AND substr(content_sha256, 1, 7) = 'sha256:'
        AND substr(content_sha256, 8) NOT GLOB '*[^0-9a-f]*'
    ),
    UNIQUE (project_revision_id, content_sha256)
) STRICT;

CREATE TABLE replay_journal_event (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES project(id),
    project_revision_id TEXT REFERENCES project_revision(id),
    sequence INTEGER NOT NULL CHECK (sequence > 0),
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    UNIQUE (project_id, sequence)
) STRICT;

CREATE TABLE sync_envelope (
    idempotency_key TEXT PRIMARY KEY NOT NULL,
    artifact_hash TEXT NOT NULL,
    parent_revision_id TEXT REFERENCES project_revision(id),
    actor_id TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    operation TEXT NOT NULL CHECK (operation IN ('create', 'upsert', 'delete')),
    CHECK (
        length(artifact_hash) = 71
        AND substr(artifact_hash, 1, 7) = 'sha256:'
        AND substr(artifact_hash, 8) NOT GLOB '*[^0-9a-f]*'
    )
) STRICT;

CREATE TRIGGER replay_event_revision_must_match_project
BEFORE INSERT ON replay_journal_event
WHEN NEW.project_revision_id IS NOT NULL
BEGIN
    SELECT CASE WHEN (SELECT project_id FROM project_revision WHERE id = NEW.project_revision_id) != NEW.project_id THEN RAISE(ABORT, 'replay project mismatch') END;
END;

CREATE TRIGGER project_revision_is_append_only_update
BEFORE UPDATE ON project_revision
BEGIN
    SELECT RAISE(ABORT, 'project_revision is append-only');
END;

CREATE TRIGGER project_revision_is_append_only_delete
BEFORE DELETE ON project_revision
BEGIN
    SELECT RAISE(ABORT, 'project_revision is append-only');
END;

CREATE TRIGGER calculation_receipt_is_append_only_update
BEFORE UPDATE ON calculation_receipt
BEGIN
    SELECT RAISE(ABORT, 'calculation_receipt is append-only');
END;

CREATE TRIGGER calculation_receipt_is_append_only_delete
BEFORE DELETE ON calculation_receipt
BEGIN
    SELECT RAISE(ABORT, 'calculation_receipt is append-only');
END;

CREATE TRIGGER replay_journal_event_is_append_only_update
BEFORE UPDATE ON replay_journal_event
BEGIN
    SELECT RAISE(ABORT, 'replay_journal_event is append-only');
END;

CREATE TRIGGER replay_journal_event_is_append_only_delete
BEFORE DELETE ON replay_journal_event
BEGIN
    SELECT RAISE(ABORT, 'replay_journal_event is append-only');
END;

CREATE TRIGGER sync_envelope_is_append_only_update
BEFORE UPDATE ON sync_envelope
BEGIN
    SELECT RAISE(ABORT, 'sync_envelope is append-only');
END;

CREATE TRIGGER sync_envelope_is_append_only_delete
BEFORE DELETE ON sync_envelope
BEGIN
    SELECT RAISE(ABORT, 'sync_envelope is append-only');
END;
