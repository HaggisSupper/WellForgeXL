CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS artifacts (
    id TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL UNIQUE CHECK(length(sha256) = 64),
    display_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    family TEXT NOT NULL,
    size_bytes INTEGER NOT NULL CHECK(size_bytes >= 0),
    modified_at TEXT,
    extraction_backend TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'ready',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS artifact_aliases (
    artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    source_uri TEXT NOT NULL,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    PRIMARY KEY (artifact_id, source_uri)
);

CREATE TABLE IF NOT EXISTS concepts (
    id TEXT PRIMARY KEY,
    concept_path TEXT NOT NULL UNIQUE,
    concept_type TEXT NOT NULL,
    title TEXT NOT NULL,
    domain TEXT NOT NULL,
    body TEXT NOT NULL,
    frontmatter_json TEXT NOT NULL DEFAULT '{}',
    provenance_state TEXT NOT NULL DEFAULT 'candidate',
    trust_state TEXT NOT NULL DEFAULT 'unverified',
    lifecycle_state TEXT NOT NULL DEFAULT 'active',
    source_confidence REAL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS concept_edges (
    id TEXT PRIMARY KEY,
    source_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    target_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,
    provenance_json TEXT NOT NULL DEFAULT '{}',
    source_artifact_id TEXT REFERENCES artifacts(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    UNIQUE(source_concept_id, target_concept_id, edge_type, source_artifact_id)
);

CREATE TABLE IF NOT EXISTS chunks (
    id TEXT PRIMARY KEY,
    concept_id TEXT REFERENCES concepts(id) ON DELETE SET NULL,
    artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK(ordinal >= 0),
    section_locator TEXT NOT NULL DEFAULT '',
    source_locator TEXT NOT NULL DEFAULT '',
    text TEXT NOT NULL,
    content_hash TEXT NOT NULL CHECK(length(content_hash) = 64),
    token_estimate INTEGER NOT NULL DEFAULT 0 CHECK(token_estimate >= 0),
    embedding_state TEXT NOT NULL DEFAULT 'pending',
    embedding_model TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(artifact_id, ordinal, content_hash)
);

CREATE TABLE IF NOT EXISTS citations (
    id TEXT PRIMARY KEY,
    concept_id TEXT REFERENCES concepts(id) ON DELETE CASCADE,
    chunk_id TEXT REFERENCES chunks(id) ON DELETE CASCADE,
    artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    locator_type TEXT NOT NULL,
    locator TEXT NOT NULL,
    label TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ingestion_runs (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    software_version TEXT NOT NULL,
    config_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    warning_count INTEGER NOT NULL DEFAULT 0,
    model_identities_json TEXT NOT NULL DEFAULT '{}',
    message TEXT
);

CREATE TABLE IF NOT EXISTS index_state (
    index_name TEXT PRIMARY KEY,
    corpus_revision INTEGER NOT NULL DEFAULT 0,
    index_revision INTEGER NOT NULL DEFAULT 0,
    model_identity TEXT,
    updated_at TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS concepts_fts USING fts5(
    concept_id UNINDEXED,
    title,
    body,
    domain,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    chunk_id UNINDEXED,
    text,
    tokenize = 'unicode61 remove_diacritics 2'
);

INSERT OR IGNORE INTO schema_migrations(version, applied_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
