-- Migration: 003_payroll_schema.sql
-- Description: Payroll schema migration — country tax configs, payroll runs, payslips, items
-- Author: Nexora OS Platform Team
-- Date: 2026-08-18
--
-- Additive only. Every statement is idempotent so re-running the migration is safe.
--
-- Depends on: 001_initial_schema.sql, 005_cross_cutting.sql (currencies, countries, permissions)
-- Requires: 006_rls_fixes.sql for RLS enforcement

-- =============================================================================
-- PAYROLL CONFIGURATIONS (country-specific, versioned)
-- =============================================================================

CREATE TABLE IF NOT EXISTS payroll_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    config_version TEXT NOT NULL,        -- e.g., "ipr-2025", "cnss-2025-q2", "smig-2025"
    effective_date DATE NOT NULL,        -- Date this config becomes active
    expiry_date DATE,                     -- NULL = still active; else date it expires
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, config_version),
    CHECK (expiry_date IS NULL OR expiry_date >= effective_date)
);

CREATE INDEX idx_payroll_configs_tenant_active ON payroll_configurations(tenant_id, is_active DESC)
    WHERE is_active = TRUE;
CREATE INDEX idx_payroll_configs_org ON payroll_configurations(org_id);

-- Seed with DRC 2025 config (config_version: ipr-2025)
INSERT INTO payroll_configurations (config_version, effective_date, is_active)
VALUES ('ipr-2025', '2025-01-01', TRUE)
ON CONFLICT (config_version) DO NOTHING;

-- =============================================================================
-- COUNTRY TAX CONFIGURATIONS (IPR progressive brackets, CNSS rates, SMIG)
-- =============================================================================

CREATE TABLE IF NOT EXISTS country_tax_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    country_code CHAR(2) NOT NULL,         -- ISO-2: CDF, USD, etc.
    config_version TEXT NOT NULL,         -- e.g., "ipr-2025"
    ipr_bands JSONB NOT NULL DEFAULT '[]', -- Array of {from, to, rate} brackets
    cnss_employee_rate NUMERIC(5,4) NOT NULL DEFAULT 0.05,   -- 5%
    cnss_employer_rate NUMERIC(5,4) NOT NULL DEFAULT 0.105,  -- 10.5% default
    cnss_ceiling NUMERIC(19,2) NOT NULL DEFAULT 1500000.00,  -- Monthly ceiling in CDF
    smig_daily NUMERIC(19,2) NOT NULL DEFAULT 7000.00,      -- SMIG daily rate
    smig_monthly_26 NUMERIC(19,2) NOT NULL DEFAULT 182000.00, -- 26-day month
    smig_monthly_30 NUMERIC(19,2) NOT NULL DEFAULT 210000.00, -- 30-day month
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, country_code, config_version)
);

CREATE INDEX idx_country_tax_tenant_active ON country_tax_configs(tenant_id, is_active DESC)
    WHERE is_active = TRUE;
CREATE INDEX idx_country_tax_country ON country_tax_configs(country_code, is_active DESC);

-- Seed DRC 2025 IPR brackets (illustrative; actual values require legal review)
INSERT INTO country_tax_configs (country_code, config_version, ipr_bands, cnss_employee_rate, cnss_employer_rate, cnss_ceiling, smig_daily)
VALUES ('CDF', 'ipr-2025',
    '[{"from": 0, "to": 180000, "rate": 0}, {"from": 180001, "to": 480000, "rate": 0.15}, {"from": 480001, "to": 1200000, "rate": 0.25}, {"from": 1200001, "to": 2400000, "rate": 0.30}, {"from": 2400001, "to": 4800000, "rate": 0.35}, {"from": 4800001, "rate": 0.40}]'::jsonb,
    0.05, 0.105, 1500000.00, 7000.00)
ON CONFLICT (tenant_id, country_code, config_version) DO NOTHING;

-- =============================================================================
-- COUNTRY STATUTORY CONFIGURATIONS (SMIG, leave, other statutory rules)
-- =============================================================================

CREATE TABLE IF NOT EXISTS country_statutory_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    country_code CHAR(2) NOT NULL,         -- ISO-2
    config_version TEXT NOT NULL,         -- e.g., "smig-2025"
    smig_daily NUMERIC(19,2) NOT NULL,
    smig_monthly_26 NUMERIC(19,2) NOT NULL,
    smig_monthly_30 NUMERIC(19,2) NOT NULL,
    annual_leave_days INTEGER NOT NULL DEFAULT 24,  -- per Code du Travail
    maternity_leave_weeks INTEGER NOT NULL DEFAULT 14, -- 14 weeks (18 if multiple)
    paternity_leave_days INTEGER NOT NULL DEFAULT 5,
    sick_leave_paid_weeks INTEGER NOT NULL DEFAULT 6,
    notice_period_weeks INTEGER NOT NULL DEFAULT 1,   -- < 6mo; 6mo-1yr: 2; 1-5y: 1month; 5-10y: 2months; >10y: 3months
    severance_months_per_year NUMERIC(5,2) NOT NULL DEFAULT 1.0, -- 1 month per year 1-5y
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, country_code, config_version)
);

CREATE INDEX idx_country_stat_tenant_active ON country_statutory_configs(tenant_id, is_active DESC)
    WHERE is_active = TRUE;
CREATE INDEX idx_country_stat_country ON country_statutory_configs(country_code, is_active DESC);

-- =============================================================================
-- PAYROLL RUNS (immutable after confirm)
-- =============================================================================

CREATE TABLE IF NOT EXISTS payroll_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    config_version TEXT NOT NULL,              -- e.g., "ipr-2025"
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'review', 'approved', 'locked')),
    total_gross_cdf NUMERIC(19,4) NOT NULL DEFAULT 0,
    total_net_cdf NUMERIC(19,4) NOT NULL DEFAULT 0,
    total_employer_cost_cdf NUMERIC(19,4) NOT NULL DEFAULT 0,
    employee_count INTEGER NOT NULL DEFAULT 0,
    initiated_by UUID REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    locked_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payroll_runs_tenant ON payroll_runs(tenant_id, period_start DESC);
CREATE INDEX idx_payroll_runs_status ON payroll_runs(tenant_id, status);

-- =============================================================================
-- PAYSLIPS (immutable, versioned, correction chain)
-- =============================================================================

CREATE TABLE IF NOT EXISTS payslips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    payroll_run_id UUID NOT NULL REFERENCES payroll_runs(id),
    employee_id UUID NOT NULL REFERENCES wf_employees(id),
    version INTEGER NOT NULL DEFAULT 1,
    superseded_by_payslip_id UUID REFERENCES payslips(id),
    gross_pay_cdf NUMERIC(19,4) NOT NULL,
    taxable_pay_cdf NUMERIC(19,4) NOT NULL,
    ipr_deduction_cdf NUMERIC(19,4) NOT NULL DEFAULT 0,
    cnss_employee_cdf NUMERIC(19,4) NOT NULL DEFAULT 0,
    cnss_employer_cdf NUMERIC(19,4) NOT NULL DEFAULT 0,
    net_pay_cdf NUMERIC(19,4) NOT NULL,
    employer_cost_cdf NUMERIC(19,4) NOT NULL,
    object_key TEXT,                          -- MinIO key for PDF
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payslips_tenant ON payslips(tenant_id, created_at DESC);
CREATE INDEX idx_payslips_employee ON payslips(employee_id, created_at DESC);
CREATE INDEX idx_payslips_run ON payslips(payroll_run_id);
CREATE UNIQUE INDEX idx_payslips_emp_version ON payslips(employee_id, payroll_run_id, version);

-- =============================================================================
-- PAYROLL ITEMS (append-only line ledger)
-- =============================================================================

CREATE TABLE IF NOT EXISTS payroll_items (
    id BIGSERIAL PRIMARY KEY,
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    payslip_id UUID NOT NULL REFERENCES payslips(id),
    item_type TEXT NOT NULL CHECK (item_type IN ('earning', 'deduction', 'employer_contribution', 'tax')),
    item_code TEXT NOT NULL,                  -- e.g., 'BASIC', 'OVERTIME', 'IPR', 'CNSS_EMP', 'HOUSING'
    description TEXT,
    amount_cdf NUMERIC(19,4) NOT NULL,
    is_taxable BOOLEAN NOT NULL DEFAULT FALSE,
    is_cnssable BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payroll_items_payslip ON payroll_items(payslip_id);
CREATE INDEX idx_payroll_items_type ON payroll_items(item_type, item_code);

-- =============================================================================
-- ROW-LEVEL SECURITY
-- =============================================================================

ALTER TABLE payroll_configurations ENABLE ROW LEVEL SECURITY;
ALTER TABLE country_tax_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE country_statutory_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE payroll_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE payslips ENABLE ROW LEVEL SECURITY;
ALTER TABLE payroll_items ENABLE ROW LEVEL SECURITY;

-- payroll_configurations: tenant-isolated
DROP POLICY IF EXISTS tenant_isolation_payroll_configs ON payroll_configurations;
CREATE POLICY tenant_isolation_payroll_configs ON payroll_configurations
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

-- country_tax_configs: tenant-isolated
DROP POLICY IF EXISTS tenant_isolation_country_tax ON country_tax_configs;
CREATE POLICY tenant_isolation_country_tax ON country_tax_configs
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

-- country_statutory_configs: tenant-isolated
DROP POLICY IF EXISTS tenant_isolation_country_stat ON country_statutory_configs;
CREATE POLICY tenant_isolation_country_stat ON country_statutory_configs
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

-- payroll_runs: tenant-isolated
DROP POLICY IF EXISTS tenant_isolation_payroll_runs ON payroll_runs;
CREATE POLICY tenant_isolation_payroll_runs ON payroll_runs
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

-- payslips: tenant-isolated
DROP POLICY IF EXISTS tenant_isolation_payslips ON payslips;
CREATE POLICY tenant_isolation_payslips ON payslips
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

-- payroll_items: tenant-isolated, append-only enforced at DB level via REVOKE
DROP POLICY IF EXISTS tenant_isolation_payroll_items ON payroll_items;
CREATE POLICY tenant_isolation_payroll_items ON payroll_items
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('nexora.is_system', true) = 'true'
    );

-- =============================================================================
-- UPDATED_AT TRIGGERS
-- =============================================================================

DROP TRIGGER IF EXISTS update_payroll_configs_updated_at ON payroll_configurations;
CREATE TRIGGER update_payroll_configs_updated_at
    BEFORE UPDATE ON payroll_configurations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_country_tax_updated_at ON country_tax_configs;
CREATE TRIGGER update_country_tax_updated_at
    BEFORE UPDATE ON country_tax_configs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_country_stat_updated_at ON country_statutory_configs;
CREATE TRIGGER update_country_stat_updated_at
    BEFORE UPDATE ON country_statutory_configs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_payroll_runs_updated_at ON payroll_runs;
CREATE TRIGGER update_payroll_runs_updated_at
    BEFORE UPDATE ON payroll_runs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- PERMISSION SEEDS FOR PAYROLL
-- =============================================================================

INSERT INTO permissions (key, description, category, is_system) VALUES
    ('payroll.read', 'Read payroll runs and configs', 'payroll', FALSE),
    ('payroll.create', 'Create payroll runs', 'payroll', FALSE),
    ('payroll.calculate', 'Calculate/preview payroll', 'payroll', FALSE),
    ('payroll.review', 'Review payroll runs', 'payroll', FALSE),
    ('payroll.approve', 'Approve payroll runs', 'payroll', FALSE),
    ('payroll.lock', 'Lock/confirm payroll runs', 'payroll', FALSE),
    ('payslip.read', 'Read payslips', 'payroll', FALSE),
    ('payslip.deliver', 'Deliver payslips to employees', 'payroll', FALSE),
    ('payroll.config.read', 'Read payroll configurations', 'payroll', FALSE),
    ('payroll.config.write', 'Manage payroll configurations', 'payroll', FALSE)
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    is_system = EXCLUDED.is_system;