-- Migration: 012_consumer_services.sql
-- Description: Consumer-facing verticals — fintech (wallets/transactions/payments),
--              retail (products/orders/inventory), government (permits/licenses/records).
-- Author: Nexora OS Platform Team
-- Date: 2026-08-28
--
-- Additive only. Every statement is idempotent so re-running the migration is safe.
-- Conventions inherited from 002/005/006:
--   * UUID primary keys, CHAR(26) ULIDs as the external id.
--   * tenant_id/org_id on every row table, isolated by RLS against the
--     nexora.current_tenant_id GUC that rls_middleware sets per request.
--   * Reference data (currencies, countries, permissions) is shared, not copied.

-- =============================================================================
-- FINTECH: WALLETS
-- =============================================================================

CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    owner_ulid CHAR(26) NOT NULL,
    currency TEXT NOT NULL DEFAULT 'CDF',
    balance NUMERIC(24, 4) NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'frozen', 'closed', 'pending')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wallets_tenant ON wallets(tenant_id);
CREATE INDEX IF NOT EXISTS idx_wallets_owner ON wallets(owner_ulid);

-- =============================================================================
-- FINTECH: TRANSACTIONS
-- =============================================================================

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    wallet_ulid CHAR(26) NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('credit', 'debit')),
    amount NUMERIC(24, 4) NOT NULL,
    status TEXT NOT NULL DEFAULT 'settled' CHECK (status IN ('pending', 'settled', 'failed', 'reversed')),
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_tenant ON transactions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_transactions_wallet ON transactions(wallet_ulid, created_at DESC);

-- =============================================================================
-- FINTECH: PAYMENTS
-- =============================================================================

CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    wallet_ulid CHAR(26) NOT NULL,
    payee TEXT NOT NULL,
    amount NUMERIC(24, 4) NOT NULL,
    method TEXT NOT NULL DEFAULT 'card' CHECK (method IN ('card', 'mobile_money', 'bank_transfer', 'cash')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'settled', 'failed', 'refunded')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payments_tenant ON payments(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payments_wallet ON payments(wallet_ulid);

-- =============================================================================
-- RETAIL: PRODUCTS
-- =============================================================================

CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    name TEXT NOT NULL,
    sku TEXT NOT NULL,
    price NUMERIC(24, 4) NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'CDF',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_products_tenant ON products(tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_products_tenant_sku ON products(tenant_id, sku) WHERE sku IS NOT NULL;

-- =============================================================================
-- RETAIL: ORDERS
-- =============================================================================

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    customer_ulid CHAR(26) NOT NULL,
    channel TEXT NOT NULL DEFAULT 'web',
    total NUMERIC(24, 4) NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'fulfilled', 'shipped', 'cancelled', 'refunded')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_tenant ON orders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_orders_customer ON orders(customer_ulid);

-- =============================================================================
-- RETAIL: INVENTORY
-- =============================================================================

CREATE TABLE IF NOT EXISTS inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    product_ulid CHAR(26) NOT NULL REFERENCES products(ulid),
    sku TEXT NOT NULL,
    available BIGINT NOT NULL DEFAULT 0,
    reserved BIGINT NOT NULL DEFAULT 0,
    warehouse TEXT NOT NULL DEFAULT 'default',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inventory_tenant ON inventory(tenant_id);
CREATE INDEX IF NOT EXISTS idx_inventory_product ON inventory(product_ulid);

-- =============================================================================
-- GOVERNMENT: PERMITS
-- =============================================================================

CREATE TABLE IF NOT EXISTS permits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    applicant_ulid CHAR(26) NOT NULL,
    permit_type TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'submitted', 'approved', 'rejected')),
    submitted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_permits_tenant ON permits(tenant_id);
CREATE INDEX IF NOT EXISTS idx_permits_applicant ON permits(applicant_ulid);

-- =============================================================================
-- GOVERNMENT: LICENSES
-- =============================================================================

CREATE TABLE IF NOT EXISTS licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    holder_ulid CHAR(26) NOT NULL,
    license_type TEXT NOT NULL,
    holder_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('active', 'expired', 'revoked', 'pending')),
    issued_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_licenses_tenant ON licenses(tenant_id);
CREATE INDEX IF NOT EXISTS idx_licenses_holder ON licenses(holder_ulid);

-- =============================================================================
-- GOVERNMENT: CIVIL RECORDS
-- =============================================================================

CREATE TABLE IF NOT EXISTS civil_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ulid CHAR(26) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL REFERENCES organizations(id),
    record_type TEXT NOT NULL CHECK (record_type IN ('birth', 'marriage', 'death', 'national_id', 'custom')),
    subject_name TEXT NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    jurisdiction TEXT NOT NULL DEFAULT 'national',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_civil_records_tenant ON civil_records(tenant_id);
CREATE INDEX IF NOT EXISTS idx_civil_records_type ON civil_records(record_type);

-- =============================================================================
-- ROW-LEVEL SECURITY
-- =============================================================================
-- Same shape as 002/005/006: readable and writable only within the caller's
-- tenant, with an explicit WITH CHECK so a caller cannot write a row into
-- another tenant. Policies are dropped first so re-running converges.

ALTER TABLE wallets ENABLE ROW LEVEL SECURITY;
ALTER TABLE transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;
ALTER TABLE products ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE inventory ENABLE ROW LEVEL SECURITY;
ALTER TABLE permits ENABLE ROW LEVEL SECURITY;
ALTER TABLE licenses ENABLE ROW LEVEL SECURITY;
ALTER TABLE civil_records ENABLE ROW LEVEL SECURITY;

DO $$
DECLARE
    t text;
    tables text[] := ARRAY[
        'wallets', 'transactions', 'payments', 'products',
        'orders', 'inventory', 'permits', 'licenses', 'civil_records'
    ];
BEGIN
    FOREACH t IN ARRAY tables LOOP
        EXECUTE format('DROP POLICY IF EXISTS tenant_isolation_%1$s ON %1$s', t);
        EXECUTE format(
            'CREATE POLICY tenant_isolation_%1$s ON %1$s '
            'USING (tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting(''nexora.current_tenant_id'', true)) '
            '   OR current_setting(''nexora.is_system'', true) = ''true'') '
            'WITH CHECK (tenant_id = (SELECT id FROM tenants WHERE ulid = current_setting(''nexora.current_tenant_id'', true)) '
            '   OR current_setting(''nexora.is_system'', true) = ''true'')',
            t
        );
    END LOOP;
END
$$;

-- =============================================================================
-- UPDATED_AT TRIGGERS (mutable tables only)
-- =============================================================================

DROP TRIGGER IF EXISTS update_wallets_updated_at ON wallets;
CREATE TRIGGER update_wallets_updated_at BEFORE UPDATE ON wallets FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_payments_updated_at ON payments;
CREATE TRIGGER update_payments_updated_at BEFORE UPDATE ON payments FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_products_updated_at ON products;
CREATE TRIGGER update_products_updated_at BEFORE UPDATE ON products FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_orders_updated_at ON orders;
CREATE TRIGGER update_orders_updated_at BEFORE UPDATE ON orders FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_inventory_updated_at ON inventory;
CREATE TRIGGER update_inventory_updated_at BEFORE UPDATE ON inventory FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_permits_updated_at ON permits;
CREATE TRIGGER update_permits_updated_at BEFORE UPDATE ON permits FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_licenses_updated_at ON licenses;
CREATE TRIGGER update_licenses_updated_at BEFORE UPDATE ON licenses FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- CONSUMER PERMISSION SEEDS
-- =============================================================================

INSERT INTO permissions (key, description, category, is_system) VALUES
    ('fintech.wallet.read', 'Read consumer wallets', 'fintech', FALSE),
    ('fintech.wallet.write', 'Create and manage consumer wallets', 'fintech', FALSE),
    ('fintech.transaction.read', 'Read consumer transactions', 'fintech', FALSE),
    ('fintech.transaction.write', 'Create consumer transactions', 'fintech', FALSE),
    ('fintech.payment.read', 'Read consumer payments', 'fintech', FALSE),
    ('fintech.admin', 'Administrative access to fintech', 'fintech', FALSE),
    ('retail.product.read', 'Read retail catalog', 'retail', FALSE),
    ('retail.product.write', 'Manage retail catalog', 'retail', FALSE),
    ('retail.order.read', 'Read retail orders', 'retail', FALSE),
    ('retail.order.write', 'Create retail orders', 'retail', FALSE),
    ('retail.inventory.read', 'Read retail inventory', 'retail', FALSE),
    ('retail.admin', 'Administrative access to retail', 'retail', FALSE),
    ('gov.permit.read', 'Read permit applications', 'government', FALSE),
    ('gov.permit.write', 'Submit and manage permit applications', 'government', FALSE),
    ('gov.license.read', 'Read licenses', 'government', FALSE),
    ('gov.license.write', 'Issue and manage licenses', 'government', FALSE),
    ('gov.record.read', 'Read civil records', 'government', FALSE),
    ('gov.admin', 'Administrative access to government services', 'government', FALSE)
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    is_system = EXCLUDED.is_system;
