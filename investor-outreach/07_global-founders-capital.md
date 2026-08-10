# Global Founders Capital
**To:** africa@gfc.vc (or via LinkedIn — search GFC Africa partners)  
**From:** mkasongo@myyahoo.com  
**Subject:** Pre-seed — Africa Operating System (workforce SaaS infrastructure)

---

Hi GFC team,

African enterprises need workforce infrastructure that was never built for them. We are building it.

The Africa Operating System (AOS) is a multi-tenant SaaS platform that provides the HR and workforce management layer African employers need: org structure, employee lifecycle, multi-currency compensation, secure document management, and compliance audit trail — all with per-tenant data isolation enforced at the database level.

**Technical differentiation:**

Most HRIS platforms enforce tenant separation in application code — a bug leaks data. AOS enforces it at the PostgreSQL row-level security layer: every query runs inside a tenant-scoped transaction. This is the right security primitive for an African market where data sovereignty and compliance are becoming regulatory requirements.

**What's live:**
- Org structure: legal entities → locations → departments → teams → positions
- Employee records + full lifecycle (hire, transfer, promote, terminate)
- Multi-currency compensation history (minor-unit precision, ISO 4217)
- Document vault: SHA-256 verified, encrypted, presigned URL access (60s TTL)
- Audit log: append-only, immutable, every action recorded
- 18-permission RBAC + PKCE auth (Keycloak)

**TAM:** 50M+ African SMEs + large enterprises. Even 0.1% penetration at $10 PEPM with 100 employees per client = $500M ARR opportunity.

**Raise:** Pre-seed. Building toward Series A on first 10 enterprise contracts + payroll engine.

Happy to share a live demo. Open to a 15-minute call?

Best,  
Miche Kasongo  
mkasongo@myyahoo.com  
Africa Operating System
