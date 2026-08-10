# Future Africa
**To:** hello@future.africa  
**From:** mkasongo@myyahoo.com  
**Subject:** Africa Operating System — workforce infrastructure for African employers

---

Hi Future Africa team,

I'm Miche Kasongo. I'm building the Africa Operating System — workforce management infrastructure designed specifically for African employers, from the database schema up.

**The problem in one line:** Global HRIS tools don't handle local labour law, multi-currency payroll, or African compliance requirements — so 50M+ African employers run on spreadsheets.

**What I've built:**

The Phase 1 workforce core is complete and running:
- Org structure (legal entities, locations, departments, teams, positions)
- Employee lifecycle management (hire, transfer, promotion, termination)
- Multi-currency compensation management with minor-unit precision
- Encrypted document vault (contracts, IDs, payslips) with presigned URL access
- Full audit trail — every action logged, immutable, per-tenant
- Multi-tenant isolation enforced at the database transaction level (row-level security)
- PKCE/JWT auth with 18 granular permissions (employee.read, compensation.write, etc.)

**Stack:** Rust + PostgreSQL + Keycloak. Chosen for correctness and performance under African infrastructure conditions (unreliable networks, high latency).

**Business model:** Per-seat SaaS. $8–15 PEPM. First target segments: DRC manufacturing, Nigerian fintech, Kenyan logistics.

**Ask:** Pre-seed to close first 5 paying tenants and build the payroll engine.

I'd love to get your feedback — is there 15 minutes on your calendar?

Miche Kasongo  
mkasongo@myyahoo.com
