-- Migration: 005_cross_cutting.sql
-- Description: Cross-cutting infrastructure tables - currencies, permissions, outbox, jobs
-- Author: AOS Platform Team
-- Date: 2026-08-07

-- =============================================================================
-- CURRENCIES & EXCHANGE RATES
-- =============================================================================

CREATE TABLE IF NOT EXISTS currencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code CHAR(3) NOT NULL UNIQUE,
    name TEXT NOT NULL,
    symbol TEXT,
    decimal_places SMALLINT NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_currencies_code ON currencies(code);

CREATE TABLE IF NOT EXISTS exchange_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_currency_id UUID NOT NULL REFERENCES currencies(id),
    to_currency_id UUID NOT NULL REFERENCES currencies(id),
    rate NUMERIC(38, 18) NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (from_currency_id, to_currency_id, valid_from)
);

CREATE INDEX idx_exchange_rates_from_to ON exchange_rates(from_currency_id, to_currency_id);
CREATE INDEX idx_exchange_rates_validity ON exchange_rates(valid_from, valid_until);

-- Seed base currencies (DRC + major trading partners)
INSERT INTO currencies (code, name, symbol, decimal_places, is_active) VALUES
    ('CDF', 'Congolese Franc', 'FC', 0, TRUE),
    ('USD', 'US Dollar', '$', 2, TRUE),
    ('EUR', 'Euro', '€', 2, TRUE),
    ('ZAR', 'South African Rand', 'R', 2, TRUE),
    ('KES', 'Kenyan Shilling', 'KSh', 2, TRUE),
    ('NGN', 'Nigerian Naira', '₦', 2, TRUE),
    ('GHS', 'Ghanaian Cedi', '₵', 2, TRUE)
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    symbol = EXCLUDED.symbol,
    decimal_places = EXCLUDED.decimal_places,
    is_active = EXCLUDED.is_active;

-- =============================================================================
-- COUNTRIES (for payroll/tax localization)
-- =============================================================================

CREATE TABLE IF NOT EXISTS countries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    iso2 CHAR(2) NOT NULL UNIQUE,
    iso3 CHAR(3) NOT NULL UNIQUE,
    name TEXT NOT NULL,
    currency_id UUID REFERENCES currencies(id),
    timezone TEXT NOT NULL DEFAULT 'UTC',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_countries_iso2 ON countries(iso2);

-- Seed DRC
INSERT INTO countries (iso2, iso3, name, currency_id, timezone)
SELECT 'CD', 'COD', 'Democratic Republic of the Congo', id, 'Africa/Lubumbashi'
FROM currencies WHERE code = 'CDF'
ON CONFLICT (iso2) DO NOTHING;

-- =============================================================================
-- PERMISSIONS (granular RBAC)
-- =============================================================================

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL UNIQUE,
    description TEXT,
    category TEXT NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_permissions_category ON permissions(category);

-- Seed core permissions
INSERT INTO permissions (key, description, category, is_system) VALUES
    -- Identity
    ('role.read', 'Read roles', 'identity', TRUE),
    ('role.create', 'Create roles', 'identity', TRUE),
    ('role.update', 'Update roles', 'identity', TRUE),
    ('role.delete', 'Delete roles', 'identity', TRUE),
    ('permission.read', 'Read permissions', 'identity', TRUE),
    ('assignment.read', 'Read role assignments', 'identity', TRUE),
    ('assignment.create', 'Create role assignments', 'identity', TRUE),
    ('assignment.revoke', 'Revoke role assignments', 'identity', TRUE),
    -- Organization/Tenant
    ('organization.read', 'Read organizations', 'organization', TRUE),
    ('organization.create', 'Create organizations', 'organization', TRUE),
    ('organization.update', 'Update organizations', 'organization', TRUE),
    ('organization.delete', 'Delete organizations', 'organization', TRUE),
    ('tenant.read', 'Read tenants', 'organization', TRUE),
    ('tenant.create', 'Create tenants', 'organization', TRUE),
    ('tenant.update', 'Update tenants', 'organization', TRUE),
    ('tenant.delete', 'Delete tenants', 'organization', TRUE),
    -- Feature flags
    ('feature.read', 'Read feature flags', 'organization', TRUE),
    ('feature.update', 'Update feature flags', 'organization', TRUE),
    -- Audit
    ('audit.read', 'Read audit events', 'audit', TRUE),
    ('audit.export', 'Export audit events', 'audit', TRUE)
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    is_system = EXCLUDED.is_system;

-- =============================================================================
-- OUTBOX EVENTS (transactional outbox pattern)
-- =============================================================================

CREATE TABLE IF NOT EXISTS outbox_events (
    id BIGSERIAL PRIMARY KEY,
    ulid CHAR(26) NOT NULL UNIQUE,
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    aggregate_ulid CHAR(26),
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    published_by TEXT
);

CREATE INDEX idx_outbox_tenant_created ON outbox_events(tenant_id, created_at);
CREATE INDEX idx_outbox_org_created ON outbox_events(org_id, created_at);
CREATE INDEX idx_outbox_unpublished ON outbox_events(published_at) WHERE published_at IS NULL;
CREATE INDEX idx_outbox_aggregate ON outbox_events(aggregate_type, aggregate_id);

-- =============================================================================
-- IDEMPOTENCY RECORDS (deduplication for financial operations)
-- =============================================================================

CREATE TABLE IF NOT EXISTS idempotency_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key TEXT NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    request_hash CHAR(64) NOT NULL,
    response_status SMALLINT,
    response_body JSONB,
    response_headers JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (tenant_id, idempotency_key)
);

CREATE INDEX idx_idempotency_expires ON idempotency_records(expires_at);
CREATE INDEX idx_idempotency_tenant_created ON idempotency_records(tenant_id, created_at DESC);

-- =============================================================================
-- BACKGROUND JOBS (Redis Streams dispatcher + SQL persistence)
-- =============================================================================

CREATE TABLE IF NOT EXISTS job_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    schedule_cron TEXT,
    payload_schema JSONB,
    timeout_seconds INT NOT NULL DEFAULT 300,
    max_retries SMALLINT NOT NULL DEFAULT 3,
    retry_backoff_seconds INT[] NOT NULL DEFAULT '{30,300,3600}',
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_definitions_enabled ON job_definitions(is_enabled);

CREATE TABLE IF NOT EXISTS job_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_definition_id UUID NOT NULL REFERENCES job_definitions(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    idempotency_key TEXT,
    payload JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'dead_letter')),
    attempt SMALLINT NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_runs_tenant_status ON job_runs(tenant_id, status, created_at DESC);
CREATE INDEX idx_job_runs_definition_status ON job_runs(job_definition_id, status);
CREATE INDEX idx_job_runs_idempotency ON job_runs(idempotency_key) WHERE idempotency_key IS NOT NULL;

-- =============================================================================
-- ROW-LEVEL SECURITY FOR NEW TABLES
-- =============================================================================

ALTER TABLE currencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE exchange_rates ENABLE ROW LEVEL SECURITY;
ALTER TABLE countries ENABLE ROW LEVEL SECURITY;
ALTER TABLE permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE outbox_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE idempotency_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE job_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE job_runs ENABLE ROW LEVEL SECURITY;

-- Reference data (currencies, countries, permissions) - globally readable, system-writable
CREATE POLICY global_read_currencies ON currencies
    USING (true)
    WITH CHECK (current_setting('aos.is_system', true) = 'true');

CREATE POLICY global_read_exchange_rates ON exchange_rates
    USING (true)
    WITH CHECK (current_setting('aos.is_system', true) = 'true');

CREATE POLICY global_read_countries ON countries
    USING (true)
    WITH CHECK (current_setting('aos.is_system', true) = 'true');

CREATE POLICY global_read_permissions ON permissions
    USING (true)
    WITH CHECK (current_setting('aos.is_system', true) = 'true');

-- Outbox events - tenant isolated
CREATE POLICY tenant_isolation_outbox ON outbox_events
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- Idempotency records - tenant isolated
CREATE POLICY tenant_isolation_idempotency ON idempotency_records
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- Job definitions - globally readable, system-writable
CREATE POLICY global_read_job_definitions ON job_definitions
    USING (true)
    WITH CHECK (current_setting('aos.is_system', true) = 'true');

-- Job runs - tenant isolated
CREATE POLICY tenant_isolation_job_runs ON job_runs
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- =============================================================================
-- UPDATED_AT TRIGGERS
-- =============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END $$;

CREATE TRIGGER update_currencies_updated_at
    BEFORE UPDATE ON currencies FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_exchange_rates_updated_at
    BEFORE UPDATE ON exchange_rates FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_countries_updated_at
    BEFORE UPDATE ON countries FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_job_definitions_updated_at
    BEFORE UPDATE ON job_definitions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_job_runs_updated_at
    BEFORE UPDATE ON job_runs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();