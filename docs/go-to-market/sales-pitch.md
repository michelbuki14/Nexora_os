# The auditor is already at the door. Your audit log is a spreadsheet. Your HR records are in three dashboards.

It's the week before a BCC, CBN, or CBK examination. Your compliance officer is stitching logs out of three separate dashboards, copying rows into Excel, and praying no one asks: *"How do you prove this record wasn't edited after the fact?"* Across DRC, Nigeria, and Kenya, that question ends licenses. And the honest answer — for most fintechs, PSPs, and SACCOs — is: you can't. The same audit pressure now reaches HR: a salary change, a contract, an employee file audit happens, and there is no tamper-evident record to hand over. Nexora OS was built to close both gaps.

**Nexora OS Audit & Compliance Log and Nexora OS Workforce are a tamper-evident, tenant-isolated, hash-chained audit logging service *and* a full multi-tenant workforce service — both running today in a reproducible local stack — designed to give African regulated institutions a regulator-grade evidence trail they can hand to an examiner on demand, once production gates (pen test, SLOs, DR test, compliance sign-off) are cleared with a design partner.**

## Why this is not another clickable log table

Most "audit" tools in the market today are a searchable events table behind a login. Most "HR" tools are a separate dashboard with no audit trail at all. Neither answers the question an examiner actually asks: *can a privileged insider silently rewrite history?* That gap is where audit findings — and enforcement actions — live.

Nexora OS closes it with three things, all **live and demonstrable today in the standalone audit service and shared across the workforce service via the same primitives**, not on a roadmap slide — the unified API gateway does not yet proxy audit or workforce (returns 'planned'); both services run on their own ports with their own health probes and their own auth+RLS stack:

- **Live OIDC authentication (Keycloak).** Every request carries a real JWT validated against a live JWKS set — signature-checked, not decoded-and-trusted. The user shown in the audit record *is* the authenticated identity. Identity is shared: the same Keycloak tenant + 18 workforce permission keys gate every HR action.
- **Database-level tenant isolation (PostgreSQL Row-Level Security).** Tenant boundaries are enforced inside the database, not in application code that a bug or a misconfigured role can silently bypass. Every one of the 9 `wf_*` workforce tables carries the same `current_setting('nexora.current_tenant_id')` policy with `WITH CHECK` — fail-closed, tested.
- **A cryptographic SHA-256 hash chain.** Each audit event is chained to the one before it via application logic. A dedicated `verify-hash-chain` endpoint replays the chain from genesis and flags the first event whose hash no longer recomputes. Tampering is detectable in seconds. **The workforce service writes into the same chain** — every hire, transfer, salary change, or document access is a chained event, dual-written to the transactional outbox for the future payroll dispatcher. Note: database-level `DELETE`/`UPDATE` revocation is on the hardening checklist before production deployment (append-only is currently application-enforced).

The credibility anchor is a single demonstrated flow we show live on screen: **a real Keycloak token → a 201 audit event → a hash-chain verification that proves the event wasn't altered** — and now the same chain ingests workforce lifecycle events. That's the demo that survives a skeptic in the room.

## What you get today (all live)

- **Append-only `audit_events`** with create, list, get-by-id, and an explicit *verify-hash-chain* call — returns 201/200 against a live dev-admin Keycloak token.
- **Full workforce service (Phase 1, backend-only):** legal entities, locations, departments, teams, positions, employee lifecycle, employment records, compensation history, and secure document management — **9 RLS-enabled tables (migration 008), 17 handlers, compiles clean**. Money stored as integer minor units (never floats); national ID SHA-256-hashed with only last-4 exposed; documents held in S3/MinIO under `{tenant}/{employee}/{doc}` keys, served only via short-lived presigned GET URLs, never public.
- **Shared, attributable identity:** per-user stable ULID derived from the Keycloak `sub` claim, with tenant and organization ULIDs carried in the token — every record (audit *and* workforce) is attributable to a real, authenticated person, not an opaque string.
- **Multi-tenant core in Rust/Axum**: an API gateway with health probes, request IDs, CORS, tracing, and a timeout layer; a tenant service with slug/tier validation and system-privilege RBAC; 18 workforce permission keys seeded into the shared `permissions` table.
- **A reproducible local stack** via Docker Compose (PostgreSQL, Keycloak, Redis, MinIO, OpenSearch, Jaeger, Redpanda) with CI/CD in GitHub Actions — so your team can run the audit flow (Keycloak token → 201 audit event → hash-chain verify) *and* the workforce flow (create employee → chained audit event) on your own laptop against the standalone services before you sign anything. The unified API gateway currently returns 'planned' for audit and workforce routes (the services run standalone).

## Pricing

**Audit & Compliance Log (standalone SaaS):** **$2,000–$5,000 per tenant per month**, tiered on event volume. The tamper-evident, tenant-isolated, hash-chained log described above — real, running, and yours to point an examiner at.

**Workforce (design-partner pilot):** **$15,000–$25,000** to fund hardening one real workforce deployment on the working core (DB-level append-only REVOKE, restore test via migration 009, frontend portal) and deliver it with you as the reference customer. The backend runs today; production gate items close with the pilot.

**Paid design-partner pilot (other verticals):** **$15,000–$25,000** to fund building one real vertical — fintech, retail, or government — on top of the working core, delivered with you as the reference customer. The platform's EKS + Aurora + Terraform blue/green production path and DR plan (RPO 5 min / RTO 30 min target) are **described in design documents — no infrastructure-as-code has been deployed yet**; a paid design-partner pilot would fund implementing and standing up that infrastructure for your workload. We never sell a vertical as shipping until it's built with you.

> **Stated plainly:** commerce, payments, ledger, KYC, citizen-ID, and permitting modules are **designed boundaries on the roadmap, gated behind compliance review** — not products you can buy today. What you can buy today is the audit log, the workforce backend, and the shared core they both run on. Design partner → reference → scale.

## Book a 20-minute demo

We'll show you the live flow — Keycloak token in, a workforce lifecycle event creating a hash-chained audit event, tamper detection on screen — and talk through what your next regulator walkthrough looks like with evidence instead of spreadsheets.

**[ Book a 20-min demo → ]**

*Built from Kinshasa for African regulated finance. Audit you can prove. Workforce you can trust.*
