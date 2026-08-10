# The auditor is already at the door. Your audit log is a spreadsheet.

It's the week before a BCC, CBN, or CBK examination. Your compliance officer is stitching logs out of three separate dashboards, copying rows into Excel, and praying no one asks: *"How do you prove this record wasn't edited after the fact?"* Across DRC, Nigeria, and Kenya, that question ends licenses. And the honest answer — for most fintechs, PSPs, and SACCOs — is: you can't.

**The AOS Audit & Compliance Log is a tamper-evident, tenant-isolated, hash-chained audit logging service — running today in a reproducible local stack — designed to give African regulated financial institutions a regulator-grade evidence trail they can hand to an examiner on demand, once production gates (pen test, SLOs, DR test, compliance sign-off) are cleared with a design partner.**

## Why this is not another clickable log table

Most "audit" tools in the market today are a searchable events table behind a login. They answer *who clicked what*. They do not answer the question an examiner actually asks: *can a privileged insider silently rewrite history?* That gap is where audit findings — and enforcement actions — live.

AOS closes it with three things, all **live and demonstrable today in the standalone audit service**, not on a roadmap slide — the unified API gateway does not yet proxy audit (returns 'planned'); the audit service runs on its own port with its own health probes:

- **Live OIDC authentication (Keycloak).** Every request carries a real JWT validated against a live JWKS set — signature-checked, not decoded-and-trusted. The user shown in the audit record *is* the authenticated identity.
- **Database-level tenant isolation (PostgreSQL Row-Level Security).** Tenant boundaries are enforced inside the database, not in application code that a bug or a misconfigured role can silently bypass. Tested fail-closed.
- **A cryptographic SHA-256 hash chain.** Each audit event is chained to the one before it via application logic. A dedicated `verify-hash-chain` endpoint replays the chain from genesis and flags the first event whose hash no longer recomputes. Tampering is detectable in seconds. Note: database-level `DELETE`/`UPDATE` revocation is on the hardening checklist before production deployment.

The credibility anchor is a single demonstrated flow we show live on screen: **a real Keycloak token → a 201 audit event → a hash-chain verification that proves the event wasn't altered.** That's the demo that survives a skeptic in the room.

## What you get today (all live)

- **Append-only `audit_events`** with create, list, get-by-id, and an explicit *verify-hash-chain* call — returns 201/200 against a live dev-admin Keycloak token.
- **Per-user stable ULID** derived from the Keycloak `sub` claim, with tenant and organization ULIDs carried in the token — so every record is attributable to a real, authenticated person, not an opaque string.
- **Multi-tenant core in Rust/Axum**: an API gateway with health probes, request IDs, CORS, tracing, and a timeout layer; a tenant service with slug/tier validation and system-privilege RBAC.
- **A reproducible local stack** via Docker Compose (PostgreSQL, Keycloak, Redis, MinIO, OpenSearch, Jaeger, Redpanda) with CI/CD in GitHub Actions — so your team can run the audit service flow (Keycloak token → 201 audit event → hash-chain verify) on your own laptop against the standalone audit service before you sign anything. The unified API gateway currently returns 'planned' for audit routes.

## Pricing

**Audit & Compliance Log (standalone SaaS):** **$2,000–$5,000 per tenant per month**, tiered on event volume. The tamper-evident, tenant-isolated, hash-chained log described above — real, running, and yours to point an examiner at.

**Paid design-partner pilot (the full backbone):** **$15,000–$25,000** to fund building one real vertical — fintech, retail, or government — on top of the working core, delivered with you as the reference customer. The platform's EKS + Aurora + Terraform blue/green production path and DR plan (RPO 5 min / RTO 30 min target) are **described in design documents — no infrastructure-as-code has been deployed yet**; a paid design-partner pilot would fund implementing and standing up that infrastructure for your workload. We never sell a vertical as shipping until it's built with you.

> **Stated plainly:** commerce, payments, ledger, KYC, citizen-ID, and permitting modules are **designed boundaries on the roadmap, gated behind compliance review** — not products you can buy today. What you can buy today is the audit log and the core it runs on. Design partner → reference → scale.

## Book a 20-minute demo

We'll show you the live flow — Keycloak token in, hash-chained audit event out, tamper detection on screen — and talk through what your next regulator walkthrough looks like with evidence instead of spreadsheets.

**[ Book a 20-min demo → ]**

*Built from Kinshasa for African regulated finance. Audit you can prove.*
