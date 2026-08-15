# Workforce Module — Architecture Assessment

> Phase 1, backend-only. Nexora OS has no frontend crate, so the Workforce prompt's
> portal/UI requirements are satisfied by this OpenAPI spec + data-contract
> docs, not by a built UI.

## What Nexora OS already provided (reused, not reinvented)

| Capability | Where it lives | How Workforce uses it |
|------------|----------------|------------------------|
| Keycloak OIDC auth | `common::auth_middleware` | unchanged — `AuthContext` carries tenant/org/user ULIDs + roles + permissions |
| RLS tenant isolation | `common::tenant_context::rls_middleware` + migrations 005/006/007 | every `wf_*` table gets the same `current_setting('nexora.current_tenant_id')` policy with `WITH CHECK` |
| Hash-chained audit | `audit_events` + `common::audit::{compute_chain_hash, GENESIS_HASH}` | `audit_emit.rs` writes a chained row on every lifecycle change |
| Transactional outbox | `outbox_events` (migration 005) | workforce lifecycle events dual-written in the same tx; future dispatcher job publishes |
| Granular RBAC keys | `permissions` table (migration 005) | migration 008 seeds 18 `category='workforce'` keys |
| Job definitions | `job_definitions` (migration 005) | 008 registers `workforce.outbox.dispatcher` (disabled) |
| Currencies | `currencies` (migration 005, CDF/USD/EUR/ZAR/KES/NGN/GHS) | compensation FK to `currencies`, amounts in minor units |
| Countries | `countries` (migration 005, DRC seeded) | legal entity country FK |
| Money policy | `common::money` (NUMERIC(19,4), no floats) | compensation stored as `BIGINT` minor units — integer, never float |
| Error/response shape | `common::error::{AosError, ErrorResponse}` | workforce handlers return `AosResult` |
| Config/observability | `common::config::{Config, S3Config}` + tracing | workforce binary mirrors `audit-service/main.rs` wiring |
| Integration-test harness | testcontainers pattern in `common/tests/tenant_isolation.rs` | mirrored in `workforce-service/tests/workforce_isolation.rs` |

## What Workforce adds new

- **9 tables** in migration 008 (`wf_legal_entities`, `wf_locations`, `wf_departments`,
  `wf_teams`, `wf_positions`, `wf_employees`, `wf_employment_records`,
  `wf_compensation_records`, `wf_documents`) — all tenant/org-scoped, RLS-enabled.
- **1 new crate** `services/workforce-service` — same shape as `audit-service`.
- **2 new columns on common** — `S3Config` added to `Config`; `s3.rs` wraps the AWS SDK S3 client (MinIO-compatible).
- **18 new permission keys** seeded into the shared `permissions` table.
- **1 gateway nest** — `/api/v1/workforce` (currently a placeholder; the workforce service runs on port 3002 with its own auth+RLS stack).

## What is deliberately NOT built this phase

- **Frontend / portals** — Nexora OS has no design system yet. OpenAPI spec + `portal-map.md` describe what the portals would call.
- **Payroll engine** — documented in `payroll-contract.md`; built in Phase 4.
- **Leave / attendance / approvals** — Phase 3.
- **Notifications provider** — abstraction documented; Mailhog runs in compose but no Rust mailer (Phase 3).
- **RS-256 / Redpanda producer** — outbox dispatcher job is *registered* but *disabled*; no consumer this phase.
- **AI** — Phase 6, hard-gated behind Phases 1-4 stable. Never lets an LLM bypass authorization.

## Carried-forward security gaps (honest ledger)

1. **`audit_events` UPDATE/DELETE REVOKE** — migration 006 explicitly deferred this to "007 or P1-AUTH-03"; migration 007 does not contain it. Append-only is currently **application-enforced only**. This must be closed before any production claim. Workforce inherits this gap.
2. **`wf_employment_records` / `wf_compensation_records` append-only** — enforced by application convention (no UPDATE path in handlers). The matching DB-level REVOKE is on the same hardening checklist as item 1.
3. **No restore test yet** — per the prompt, *"Do not claim backups are valid until a restore test succeeds."* A rollback migration (009, planned) is the restore-test artifact; until it runs green, backups are unverified.

## "Must feel like it was always part of Nexora OS" — how Phase 1 satisfies this

- Shared identity (Keycloak OIDC, stable ULID from `sub`).
- Shared organizations / tenants (FK, not duplication).
- Shared permissions (one `permissions` table, `category='workforce'`).
- Shared DB conventions (UUID PKs, CHAR(26) ULIDs, RLS WITH CHECK, `update_updated_at_column()` triggers).
- Shared error shape (`AosError` → `ErrorResponse`).
- Shared audit hash chain (same `compute_chain_hash` / canonical-payload algorithm).
- Shared config/observability (Figment + tracing + health probes).
- Shared deployment shape (same `main.rs` wiring, same CI, same Docker Compose).
