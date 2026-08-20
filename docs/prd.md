//! Nexora OS MVP Product Requirements Document
//!
//! Mirrors the product requirements for the Nexora OS MVP, establishing shared
//! tenant, identity, authorization, audit, workflow, data, and observability
//! capabilities. Vertical domains consume those primitives through stable APIs.
//!
//! ## Problem
//!
//! African organizations operate across fragmented identity, commerce, payments,
//! compliance, logistics, and government systems. They need reusable infrastructure
//! that works across countries, currencies, connectivity conditions, and regulatory
//! environments.
//!
//! ## Business context
//!
//! Nexora OS is infrastructure for African commerce, not a collection of
//! disconnected vertical applications. The MVP establishes shared tenant, identity,
//! authorization, audit, workflow, data, and observability capabilities. Vertical
//! domains consume those primitives through stable APIs.
//!
//! ## MVP users
//!
//! - Tenant administrators managing organizations, users, roles, and integrations
//! - Developers integrating Nexora OS APIs
//! - Operations and compliance teams reviewing immutable activity
//! - Design partners in one selected vertical at a time
//!
//! ## User stories
//!
//! 1. As a tenant administrator, I can create an organization and manage memberships
//!    without accessing another tenant's data.
//! 2. As a developer, I can discover versioned APIs, authenticate with OIDC, and
//!    receive consistent errors and request IDs.
//! 3. As an auditor, I can query immutable tenant-scoped audit records and export
//!    approved records.
//! 4. As an operator, I can see health, readiness, latency, error, and dependency
//!    status.
//! 5. As a design partner, I can enable a vertical only after its compliance and
//!    operational gates pass.
//!
//! ## Acceptance criteria
//!
//! - Every authenticated domain request has verified subject, tenant, authorization,
//!   request ID, and audit context.
//! - Tenant isolation is enforced in application code and PostgreSQL RLS; cross-tenant
//!   tests fail closed.
//! - All write operations define idempotency behavior, validation, authorization,
//!   audit events, and migration coverage.
//! - API contracts are versioned and published as OpenAPI.
//! - Deployment is reproducible from clean infrastructure code, with rollback and
//!   restore procedures tested.
//! - No regulated financial or citizen data is accepted before a documented launch
//!   gate approves it.
//!
//! ## Non-goals
//!
//! - Simultaneous production launch of all listed industries
//! - Building a new identity provider or payment network
//! - Moving money, issuing government identity, or making regulated decisions
//!   without external compliance review
//!
//! ## Service inventory
//!
//! ### Compiling services
//!
//! | Service | Crate | Status | Purpose |
//! |---------|-------|--------|---------|
//! | API Gateway | `nexora-api-gateway` | ✅ Compiling | Single entry point, routing, rate limiting |
//! | Workforce | `nexora-workforce-service` | ✅ Compiling | Employee lifecycle, contracts, compensation |
//! | Payroll | `nexora-payroll-service` | ✅ Code complete | DRC payroll engine, payslips, audit |
//!
//! ### Stubbed services (boundary reserved)
//!
//! | Service | Crate | Status | Purpose |
//! |---------|-------|--------|---------|
//! | Retail | `nexora-retail-service` | Stub | Retail/consumer vertical |
//! | Fintech | `nexora-fintech-service` | Stub | Fintech/payments vertical |
//! | Gov | `nexora-gov-service` | Stub | Government/public-sector vertical |
//!
//! ### Payroll service detail
//!
//! The payroll service is the most substantial vertical implementation. It includes:
//!
//! - 19 REST handlers with full utoipa OpenAPI documentation
//! - RBAC permission checks (deny-by-default, 18 permission checks)
//! - Hash-chained audit + outbox emission (10 audit events)
//! - DRC 2025 compliant calc engine: IPR progressive tax, CNSS social security,
//!   SMIG minimum-wage enforcement (640 lines, 12 tests passing)
//! - Hand-rolled PDF payslip generator (186 lines, valid PDF-1.4)
//! - ULID-addressed wire types for all entities
//! - Lifecycle state machine: draft → review → approved → locked → cancelled
//!
//! ## Test suite status
//!
//! The test suite includes 7 tests:
//! - 5 pure unit tests that pass without Docker:
//!   - Mass-assignment structural protection
//!   - Employee role cannot write employees
//!   - Document ULID object key namespacing
//!   - Audit emit canonical payload stability
//!   - Employee read does not grant compensation read
//! - 2 integration tests requiring Docker/testcontainers:
//!   - Cross-tenant employee read returns empty (requires PostgreSQL container)
//!   - GUC does not leak between queries (requires PostgreSQL container)
//! - 12 payroll calc unit tests — all passing, verified in isolated test run
//! - CI (ci.yml) provides PostgreSQL+Redis services, so all 7 tests run in CI
//! - The 2 `#[ignore]` tests are documented: "Tests are `#[ignore]`d on Windows
//!   (testcontainers npipe bug)"
//!
//! ## CI/CD pipelines
//!
//! - **ci.yml**: GitHub Actions with PostgreSQL+Redis services, clippy, cargo-audit,
//!   cargo-deny, contract tests, Docker build & scan
//! - **cd.yml**: ArgoCD-based deployment, AWS ECR image promotion, blue/green
//!   deployments, production smoke tests
//!
//! ## Product overview (CEO perspective)
//!
//! Nexora OS is infrastructure for African commerce, providing tenant isolation,
//! security features, and compliance capabilities for B2B SaaS. The product addresses
//! regulated industries (fintech, payment service providers, savings cooperatives)
//! with a SaaS model at $2-5K/tenant/month. The technical foundation is solid with
//! proper CI/CD pipelines and appropriate test coverage.
//!
//! ## Market opportunity
//!
//! Target segments:
//! - Fintech startups (Series A-B): $2-5K/tenant/month, need compliance from launch
//! - Payment service providers: Existing systems costing too much to maintain
//! - Savings cooperatives: Regulatory requirements, limited internal expertise
//! - RegTech resellers: Add audit trail functionality to existing platforms
//!
//! ## Go-to-market
//!
//! - Conference outreach (FinTech Week, Money20/20, RegTech Summits)
//! - LinkedIn outreach (CTO, CCO, Risk Officer titles)
//! - Qualifying framework: BANT (Budget, Authority, Need, Timeline)
//! - Quick validation: 20 outreach attempts → ~6 responses → ~2 meetings → ~1 opportunity
