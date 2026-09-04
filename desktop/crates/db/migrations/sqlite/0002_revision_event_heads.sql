-- Revision IDs identify immutable save events, independently from repeatable content hashes.
-- The current head is advanced by the same write transaction that appends each event.

PRAGMA defer_foreign_keys = ON;

DROP TRIGGER project_revision_parent_must_match_project;
DROP TRIGGER replay_event_revision_must_match_project;
DROP TRIGGER project_revision_is_append_only_update;
DROP TRIGGER project_revision_is_append_only_delete;
DROP TRIGGER calculation_receipt_is_append_only_update;
DROP TRIGGER calculation_receipt_is_append_only_delete;
DROP TRIGGER replay_journal_event_is_append_only_update;
DROP TRIGGER replay_journal_event_is_append_only_delete;
DROP TRIGGER sync_envelope_is_append_only_update;
DROP TRIGGER sync_envelope_is_append_only_delete;
DROP INDEX project_revision_project_created_idx;

ALTER TABLE calculation_receipt RENAME TO calculation_receipt_0001;
ALTER TABLE replay_journal_event RENAME TO replay_journal_event_0001;
ALTER TABLE sync_envelope RENAME TO sync_envelope_0001;
ALTER TABLE project_revision RENAME TO project_revision_0001;

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
    )
) STRICT;

INSERT INTO project_revision (
    id, project_id, parent_revision_id, content, content_sha256, created_at, actor_id
)
SELECT id, project_id, parent_revision_id, content, content_sha256, created_at, actor_id
FROM project_revision_0001
ORDER BY rowid;

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

INSERT INTO calculation_receipt
SELECT * FROM calculation_receipt_0001;

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

INSERT INTO replay_journal_event
SELECT * FROM replay_journal_event_0001;

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

INSERT INTO sync_envelope
SELECT * FROM sync_envelope_0001;

DROP TABLE calculation_receipt_0001;
DROP TABLE replay_journal_event_0001;
DROP TABLE sync_envelope_0001;
DROP TABLE project_revision_0001;

CREATE INDEX project_revision_project_created_idx
    ON project_revision (project_id, created_at);

CREATE TABLE project_head (
    project_id TEXT PRIMARY KEY NOT NULL REFERENCES project(id),
    revision_id TEXT NOT NULL UNIQUE REFERENCES project_revision(id)
) STRICT;

INSERT INTO project_head (project_id, revision_id)
SELECT candidate.project_id, candidate.id
FROM project_revision AS candidate
WHERE candidate.rowid = (
    SELECT MAX(head.rowid)
    FROM project_revision AS head
    WHERE head.project_id = candidate.project_id
);

CREATE TRIGGER project_revision_parent_must_match_project
BEFORE INSERT ON project_revision
WHEN NEW.parent_revision_id IS NOT NULL
BEGIN
    SELECT CASE WHEN NEW.parent_revision_id = NEW.id THEN RAISE(ABORT, 'self parent revision') END;
    SELECT CASE WHEN (SELECT project_id FROM project_revision WHERE id = NEW.parent_revision_id) != NEW.project_id THEN RAISE(ABORT, 'parent project mismatch') END;
END;

CREATE TRIGGER project_revision_must_extend_current_head
BEFORE INSERT ON project_revision
BEGIN
    SELECT CASE
        WHEN EXISTS(SELECT 1 FROM project_head WHERE project_id = NEW.project_id)
             AND (
                 NEW.parent_revision_id IS NULL
                 OR NEW.parent_revision_id != (
                     SELECT revision_id FROM project_head WHERE project_id = NEW.project_id
                 )
             )
        THEN RAISE(ABORT, 'stale parent revision')
    END;
    SELECT CASE
        WHEN NOT EXISTS(SELECT 1 FROM project_head WHERE project_id = NEW.project_id)
             AND NEW.parent_revision_id IS NOT NULL
        THEN RAISE(ABORT, 'first revision must be a root')
    END;
END;

CREATE TRIGGER project_revision_advances_head
AFTER INSERT ON project_revision
BEGIN
    INSERT INTO project_head (project_id, revision_id)
    VALUES (NEW.project_id, NEW.id)
    ON CONFLICT(project_id) DO UPDATE SET revision_id = excluded.revision_id;
END;

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
