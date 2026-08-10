# AOS — Africa Operating System
### Investor Speech (4-5 min, ~850 words)

---

## The Problem: A Continent Running on Disconnected Systems

Good morning. Let me start with a number most people outside Africa don't sit with: this continent holds over a billion people, dozens of currencies, and some of the highest mobile-money penetration on earth. Commerce here doesn't lack activity. It lacks *connective tissue*.

A fintech in Lagos, a savings cooperative in Nairobi, a payment processor in Kinshasa — they all hit the same wall. Identity is fragmented: every country, sometimes every institution, runs its own notion of who a user is. Mobile money dominates, but each rail is its own world. And above all of it sits the regulator — the BCC in Congo, the CBN in Nigeria, the CBK in Kenya — asking the same uncomfortable question: *prove to us, with evidence that can't be tampered with, who did what, when, and under whose authority.*

That audit question is not a feature request. It is a license-to-operate requirement. And today, across the region, it is answered with spreadsheets, copy-pasted logs, and hope. We built AOS to answer it properly — and to turn that answer into a platform.

---

## The Wedge: One Demo, Not Slideware

I could show you a roadmap slide. I'd rather show you a token.

Today, live, you can take a real Keycloak OpenID Connect token — not a mock, a validated JWT with its signature checked against live JWKS — send it to our API, and POST an audit event. You get a 201 back. That event lands in an append-only table (with database-level `DELETE`/`UPDATE` revocation on the hardening checklist before production deployment), and it is chained: every event carries a SHA-256 hash computed from the previous event's hash and its own canonical payload. You can call verify-hash-chain and we walk the chain and tell you whether it's intact.

Underneath, that event is tenant-isolated by Row-Level Security enforced *in the database itself* — not in our application code, where a bug could leak it, but in PostgreSQL, fail-closed. The actor is a stable per-user ULID derived from the Keycloak subject claim. The gateway in front of it already does health probes, request IDs, tracing, and timeouts.

The audit service runs as a standalone binary today; the gateway's audit route is a stub pending final wiring — the live demo goes directly to the audit service. That is not a vision statement. That is a working, demonstrable system, built in Rust on Axum, running today. And here is why that specific demo matters: it is the exact artifact a regulator's examiner asks for. Who acted? What did they do? Under which tenant? Can the record be proven unbroken? The BCC, the CBN, the CBK — they don't care about your dashboards. They care that the audit trail is tamper-evident and that it cannot bleed across tenants. We built the one thing they audit first.

---

## From Wedge to Moat: The Hard Primitives Nobody Wants to Rebuild

Here is the part that isn't obvious from one demo. To make that audit chain trustworthy, we were *forced* to solve the hard shared primitives of any African vertical: multi-tenancy with real isolation, identity federation through OIDC, authorization and RBAC, observability, tracing, and row-level security done at the database.

Every fintech, retailer, and government service on this continent needs those primitives. None of them want to build them. They want to ship product. The audit wedge was deliberate, because it made us solve the layer that is boring, expensive, and easy to get wrong — the layer that becomes the moat once it's done and proven.

So we are not a point product. We are a backbone, proven under the most demanding requirement — auditability — and now ready to carry verticals on top of it.

---

## The Business Model: What We Sell Today

Two things, both honest.

First, the audit and compliance log as a standalone service — tamper-evident, tenant-isolated, hash-chained, OIDC-authenticated — target pricing in the range of two to five thousand dollars a month per tenant based on early conversations. The buyer is any fintech, payment service provider, or savings cooperative that needs a regulator-grade audit trail and doesn't want to build one. That revenue can start now, because the product exists now.

Second, paid design-partner pilots — a target setup fee of fifteen to twenty-five thousand dollars to fund building one real vertical on the working core. We never sell slides. We sell the working backbone plus a committed build, behind a paying partner.

I want to be blunt about how we scale. It is *design partner to reference to scale* — not a five-year ARR projection made up on a flight. One signed partner proves one vertical. One reference closes the next. That is the only honest growth curve for infrastructure this early, and it is the curve the best African infrastructure companies have actually followed.

---

## The Roadmap — and the Honest Gate

Finance, commerce, and government are *designed boundaries* in our architecture, not shipped products. I have to tell you plainly: today those services are three-line stubs that print "boundary reserved." That is intentional. We will build each one only inside a paid partner engagement, and only behind a compliance gate.

That gate is non-negotiable. No moving money until reconciliation and settlement controls are reviewed. No citizen ID until data residency and retention are approved. No regulated decision leaves our hands without external compliance sign-off. Our production-readiness checklist is fully unchecked today — no pen test, no load test, no disaster-recovery run, no external sign-off — and I'm telling you that deliberately, because a founder who hides that from investors is one you shouldn't fund. We know exactly what is live and what is the road ahead.

We start in Kinshasa, in the DRC, because that is home ground and because the BCC's audit expectations are a clean fit for what we have already proven.

---

## The Ask: Why Now

What we need now is specific and bounded. First, design-partner introductions — fintechs, PSPs, SACCOs, ideally in DRC or a neighboring francophone market — who feel the regulator-audit pain today and would fund one vertical build. Second, seed capital to convert the first paid pilot into a reference and harden the path to the compliance gate.

Why now, not later? Because the proof that buys everything else — a real OIDC token, a 201, a verified hash chain, database-enforced isolation — already runs. The work that remains is gated by partners and capital, not by a research problem we haven't solved. The timing is now because the evidence is now. And on this continent, the regulator's question is not getting any softer — it is getting sharper every cycle.

We have built the connective tissue. We have proven the hardest part under the harshest requirement. Help us turn it into the infrastructure African commerce runs on.

Thank you.
