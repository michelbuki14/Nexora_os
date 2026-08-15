-- migrate:no-transaction
-- Migration: 006_rls_fixes.sql
-- Description: Fix RLS policies for existing tables - add WITH CHECK, fix organizations policy
-- Author: AOS Platform Team
-- Date: 2026-08-07

-- =============================================================================
-- RLS FIXES FOR EXISTING TABLES (from 001_initial_schema)
-- =============================================================================

-- Drop and recreate policies with WITH CHECK clauses

-- organizations: org-isolated (not tenant-isolated). Visible to system + org members.
DROP POLICY IF EXISTS tenant_isolation_organizations ON organizations;
CREATE POLICY org_isolation_organizations ON organizations
    USING (
        id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- tenants: tenant-isolated
DROP POLICY IF EXISTS tenant_isolation_tenants ON tenants;
CREATE POLICY tenant_isolation_tenants ON tenants
    USING (
        ulid = current_setting('nexora.current_tenant_id', true)
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        ulid = current_setting('nexora.current_tenant_id', true)
        OR current_setting('aos.is_system', true) = 'true'
    );

-- users: tenant-isolated for tenant-scoped users, org-isolated for org-level users (tenant_id IS NULL)
DROP POLICY IF EXISTS tenant_isolation_users ON users;
CREATE POLICY tenant_isolation_users ON users
    USING (
        (tenant_id IS NOT NULL AND tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)))
        OR (tenant_id IS NULL AND org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        (tenant_id IS NOT NULL AND tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)))
        OR (tenant_id IS NULL AND org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- roles: org-isolated
DROP POLICY IF EXISTS tenant_isolation_roles ON roles;
CREATE POLICY org_isolation_roles ON roles
    USING (
        org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- memberships: tenant + org isolated
DROP POLICY IF EXISTS tenant_isolation_memberships ON memberships;
CREATE POLICY tenant_isolation_memberships ON memberships
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- tenant_features: tenant-isolated
DROP POLICY IF EXISTS tenant_isolation_features ON tenant_features;
CREATE POLICY tenant_isolation_features ON tenant_features
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- audit_events: tenant-isolated with append-only enforcement
DROP POLICY IF EXISTS tenant_isolation_audit ON audit_events;
CREATE POLICY tenant_isolation_audit ON audit_events
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

-- =============================================================================
-- APPEND-ONLY ENFORCEMENT FOR audit_events
-- =============================================================================
-- REVOKE UPDATE/DELETE for application roles (roles created in P1-AUTH-03).
-- The migration runner executes as a superuser, so we create a helper function
-- that can be called by the application role setup.
-- Actual REVOKE will be done by the roles migration (007 or P1-AUTH-03).
-- Here we just ensure the table structure supports it.

-- No structural changes needed; REVOKE is a DCL statement run separately.

-- =============================================================================
-- GUC DEFAULTS (set at database level for safety)
-- =============================================================================

-- These ensure the GUCs exist and have safe defaults even if middleware fails.
-- The middleware overrides per-request via set_config(..., false). ALTER DATABASE
-- cannot run inside a transaction block (hence `migrate:no-transaction` above) and
-- requires a concrete database name, so it is executed via dynamic SQL against
-- current_database().
DO $$
DECLARE
    db_name text := current_database();
BEGIN
    EXECUTE format('ALTER DATABASE %I SET nexora.current_tenant_id = %L', db_name, '');
    EXECUTE format('ALTER DATABASE %I SET aos.is_system = %L', db_name, 'false');
END
$$;