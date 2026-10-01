-- Migration: 011_revoke_append_only.sql
-- Description: Enforce append-only at database level by REVOKE UPDATE/DELETE on sensitive tables
-- Author: Nexora OS Platform Team
-- Date: 2026-08-18

-- This migration runs as a superuser (the migration runner or container bootstrap).
-- It revokes UPDATE and DELETE privileges from the application role 'aos_app'
-- on tables that must be append-only for audit/compliance:
--   - audit_events: tamper-evident audit log
--   - wf_employment_records: employment history (payroll reproducibility)
--   - wf_compensation_records: compensation history (payroll reproducibility)
--   - wf_documents: document metadata (integrity via SHA-256 + presigned URLs)
--   - outbox_events: event sourcing (must not be modified after publish)
--   - idempotency_records: deduplication (must not be modified after completion)

-- Ensure we only act when run by a superuser (same pattern as 007_app_role.sql)
DO $$
BEGIN
    IF NOT (SELECT rolsuper FROM pg_roles WHERE rolname = current_user) THEN
        RAISE NOTICE 'REVOKE append-only skipped: current_user (%) is not a superuser', current_user;
    ELSE
        -- Revoke UPDATE and DELETE on audit_events
        IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'audit_events') THEN
            REVOKE UPDATE, DELETE ON audit_events FROM aos_app;
            RAISE NOTICE 'REVOKEd UPDATE, DELETE on audit_events from aos_app';
        END IF;

        -- Revoke UPDATE and DELETE on workforce history tables
        IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'wf_employment_records') THEN
            REVOKE UPDATE, DELETE ON wf_employment_records FROM aos_app;
            RAISE NOTICE 'REVOKEd UPDATE, DELETE on wf_employment_records from aos_app';
        END IF;

        IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'wf_compensation_records') THEN
            REVOKE UPDATE, DELETE ON wf_compensation_records FROM aos_app;
            RAISE NOTICE 'REVOKEd UPDATE, DELETE on wf_compensation_records from aos_app';
        END IF;

        IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'wf_documents') THEN
            REVOKE UPDATE, DELETE ON wf_documents FROM aos_app;
            RAISE NOTICE 'REVOKEd UPDATE, DELETE on wf_documents from aos_app';
        END IF;

        -- Revoke UPDATE and DELETE on outbox_events
        IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'outbox_events') THEN
            REVOKE UPDATE, DELETE ON outbox_events FROM aos_app;
            RAISE NOTICE 'REVOKEd UPDATE, DELETE on outbox_events from aos_app';
        END IF;

        -- Revoke UPDATE and DELETE on idempotency_records
        IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'idempotency_records') THEN
            REVOKE UPDATE, DELETE ON idempotency_records FROM aos_app;
            RAISE NOTICE 'REVOKEd UPDATE, DELETE on idempotency_records from aos_app';
        END IF;

        -- Note: We do NOT revoke on wf_employees, wf_departments, etc. because those
        -- are mutable reference data (employees change departments, get promoted, etc.)
        -- The history tables (wf_employment_records, wf_compensation_records) are the
        -- append-only audit trail for those changes.
    END IF;
END
$$;