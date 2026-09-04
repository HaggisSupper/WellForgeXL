-- HISTORICAL / INACTIVE: PostgreSQL foundation schema retained for reference.
-- SQLite migrations under sqlite/ are the active local authority path.
--
-- This historical model previously treated PostgreSQL as the authority. DuckDB and Polars may
-- read approved extracts for analytics and transformation, but they must not
-- publish releases, alter revisions, or write audit decisions.
--
-- This is an initial bootstrap migration: run it once through the controlled
-- migration runner, using a schema-owner role. Do not apply it from a desktop
-- client or a runtime application role.

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE SCHEMA IF NOT EXISTS iam;
CREATE SCHEMA IF NOT EXISTS ref;
CREATE SCHEMA IF NOT EXISTS catalog;
CREATE SCHEMA IF NOT EXISTS project;
CREATE SCHEMA IF NOT EXISTS audit;
CREATE SCHEMA IF NOT EXISTS stage;

-- These group roles own no credentials. Deployment automation grants login
-- principals membership in exactly one of them.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wellforge_migrator') THEN
        CREATE ROLE wellforge_migrator NOLOGIN NOINHERIT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wellforge_runtime') THEN
        CREATE ROLE wellforge_runtime NOLOGIN NOINHERIT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wellforge_reader') THEN
        CREATE ROLE wellforge_reader NOLOGIN NOINHERIT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wellforge_importer') THEN
        CREATE ROLE wellforge_importer NOLOGIN NOINHERIT;
    END IF;
END;
$$;

CREATE TABLE iam.organization (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code text NOT NULL UNIQUE CHECK (code ~ '^[a-z0-9][a-z0-9-]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    lifecycle_state text NOT NULL DEFAULT 'active'
        CHECK (lifecycle_state IN ('active', 'suspended', 'retired')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE iam.user_account (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject text NOT NULL UNIQUE CHECK (btrim(subject) <> ''),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    email text,
    lifecycle_state text NOT NULL DEFAULT 'active'
        CHECK (lifecycle_state IN ('active', 'disabled')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (email IS NULL OR email = lower(email))
);

CREATE TABLE iam.role (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid REFERENCES iam.organization(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[a-z][a-z0-9_]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    is_system boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE NULLS NOT DISTINCT (organization_id, code),
    CHECK ((is_system AND organization_id IS NULL) OR (NOT is_system AND organization_id IS NOT NULL))
);

CREATE TABLE iam.membership (
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    user_id uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    role_id uuid NOT NULL REFERENCES iam.role(id) ON DELETE RESTRICT,
    lifecycle_state text NOT NULL DEFAULT 'active'
        CHECK (lifecycle_state IN ('active', 'suspended', 'revoked')),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (organization_id, user_id, role_id)
);

CREATE TABLE iam.role_permission (
    role_id uuid NOT NULL REFERENCES iam.role(id) ON DELETE RESTRICT,
    scope_type text NOT NULL CHECK (scope_type IN ('catalog', 'project', 'release', 'reference', 'administration')),
    permission text NOT NULL CHECK (permission IN ('read', 'write', 'approve', 'admin')),
    PRIMARY KEY (role_id, scope_type, permission)
);

CREATE TABLE iam.scope_grant (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    user_id uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    scope_type text NOT NULL CHECK (scope_type IN ('catalog', 'project', 'release', 'reference', 'administration')),
    scope_id uuid,
    permission text NOT NULL CHECK (permission IN ('read', 'write', 'approve', 'admin')),
    expires_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE NULLS NOT DISTINCT (organization_id, user_id, scope_type, scope_id, permission)
);

CREATE TABLE ref.dimension (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code text NOT NULL UNIQUE CHECK (code ~ '^[a-z][a-z0-9_]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    symbol text NOT NULL CHECK (btrim(symbol) <> ''),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE ref.unit_system (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code text NOT NULL UNIQUE CHECK (code ~ '^[a-z][a-z0-9_]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE ref.unit (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dimension_id uuid NOT NULL REFERENCES ref.dimension(id) ON DELETE RESTRICT,
    code text NOT NULL UNIQUE CHECK (code ~ '^[a-zA-Z][a-zA-Z0-9_/-]{0,62}$'),
    symbol text NOT NULL CHECK (btrim(symbol) <> ''),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    scale_to_canonical numeric(30,12) NOT NULL CHECK (scale_to_canonical > 0),
    offset_to_canonical numeric(30,12) NOT NULL DEFAULT 0,
    is_canonical boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dimension_id, symbol)
);

CREATE UNIQUE INDEX ref_unit_one_canonical_per_dimension
    ON ref.unit (dimension_id) WHERE is_canonical;

CREATE TABLE ref.unit_system_preference (
    unit_system_id uuid NOT NULL REFERENCES ref.unit_system(id) ON DELETE RESTRICT,
    dimension_id uuid NOT NULL REFERENCES ref.dimension(id) ON DELETE RESTRICT,
    unit_id uuid NOT NULL REFERENCES ref.unit(id) ON DELETE RESTRICT,
    PRIMARY KEY (unit_system_id, dimension_id)
);

CREATE TABLE ref.manufacturer (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid REFERENCES iam.organization(id) ON DELETE RESTRICT,
    normalized_name text NOT NULL CHECK (btrim(normalized_name) <> ''),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE NULLS NOT DISTINCT (organization_id, normalized_name)
);

CREATE TABLE ref.material (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid REFERENCES iam.organization(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[A-Za-z0-9][A-Za-z0-9_.-]{0,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    properties jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(properties) = 'object'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE NULLS NOT DISTINCT (organization_id, code)
);

CREATE TABLE ref.connection (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid REFERENCES iam.organization(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[A-Za-z0-9][A-Za-z0-9_.-]{0,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    properties jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(properties) = 'object'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE NULLS NOT DISTINCT (organization_id, code)
);

CREATE TABLE catalog.family (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[a-z][a-z0-9_-]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code)
);

CREATE TABLE catalog.type (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    family_id uuid NOT NULL REFERENCES catalog.family(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[a-z][a-z0-9_-]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (family_id, code)
);

CREATE TABLE catalog.category (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    type_id uuid NOT NULL REFERENCES catalog.type(id) ON DELETE RESTRICT,
    parent_id uuid REFERENCES catalog.category(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[a-z][a-z0-9_-]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (type_id, parent_id, code),
    CHECK (parent_id IS NULL OR parent_id <> id)
);

CREATE TABLE catalog.attribute_definition (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[a-z][a-z0-9_]{1,62}$'),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    value_kind text NOT NULL CHECK (value_kind IN ('number', 'text', 'boolean', 'timestamp', 'json')),
    dimension_id uuid REFERENCES ref.dimension(id) ON DELETE RESTRICT,
    description text,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code),
    CHECK ((value_kind = 'number') = (dimension_id IS NOT NULL))
);

CREATE TABLE catalog.item (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    manufacturer_id uuid REFERENCES ref.manufacturer(id) ON DELETE RESTRICT,
    part_number text NOT NULL CHECK (btrim(part_number) <> ''),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    created_at timestamptz NOT NULL DEFAULT now(),
    created_by uuid REFERENCES iam.user_account(id) ON DELETE RESTRICT
);

CREATE UNIQUE NULLS NOT DISTINCT INDEX catalog_item_business_key
    ON catalog.item (organization_id, manufacturer_id, part_number);

CREATE TABLE catalog.item_revision (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id uuid NOT NULL REFERENCES catalog.item(id) ON DELETE RESTRICT,
    revision_no integer NOT NULL CHECK (revision_no > 0),
    category_id uuid NOT NULL REFERENCES catalog.category(id) ON DELETE RESTRICT,
    lifecycle_state text NOT NULL DEFAULT 'draft'
        CHECK (lifecycle_state IN ('draft', 'review', 'retired')),
    source_checksum text NOT NULL CHECK (source_checksum ~ '^sha256:[0-9a-f]{64}$'),
    source_document jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(source_document) = 'object'),
    created_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (item_id, revision_no)
);

CREATE INDEX catalog_item_revision_lookup
    ON catalog.item_revision (category_id, lifecycle_state, created_at DESC);

CREATE TABLE catalog.item_revision_approval (
    item_revision_id uuid PRIMARY KEY REFERENCES catalog.item_revision(id) ON DELETE RESTRICT,
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    status text NOT NULL CHECK (status IN ('approved', 'rejected')),
    decided_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    decided_at timestamptz NOT NULL DEFAULT now(),
    rationale text NOT NULL CHECK (btrim(rationale) <> '')
);

CREATE TABLE catalog.item_attribute (
    item_revision_id uuid NOT NULL REFERENCES catalog.item_revision(id) ON DELETE RESTRICT,
    attribute_definition_id uuid NOT NULL REFERENCES catalog.attribute_definition(id) ON DELETE RESTRICT,
    ordinal integer NOT NULL DEFAULT 0 CHECK (ordinal >= 0),
    value_number numeric(30,12),
    value_text text,
    value_boolean boolean,
    value_timestamp timestamptz,
    value_json jsonb,
    source_value jsonb,
    source_unit_id uuid REFERENCES ref.unit(id) ON DELETE RESTRICT,
    PRIMARY KEY (item_revision_id, attribute_definition_id, ordinal),
    CHECK (num_nonnulls(value_number, value_text, value_boolean, value_timestamp, value_json) = 1),
    CHECK (value_json IS NULL OR jsonb_typeof(value_json) IN ('object', 'array'))
);

CREATE INDEX catalog_item_attribute_numeric_lookup
    ON catalog.item_attribute (attribute_definition_id, value_number, item_revision_id)
    WHERE value_number IS NOT NULL;

CREATE TABLE catalog.item_relation (
    source_item_revision_id uuid NOT NULL REFERENCES catalog.item_revision(id) ON DELETE RESTRICT,
    target_item_revision_id uuid NOT NULL REFERENCES catalog.item_revision(id) ON DELETE RESTRICT,
    relation_kind text NOT NULL CHECK (relation_kind IN ('compatible_with', 'contains', 'requires', 'alternative_to')),
    sibling_position integer NOT NULL DEFAULT 0 CHECK (sibling_position >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (source_item_revision_id, target_item_revision_id, relation_kind),
    UNIQUE (source_item_revision_id, relation_kind, sibling_position),
    CHECK (source_item_revision_id <> target_item_revision_id)
);

CREATE TABLE catalog.change_request (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    target_item_revision_id uuid REFERENCES catalog.item_revision(id) ON DELETE RESTRICT,
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'approved', 'rejected', 'withdrawn')),
    requested_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    requested_at timestamptz NOT NULL DEFAULT now(),
    decision_by uuid REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    decision_at timestamptz,
    rationale text NOT NULL CHECK (btrim(rationale) <> ''),
    CHECK ((decision_by IS NULL) = (decision_at IS NULL)),
    CHECK (status NOT IN ('approved', 'rejected') OR decision_by IS NOT NULL)
);

CREATE TABLE catalog.release (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (code ~ '^[A-Za-z0-9][A-Za-z0-9_.-]{0,62}$'),
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'approved', 'published', 'withdrawn')),
    approved_by uuid REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    approved_at timestamptz,
    published_at timestamptz,
    created_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code),
    CHECK ((approved_by IS NULL) = (approved_at IS NULL)),
    CHECK (status NOT IN ('approved', 'published') OR approved_by IS NOT NULL),
    CHECK (status <> 'published' OR published_at IS NOT NULL)
);

CREATE TABLE catalog.release_item (
    release_id uuid NOT NULL REFERENCES catalog.release(id) ON DELETE RESTRICT,
    item_revision_id uuid NOT NULL REFERENCES catalog.item_revision(id) ON DELETE RESTRICT,
    added_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (release_id, item_revision_id)
);

CREATE TABLE project.project (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    code text NOT NULL CHECK (btrim(code) <> ''),
    display_name text NOT NULL CHECK (btrim(display_name) <> ''),
    lifecycle_state text NOT NULL DEFAULT 'active' CHECK (lifecycle_state IN ('active', 'archived')),
    created_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (organization_id, code)
);

CREATE TABLE project.project_revision (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES project.project(id) ON DELETE RESTRICT,
    revision_no integer NOT NULL CHECK (revision_no > 0),
    parent_revision_id uuid REFERENCES project.project_revision(id) ON DELETE RESTRICT,
    content_hash text NOT NULL CHECK (content_hash ~ '^sha256:[0-9a-f]{64}$'),
    artifact_uri text NOT NULL CHECK (btrim(artifact_uri) <> ''),
    artifact_format text NOT NULL CHECK (artifact_format IN ('xml', 'json')),
    schema_version text NOT NULL CHECK (btrim(schema_version) <> ''),
    created_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (id),
    UNIQUE (project_id, revision_no),
    UNIQUE (project_id, content_hash),
    CHECK (parent_revision_id IS NULL OR parent_revision_id <> id)
);

CREATE TABLE project.project_access (
    project_id uuid NOT NULL REFERENCES project.project(id) ON DELETE RESTRICT,
    user_id uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    permission text NOT NULL CHECK (permission IN ('read', 'write', 'admin')),
    granted_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    granted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, user_id)
);

CREATE TABLE project.calculation_run (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_revision_id uuid NOT NULL REFERENCES project.project_revision(id) ON DELETE RESTRICT,
    algorithm_id text NOT NULL CHECK (btrim(algorithm_id) <> ''),
    algorithm_version text NOT NULL CHECK (btrim(algorithm_version) <> ''),
    input_checksum text NOT NULL CHECK (input_checksum ~ '^sha256:[0-9a-f]{64}$'),
    status text NOT NULL CHECK (status IN ('succeeded', 'failed', 'cancelled')),
    started_at timestamptz NOT NULL,
    completed_at timestamptz NOT NULL,
    created_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (completed_at >= started_at)
);

CREATE INDEX project_calculation_run_lookup
    ON project.calculation_run (project_revision_id, algorithm_id, algorithm_version, created_at DESC);

CREATE TABLE project.calculation_artifact (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    calculation_run_id uuid NOT NULL REFERENCES project.calculation_run(id) ON DELETE RESTRICT,
    artifact_kind text NOT NULL CHECK (artifact_kind IN ('result', 'report_input', 'plot', 'diagnostic')),
    content_hash text NOT NULL CHECK (content_hash ~ '^sha256:[0-9a-f]{64}$'),
    artifact_uri text NOT NULL CHECK (btrim(artifact_uri) <> ''),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (calculation_run_id, artifact_kind, content_hash)
);

CREATE TABLE audit.event (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    actor_id uuid REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    aggregate_type text NOT NULL CHECK (btrim(aggregate_type) <> ''),
    aggregate_id uuid NOT NULL,
    event_type text NOT NULL CHECK (btrim(event_type) <> ''),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    correlation_id uuid,
    idempotency_key text NOT NULL CHECK (btrim(idempotency_key) <> ''),
    details jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(details) = 'object'),
    content_hash text CHECK (content_hash IS NULL OR content_hash ~ '^sha256:[0-9a-f]{64}$'),
    PRIMARY KEY (id),
    UNIQUE (organization_id, idempotency_key)
);

CREATE INDEX audit_event_organization_time ON audit.event (organization_id, occurred_at DESC);
CREATE INDEX audit_event_aggregate_time ON audit.event (aggregate_type, aggregate_id, occurred_at DESC);

CREATE TABLE stage.import_batch (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES iam.organization(id) ON DELETE RESTRICT,
    source_system text NOT NULL CHECK (btrim(source_system) <> ''),
    source_checksum text NOT NULL CHECK (source_checksum ~ '^sha256:[0-9a-f]{64}$'),
    transformation_version text,
    status text NOT NULL DEFAULT 'received'
        CHECK (status IN ('received', 'profiling', 'validated', 'promoted', 'rejected')),
    received_by uuid NOT NULL REFERENCES iam.user_account(id) ON DELETE RESTRICT,
    received_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (organization_id, source_system, source_checksum)
);

CREATE TABLE stage.source_record (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    import_batch_id uuid NOT NULL REFERENCES stage.import_batch(id) ON DELETE RESTRICT,
    source_table text NOT NULL CHECK (btrim(source_table) <> ''),
    source_key text NOT NULL CHECK (btrim(source_key) <> ''),
    source_hash text NOT NULL CHECK (source_hash ~ '^sha256:[0-9a-f]{64}$'),
    raw_document jsonb NOT NULL CHECK (jsonb_typeof(raw_document) = 'object'),
    received_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (import_batch_id, source_table, source_key, source_hash)
);

CREATE TABLE stage.source_identity_map (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    import_batch_id uuid NOT NULL REFERENCES stage.import_batch(id) ON DELETE RESTRICT,
    source_table text NOT NULL CHECK (btrim(source_table) <> ''),
    source_key text NOT NULL CHECK (btrim(source_key) <> ''),
    target_schema text NOT NULL CHECK (target_schema IN ('ref', 'catalog', 'project')),
    target_table text NOT NULL CHECK (btrim(target_table) <> ''),
    target_id uuid NOT NULL,
    mapped_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (import_batch_id, source_table, source_key),
    UNIQUE (target_schema, target_table, target_id)
);

CREATE TABLE stage.validation_result (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_record_id uuid NOT NULL REFERENCES stage.source_record(id) ON DELETE RESTRICT,
    rule_code text NOT NULL CHECK (btrim(rule_code) <> ''),
    severity text NOT NULL CHECK (severity IN ('error', 'warning', 'info')),
    message text NOT NULL CHECK (btrim(message) <> ''),
    details jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(details) = 'object'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (source_record_id, rule_code)
);

-- Revision and audit data are immutable. A correction is represented by a
-- later revision or a compensating audit event, never by an in-place rewrite.
CREATE FUNCTION audit.reject_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'immutable table %.% does not permit %', TG_TABLE_SCHEMA, TG_TABLE_NAME, TG_OP
        USING ERRCODE = '55000';
END;
$$;

CREATE FUNCTION iam.touch_updated_at() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at := now();
    RETURN NEW;
END;
$$;

CREATE FUNCTION iam.assert_active_member(p_organization_id uuid, p_user_id uuid) RETURNS void
LANGUAGE plpgsql STABLE SECURITY DEFINER SET search_path = iam, pg_temp AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
          FROM iam.membership m
         WHERE m.organization_id = p_organization_id
           AND m.user_id = p_user_id
           AND m.lifecycle_state = 'active'
    ) THEN
        RAISE EXCEPTION 'user % is not an active member of organization %', p_user_id, p_organization_id
            USING ERRCODE = '23514';
    END IF;
END;
$$;

CREATE FUNCTION iam.validate_membership() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = iam, pg_temp AS $$
DECLARE
    role_organization_id uuid;
    role_is_system boolean;
BEGIN
    SELECT organization_id, is_system INTO role_organization_id, role_is_system FROM iam.role WHERE id = NEW.role_id;
    IF NOT FOUND OR (NOT role_is_system AND role_organization_id IS DISTINCT FROM NEW.organization_id) THEN
        RAISE EXCEPTION 'membership role must belong to the membership organization or be a system role'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION iam.validate_scope_grant() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = iam, pg_temp AS $$
BEGIN
    PERFORM iam.assert_active_member(NEW.organization_id, NEW.user_id);
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_item() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = catalog, iam, ref, pg_temp AS $$
DECLARE
    manufacturer_organization_id uuid;
BEGIN
    IF NEW.manufacturer_id IS NOT NULL THEN
        SELECT organization_id INTO manufacturer_organization_id FROM ref.manufacturer WHERE id = NEW.manufacturer_id;
        IF manufacturer_organization_id IS NOT NULL AND manufacturer_organization_id IS DISTINCT FROM NEW.organization_id THEN
            RAISE EXCEPTION 'manufacturer must be global or belong to the item organization' USING ERRCODE = '23514';
        END IF;
    END IF;
    IF NEW.created_by IS NOT NULL THEN
        PERFORM iam.assert_active_member(NEW.organization_id, NEW.created_by);
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_category() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = catalog, pg_temp AS $$
DECLARE
    parent_type_id uuid;
BEGIN
    IF NEW.parent_id IS NOT NULL THEN
        SELECT type_id INTO parent_type_id FROM catalog.category WHERE id = NEW.parent_id;
        IF parent_type_id IS DISTINCT FROM NEW.type_id THEN
            RAISE EXCEPTION 'category parent must belong to the same catalog type' USING ERRCODE = '23514';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_item_revision() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = catalog, iam, pg_temp AS $$
DECLARE
    item_organization_id uuid;
    category_organization_id uuid;
BEGIN
    SELECT organization_id INTO item_organization_id FROM catalog.item WHERE id = NEW.item_id;
    SELECT f.organization_id INTO category_organization_id
      FROM catalog.category c
      JOIN catalog.type t ON t.id = c.type_id
      JOIN catalog.family f ON f.id = t.family_id
     WHERE c.id = NEW.category_id;
    IF item_organization_id IS DISTINCT FROM category_organization_id THEN
        RAISE EXCEPTION 'item revision category must belong to the item organization' USING ERRCODE = '23514';
    END IF;
    PERFORM iam.assert_active_member(item_organization_id, NEW.created_by);
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_item_attribute_tenant() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = catalog, iam, pg_temp AS $$
DECLARE
    revision_organization_id uuid;
    definition_organization_id uuid;
BEGIN
    SELECT i.organization_id INTO revision_organization_id
      FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id
     WHERE r.id = NEW.item_revision_id;
    SELECT organization_id INTO definition_organization_id FROM catalog.attribute_definition WHERE id = NEW.attribute_definition_id;
    IF revision_organization_id IS DISTINCT FROM definition_organization_id THEN
        RAISE EXCEPTION 'attribute definition must belong to the revision organization' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_item_relation_tenant() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = catalog, pg_temp AS $$
DECLARE
    source_organization_id uuid;
    target_organization_id uuid;
BEGIN
    SELECT i.organization_id INTO source_organization_id FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = NEW.source_item_revision_id;
    SELECT i.organization_id INTO target_organization_id FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = NEW.target_item_revision_id;
    IF source_organization_id IS DISTINCT FROM target_organization_id THEN
        RAISE EXCEPTION 'related item revisions must belong to the same organization' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_item_approval() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = catalog, iam, pg_temp AS $$
DECLARE
    revision_organization_id uuid;
BEGIN
    SELECT i.organization_id INTO revision_organization_id FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = NEW.item_revision_id;
    IF revision_organization_id IS DISTINCT FROM NEW.organization_id THEN
        RAISE EXCEPTION 'approval organization must match the revision organization' USING ERRCODE = '23514';
    END IF;
    PERFORM iam.assert_active_member(NEW.organization_id, NEW.decided_by);
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_release_item() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = catalog, pg_temp AS $$
DECLARE
    release_organization_id uuid;
    revision_organization_id uuid;
    is_approved boolean;
BEGIN
    SELECT organization_id INTO release_organization_id FROM catalog.release WHERE id = NEW.release_id;
    SELECT i.organization_id INTO revision_organization_id FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = NEW.item_revision_id;
    SELECT status = 'approved' INTO is_approved FROM catalog.item_revision_approval WHERE item_revision_id = NEW.item_revision_id;
    IF release_organization_id IS DISTINCT FROM revision_organization_id OR is_approved IS DISTINCT FROM true THEN
        RAISE EXCEPTION 'release membership requires an approved revision from the same organization' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION project.validate_project_revision() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = project, iam, pg_temp AS $$
DECLARE
    parent_project_id uuid;
    current_organization_id uuid;
BEGIN
    SELECT p.organization_id INTO current_organization_id FROM project.project p WHERE p.id = NEW.project_id;
    IF NEW.parent_revision_id IS NOT NULL THEN
        SELECT project_id INTO parent_project_id FROM project.project_revision WHERE id = NEW.parent_revision_id;
        IF parent_project_id IS DISTINCT FROM NEW.project_id THEN
            RAISE EXCEPTION 'parent revision must belong to the same project' USING ERRCODE = '23514';
        END IF;
    END IF;
    PERFORM iam.assert_active_member(current_organization_id, NEW.created_by);
    RETURN NEW;
END;
$$;

CREATE FUNCTION project.validate_project_access() RETURNS trigger
LANGUAGE plpgsql SECURITY DEFINER SET search_path = project, iam, pg_temp AS $$
DECLARE
    current_organization_id uuid;
BEGIN
    SELECT p.organization_id INTO current_organization_id FROM project.project p WHERE p.id = NEW.project_id;
    PERFORM iam.assert_active_member(current_organization_id, NEW.user_id);
    PERFORM iam.assert_active_member(current_organization_id, NEW.granted_by);
    RETURN NEW;
END;
$$;

CREATE FUNCTION catalog.validate_item_attribute() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    expected_kind text;
    expected_dimension_id uuid;
    source_dimension_id uuid;
BEGIN
    SELECT value_kind, dimension_id
      INTO expected_kind, expected_dimension_id
      FROM catalog.attribute_definition
     WHERE id = NEW.attribute_definition_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'unknown catalog attribute definition %', NEW.attribute_definition_id
            USING ERRCODE = '23503';
    END IF;

    IF (expected_kind = 'number' AND NEW.value_number IS NULL)
       OR (expected_kind = 'text' AND NEW.value_text IS NULL)
       OR (expected_kind = 'boolean' AND NEW.value_boolean IS NULL)
       OR (expected_kind = 'timestamp' AND NEW.value_timestamp IS NULL)
       OR (expected_kind = 'json' AND NEW.value_json IS NULL) THEN
        RAISE EXCEPTION 'value column does not match attribute kind %', expected_kind
            USING ERRCODE = '23514';
    END IF;

    IF NEW.source_unit_id IS NOT NULL THEN
        IF expected_kind <> 'number' THEN
            RAISE EXCEPTION 'only numeric attributes may carry source units'
                USING ERRCODE = '23514';
        END IF;

        SELECT dimension_id INTO source_dimension_id FROM ref.unit WHERE id = NEW.source_unit_id;
        IF source_dimension_id IS DISTINCT FROM expected_dimension_id THEN
            RAISE EXCEPTION 'source unit dimension does not match attribute dimension'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER catalog_item_revision_immutable
    BEFORE UPDATE OR DELETE ON catalog.item_revision
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER catalog_item_attribute_immutable
    BEFORE UPDATE OR DELETE ON catalog.item_attribute
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER catalog_item_attribute_value_type
    BEFORE INSERT OR UPDATE ON catalog.item_attribute
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_item_attribute();
CREATE TRIGGER iam_organization_touch_updated_at
    BEFORE UPDATE ON iam.organization
    FOR EACH ROW EXECUTE FUNCTION iam.touch_updated_at();
CREATE TRIGGER iam_user_account_touch_updated_at
    BEFORE UPDATE ON iam.user_account
    FOR EACH ROW EXECUTE FUNCTION iam.touch_updated_at();
CREATE TRIGGER iam_membership_tenant_integrity
    BEFORE INSERT OR UPDATE ON iam.membership
    FOR EACH ROW EXECUTE FUNCTION iam.validate_membership();
CREATE TRIGGER iam_scope_grant_tenant_integrity
    BEFORE INSERT OR UPDATE ON iam.scope_grant
    FOR EACH ROW EXECUTE FUNCTION iam.validate_scope_grant();
CREATE TRIGGER catalog_item_tenant_integrity
    BEFORE INSERT OR UPDATE ON catalog.item
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_item();
CREATE TRIGGER catalog_category_tenant_integrity
    BEFORE INSERT OR UPDATE ON catalog.category
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_category();
CREATE TRIGGER catalog_item_revision_tenant_integrity
    BEFORE INSERT ON catalog.item_revision
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_item_revision();
CREATE TRIGGER catalog_item_attribute_tenant_integrity
    BEFORE INSERT ON catalog.item_attribute
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_item_attribute_tenant();
CREATE TRIGGER catalog_item_relation_immutable
    BEFORE UPDATE OR DELETE ON catalog.item_relation
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER catalog_item_relation_tenant_integrity
    BEFORE INSERT ON catalog.item_relation
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_item_relation_tenant();
CREATE TRIGGER catalog_item_approval_tenant_integrity
    BEFORE INSERT ON catalog.item_revision_approval
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_item_approval();
CREATE TRIGGER catalog_item_approval_immutable
    BEFORE UPDATE OR DELETE ON catalog.item_revision_approval
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER catalog_release_item_immutable
    BEFORE UPDATE OR DELETE ON catalog.release_item
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER catalog_release_item_tenant_integrity
    BEFORE INSERT ON catalog.release_item
    FOR EACH ROW EXECUTE FUNCTION catalog.validate_release_item();
CREATE TRIGGER project_revision_immutable
    BEFORE UPDATE OR DELETE ON project.project_revision
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER project_revision_tenant_integrity
    BEFORE INSERT ON project.project_revision
    FOR EACH ROW EXECUTE FUNCTION project.validate_project_revision();
CREATE TRIGGER project_access_tenant_integrity
    BEFORE INSERT OR UPDATE ON project.project_access
    FOR EACH ROW EXECUTE FUNCTION project.validate_project_access();
CREATE TRIGGER project_calculation_run_immutable
    BEFORE UPDATE OR DELETE ON project.calculation_run
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER project_calculation_artifact_immutable
    BEFORE UPDATE OR DELETE ON project.calculation_artifact
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();
CREATE TRIGGER audit_event_append_only
    BEFORE UPDATE OR DELETE ON audit.event
    FOR EACH ROW EXECUTE FUNCTION audit.reject_mutation();

-- The Rust-controlled transaction boundary must set app.organization_id and
-- app.actor_id with SET LOCAL after authenticating the caller. The desktop
-- client never obtains database credentials.
CREATE FUNCTION iam.current_organization_id() RETURNS uuid
LANGUAGE sql STABLE PARALLEL SAFE AS $$
    SELECT NULLIF(current_setting('app.organization_id', true), '')::uuid
$$;

CREATE FUNCTION iam.current_actor_id() RETURNS uuid
LANGUAGE sql STABLE PARALLEL SAFE AS $$
    SELECT NULLIF(current_setting('app.actor_id', true), '')::uuid
$$;

CREATE FUNCTION iam.permission_rank(p_permission text) RETURNS integer
LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $$
    SELECT CASE p_permission
        WHEN 'read' THEN 1
        WHEN 'write' THEN 2
        WHEN 'approve' THEN 3
        WHEN 'admin' THEN 4
        ELSE 0
    END
$$;

CREATE FUNCTION iam.actor_has_active_membership(p_organization_id uuid) RETURNS boolean
LANGUAGE sql STABLE SECURITY DEFINER SET search_path = iam, pg_temp AS $$
    SELECT EXISTS (
        SELECT 1
          FROM iam.membership m
         WHERE m.organization_id = p_organization_id
           AND m.user_id = iam.current_actor_id()
           AND m.lifecycle_state = 'active'
    )
$$;

CREATE FUNCTION iam.actor_can(
    p_organization_id uuid,
    p_scope_type text,
    p_scope_id uuid,
    p_permission text
) RETURNS boolean
LANGUAGE sql STABLE SECURITY DEFINER SET search_path = iam, pg_temp AS $$
    SELECT EXISTS (
        SELECT 1
          FROM iam.membership m
         WHERE m.organization_id = p_organization_id
           AND m.user_id = iam.current_actor_id()
           AND m.lifecycle_state = 'active'
           AND (
                EXISTS (
                    SELECT 1
                      FROM iam.role_permission rp
                     WHERE rp.role_id = m.role_id
                       AND iam.permission_rank(rp.permission) >= iam.permission_rank(p_permission)
                       AND (rp.scope_type = p_scope_type OR (rp.scope_type = 'administration' AND rp.permission = 'admin'))
                )
                OR EXISTS (
                    SELECT 1
                      FROM iam.scope_grant sg
                     WHERE sg.organization_id = p_organization_id
                       AND sg.user_id = m.user_id
                       AND sg.scope_type = p_scope_type
                       AND (sg.scope_id IS NULL OR sg.scope_id = p_scope_id)
                       AND (sg.expires_at IS NULL OR sg.expires_at > now())
                       AND iam.permission_rank(sg.permission) >= iam.permission_rank(p_permission)
                )
           )
    )
$$;

ALTER TABLE iam.organization ENABLE ROW LEVEL SECURITY;
ALTER TABLE iam.organization FORCE ROW LEVEL SECURITY;
CREATE POLICY organization_tenant_scope ON iam.organization
    USING (id = iam.current_organization_id())
    WITH CHECK (id = iam.current_organization_id());

ALTER TABLE iam.user_account ENABLE ROW LEVEL SECURITY;
CREATE POLICY user_account_actor_scope ON iam.user_account
    USING (id = iam.current_actor_id())
    WITH CHECK (id = iam.current_actor_id());

ALTER TABLE iam.membership ENABLE ROW LEVEL SECURITY;
CREATE POLICY membership_tenant_scope ON iam.membership
    USING (organization_id = iam.current_organization_id())
    WITH CHECK (organization_id = iam.current_organization_id());

ALTER TABLE iam.scope_grant ENABLE ROW LEVEL SECURITY;
CREATE POLICY scope_grant_tenant_scope ON iam.scope_grant
    USING (organization_id = iam.current_organization_id())
    WITH CHECK (organization_id = iam.current_organization_id());

ALTER TABLE iam.role_permission ENABLE ROW LEVEL SECURITY;
CREATE POLICY role_permission_administration_scope ON iam.role_permission
    USING (iam.actor_can(iam.current_organization_id(), 'administration', NULL, 'admin'))
    WITH CHECK (iam.actor_can(iam.current_organization_id(), 'administration', NULL, 'admin'));

ALTER TABLE ref.manufacturer ENABLE ROW LEVEL SECURITY;
ALTER TABLE ref.material ENABLE ROW LEVEL SECURITY;
ALTER TABLE ref.connection ENABLE ROW LEVEL SECURITY;
CREATE POLICY manufacturer_visibility ON ref.manufacturer
    USING ((organization_id IS NULL AND iam.actor_has_active_membership(iam.current_organization_id()))
        OR iam.actor_can(organization_id, 'reference', id, 'read'))
    WITH CHECK (organization_id IS NOT NULL AND iam.actor_can(organization_id, 'reference', id, 'write'));
CREATE POLICY material_visibility ON ref.material
    USING ((organization_id IS NULL AND iam.actor_has_active_membership(iam.current_organization_id()))
        OR iam.actor_can(organization_id, 'reference', id, 'read'))
    WITH CHECK (organization_id IS NOT NULL AND iam.actor_can(organization_id, 'reference', id, 'write'));
CREATE POLICY connection_visibility ON ref.connection
    USING ((organization_id IS NULL AND iam.actor_has_active_membership(iam.current_organization_id()))
        OR iam.actor_can(organization_id, 'reference', id, 'read'))
    WITH CHECK (organization_id IS NOT NULL AND iam.actor_can(organization_id, 'reference', id, 'write'));

ALTER TABLE catalog.family ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.type ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.category ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.attribute_definition ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.item ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.change_request ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.release ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_attribute ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_relation ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.release_item ENABLE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_revision_approval ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.project ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.project_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.project_access ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.calculation_run ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.calculation_artifact ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit.event ENABLE ROW LEVEL SECURITY;
ALTER TABLE stage.import_batch ENABLE ROW LEVEL SECURITY;
ALTER TABLE stage.source_record ENABLE ROW LEVEL SECURITY;
ALTER TABLE stage.source_identity_map ENABLE ROW LEVEL SECURITY;
ALTER TABLE stage.validation_result ENABLE ROW LEVEL SECURITY;

ALTER TABLE catalog.family FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.attribute_definition FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.item FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.type FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.category FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.change_request FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.release FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_revision FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_attribute FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_relation FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.release_item FORCE ROW LEVEL SECURITY;
ALTER TABLE catalog.item_revision_approval FORCE ROW LEVEL SECURITY;
ALTER TABLE project.project FORCE ROW LEVEL SECURITY;
ALTER TABLE project.project_revision FORCE ROW LEVEL SECURITY;
ALTER TABLE project.project_access FORCE ROW LEVEL SECURITY;
ALTER TABLE project.calculation_run FORCE ROW LEVEL SECURITY;
ALTER TABLE project.calculation_artifact FORCE ROW LEVEL SECURITY;
ALTER TABLE audit.event FORCE ROW LEVEL SECURITY;
ALTER TABLE stage.import_batch FORCE ROW LEVEL SECURITY;
ALTER TABLE stage.source_record FORCE ROW LEVEL SECURITY;
ALTER TABLE stage.source_identity_map FORCE ROW LEVEL SECURITY;
ALTER TABLE stage.validation_result FORCE ROW LEVEL SECURITY;

CREATE POLICY catalog_family_tenant_scope ON catalog.family
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY catalog_type_tenant_scope ON catalog.type
    USING (EXISTS (SELECT 1 FROM catalog.family f WHERE f.id = family_id AND f.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.family f WHERE f.id = family_id AND f.organization_id = iam.current_organization_id()));
CREATE POLICY catalog_category_tenant_scope ON catalog.category
    USING (EXISTS (SELECT 1 FROM catalog.type t JOIN catalog.family f ON f.id = t.family_id WHERE t.id = type_id AND f.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.type t JOIN catalog.family f ON f.id = t.family_id WHERE t.id = type_id AND f.organization_id = iam.current_organization_id()));
CREATE POLICY catalog_attribute_definition_tenant_scope ON catalog.attribute_definition
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY catalog_item_tenant_scope ON catalog.item
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY catalog_change_request_tenant_scope ON catalog.change_request
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY catalog_release_tenant_scope ON catalog.release
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY catalog_item_revision_tenant_scope ON catalog.item_revision
    USING (EXISTS (SELECT 1 FROM catalog.item i WHERE i.id = item_id AND i.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.item i WHERE i.id = item_id AND i.organization_id = iam.current_organization_id()));
CREATE POLICY catalog_item_attribute_tenant_scope ON catalog.item_attribute
    USING (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = item_revision_id AND i.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = item_revision_id AND i.organization_id = iam.current_organization_id()));
CREATE POLICY catalog_item_relation_tenant_scope ON catalog.item_relation
    USING (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = source_item_revision_id AND i.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = source_item_revision_id AND i.organization_id = iam.current_organization_id()));
CREATE POLICY catalog_release_item_tenant_scope ON catalog.release_item
    USING (EXISTS (SELECT 1 FROM catalog.release r WHERE r.id = release_id AND r.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.release r WHERE r.id = release_id AND r.organization_id = iam.current_organization_id()));
CREATE POLICY catalog_item_approval_tenant_scope ON catalog.item_revision_approval
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY project_tenant_scope ON project.project
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY project_revision_tenant_scope ON project.project_revision
    USING (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND p.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND p.organization_id = iam.current_organization_id()));
CREATE POLICY project_access_tenant_scope ON project.project_access
    USING (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND p.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND p.organization_id = iam.current_organization_id()));
CREATE POLICY project_calculation_run_tenant_scope ON project.calculation_run
    USING (EXISTS (SELECT 1 FROM project.project_revision r JOIN project.project p ON p.id = r.project_id WHERE r.id = project_revision_id AND p.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM project.project_revision r JOIN project.project p ON p.id = r.project_id WHERE r.id = project_revision_id AND p.organization_id = iam.current_organization_id()));
CREATE POLICY project_calculation_artifact_tenant_scope ON project.calculation_artifact
    USING (EXISTS (SELECT 1 FROM project.calculation_run c JOIN project.project_revision r ON r.id = c.project_revision_id JOIN project.project p ON p.id = r.project_id WHERE c.id = calculation_run_id AND p.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM project.calculation_run c JOIN project.project_revision r ON r.id = c.project_revision_id JOIN project.project p ON p.id = r.project_id WHERE c.id = calculation_run_id AND p.organization_id = iam.current_organization_id()));
CREATE POLICY audit_event_tenant_scope ON audit.event
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY stage_import_batch_tenant_scope ON stage.import_batch
    USING (organization_id = iam.current_organization_id()) WITH CHECK (organization_id = iam.current_organization_id());
CREATE POLICY stage_source_record_tenant_scope ON stage.source_record
    USING (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND b.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND b.organization_id = iam.current_organization_id()));
CREATE POLICY stage_source_identity_map_tenant_scope ON stage.source_identity_map
    USING (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND b.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND b.organization_id = iam.current_organization_id()));
CREATE POLICY stage_validation_result_tenant_scope ON stage.validation_result
    USING (EXISTS (SELECT 1 FROM stage.source_record s JOIN stage.import_batch b ON b.id = s.import_batch_id WHERE s.id = source_record_id AND b.organization_id = iam.current_organization_id()))
    WITH CHECK (EXISTS (SELECT 1 FROM stage.source_record s JOIN stage.import_batch b ON b.id = s.import_batch_id WHERE s.id = source_record_id AND b.organization_id = iam.current_organization_id()));

-- Restrictive policies turn a matching tenant into an authorized actor. They
-- are evaluated in addition to the tenant-scoping policies above.
CREATE POLICY membership_actor_authorization ON iam.membership AS RESTRICTIVE
    USING (user_id = iam.current_actor_id() OR iam.actor_can(organization_id, 'administration', NULL, 'admin'))
    WITH CHECK (iam.actor_can(organization_id, 'administration', NULL, 'admin'));
CREATE POLICY scope_grant_actor_authorization ON iam.scope_grant AS RESTRICTIVE
    USING (user_id = iam.current_actor_id() OR iam.actor_can(organization_id, 'administration', NULL, 'admin'))
    WITH CHECK (iam.actor_can(organization_id, 'administration', NULL, 'admin'));
CREATE POLICY catalog_family_actor_authorization ON catalog.family AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'catalog', NULL, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'catalog', NULL, 'write'));
CREATE POLICY catalog_type_actor_authorization ON catalog.type AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM catalog.family f WHERE f.id = family_id AND iam.actor_can(f.organization_id, 'catalog', NULL, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.family f WHERE f.id = family_id AND iam.actor_can(f.organization_id, 'catalog', NULL, 'write')));
CREATE POLICY catalog_category_actor_authorization ON catalog.category AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM catalog.type t JOIN catalog.family f ON f.id = t.family_id WHERE t.id = type_id AND iam.actor_can(f.organization_id, 'catalog', NULL, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.type t JOIN catalog.family f ON f.id = t.family_id WHERE t.id = type_id AND iam.actor_can(f.organization_id, 'catalog', NULL, 'write')));
CREATE POLICY catalog_attribute_definition_actor_authorization ON catalog.attribute_definition AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'catalog', NULL, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'catalog', NULL, 'write'));
CREATE POLICY catalog_item_actor_authorization ON catalog.item AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'catalog', id, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'catalog', id, 'write'));
CREATE POLICY catalog_item_revision_actor_authorization ON catalog.item_revision AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM catalog.item i WHERE i.id = item_id AND iam.actor_can(i.organization_id, 'catalog', i.id, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.item i WHERE i.id = item_id AND iam.actor_can(i.organization_id, 'catalog', i.id, 'write')));
CREATE POLICY catalog_item_approval_actor_authorization ON catalog.item_revision_approval AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'catalog', item_revision_id, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'catalog', item_revision_id, 'approve'));
CREATE POLICY catalog_item_attribute_actor_authorization ON catalog.item_attribute AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = item_revision_id AND iam.actor_can(i.organization_id, 'catalog', i.id, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = item_revision_id AND iam.actor_can(i.organization_id, 'catalog', i.id, 'write')));
CREATE POLICY catalog_item_relation_actor_authorization ON catalog.item_relation AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = source_item_revision_id AND iam.actor_can(i.organization_id, 'catalog', i.id, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.item_revision r JOIN catalog.item i ON i.id = r.item_id WHERE r.id = source_item_revision_id AND iam.actor_can(i.organization_id, 'catalog', i.id, 'write')));
CREATE POLICY catalog_change_request_actor_authorization ON catalog.change_request AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'catalog', target_item_revision_id, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'catalog', target_item_revision_id, 'write'));
CREATE POLICY catalog_release_actor_authorization ON catalog.release AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'release', id, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'release', id, 'approve'));
CREATE POLICY catalog_release_item_actor_authorization ON catalog.release_item AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM catalog.release r WHERE r.id = release_id AND iam.actor_can(r.organization_id, 'release', r.id, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM catalog.release r WHERE r.id = release_id AND iam.actor_can(r.organization_id, 'release', r.id, 'approve')));
CREATE POLICY project_actor_authorization ON project.project AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'project', id, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'project', id, 'write'));
CREATE POLICY project_revision_actor_authorization ON project.project_revision AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND iam.actor_can(p.organization_id, 'project', p.id, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND iam.actor_can(p.organization_id, 'project', p.id, 'write')));
CREATE POLICY project_access_actor_authorization ON project.project_access AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND (user_id = iam.current_actor_id() OR iam.actor_can(p.organization_id, 'project', p.id, 'admin'))))
    WITH CHECK (EXISTS (SELECT 1 FROM project.project p WHERE p.id = project_id AND iam.actor_can(p.organization_id, 'project', p.id, 'admin')));
CREATE POLICY project_calculation_run_actor_authorization ON project.calculation_run AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM project.project_revision r JOIN project.project p ON p.id = r.project_id WHERE r.id = project_revision_id AND iam.actor_can(p.organization_id, 'project', p.id, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM project.project_revision r JOIN project.project p ON p.id = r.project_id AND r.id = project_revision_id AND iam.actor_can(p.organization_id, 'project', p.id, 'write')));
CREATE POLICY project_calculation_artifact_actor_authorization ON project.calculation_artifact AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM project.calculation_run c JOIN project.project_revision r ON r.id = c.project_revision_id JOIN project.project p ON p.id = r.project_id WHERE c.id = calculation_run_id AND iam.actor_can(p.organization_id, 'project', p.id, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM project.calculation_run c JOIN project.project_revision r ON r.id = c.project_revision_id JOIN project.project p ON p.id = r.project_id WHERE c.id = calculation_run_id AND iam.actor_can(p.organization_id, 'project', p.id, 'write')));
CREATE POLICY audit_event_actor_authorization ON audit.event AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'administration', NULL, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'administration', NULL, 'write'));
CREATE POLICY stage_import_batch_actor_authorization ON stage.import_batch AS RESTRICTIVE
    USING (iam.actor_can(organization_id, 'administration', NULL, 'read'))
    WITH CHECK (iam.actor_can(organization_id, 'administration', NULL, 'write'));
CREATE POLICY stage_source_record_actor_authorization ON stage.source_record AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND iam.actor_can(b.organization_id, 'administration', NULL, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND iam.actor_can(b.organization_id, 'administration', NULL, 'write')));
CREATE POLICY stage_source_identity_map_actor_authorization ON stage.source_identity_map AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND iam.actor_can(b.organization_id, 'administration', NULL, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM stage.import_batch b WHERE b.id = import_batch_id AND iam.actor_can(b.organization_id, 'administration', NULL, 'write')));
CREATE POLICY stage_validation_result_actor_authorization ON stage.validation_result AS RESTRICTIVE
    USING (EXISTS (SELECT 1 FROM stage.source_record s JOIN stage.import_batch b ON b.id = s.import_batch_id WHERE s.id = source_record_id AND iam.actor_can(b.organization_id, 'administration', NULL, 'read')))
    WITH CHECK (EXISTS (SELECT 1 FROM stage.source_record s JOIN stage.import_batch b ON b.id = s.import_batch_id WHERE s.id = source_record_id AND iam.actor_can(b.organization_id, 'administration', NULL, 'write')));

-- Privileges are granted to group roles only. Login principals are provisioned
-- separately and must be assigned to a single group role by deployment policy.
REVOKE ALL ON SCHEMA public FROM PUBLIC;
REVOKE ALL ON SCHEMA iam, ref, catalog, project, audit, stage FROM PUBLIC;
REVOKE ALL ON ALL TABLES IN SCHEMA iam, ref, catalog, project, audit, stage FROM PUBLIC;
REVOKE ALL ON ALL FUNCTIONS IN SCHEMA iam, ref, catalog, project, audit, stage FROM PUBLIC;

GRANT USAGE ON SCHEMA iam, ref, catalog, project, audit TO wellforge_runtime, wellforge_reader;
GRANT USAGE ON SCHEMA iam, stage TO wellforge_importer;
GRANT USAGE ON SCHEMA ref TO wellforge_importer;

GRANT SELECT ON ALL TABLES IN SCHEMA iam, ref, catalog, project, audit TO wellforge_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA iam, ref, catalog, project, audit TO wellforge_runtime;
GRANT INSERT ON iam.user_account, iam.membership, iam.scope_grant TO wellforge_runtime;
GRANT UPDATE ON iam.user_account, iam.membership, iam.scope_grant TO wellforge_runtime;
GRANT INSERT, UPDATE ON ref.manufacturer, ref.material, ref.connection TO wellforge_runtime;
GRANT INSERT, UPDATE ON catalog.family, catalog.type, catalog.category, catalog.attribute_definition,
    catalog.item, catalog.change_request, catalog.release TO wellforge_runtime;
GRANT INSERT ON catalog.item_revision, catalog.item_revision_approval, catalog.item_attribute,
    catalog.item_relation, catalog.release_item TO wellforge_runtime;
GRANT INSERT, UPDATE ON project.project, project.project_access TO wellforge_runtime;
GRANT INSERT ON project.project_revision, project.calculation_run, project.calculation_artifact,
    audit.event TO wellforge_runtime;

GRANT SELECT ON ref.dimension, ref.unit, ref.unit_system, ref.unit_system_preference TO wellforge_importer;
GRANT SELECT, INSERT, UPDATE ON stage.import_batch, stage.source_record,
    stage.source_identity_map, stage.validation_result TO wellforge_importer;

GRANT EXECUTE ON FUNCTION iam.current_organization_id(), iam.current_actor_id(),
    iam.actor_has_active_membership(uuid), iam.actor_can(uuid, text, uuid, text)
    TO wellforge_runtime, wellforge_reader, wellforge_importer;
GRANT USAGE ON SCHEMA iam, ref, catalog, project, audit, stage TO wellforge_migrator;
GRANT CREATE ON SCHEMA iam, ref, catalog, project, audit, stage TO wellforge_migrator;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA iam, ref, catalog, project, audit, stage TO wellforge_migrator;

COMMIT;
