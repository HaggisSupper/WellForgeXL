CREATE TABLE IF NOT EXISTS inferred_causality (
    id TEXT PRIMARY KEY,
    episode_key TEXT NOT NULL UNIQUE,
    start_at TEXT NOT NULL,
    end_at TEXT NOT NULL,
    actor TEXT NOT NULL CHECK(actor IN (
        'manual_confirmed',
        'manual_inferred',
        'automation_command',
        'automation_inferred',
        'unknown'
    )),
    claim_level TEXT NOT NULL CHECK(claim_level IN (
        'observed_sequence',
        'likely_reaction',
        'causal_claim'
    )),
    situation_json TEXT NOT NULL,
    intervention_json TEXT NOT NULL,
    observed_response_json TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK(outcome IN (
        'effective',
        'partially_effective',
        'ineffective',
        'adverse',
        'inconclusive'
    )),
    confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
    inference_method TEXT NOT NULL,
    canonical_state_ref TEXT,
    source_artifact_id TEXT REFERENCES artifacts(id) ON DELETE SET NULL,
    authority TEXT NOT NULL DEFAULT 'Inferred_Causality'
        CHECK(authority = 'Inferred_Causality'),
    is_canonical INTEGER NOT NULL DEFAULT 0
        CHECK(is_canonical = 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK(end_at >= start_at)
);

CREATE INDEX IF NOT EXISTS idx_inferred_causality_time
ON inferred_causality(start_at, end_at);

CREATE INDEX IF NOT EXISTS idx_inferred_causality_actor_outcome
ON inferred_causality(actor, outcome);

CREATE INDEX IF NOT EXISTS idx_inferred_causality_source_artifact
ON inferred_causality(source_artifact_id);

INSERT OR IGNORE INTO schema_migrations(version, applied_at)
VALUES (2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
