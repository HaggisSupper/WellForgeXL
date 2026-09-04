-- HISTORICAL / INACTIVE: PostgreSQL staging alignment retained for reference.
-- SQLite migrations under sqlite/ are the active local authority path.
--
-- Align the non-authoritative staging area with the Rust import contracts.
--
-- This migration is append-only. Corrections are recorded as later staged
-- records or validation results; the original evidence is never overwritten.
-- This historical model previously treated PostgreSQL as the authority.

BEGIN;

-- `id` remains the internal UUID boundary. `source_batch_id` is the external
-- batch identifier carried by ImportBatchMetadata.
ALTER TABLE stage.import_batch
    ADD COLUMN source_batch_id text,
    ADD COLUMN source_location text,
    ADD COLUMN extracted_at timestamptz,
    ADD COLUMN operator_id text;

UPDATE stage.import_batch
   SET source_batch_id = id::text,
       source_location = 'legacy://import-batch/' || id::text,
       extracted_at = received_at,
       operator_id = received_by::text
 WHERE source_batch_id IS NULL;

ALTER TABLE stage.import_batch
    ALTER COLUMN source_batch_id SET NOT NULL,
    ALTER COLUMN source_location SET NOT NULL,
    ALTER COLUMN extracted_at SET NOT NULL,
    ALTER COLUMN operator_id SET NOT NULL,
    ADD CONSTRAINT import_batch_source_batch_id_check
        CHECK (btrim(source_batch_id) <> ''),
    ADD CONSTRAINT import_batch_source_location_check
        CHECK (btrim(source_location) <> ''),
    ADD CONSTRAINT import_batch_operator_id_check
        CHECK (btrim(operator_id) <> ''),
    ADD CONSTRAINT import_batch_source_batch_identity_key
        UNIQUE (organization_id, source_system, source_batch_id),
    ADD CONSTRAINT import_batch_id_source_system_key
        UNIQUE (id, source_system);

-- Keep an internal UUID primary key while storing the complete source-side
-- identity and provenance needed by RawSourceRecord.
ALTER TABLE stage.source_record
    RENAME COLUMN source_table TO source_entity_kind;
ALTER TABLE stage.source_record
    RENAME COLUMN source_key TO source_record_key;
ALTER TABLE stage.source_record
    RENAME COLUMN source_hash TO source_checksum;
ALTER TABLE stage.source_record
    RENAME COLUMN raw_document TO payload;

ALTER TABLE stage.source_record
    ADD COLUMN source_system text,
    ADD COLUMN source_entity_id text,
    ADD COLUMN source_location text,
    ADD COLUMN extracted_at timestamptz;

UPDATE stage.source_record AS record
   SET source_system = batch.source_system,
       source_entity_id = record.source_record_key,
       source_location = batch.source_location,
       extracted_at = batch.extracted_at
  FROM stage.import_batch AS batch
 WHERE batch.id = record.import_batch_id
   AND record.source_system IS NULL;

ALTER TABLE stage.source_record
    ALTER COLUMN source_system SET NOT NULL,
    ALTER COLUMN source_entity_id SET NOT NULL,
    ALTER COLUMN source_location SET NOT NULL,
    ALTER COLUMN extracted_at SET NOT NULL,
    ADD CONSTRAINT source_record_source_system_check
        CHECK (btrim(source_system) <> ''),
    ADD CONSTRAINT source_record_source_entity_kind_check
        CHECK (btrim(source_entity_kind) <> ''),
    ADD CONSTRAINT source_record_source_entity_id_check
        CHECK (btrim(source_entity_id) <> ''),
    ADD CONSTRAINT source_record_source_location_check
        CHECK (btrim(source_location) <> ''),
    ADD CONSTRAINT source_record_batch_source_system_fk
        FOREIGN KEY (import_batch_id, source_system)
        REFERENCES stage.import_batch (id, source_system) ON DELETE RESTRICT,
    ADD CONSTRAINT source_record_batch_record_key
        UNIQUE (import_batch_id, source_record_key),
    ADD CONSTRAINT source_record_id_batch_key
        UNIQUE (id, import_batch_id);

-- Each mapping retains the immutable source checksum which establishes the
-- exact source-side identity used during reconciliation.
ALTER TABLE stage.source_identity_map
    RENAME COLUMN source_table TO source_entity_kind;
ALTER TABLE stage.source_identity_map
    RENAME COLUMN source_key TO source_entity_id;

ALTER TABLE stage.source_identity_map
    ADD COLUMN source_system text,
    ADD COLUMN source_checksum text;

UPDATE stage.source_identity_map AS mapping
   SET source_system = batch.source_system,
       source_checksum = batch.source_checksum
  FROM stage.import_batch AS batch
 WHERE batch.id = mapping.import_batch_id
   AND mapping.source_system IS NULL;

ALTER TABLE stage.source_identity_map
    ALTER COLUMN source_system SET NOT NULL,
    ALTER COLUMN source_checksum SET NOT NULL,
    ADD CONSTRAINT source_identity_map_source_system_check
        CHECK (btrim(source_system) <> ''),
    ADD CONSTRAINT source_identity_map_source_entity_kind_check
        CHECK (btrim(source_entity_kind) <> ''),
    ADD CONSTRAINT source_identity_map_source_entity_id_check
        CHECK (btrim(source_entity_id) <> ''),
    ADD CONSTRAINT source_identity_map_source_checksum_check
        CHECK (source_checksum ~ '^sha256:[0-9a-f]{64}$'),
    ADD CONSTRAINT source_identity_map_batch_source_system_fk
        FOREIGN KEY (import_batch_id, source_system)
        REFERENCES stage.import_batch (id, source_system) ON DELETE RESTRICT,
    ADD CONSTRAINT source_identity_map_identity_checksum_key
        UNIQUE (import_batch_id, source_system, source_entity_kind, source_entity_id, source_checksum);

-- The original table represented a finding per row. Preserve it as a factual
-- finding ledger, then add the contract-level validation summary used by
-- StagedValidationResult.
ALTER TABLE stage.validation_result RENAME TO validation_finding;

CREATE TABLE stage.validation_result (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    import_batch_id uuid NOT NULL REFERENCES stage.import_batch(id) ON DELETE RESTRICT,
    source_record_id uuid NOT NULL,
    disposition text NOT NULL
        CHECK (disposition IN ('accepted', 'rejected', 'needs_review')),
    rule_set_version text NOT NULL CHECK (btrim(rule_set_version) <> ''),
    validated_at timestamptz NOT NULL,
    findings jsonb NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(findings) = 'array'),
    recorded_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (source_record_id, import_batch_id)
        REFERENCES stage.source_record (id, import_batch_id) ON DELETE RESTRICT,
    CHECK (disposition = 'accepted' OR jsonb_array_length(findings) > 0),
    UNIQUE (source_record_id, rule_set_version, validated_at)
);

-- Convert any pre-existing per-rule results into one auditable legacy summary
-- per source record. New writes use the contract-level table directly.
INSERT INTO stage.validation_result (
    import_batch_id,
    source_record_id,
    disposition,
    rule_set_version,
    validated_at,
    findings,
    recorded_at
)
SELECT
    record.import_batch_id,
    finding.source_record_id,
    CASE
        WHEN bool_or(finding.severity = 'error') THEN 'rejected'
        WHEN bool_or(finding.severity = 'warning') THEN 'needs_review'
        ELSE 'accepted'
    END,
    'legacy-unversioned',
    min(finding.created_at),
    jsonb_agg(
        jsonb_build_object('code', finding.rule_code, 'message', finding.message)
        ORDER BY finding.created_at, finding.id
    ),
    min(finding.created_at)
  FROM stage.validation_finding AS finding
  JOIN stage.source_record AS record ON record.id = finding.source_record_id
 GROUP BY record.import_batch_id, finding.source_record_id;

-- The policies that were attached to the original table moved with the
-- finding ledger. Apply equivalent tenant and actor controls to the summary.
ALTER TABLE stage.validation_result ENABLE ROW LEVEL SECURITY;
ALTER TABLE stage.validation_result FORCE ROW LEVEL SECURITY;

CREATE POLICY stage_validation_summary_tenant_scope ON stage.validation_result
    USING (
        EXISTS (
            SELECT 1
              FROM stage.import_batch AS batch
             WHERE batch.id = import_batch_id
               AND batch.organization_id = iam.current_organization_id()
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1
              FROM stage.import_batch AS batch
             WHERE batch.id = import_batch_id
               AND batch.organization_id = iam.current_organization_id()
        )
    );

CREATE POLICY stage_validation_summary_actor_authorization ON stage.validation_result AS RESTRICTIVE
    USING (
        EXISTS (
            SELECT 1
              FROM stage.import_batch AS batch
             WHERE batch.id = import_batch_id
               AND iam.actor_can(batch.organization_id, 'administration', NULL, 'read')
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1
              FROM stage.import_batch AS batch
             WHERE batch.id = import_batch_id
               AND iam.actor_can(batch.organization_id, 'administration', NULL, 'write')
        )
    );

-- The importer can append evidence, but cannot change or erase it. The schema
-- trigger covers privileged mistakes in addition to these least-privilege grants.
REVOKE UPDATE, DELETE ON stage.import_batch, stage.source_record,
    stage.source_identity_map, stage.validation_finding, stage.validation_result
    FROM wellforge_importer;
GRANT SELECT, INSERT ON stage.validation_finding, stage.validation_result TO wellforge_importer;
GRANT ALL PRIVILEGES ON stage.validation_finding, stage.validation_result TO wellforge_migrator;

CREATE TRIGGER stage_import_batch_append_only
    BEFORE UPDATE OR DELETE ON stage.import_batch
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER stage_source_record_append_only
    BEFORE UPDATE OR DELETE ON stage.source_record
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER stage_source_identity_map_append_only
    BEFORE UPDATE OR DELETE ON stage.source_identity_map
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER stage_validation_finding_append_only
    BEFORE UPDATE OR DELETE ON stage.validation_finding
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER stage_validation_result_append_only
    BEFORE UPDATE OR DELETE ON stage.validation_result
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();

CREATE INDEX stage_source_record_identity_lookup
    ON stage.source_record (import_batch_id, source_entity_kind, source_entity_id);
CREATE INDEX stage_source_identity_map_lookup
    ON stage.source_identity_map (source_system, source_entity_kind, source_entity_id);
CREATE INDEX stage_validation_result_batch_disposition
    ON stage.validation_result (import_batch_id, disposition, validated_at DESC);

COMMIT;
