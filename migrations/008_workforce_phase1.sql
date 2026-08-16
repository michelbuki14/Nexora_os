-- Migration: 008_workforce_phase1.sql
-- Description: Workforce module Phase 1 - org structure, employees, employment and
--              compensation history, document metadata, RBAC permission seeds.
-- Author: Nexora OS Platform Team
-- Date: 2026-08-10
--
-- Additive only. Every statement is idempotent so re-running the migration is safe.
--
-- Conventions inherited from 001/005/006 and deliberately NOT re-invented here:
--   * UUID primary keys (`gen_random_uuid()`), `CHAR(26)` ULIDs as the external id.
--   * `tenant_id`/`org_id` on every row table, isolated by RLS against the
--     `nexora.current_tenant_id` GUC that `rls_middleware` sets per request.
--   * Reference data (currencies, countries, permissions) is shared, not copied.
--
-- History tables (`wf_employment_records`, `wf_compensation_records`) and document
-- metadata are APPEND-ONLY by design: they carry no `updated_at` and the application
-- never issues UPDATE against them. Corrections are new rows with a new
-- `effective_date`, which is what makes a payroll run reproducible after the fact.
-- Enforcing that in the database (REVOKE UPDATE, DELETE) is on the hardening
-- checklist together with the same REVOKE for `audit_events`.

-- =============================================================================
-- LEGAL ENTITIES
-- =============================================================================

CREATE TABLE IF NOT EXISTS wf_legal_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    name TEXT NOT NULL,
    registration_number TEXT,
    country_id UUID REFERENCES countries(id),
    address JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'dissolved')),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wf_legal_entities_tenant ON wf_legal_entities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_legal_entities_org ON wf_legal_entities(org_id);

-- =============================================================================
-- LOCATIONS
-- =============================================================================

CREATE TABLE IF NOT EXISTS wf_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    legal_entity_id UUID NOT NULL REFERENCES wf_legal_entities(id),
    name TEXT NOT NULL,
    code TEXT,
    address JSONB NOT NULL DEFAULT '{}',
    timezone TEXT NOT NULL DEFAULT 'Africa/Lubumbashi',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wf_locations_tenant ON wf_locations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_locations_entity ON wf_locations(legal_entity_id);

-- =============================================================================
-- DEPARTMENTS (self-referential hierarchy)
-- =============================================================================

CREATE TABLE IF NOT EXISTS wf_departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    name TEXT NOT NULL,
    code TEXT,
    parent_department_id UUID REFERENCES wf_departments(id),
    location_id UUID REFERENCES wf_locations(id),
    -- head_employee_id added after wf_employees exists (see deferred FKs below).
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wf_departments_tenant ON wf_departments(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_departments_parent ON wf_departments(parent_department_id);
-- Partial unique index: codes are optional, but must be unique per tenant when set.
CREATE UNIQUE INDEX IF NOT EXISTS idx_wf_departments_tenant_code
    ON wf_departments(tenant_id, code) WHERE code IS NOT NULL;

-- =============================================================================
-- TEAMS
-- =============================================================================

CREATE TABLE IF NOT EXISTS wf_teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    department_id UUID NOT NULL REFERENCES wf_departments(id),
    name TEXT NOT NULL,
    code TEXT,
    -- lead_employee_id added after wf_employees exists (see deferred FKs below).
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wf_teams_tenant ON wf_teams(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_teams_department ON wf_teams(department_id);

-- =============================================================================
-- POSITIONS
-- =============================================================================

CREATE TABLE IF NOT EXISTS wf_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    department_id UUID REFERENCES wf_departments(id),
    title TEXT NOT NULL,
    code TEXT,
    job_grade TEXT,
    description TEXT,
    responsibilities TEXT,
    employment_type TEXT NOT NULL DEFAULT 'permanent'
        CHECK (employment_type IN ('permanent', 'contract', 'intern', 'consultant')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wf_positions_tenant ON wf_positions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_positions_department ON wf_positions(department_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_wf_positions_tenant_code
    ON wf_positions(tenant_id, code) WHERE code IS NOT NULL;

-- =============================================================================
-- EMPLOYEES (sensitive)
-- =============================================================================
-- National identifiers are never stored in the clear: only a SHA-256 digest for
-- duplicate detection and the last four characters for human disambiguation.

CREATE TABLE IF NOT EXISTS wf_employees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    -- Nullable: an employee record can exist before an IAM login is provisioned.
    user_id UUID REFERENCES users(id),
    employee_number TEXT NOT NULL,
    legal_name TEXT NOT NULL,
    preferred_name TEXT,
    date_of_birth DATE,
    gender TEXT CHECK (gender IN ('male', 'female', 'other', 'prefer_not_to_say')),
    national_id_hash TEXT,
    national_id_last4 CHAR(4),
    email TEXT NOT NULL,
    phone TEXT,
    status TEXT NOT NULL DEFAULT 'onboarding'
        CHECK (status IN ('onboarding', 'active', 'on_leave', 'terminated')),
    hire_date DATE,
    termination_date DATE,
    termination_reason TEXT,
    current_department_id UUID REFERENCES wf_departments(id),
    current_position_id UUID REFERENCES wf_positions(id),
    current_location_id UUID REFERENCES wf_locations(id),
    manager_employee_id UUID REFERENCES wf_employees(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (tenant_id, employee_number)
);

CREATE INDEX IF NOT EXISTS idx_wf_employees_tenant ON wf_employees(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_employees_org ON wf_employees(org_id);
CREATE INDEX IF NOT EXISTS idx_wf_employees_user ON wf_employees(user_id);
CREATE INDEX IF NOT EXISTS idx_wf_employees_manager ON wf_employees(manager_employee_id);
CREATE INDEX IF NOT EXISTS idx_wf_employees_department ON wf_employees(current_department_id);
CREATE INDEX IF NOT EXISTS idx_wf_employees_status ON wf_employees(tenant_id, status);

-- Deferred self/forward references: these columns point at wf_employees, so they
-- can only be added once that table exists.
ALTER TABLE wf_departments
    ADD COLUMN IF NOT EXISTS head_employee_id UUID REFERENCES wf_employees(id);
ALTER TABLE wf_teams
    ADD COLUMN IF NOT EXISTS lead_employee_id UUID REFERENCES wf_employees(id);

-- =============================================================================
-- EMPLOYMENT HISTORY (append-only)
-- =============================================================================

CREATE TABLE IF NOT EXISTS wf_employment_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    employee_id UUID NOT NULL REFERENCES wf_employees(id),
    position_id UUID REFERENCES wf_positions(id),
    department_id UUID REFERENCES wf_departments(id),
    location_id UUID REFERENCES wf_locations(id),
    employment_type TEXT NOT NULL DEFAULT 'permanent'
        CHECK (employment_type IN ('permanent', 'contract', 'intern', 'consultant')),
    effective_date DATE NOT NULL,
    -- NULL end_date marks the currently-open record for an employee.
    end_date DATE,
    changes JSONB NOT NULL DEFAULT '{}',
    change_reason TEXT,
    changed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wf_employment_tenant ON wf_employment_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_employment_employee ON wf_employment_records(employee_id, effective_date DESC);
CREATE INDEX IF NOT EXISTS idx_wf_employment_open ON wf_employment_records(employee_id) WHERE end_date IS NULL;

-- =============================================================================
-- COMPENSATION HISTORY (append-only, sensitive)
-- =============================================================================
-- Amounts are integer minor units (centimes/cents) against a currency from the
-- shared `currencies` table — never a float, per the money policy in
-- services/common/src/money.rs.

CREATE TABLE IF NOT EXISTS wf_compensation_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    employee_id UUID NOT NULL REFERENCES wf_employees(id),
    gross_amount_minor BIGINT NOT NULL,
    currency_id UUID NOT NULL REFERENCES currencies(id),
    frequency TEXT NOT NULL DEFAULT 'monthly'
        CHECK (frequency IN ('monthly', 'annual', 'hourly', 'weekly')),
    effective_date DATE NOT NULL,
    end_date DATE,
    changes JSONB NOT NULL DEFAULT '{}',
    change_reason TEXT,
    changed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wf_compensation_tenant ON wf_compensation_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_compensation_employee ON wf_compensation_records(employee_id, effective_date DESC);
CREATE INDEX IF NOT EXISTS idx_wf_compensation_open ON wf_compensation_records(employee_id) WHERE end_date IS NULL;

-- =============================================================================
-- DOCUMENT METADATA (bytes live in object storage, never in the database)
-- =============================================================================
-- `object_key` is namespaced `{tenant_ulid}/{employee_ulid}/{doc_ulid}` so that a
-- key from another tenant is not merely unauthorized but unguessable, and is
-- served exclusively through short-lived presigned URLs.

CREATE TABLE IF NOT EXISTS wf_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    employee_id UUID NOT NULL REFERENCES wf_employees(id),
    doc_type TEXT NOT NULL
        CHECK (doc_type IN ('contract', 'id_document', 'certificate', 'offer_letter', 'payslip', 'other')),
    object_key TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    sha256 CHAR(64),
    size_bytes BIGINT,
    uploaded_by UUID REFERENCES users(id),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wf_documents_tenant ON wf_documents(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_documents_employee ON wf_documents(employee_id, created_at DESC);

-- =============================================================================
-- ROW-LEVEL SECURITY
-- =============================================================================
-- Same shape as 005/006: readable and writable only within the caller's tenant,
-- with an explicit WITH CHECK so a caller cannot write a row into another tenant.
-- Policies are dropped first so re-running the migration converges.

ALTER TABLE wf_legal_entities ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_locations ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_departments ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_teams ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_positions ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_employees ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_employment_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_compensation_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_documents ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_wf_legal_entities ON wf_legal_entities;
CREATE POLICY tenant_isolation_wf_legal_entities ON wf_legal_entities
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_locations ON wf_locations;
CREATE POLICY tenant_isolation_wf_locations ON wf_locations
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_departments ON wf_departments;
CREATE POLICY tenant_isolation_wf_departments ON wf_departments
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_teams ON wf_teams;
CREATE POLICY tenant_isolation_wf_teams ON wf_teams
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_positions ON wf_positions;
CREATE POLICY tenant_isolation_wf_positions ON wf_positions
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_employees ON wf_employees;
CREATE POLICY tenant_isolation_wf_employees ON wf_employees
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_employment_records ON wf_employment_records;
CREATE POLICY tenant_isolation_wf_employment_records ON wf_employment_records
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_compensation_records ON wf_compensation_records;
CREATE POLICY tenant_isolation_wf_compensation_records ON wf_compensation_records
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

DROP POLICY IF EXISTS tenant_isolation_wf_documents ON wf_documents;
CREATE POLICY tenant_isolation_wf_documents ON wf_documents
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

-- =============================================================================
-- UPDATED_AT TRIGGERS (only for the mutable tables)
-- =============================================================================
-- `update_updated_at_column()` is defined in 005. History and document tables are
-- append-only and deliberately excluded.

DROP TRIGGER IF EXISTS update_wf_legal_entities_updated_at ON wf_legal_entities;
CREATE TRIGGER update_wf_legal_entities_updated_at
    BEFORE UPDATE ON wf_legal_entities FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_wf_locations_updated_at ON wf_locations;
CREATE TRIGGER update_wf_locations_updated_at
    BEFORE UPDATE ON wf_locations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_wf_departments_updated_at ON wf_departments;
CREATE TRIGGER update_wf_departments_updated_at
    BEFORE UPDATE ON wf_departments FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_wf_teams_updated_at ON wf_teams;
CREATE TRIGGER update_wf_teams_updated_at
    BEFORE UPDATE ON wf_teams FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_wf_positions_updated_at ON wf_positions;
CREATE TRIGGER update_wf_positions_updated_at
    BEFORE UPDATE ON wf_positions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_wf_employees_updated_at ON wf_employees;
CREATE TRIGGER update_wf_employees_updated_at
    BEFORE UPDATE ON wf_employees FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- WORKFORCE PERMISSION SEEDS
-- =============================================================================
-- Granular keys consumed by `AuthContext::has_permission` (deny-by-default).
-- Compensation and export keys are separated from the general employee keys so a
-- role can read staff records without ever seeing pay.

INSERT INTO permissions (key, description, category, is_system) VALUES
    ('employee.read', 'Read employee records', 'workforce', FALSE),
    ('employee.write', 'Create and update employees', 'workforce', FALSE),
    ('employee.terminate', 'Terminate employees', 'workforce', FALSE),
    ('employee.export', 'Export employee data in bulk', 'workforce', FALSE),
    ('employee.compensation.read', 'Read compensation records (sensitive)', 'workforce', FALSE),
    ('employee.compensation.write', 'Write compensation records (sensitive)', 'workforce', FALSE),
    ('employee.documents.read', 'Read employee documents', 'workforce', FALSE),
    ('employee.documents.write', 'Upload employee documents', 'workforce', FALSE),
    ('department.read', 'Read departments', 'workforce', FALSE),
    ('department.write', 'Create and update departments', 'workforce', FALSE),
    ('position.read', 'Read job positions', 'workforce', FALSE),
    ('position.write', 'Create and update job positions', 'workforce', FALSE),
    ('location.read', 'Read locations', 'workforce', FALSE),
    ('location.write', 'Create and update locations', 'workforce', FALSE),
    ('legal_entity.read', 'Read legal entities', 'workforce', FALSE),
    ('legal_entity.write', 'Create and update legal entities', 'workforce', FALSE),
    ('team.read', 'Read teams', 'workforce', FALSE),
    ('team.write', 'Create and update teams', 'workforce', FALSE)
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    is_system = EXCLUDED.is_system;

-- =============================================================================
-- OUTBOX DISPATCHER JOB DEFINITION
-- =============================================================================
-- Registered but disabled: workforce lifecycle events are written to
-- `outbox_events` today, and this job publishes them onward once a broker
-- producer is wired. Defining it now keeps the contract visible.

INSERT INTO job_definitions (key, display_name, description, payload_schema, is_enabled)
VALUES (
    'workforce.outbox.dispatcher',
    'Workforce Outbox Dispatcher',
    'Publishes unpublished outbox_events rows with aggregate_type=workforce to the event broker',
    '{"type":"object","properties":{"batch_size":{"type":"integer","default":100}}}',
    FALSE
) ON CONFLICT (key) DO NOTHING;
