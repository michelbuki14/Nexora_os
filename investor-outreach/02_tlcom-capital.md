# TLcom Capital
**To:** investments@tlcomcapital.com  
**From:** mkasongo@myyahoo.com  
**Subject:** Pre-seed — Africa Operating System (B2B workforce infrastructure)

---

Hi TLcom team,

African enterprises — from 50-person manufacturers in Lagos to 500-person banks in Kinshasa — share the same problem: no workforce management platform was built for them. Local labour law, multi-currency payroll, and country-specific compliance are afterthoughts in every global HRIS. The result is spreadsheets, compliance risk, and payroll errors at scale.

The Africa Operating System (AOS) is a multi-tenant B2B SaaS platform that solves this at the infrastructure layer.

**Technical proof — what's built and running today:**

- **Org structure:** legal entities, locations, departments, teams, positions — all multi-tenant, all isolated at the database row level
- **Employee lifecycle management:** full hire-to-terminate flow, employment transfers, status tracking
- **Compensation engine:** multi-currency, minor-unit precision (no floating-point rounding errors on payroll), multiple frequency types
- **Document vault:** encrypted storage, presigned URL access (60s TTL), document type classification — contracts, IDs, payslips, certificates
- **Audit trail:** append-only event log, every write action recorded with actor, timestamp, and tenant context
- **Security:** JWT validation + PKCE auth, 18 granular RBAC permissions, row-level security enforced at the DB transaction layer

**Architecture:** Rust microservices, PostgreSQL, Keycloak, MinIO — built for horizontal scale and multi-region deployment.

**Business model:** per-seat SaaS subscription, tiered by employee count. Target: $8–15 PEPM (per employee per month).

**Ask:** Pre-seed capital to close our first enterprise contracts and build the payroll engine (Phase 2).

Happy to share a live demo or technical deep-dive. Is there a slot on your calendar this month?

Best,  
Miche Kasongo  
mkasongo@myyahoo.com  
Africa Operating System
