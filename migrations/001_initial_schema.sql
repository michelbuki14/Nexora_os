-- Migration: 001_initial_schema.sql
-- Description: Core AOS schema - tenants, organizations, users, audit log
-- Author: AOS Platform Team
-- Date: 2026-01-01

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Core tenant/organization tables
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'pending_verification', 'deleted')),
    tier TEXT NOT NULL DEFAULT 'free' CHECK (tier IN ('free', 'starter', 'professional', 'enterprise', 'custom')),
    parent_org_id UUID REFERENCES organizations(id),
    settings JSONB NOT NULL DEFAULT '{}',
    features JSONB NOT NULL DEFAULT '{}',
    quotas JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_organizations_slug ON organizations(slug);
CREATE INDEX IF NOT EXISTS idx_organizations_parent ON organizations(parent_org_id);

-- Tenants (child units within organizations)
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES organizations(id),
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'pending_verification', 'deleted')),
    tier TEXT NOT NULL DEFAULT 'free' CHECK (tier IN ('free', 'starter', 'professional', 'enterprise', 'custom')),
    settings JSONB NOT NULL DEFAULT '{}',
    features JSONB NOT NULL DEFAULT '{}',
    quotas JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (org_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_tenants_org ON tenants(org_id);
CREATE INDEX IF NOT EXISTS idx_tenants_slug ON tenants(org_id, slug);

-- Users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES organizations(id),
    tenant_id UUID REFERENCES tenants(id),
    email TEXT NOT NULL,
    phone TEXT,
    display_name TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'pending_verification', 'deleted')),
    keycloak_user_id TEXT UNIQUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (org_id, email)
);

CREATE INDEX IF NOT EXISTS idx_users_org ON users(org_id);
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_keycloak ON users(keycloak_user_id);

-- Roles
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES organizations(id),
    name TEXT NOT NULL,
    description TEXT,
    permissions TEXT[] NOT NULL DEFAULT '{}',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (org_id, name)
);

CREATE INDEX IF NOT EXISTS idx_roles_org ON roles(org_id);

-- User-Role assignments (through memberships)
CREATE TABLE IF NOT EXISTS memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    tenant_id UUID REFERENCES tenants(id),
    role_id UUID NOT NULL REFERENCES roles(id),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'pending', 'revoked')),
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE (user_id, org_id, role_id, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_memberships_user ON memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_memberships_org ON memberships(org_id);
CREATE INDEX IF NOT EXISTS idx_memberships_tenant ON memberships(tenant_id);

-- Feature flags per tenant
CREATE TABLE IF NOT EXISTS tenant_features (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    feature_key TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    config JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, feature_key)
);

-- Append-only audit log (immutable)
CREATE TABLE IF NOT EXISTS audit_events (
    id BIGSERIAL PRIMARY KEY,
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    actor_type TEXT NOT NULL CHECK (actor_type IN ('user', 'system', 'service', 'api_key')),
    actor_id UUID,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID,
    resource_ulid CHAR(26),
    changes JSONB,
    metadata JSONB NOT NULL DEFAULT '{}',
    request_id TEXT,
    correlation_id TEXT,
    ip_address INET,
    user_agent TEXT,
    result TEXT NOT NULL CHECK (result IN ('success', 'failure', 'partial')),
    error_message TEXT,
    hash CHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_tenant_created ON audit_events(tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_org_created ON audit_events(org_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_events(actor_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_events(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_correlation ON audit_events(correlation_id);
CREATE INDEX IF NOT EXISTS idx_audit_hash ON audit_events(hash);

-- Row-Level Security policies (tenant isolation)
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_features ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_events ENABLE ROW LEVEL SECURITY;

-- RLS policies require current_setting('aos.current_tenant_id', true) to be set
-- Application middleware must call SELECT set_config('aos.current_tenant_id', <tenant_ulid>, false)

CREATE POLICY tenant_isolation_organizations ON organizations
    USING (ulid = current_setting('aos.current_tenant_id', true) OR current_setting('aos.is_system', true) = 'true');

CREATE POLICY tenant_isolation_tenants ON tenants
    USING (ulid = current_setting('aos.current_tenant_id', true) OR current_setting('aos.is_system', true) = 'true');

CREATE POLICY tenant_isolation_users ON users
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('aos.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

CREATE POLICY tenant_isolation_roles ON roles
    USING (
        org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('aos.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

CREATE POLICY tenant_isolation_memberships ON memberships
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('aos.current_tenant_id', true))
        OR org_id = (SELECT org_id FROM tenants WHERE ulid = current_setting('aos.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

CREATE POLICY tenant_isolation_features ON tenant_features
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('aos.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );

CREATE POLICY tenant_isolation_audit ON audit_events
    USING (
        tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting('aos.current_tenant_id', true))
        OR current_setting('aos.is_system', true) = 'true'
    );