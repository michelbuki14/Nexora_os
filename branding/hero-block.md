# Nexora OS Landing Page Hero Block

## Headline (≤12 words)
**Honest infrastructure for African commerce — built, not decked.**

## Subhead (≤25 words)
> A live OIDC token writes a SHA-256 hash-chained, tenant-isolated audit event today — and now powers a multi-tenant workforce service on the same core.

## Three Proof Points (live, demonstrable)

### 1. Tamper-evident by construction
> Every audit event is SHA-256 chained to the previous one; a verify endpoint replays the full chain and flags any tampering. Workforce lifecycle events write into the same chain.

### 2. Tenant isolation enforced in the database
> PostgreSQL Row-Level Security across the audit table *and* all 9 workforce tables, fail-closed; one tenant physically cannot see another's rows.

### 3. OIDC-authenticated end to end
> Real Keycloak JWT, JWKS signature validation; a live token returns a `201` audit event and authenticates every workforce action, not a mock.

## CTA Button
> **See the live audit chain →**

## Honesty Ledger (internal — do not publish)

| What | Status |
|------|--------|
| Audit/compliance log SaaS | ✅ Live, demonstrable, compiles clean |
| Workforce service — Phase 1 backend | ✅ Live, compiles clean (lib + tests). 9 RLS tables (migration 008), 17 handlers, audit emit, S3 presigned docs, 18 RBAC keys, testcontainers isolation test scaffolded (ignored on Windows pending Linux/CI run) |
| Platform backbone, design-partner pilot | ✅ Sellable now |
| Workforce frontend / portal | ❌ Not built — backend-only Phase 1; OpenAPI spec + data-contract/portal-map docs describe the surface |
| Fintech / retail / gov vertical mains | ❌ 3-line stubs — `println!("boundary reserved")` |
| Gateway audit + workforce routes | ❌ Return `{"status":"planned"}` — both services run standalone on their own ports with their own auth+RLS stack; gateway forwarding not wired |
| No ledger, KYC, payments, citizen ID, permits | ❌ Does not exist |
| `audit_events` UPDATE/DELETE DB-REVOKE | ❌ Deferred — append-only currently app-enforced only (incl. workforce history/compensation tables). Must close before production |
| Restore test | ❌ Not yet run green (migration 009 planned) — backups unverified |
| EKS / Aurora / Terraform / blue-green / DR | 📄 Documented design, not deployed — no `.tf` files, production-readiness checklist fully unchecked |
| Revenue numbers used | $2–5K/mo audit SaaS, $15–25K pilot setup (harden workforce OR build new vertical) — no fabricated ARR |
| Moving money / issuing ID | ❌ Never claimed |

> Last updated: 2026-08-10 (post-workforce-Phase-1).