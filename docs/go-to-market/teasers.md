# AOS Teaser Series — "We Actually Built It"

> Audience: African fintech operators, developers, frontier-tech investors
> Channels: LinkedIn, X/Twitter, landing-page hero
> Anchor proof (live, demonstrable): a real Keycloak OIDC token authenticates → a `201` audit event is written → it is chained via SHA-256 (`prev_hash || canonical_payload`) to the previous event, under PostgreSQL Row-Level Security tenant isolation. Verify-hash-chain endpoint replays the chain. Not a deck. A running service.

---

## (a) LinkedIn Posts

### LinkedIn #1 — The credibility anchor

**AOS — "We actually built it, not just decked it."**

Most pitch decks describe an audit trail. We shipped one. Today a live Keycloak OIDC token authenticates a request, our Rust/Axum service writes a `201` audit event, and that event is SHA-256 chained to the one before it — under PostgreSQL Row-Level Security. Tenant isolation is enforced in the database, fail-closed. There's a verify-hash-chain endpoint that replays the whole chain and flags tampering.

This is AOS: infrastructure for African commerce, one honest vertical at a time.

---

### LinkedIn #2 — Honest scope, designed boundaries

**What's live vs. what's roadmap — said plainly.**

Live now: a tenant-isolated, hash-chained, OIDC-authenticated audit log. The kind regulators (BCC, CBN, CBK) keep asking fintechs to produce on demand.

Designed boundaries, not yet built: fintech, retail, gov. Their service mains print `boundary reserved`. We won't sell stubs. We'll build one real vertical with one paying design partner on the working core — then reference, then scale.

If you're a PSP, SACCO, or fintech needing a regulator-grade audit trail today, that part is real and demonstrable. DM for a walkthrough.

---

### LinkedIn #3 — The investor / design-partner frame

**Built before funded. Seeking one design partner, not ten customers.**

AOS runs locally on Docker Compose (Postgres, Keycloak, Redis, MinIO, OpenSearch, Jaeger, Redpanda) with CI/CD wired. The audit service returns clean HTTP codes against a live dev-admin token. Infrastructure architecture is documented; no production AWS environment is deployed yet — that's what a design-partner pilot funds.

Pilot: ~$15–25K setup to build one vertical on this core. Or ~$2–5K/mo audit SaaS standalone.

Design partner → reference → scale. No invented ARR. Built, not decked.

---

## (b) X / Twitter Posts

### X #1 — the proof in one shot

AOS — "we actually built it, not just decked it."

Live Keycloak OIDC token → Rust/Axum 201 audit event → SHA-256 hash chain under PostgreSQL Row-Level Security. Tenant isolation enforced in DB, fail-closed. Tamper-evident, replay-verifiable. #fintech #Africa

---

### X #2 — honest roadmap tagging

[LIVE] Tenant-isolated, hash-chained audit log. OIDC-auth. Regulator-grade.
[ROADMAP] fintech / retail / gov — designed boundaries, gated behind compliance & a design partner.

We don't sell stubs. We build one real vertical on a working core.
Africa commerce infra, one honest vertical at a time.

---

### X #3 — design-partner call

Seeking ONE design partner for AOS.

The core works today: Keycloak JWT → tamper-evident audit chain, RLS-isolated, ~$2–5K/mo standalone or ~$15–25K pilot setup to build one vertical on it.

No invented ARR. Design partner → reference → scale. Built before funded, from Kinshasa.

---

## (c) Landing-Page Hero Block

**Headline** (≤12 words):
> Honest infrastructure for African commerce — built, not decked.

**Subhead** (≤25 words):
> A live OIDC token writes a SHA-256 hash-chained, tenant-isolated audit event today. The rest is built to order with a design partner — never sold as stubs.

**Three proof-points (live, demonstrable):**
- **Tamper-evident by construction** — every audit event is SHA-256 chained to the previous one; a verify endpoint replays the full chain and flags any tampering.
- **Tenant isolation enforced in the database** — PostgreSQL Row-Level Security, fail-closed; one tenant physically cannot see another's rows.
- **OIDC-authenticated end to end** — real Keycloak JWT, JWKS signature validation; a live token returns a `201` audit event, not a mock.

**CTA button:**
> See the live audit chain →

---

## Honesty ledger (internal — do not publish)

| What | Status |
|------|--------|
| Audit/compliance log SaaS | ✅ Live, demonstrable |
| Platform backbone, design-partner pilot | ✅ Sellable now |
| Fintech / retail / gov vertical mains | ❌ 3-line stubs — `println!("boundary reserved")` |
| Gateway audit route | ❌ Returns `{"status":"planned"}` |
| No ledger, KYC, payments, citizen ID, permits | ❌ Does not exist |
| EKS / Aurora / Terraform / blue-green / DR | 📄 Documented design, not deployed — no `.tf` files, production-readiness checklist fully unchecked |
| Revenue numbers used | $2–5K/mo audit SaaS, $15–25K pilot setup — no fabricated ARR |
| Moving money / issuing ID | ❌ Never claimed |
