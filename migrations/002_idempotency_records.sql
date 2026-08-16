-- Migration: 002_idempotency_records.sql
-- Description: Idempotency records table for deduplicating payment/payroll requests
-- Author: Nexora Os Team
-- Date: 2026-08-15

-- Create the idempotency_records table
CREATE TABLE IF NOT EXISTS idempotency_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    requestor_id UUID NOT NULL,       -- The user/service that made the request
    idempotency_key UUID NOT NULL,    -- Unique key identifying the idempotent operation
    operation TEXT NOT NULL,          -- Type of operation (e.g., "create_payment", "create_payroll_run")
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'failed')),
    response_payload JSONB NOT NULL DEFAULT '{}',  -- Cached response payload
    response_status_code SMALLINT,  -- HTTP status code from the original operation
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    UNIQUE (tenant_id, idempotency_key, operation),
    CHECK (char_length(idempotency_key) = 36)  -- UUID format
);

-- Create indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_idempotency_tenant_key ON idempotency_records(tenant_id, idempotency_key);
CREATE INDEX IF NOT EXISTS idx_idempotency_operation ON idempotency_records(operation);
CREATE INDEX IF NOT EXISTS idx_idempotency_status ON idempotency_records(status);
CREATE INDEX IF NOT EXISTS idx_idempotency_completed ON idempotency_records(completed_at);

-- Grant permissions
GRANT SELECT, INSERT, UPDATE ON idempotency_records TO service_role;
ALTER TABLE idempotency_records ENABLE ROW LEVEL SECURITY;

-- Row-level security: tenants can only see their own records
CREATE POLICY tenant_isolation ON idempotency_records
    USING (tenant_id IN (
        SELECT id FROM tenants WHERE org_id = current_setting('nexora.current_tenant_id')::uuid
    ));

-- Index for pruning old records (keep 7 days)
CREATE INDEX IF NOT EXISTS idx_idempotency_prune ON idempotency_records(updated_at)
    WHERE status = 'pending' AND updated_at < NOW() - INTERVAL '7 days';

COMMENT ON TABLE idempotency_records IS 'Append-only idempotency records for deduplicating payment and payroll operations';
COMMENT ON COLUMN idempotency_records.operation IS 'Type of idempotent operation (create_payment, create_payroll_run, etc.)';
COMMENT ON COLUMN idempotency_records.idempotency_key IS 'Unique key identifying the idempotent operation, provided by the client';