# Partech Africa
**To:** africa@partechpartners.com  
**From:** mkasongo@myyahoo.com  
**Subject:** Pre-seed — Africa Operating System (workforce OS for African employers)

---

Hi Partech Africa team,

50 million African SMEs run payroll and HR on spreadsheets. Global platforms (SAP, Workday, BambooHR) don't handle local labour law, multi-currency payroll, or country-specific compliance — so African employers are left stitching together disconnected tools that were never built for them.

We are building the Africa Operating System (AOS) — a multi-tenant workforce OS designed from the ground up for African employers.

**What's live today:**
- Full org-structure engine: legal entities, locations, departments, teams, positions
- Employee lifecycle: hire → transfer → terminate, with full audit trail
- Compensation management (multi-currency, minor-unit precision, frequency-aware)
- Secure document vault: contracts, IDs, payslips — stored encrypted in S3, accessed only via short-lived presigned URLs
- Row-level security: every query is tenant-isolated at the database layer — no tenant can ever see another's data
- JWT/PKCE auth via Keycloak with per-user permission claims (18 granular permissions)
- API gateway routing to microservices, built in Rust for performance and reliability

**Stack signal:** Rust + PostgreSQL + Keycloak — chosen for data integrity and security compliance, not hype.

**The roadmap:** payroll engine (Phase 2), local tax and labour law compliance per country (Phase 3), government integrations and fintech rails (Phase 4).

**We are raising a pre-seed round** to fund 12 months of product development and our first 10 paying enterprise tenants across DRC, Kenya, and Nigeria.

Would you have 15 minutes for a call this month?

Best,  
Miche Kasongo  
mkasongo@myyahoo.com  
Africa Operating System
