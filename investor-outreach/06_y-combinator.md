# Y Combinator (Application Supplement / Cold Intro)
**To:** Apply at ycombinator.com/apply — use this as your application narrative  
**From:** mkasongo@myyahoo.com  
**Subject:** YC Application — Africa Operating System

---

**Company:** Nexora OS  
**Founder:** Miche Kasongo — mkasongo@myyahoo.com  
**Stage:** Pre-seed, Phase 1 complete  

---

**What does your company do?**

Nexora OS is the workforce operating system for African employers. We provide multi-tenant HR and workforce management infrastructure purpose-built for African labour law, multi-currency payroll, and compliance requirements that global platforms ignore.

**What's the problem?**

50+ million African SMEs and enterprises run HR and payroll on spreadsheets because no platform was designed for their context. Global HRIS tools (SAP, Workday, BambooHR) assume single-currency payroll, OECD tax law, and stable internet. African businesses have multi-currency operations (CDF, NGN, KES, GHS, ZAR), country-specific labour codes, and mobile-first users. The gap is a structural one — it requires a platform rebuilt from scratch, not a localization patch.

**What have you built?**

Phase 1 is complete and running:
- Multi-tenant org structure engine: legal entities, locations, departments, teams, positions
- Full employee lifecycle: hire, transfer, promotion, termination — with immutable audit trail
- Compensation management: multi-currency, minor-unit precision (ISO 4217), frequency-aware
- Secure document vault: SHA-256 integrity, encrypted storage, 60-second presigned URL access
- 18-permission RBAC system: granular permissions mapped to HR roles (HR Admin, Manager, Employee)
- Row-level security: tenant isolation enforced at the database transaction layer — not in application code
- PKCE + JWT auth via Keycloak with custom claims (tenant_id, org_id, permissions)

Built in Rust + PostgreSQL for correctness, performance, and data integrity under African infrastructure conditions.

**Why you?**

I am African, I understand the market from the inside, and I have built the core infrastructure. Phase 2 (payroll engine) is designed. Phase 3 (country-specific tax compliance) and Phase 4 (government integrations) have clear technical paths. The moat is domain knowledge + data — once tenants' employee records are on Nexora OS, switching cost is high.

**Business model:** Per-seat SaaS, $8–15 PEPM. 100 employees × $10 = $1,000 MRR per tenant.

**Ask:** YC standard terms. First batch preference: W25 or S25.

Miche Kasongo  
mkasongo@myyahoo.com
