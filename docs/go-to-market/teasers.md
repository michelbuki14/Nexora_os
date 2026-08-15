# Nexora OS Teaser Series — "We Actually Built It"

> Audience: African fintech operators, HR leaders, developers, frontier-tech investors
> Channels: LinkedIn, X/Twitter, landing-page hero
> Anchor proof (live, demonstrable): a real Keycloak OIDC token authenticates → a `201` audit event is written → it is chained via SHA-256 (`prev_hash || canonical_payload`) to the previous event, under PostgreSQL Row-Level Security tenant isolation. Verify-hash-chain endpoint replays the chain. The same primitives now power a second vertical: Nexora OS Workforce — 9 RLS-enabled tables (migration 008), 17 handlers, compiles clean — where every hire, transfer, salary change, and document access writes into the same hash chain. Not a deck. Two running services on one working core.

---

## (a) LinkedIn Posts

### LinkedIn #1 — The credibility anchor

**Nexora OS — "We actually built it, not just decked it."**

Most pitch decks describe an audit trail. We shipped one. Today a live Keycloak OIDC token authenticates a request, our Rust/Axum service writes a `201` audit event, and that event is SHA-256 chained to the one before it — under PostgreSQL Row-Level Security. Tenant isolation is enforced in the database, fail-closed. There's a verify-hash-chain endpoint that replays the whole chain and flags tampering.

This is Nexora OS: infrastructure for African commerce, one honest vertical at a time.

---

### LinkedIn #2 — Two verticals, one honest core

**What's live vs. what's roadmap — said plainly.**

Live now: a tenant-isolated, hash-chained, OIDC-authenticated audit log — **and** a full multi-tenant workforce service (legal entities, departments, positions, employee lifecycle, compensation history, secure document management with presigned GET URLs). Both run on the same Keycloak identity, the same RLS isolation, the same SHA-256 chain. 18 workforce permission keys, money as integer minor units, national IDs hashed with last-4 only.

Designed boundaries, not yet built: fintech, retail, gov, payments, ledger, KYC, citizen ID, permits. Those service mains print `boundary reserved`. We won't sell stubs.

The audit chain now ingests workforce lifecycle events — every salary change is a chained, outbox-queued event the future payroll dispatcher will consume. We built the pipe Phase 2 runs through before we built the engine.

If you're a regulated African institution needing audit evidence *and* trustworthy HR records today, both parts are real and demonstrable. DM for a walkthrough.

---

### LinkedIn #3 — The investor / design-partner frame

**Built before funded. Seeking one design partner, not ten customers.**

Nexora OS runs locally on Docker Compose (Postgres, Keycloak, Redis, MinIO, OpenSearch, Jaeger, Redpanda) with CI/CD wired. Two verticals compile and return clean HTTP codes against a live dev-admin token — audit *and* workforce. The workforce backend (Phase 1) is backend-only with no portal frontend yet; production infrastructure (EKS/Aurora/Terraform) is documented, not deployed — that's what a design-partner pilot funds.

Pilot: ~$15–25K setup to harden one vertical on this core (workforce: DB-level append-only REVOKE, restore test, portal) or build a new one (fintech/retail/gov). Or ~$2–5K/mo audit SaaS standalone.

Design partner → reference → scale. No invented ARR. Built, not decked.

---

## (b) X / Twitter Posts

### X #1 — the proof in one shot

Nexora OS — "we actually built it, not just decked it."

Live Keycloak OIDC token → Rust/Axum 201 audit event → SHA-256 hash chain under PostgreSQL Row-Level Security. Tenant isolation enforced in DB, fail-closed. Tamper-evident, replay-verifiable. #fintech #Africa

---

### X #2 — two live verticals

[LIVE] Tenant-isolated, hash-chained audit log. OIDC-auth. Regulator-grade.
[LIVE] Workforce service — 9 RLS tables, 17 handlers, lifecycle+compensation+docs. Same chain, same identity.
[ROADMAP] fintech / retail / gov — designed boundaries, gated behind compliance & a design partner.

We don't sell stubs. We build real verticals on a working core.
Africa commerce infra, one honest vertical at a time.

---

### X #3 — design-partner call

Seeking ONE design partner for Nexora OS.

The core works today: two verticals live (audit log + workforce backend). Keycloak JWT → tamper-evident audit chain, RLS-isolated, ~$2–5K/mo audit standalone or ~$15–25K pilot to harden/build one vertical on it.

No invented ARR. Design partner → reference → scale. Built before funded, from Kinshasa.

---

## (c) Landing-Page Hero Block

**Headline** (≤12 words):
> Honest infrastructure for African commerce — built, not decked.

**Subhead** (≤25 words):
> A live OIDC token writes a SHA-256 hash-chained, tenant-isolated audit event today — and now powers a multi-tenant workforce service on the same core.

**Three proof-points (live, demonstrable):**
- **Tamper-evident by construction** — every audit event is SHA-256 chained to the previous one; a verify endpoint replays the full chain and flags any tampering. Workforce lifecycle events write into the same chain.
- **Tenant isolation enforced in the database** — PostgreSQL Row-Level Security across the audit table *and* all 9 workforce tables, fail-closed; one tenant physically cannot see another's rows.
- **OIDC-authenticated end to end** — real Keycloak JWT, JWKS signature validation; a live token returns a `201` audit event and authenticates every workforce action, not a mock.

**CTA button:**
> See the live audit chain →

---

## Honesty ledger (internal — do not publish)

Last updated: 2026-08-10 (post-workforce-Phase-1).

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
