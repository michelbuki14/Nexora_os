-- migrate:no-transaction
-- Bootstrap a non-superuser application role so row-level security is enforced.
--
-- PostgreSQL superusers bypass row-level security, so if the runtime services
-- connect as the `aos` superuser (the default POSTGRES_USER), the `nexora.current_tenant_id`
-- / `aos.is_system` GUCs are ignored and every request sees every tenant's rows.
--
-- This migration creates `aos_app`, a LOGIN role with only DML privileges on the
-- public schema, and grants it access to future tables/sequences too (via
-- ALTER DEFAULT PRIVILEGES, applied to objects owned by the migration user).
-- Because `aos_app` is neither a superuser nor a table owner, RLS policies apply
-- to every query it runs.
--
-- The block is idempotent and self-guarding: it only acts when run by a superuser
-- (the initial container bootstrap or `aos-migrate`). When a migration run is
-- invoked as `aos_app` itself (role already bootstrapped), the block no-ops, so
-- running migrations does not require superuser credentials post-bootstrap.

DO $$
BEGIN
    IF NOT (SELECT rolsuper FROM pg_roles WHERE rolname = current_user) THEN
        RAISE NOTICE 'aos_app bootstrap skipped: current_user (%) is not a superuser', current_user;
    ELSE
        IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'aos_app') THEN
            CREATE ROLE aos_app LOGIN PASSWORD 'aos_app_dev_password'
                NOSUPERUSER NOCREATEDB NOCREATEROLE;
            RAISE NOTICE 'created application role aos_app';
        END IF;

        EXECUTE format('GRANT CONNECT ON DATABASE %I TO aos_app', current_database());
        GRANT USAGE ON SCHEMA public TO aos_app;
        GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO aos_app;
        GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO aos_app;
    END IF;
END
$$;

-- Default privileges apply to objects created by the migration user *after* this
-- migration runs; guarded the same way so post-bootstrap runs by aos_app no-op.
DO $$
BEGIN
    IF (SELECT rolsuper FROM pg_roles WHERE rolname = current_user) THEN
        ALTER DEFAULT PRIVILEGES IN SCHEMA public
            GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO aos_app;
        ALTER DEFAULT PRIVILEGES IN SCHEMA public
            GRANT USAGE, SELECT ON SEQUENCES TO aos_app;
    END IF;
END
$$;
