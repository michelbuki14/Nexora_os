# AOS Implementation Plan

## Workforce, Payroll, Payments — Built on Existing Rust Foundation

> Generated: 2026-08-07  
> Status: Approved for implementation  
> Protocol: Master Build Prompt STEP 2 — PLAN

---

## Executive Summary

AOS will be built as a **Rust modular monolith** using Axum, SQLx, PostgreSQL 16, Tokio, and Keycloak — preserving the existing foundation. The plan adds three major commercial verticals (Workforce, Payroll, Payments) plus a Finance foundation over **4 phases across 16 weeks**.

The current repository provides a solid base (Docker Compose stack, Core schema with RLS, CI/CD pipeline, Keycloak realm, observability primitives) but needs critical fixes before vertical implementation can begin.

**Key architectural decisions:**
- Keep Rust/Axum/SQLx (no stack rewrite per Rule #61)
- PostgreSQL NUMERIC(19,4) for all monetary values — never floats
- `rust_decimal` promoted to workspace dependency
- Transactional outbox pattern for reliable background jobs (no BullMQ — use Redis Streams + worker)
- RLS enforced at DB level for ALL new tables
- Append-only, tamper-evident audit for all financial/payroll operations
- Deterministic payroll engine with versioned, effective-dated rules
- Payment orchestration behind provider adapter ports (not direct bank integration)

---

## Critical Gaps Found (Validation Gate)

The adversarial review identified **8 critical gaps** that must be addressed before or during Phase 1:

| # | Gap | Severity | Resolution |
|---|-----|----------|------------|
| 1 | **Workspace doesn't compile** — `services/common/src/health.rs` uses `sqlx::PgPool` and `redis::Client` but `Cargo.toml` declares neither; ~53 errors | CRITICAL | Add sqlx/redis deps to aos-common, fix unused imports |
| 2 | **No DRC payroll analysis anywhere** — grep for payroll/DRC/IPR/CNSS/SMIG returns nothing | CRITICAL | Produce DRC payroll design artifact with regulatory review before payroll implementation |
| 3 | **No money representation standard** — no NUMERIC columns, no money types, no rounding policy | CRITICAL | Adopt NUMERIC(19,4) + `rust_decimal` everywhere, clippy guard banning f32/f64 for money |
| 4 | **RLS is inert** — no code ever calls `set_config('aos.current_tenant_id', ...)`, no WITH CHECK policies, organizations policy has wrong dimension | CRITICAL | Wire RLS per-request, add WITH CHECK, fix organizations policy |
| 5 | **Audit is not tamper-evident** — no hash-chain computation, no append-only enforcement, audit-service is a stub | CRITICAL | Keyed HMAC hash chain, REVOKE UPDATE/DELETE, verification job |
| 6 | **No idempotency anywhere** — only an error variant exists; no dedup table, no key middleware | CRITICAL | Idempotency-key layer on every POST/PUT, unique constraints on payroll/payment |
| 7 | **No deterministic payroll engine** — no computation engine, no versioned rules, no DRC calendar/timezone | CRITICAL | Spec deterministic engine with versioned effective-dated rule sets |
| 8 | **Zero tests** — no `#[test]` anywhere, no property-based testing, no golden files | CRITICAL | Unit + proptest + golden file strategy for financial calculations |

**All 8 must pass before Phase 1 completes.** Gaps 3–8 are addressed in Phase 1 tasks below.

---

## Reusable Assets (Preserve)

| Asset | Adaptation |
|-------|-----------|
| `aos-common` crate (config/error/jwt/tenant/validation/ulid/time/logging/health) | Add PermissionExtractor, authz middleware, shared money types |
| `rust_decimal` (transitive dep in fintech-service) | Promote to workspace dep, add sqlx NUMERIC support |
| `AosError` variants (IdempotencyConflict, InsufficientFunds, ExternalService) | Map to payment retry, payroll validation, reconciliation |
| `audit_events` schema (append-only, SHA-256 hash, RLS) | Reuse for payroll/payment/lifecycle; add verifier job |
| RLS policy pattern (`current_setting('aos.current_tenant_id')`) | Replicate for all 35+ new tables |
| `TenantResolver`/`TenantExtractor` middleware | Extend into full authz chain: JWT → tenant → permissions → RLS GUC |
| `JwtValidator` + `AosClaims` | Add employee_id, country_code, currency claims |
| Health checks (DatabaseHealthCheck, RedisHealthCheck) | Wire into new service routers |
| Docker Compose stack (PostgreSQL, Redis, Keycloak, MinIO, Redpanda, Jaeger, Mailhog) | Wire Redis (jobs), Redpanda (events), MinIO (payslips), Mailhog (email) |
| CI workflow (fmt, clippy, test, audit, deny, build) | Auto-includes new workspace crates |
| `utoipa`/`utoipa-swagger-ui` workspace deps | Generate OpenAPI for all new endpoints |
| `time.rs` helpers (start_of_month, end_of_month, calculate_age) | Reuse for payroll periods, leave, contract dates |

---

## New Services

| Service | Purpose | Dependencies |
|---------|---------|-------------|
| `aos-workforce-service` | Employee lifecycle, contracts, compensation, attendance, leave, documents, compliance | aos-common, Keycloak, PostgreSQL, Redis, MinIO, Redpanda |
| `aos-payroll-service` | Deterministic payroll engine, DRC localization, payslips, approval workflow | aos-common, aos-workforce-service, PostgreSQL, Redis |
| `aos-payments-service` | Payment orchestration, provider adapters, transaction ledger, reconciliation, idempotency | aos-common, PostgreSQL, Redis, Redpanda |
| `aos-finance-service` | Chart of accounts, double-entry journal, payroll-to-accounting posting (replaces fintech stub) | aos-common, PostgreSQL |
| `aos-worker` | Background job runner: payroll processing, payslip generation, payment retries, reconciliation | aos-common, Redis, Redpanda, MinIO, PostgreSQL |

---

## Database Schema

### Migration Plan

| Migration | Tables | Purpose |
|-----------|--------|---------|
| `001_initial_schema.sql` | organizations, tenants, users, roles, memberships, tenant_features, audit_events | ✅ Exists |
| `002_workforce_schema.sql` | departments, job_titles, pay_grades, cost_centers, employees, employment_contracts, salaries, salary_components, employee_salary_components, attendance_records, leave_types, leave_requests, leave_balances | Workforce vertical |
| `003_payroll_schema.sql` | payroll_configurations, country_tax_configs, country_statutory_configs, payroll_runs, payslips, payroll_items | Payroll vertical |
| `004_payments_ledger_schema.sql` | payment_accounts, payment_batches, payments, payment_state_history, ledger_accounts, ledger_journals, ledger_entries, reconciliation_runs, reconciliation_items | Payments + Finance |
| `005_cross_cutting.sql` | currencies, exchange_rates, countries, permissions, outbox_events, idempotency_records, job_definitions, job_runs | Infrastructure |

### Schema Design Principles
- Every new table has `tenant_id UUID NOT NULL REFERENCES tenants(id)` or `org_id UUID NOT NULL REFERENCES organizations(id)`
- Every new table has RLS enabled with `tenant_isolation_*` or `org_isolation_*` policies
- All monetary columns use `NUMERIC(19,4)`, all rate columns use `NUMERIC(38,10)`
- Append-only tables (`payroll_items`, `ledger_entries`, `payment_state_history`) have `REVOKE UPDATE, DELETE` for app roles
- `payroll_runs` and `payments` have unique constraints preventing duplicate periods/batches
- All tables have `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`; mutable tables also have `updated_at`

### Key Relationships

```
Organization
  └── Tenant
        ├── Employee
        │     ├── EmploymentContract (1 active at a time, effective-dated)
        │     ├── Salary (effective-dated history)
        │     │     └── EmployeeSalaryComponent (overrides)
        │     ├── AttendanceRecord
        │     ├── LeaveRequest → LeaveBalance
        │     └── PaymentAccount (bank/mobile_money)
        │
        ├── PayrollConfiguration (pins country_tax_config + statutory_config versions)
        │     └── CountryTaxConfig (IPR progressive brackets)
        │     └── CountryStatutoryConfig (CNSS rates, SMIG)
        │
        ├── PayrollRun (immutable after confirm, snapshots config version)
        │     └── Payslip (immutable, versioned, correction chain)
        │           └── PayrollItem (append-only line ledger)
        │
        ├── PaymentBatch (from payroll or manual)
        │     └── Payment (state machine, idempotency key)
        │           └── PaymentStateHistory (append-only)
        │
        └── LedgerJournal (double-entry header)
              └── LedgerEntry (append-only, debit or credit)
```

---

## API Design

### Endpoint Map

| Domain | Base Path | Key Endpoints | Permission Model |
|--------|-----------|---------------|------------------|
| **Identity** | `/api/v1/identity` | roles, permissions, assignments | SUPER_ADMIN, PLATFORM_ADMIN |
| **Workforce** | `/api/v1/workforce` | employees, departments, positions, contracts, compensation, attendance, leave, documents, compliance, reports | employee.read/write, hr.manage |
| **Payroll** | `/api/v1/payroll` | periods, runs, preview, calculate, review, confirm, post, payslips, rules, countries/{code}/config | payroll.read/create/calculate/review/approve |
| **Payments** | `/api/v1/payments` | orders, approve, submit, retry, cancel, transactions, reconciliations, providers, webhooks | payment.read/create/approve |
| **Finance** | `/api/v1/finance` | accounts, journal-entries, trial-balance, balance-sheet, income-statement, periods | finance.read/post |
| **Portal** | `/api/v1/portal` | me, payslips, leave-requests, leave/balances, attendance, documents, payroll-info | EMPLOYEE (self-scoped) |

### Authentication Flow
```
Client → Keycloak (PKCE auth code) → Access Token (tenant_id, org_id, roles, employee_id, country_code)
→ API Gateway → JWT Validation (JWKS) → Tenant Context Extraction → RLS GUC → Permission Check → Handler
```

---

## Payroll Engine Design

### Determinism Requirements
- **Same inputs + same ruleset = same output** — always
- Pure calculation functions; no I/O, no randomness, no time-of-day dependency
- Snapshot `payroll_config_version` in every run
- Versioned effective-dated rule sets (`country_tax_configs`, `country_statutory_configs`)
- Fixed rounding mode per configuration (round_half_up, round_down, round_up)
- DRC timezone: `Africa/Lubumbashi` (CAT, UTC+2)

### Calculation Flow
```
Employee → Contract → Salary + Compensation Components
  ↓
Attendance (hours_worked, overtime_hours)
  ↓
Earnings: base_salary + allowances + overtime + bonus + ... 
  ↓
Proration: (actual_days / period_days) × component_amount
  ↓
Taxable Pay: sum(earnings where is_taxable=true) - tax_exemptions
  ↓
IPR: progressive brackets from CountryTaxConfig
  ↓
CNSS (Employee): min(taxable_pay × rate, ceiling) from CountryStatutoryConfig
  ↓
CNSS (Employer): min(gross_pay × rate, ceiling)
  ↓
Net Pay: gross_pay - deductions - IPR - CNSS_employee
  ↓
Employer Cost: gross_pay + CNSS_employer + other_employer_contributions
```

### DRC Localization (IPR)
Progressive tax brackets stored in `country_tax_configs.config`:
```json
{
  "bands": [
    {"from": 0, "to": 180000, "rate": 0},
    {"from": 180001, "to": 400000, "rate": 0.15},
    {"from": 400001, "to": 1000000, "rate": 0.20},
    {"from": 1000001, "to": 2000000, "rate": 0.25},
    {"from": 2000001, "to": 3500000, "rate": 0.30},
    {"from": 3500001, "to": Infinity, "rate": 0.40}
  ],
  "thresholds": {...},
  "currency": "CDF"
}
```
> **Note:** Actual DRC IPR rates must be verified with DRC tax authority (DGI). Values above are illustrative placeholders. CONFIGURATION_REQUIRED for unverified rates.

### Immutability
- Once `payroll_runs.status = 'confirmed'` → `REVOKE UPDATE, DELETE` on `payroll_items` and `payslips`
- Corrections require: Adjustment, Reversal, Off-cycle payroll, Correction payroll
- `payroll_items.id` uses `BIGSERIAL` (append-only, no reuse)
- Each payslip has `version` and `superseded_by_payslip_id` for correction chaining

---

## Payment Orchestration Design

### Provider Adapter Interface
```rust
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_payment(&self, req: CreatePaymentRequest) -> Result<PaymentResponse, ProviderError>;
    async fn get_status(&self, provider_ref: &str) -> Result<PaymentStatus, ProviderError>;
    async fn cancel(&self, provider_ref: &str) -> Result<(), ProviderError>;
    async fn verify_webhook(&self, payload: &[u8], signature: &str) -> bool;
}
```

### State Machine
```
created → validated → authorized → submitted → in_transit → completed
                    ↓                    ↓
                cancelled            failed → retrying → submitted
                                        ↓
                                    reversed/refunded
```

### Idempotency
- Every payment request carries an `idempotency_key`
- `UNIQUE(tenant_id, idempotency_key)` enforced at DB level
- Replay returns cached `response_payload` from `idempotency_records`
- TTL: 7 days, then cleanup

### Reconciliation
- Daily automated matching of `payments` vs `provider_transactions`
- Statuses: matched, missing, extra, amount_mismatch, disputed
- Dashboard: total / submitted / successful / pending / failed / reversed / unreconciled

---

## Background Jobs Architecture

### Pattern: Transactional Outbox + Redis Streams
- Domain mutations write to `outbox_events` in the same transaction
- `aos-worker` polls `outbox_events` → dispatches to Redis Streams
- Consumers process with exactly-once semantics (idempotency key + dedup table)
- Failed jobs retry with exponential backoff → dead letter after max attempts

### Job Catalog

| Job | Trigger | Purpose |
|-----|---------|---------|
| payroll_process | Period close / cron | Calculate draft payroll run |
| payroll_confirm | Reviewer approval | Lock run as immutable, snapshot rules |
| payslip_generate | After confirm | Render PDFs to MinIO |
| payslip_deliver | After generation | Email + portal notification |
| payroll_post_to_ledger | After confirm | Double-entry journal posting |
| payroll_execute_payments | After posting | Create payment orders |
| payment_retry | Failed attempt | Retry with backoff |
| payment_reconcile | Daily cron | Match provider vs internal |
| report_generate | Scheduled/on-demand | Generate reports to MinIO |
| leave_accrual | Monthly | Accrue leave balances |
| attendance_import | File upload | Parse + load batch records |
| contract_expiry_reminder | Daily | Notify HR before expiry |
| compliance_reminder | Weekly | CNSS/IPR filing deadlines |
| exchange_rate_refresh | Daily | FX rates for multi-currency |

---

## RBAC & Permissions

### Roles
| Role | Composite Contains | Key Permissions |
|------|-------------------|----------------|
| SUPER_ADMIN | PLATFORM_ADMIN + all | Full platform access |
| PLATFORM_ADMIN | TENANT_ADMIN | Platform-level operations |
| TENANT_OWNER | HR_ADMIN, PAYROLL_ADMIN, FINANCE_ADMIN, APPROVER, AUDITOR | Tenant full access |
| HR_ADMIN | — | employee.*, contract.*, department.*, position.* |
| PAYROLL_ADMIN | — | payroll.*, payslip.*, compensation.* |
| FINANCE_ADMIN | — | finance.*, payment.approve, reconciliation.* |
| ACCOUNTANT | — | finance.read, journal.create, report.read |
| MANAGER | — | employee.read, leave.approve, attendance.read |
| APPROVER | — | payroll.approve, payment.approve |
| AUDITOR | — | audit.*, report.*, finance.read |
| EMPLOYEE | — | portal.me.*, leave.create, attendance.self |

### Permission Catalog
```
employee.read, employee.create, employee.update, employee.lifecycle.transition
contract.read, contract.create, contract.update
department.read, department.create, department.update
compensation.read, compensation.create, compensation.update
attendance.read, attendance.create, attendance.approve
leave.read, leave.create, leave.approve
payroll.read, payroll.create, payroll.calculate, payroll.review, payroll.approve, payroll.lock
payslip.read, payslip.deliver
payment.read, payment.create, payment.approve, payment.retry, payment.cancel
finance.read, finance.post, finance.journal.create
reconciliation.read, reconciliation.run
audit.read, audit.export
report.read, report.export
```

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1–4)

**Goal:** Hardened core platform. Compilation fixes, tenant-scoped RLS, RBAC enforcement, append-only audit, hardened gateway, observability, and deployable staging. Security review gate passes.

| Track | Task ID | Title | Days | Dependencies |
|-------|---------|-------|------|-------------|
| Backend | P1-BE-01 | Fix workspace compilation (sqlx/redis in aos-common, unused imports) | 2 | — |
| Backend | P1-BE-02 | Harden API gateway (request-ID, timeouts, body limits, CORS, rate-limit) | 5 | — |
| Backend | P1-BE-03 | Tenant context + RLS middleware (set_config per-request) | 4 | P1-DB-01 |
| Backend | P1-BE-04 | Tenant service CRUD (org, tenant, user, role, membership) | 6 | P1-BE-03 |
| Backend | P1-BE-05 | Audit service (hash-chain, append-only enforcement, export) | 5 | P1-BE-03 |
| Backend | P1-BE-06 | Feature flag read path (default-disabled regulated features) | 3 | P1-BE-03 |
| Backend | P1-BE-07 | RBAC policy middleware (deny-by-default) | 4 | P1-AUTH-01 |
| Backend | P1-BE-08 | Shared money types (Money, CurrencyCode, rust_decimal policy) | 2 | — |
| Backend | P1-BE-09 | Idempotency-key middleware + idempotency_records table | 3 | P1-DB-01 |
| Database | P1-DB-01 | Migration runner + sqlx compile-time checks | 3 | — |
| Database | P1-DB-02 | Workforce schema migration (002_workforce_schema.sql) | 5 | P1-DB-01 |
| Database | P1-DB-03 | Payroll schema migration (003_payroll_schema.sql) | 4 | P1-DB-02 |
| Database | P1-DB-04 | Payments/finance schema migration (004_payments_ledger_schema.sql) | 5 | P1-DB-03 |
| Database | P1-DB-05 | Cross-cutting migration (005: currencies, permissions, outbox, jobs) | 3 | P1-DB-04 |
| Auth | P1-AUTH-01 | Keycloak production realm (MFA, key rotation, backup/restore) | 4 | — |
| Auth | P1-AUTH-02 | JWT validation + claim mapping | 4 | P1-AUTH-01 |
| Auth | P1-AUTH-03 | Seed roles/permissions + backup runbook | 3 | P1-AUTH-01 |
| CI/CD | P1-CICD-01 | Branch protection + secret scanning (gitleaks) | 2 | — |
| CI/CD | P1-CICD-02 | Terraform dev/staging environments | 5 | — |
| CI/CD | P1-CICD-03 | CD promotion to staging + ArgoCD apps | 4 | P1-CICD-02 |
| CI/CD | P1-CICD-04 | SBOM, image signing (cosign), cargo-deny baseline | 3 | P1-CICD-01 |
| Docs | P1-DOC-01 | OpenAPI v1 published + API reference | 3 | P1-BE-02 |
| Docs | P1-DOC-02 | Operational runbooks (incident, rotation, backup, DR) | 4 | — |
| Docs | P1-DOC-03 | Onboarding guide + ADR0003 (tenant-context/RLS model) | 3 | P1-BE-03 |
| Testing | P1-T-01 | Core unit tests (validation, JWT, tenant, feature-flag) | 4 | P1-BE-03, P1-AUTH-02 |
| Testing | P1-T-02 | Tenant isolation integration tests (fail-closed) | 4 | P1-BE-03 |
| Testing | P1-T-03 | Audit hash-chain + append-only enforcement tests | 3 | P1-BE-05 |

**Phase 1 Gate:** Security review, workspace compiles clean, CI green, RLS verified, audit tamper-evident, MFA/key rotation tested, staging environment healthy.

---

### Phase 2: Workforce MVP (Weeks 5–8)

**Goal:** Employee lifecycle, contracts, compensation, attendance, leave, documents, compliance records. HR_admin and employee portal can manage workforce.

| Track | Task ID | Title | Days | Dependencies |
|-------|---------|-------|------|-------------|
| Backend | P2-BE-01 | Employee service (CRUD, lifecycle state machine) | 6 | Phase 1 |
| Backend | P2-BE-02 | Department, position, cost center CRUD | 3 | P2-BE-01 |
| Backend | P2-BE-03 | Contract management (8 types, versioning) | 4 | P2-BE-01 |
| Backend | P2-BE-04 | Compensation engine (15+ component types, packages) | 5 | P2-BE-01, P1-BE-08 |
| Backend | P2-BE-05 | Attendance (clock in/out, batch import, approval) | 4 | P2-BE-01 |
| Backend | P2-BE-06 | Leave (types, requests, balances, approval workflow) | 4 | P2-BE-01 |
| Backend | P2-BE-07 | Document management (MinIO upload, metadata, expiry) | 3 | P2-BE-01 |
| Backend | P2-BE-08 | Compliance records (CNSS/IPR registration, filing tracking) | 3 | P2-BE-01 |
| Backend | P2-BE-09 | Workforce reports (headcount, turnover, attendance, leave) | 4 | P2-BE-01 |
| Backend | P2-BE-10 | Outbox event publisher (employee.changed, contract.created) | 3 | Phase 1 |
| Backend | P2-BE-11 | aos-worker: outbox dispatcher + basic job runner | 4 | Phase 1, P2-BE-10 |
| Frontend | P2-FE-01 | Next.js app shell + Keycloak PKCE auth flow | 5 | Phase 1 |
| Frontend | P2-FE-02 | Employee list + detail pages | 5 | P2-BE-01, P2-FE-01 |
| Frontend | P2-FE-03 | Contract + compensation management | 4 | P2-BE-03, P2-BE-04 |
| Frontend | P2-FE-04 | Attendance + leave request UI | 4 | P2-BE-05, P2-BE-06 |
| Frontend | P2-FE-05 | Employee self-service portal | 3 | P2-BE-01, P2-FE-01 |
| Testing | P2-T-01 | Employee lifecycle integration tests | 4 | P2-BE-01 |
| Testing | P2-T-02 | Contract + compensation edge-case tests | 3 | P2-BE-03, P2-BE-04 |
| Testing | P2-T-03 | Cross-tenant isolation regression suite | 3 | All P2 |

**Phase 2 Gate:** Load test, compliance review (employee data handling), E2E workforce flow, portal auth flow verified.

---

### Phase 3: Payroll MVP (Weeks 9–12)

**Goal:** DRC payroll engine, payslips, approval workflow, compliance filings. Deterministic calculation, immutable runs, audit trail.

| Track | Task ID | Title | Days | Dependencies |
|-------|---------|-------|------|-------------|
| Backend | P3-BE-01 | Country config service (IPR brackets, CNSS rates, SMIG) | 4 | Phase 2 |
| Backend | P3-BE-02 | Payroll configuration service | 3 | P3-BE-01 |
| Backend | P3-BE-03 | Deterministic payroll engine core | 8 | P3-BE-01, P3-BE-02, P2-BE-04 |
| Backend | P3-BE-04 | DRC IPR calculation (progressive brackets, exemptions) | 5 | P3-BE-03 |
| Backend | P3-BE-05 | CNSS/INSS calculation (employee + employer rates, ceiling) | 4 | P3-BE-03 |
| Backend | P3-BE-06 | Proration engine (partial periods, hire/terminate mid-period) | 3 | P3-BE-03 |
| Backend | P3-BE-07 | Payslip generation (HTML template → PDF → MinIO) | 4 | P3-BE-03 |
| Backend | P3-BE-08 | Payroll approval workflow (draft → review → approved → locked) | 4 | P3-BE-03 |
| Backend | P3-BE-09 | Payroll → ledger posting (double-entry journal) | 4 | P3-BE-08, P1-BE-04 |
| Backend | P3-BE-10 | Payroll → payments handoff | 3 | P3-BE-09 |
| Backend | P3-BE-11 | Payslip delivery (email + portal) | 3 | P3-BE-07 |
| Backend | P3-BE-12 | Rule versioning + golden file test framework | 5 | P3-BE-03 |
| Frontend | P3-FE-01 | Payroll period management UI | 3 | P3-BE-02 |
| Frontend | P3-FE-02 | Payroll run list + detail + preview | 5 | P3-BE-03 |
| Frontend | P3-FE-03 | Payslip viewer + download | 3 | P3-BE-07 |
| Frontend | P3-FE-04 | Approval workflow UI | 3 | P3-BE-08 |
| Frontend | P3-FE-05 | Employee portal: my payslips, payroll info | 2 | P3-BE-07 |
| Testing | P3-T-01 | DRC payroll golden tests (IPR, CNSS, SMIG, proration) | 5 | P3-BE-03 |
| Testing | P3-T-02 | Payroll determinism tests (same inputs = same output) | 3 | P3-BE-03 |
| Testing | P3-T-03 | Immutability enforcement tests (post-confirm no-modify) | 3 | P3-BE-08 |
| Testing | P3-T-04 | Payroll-to-ledger integration tests | 3 | P3-BE-09 |
| Testing | P3-T-05 | End-to-end payroll flow (employees → calculation → payslip → delivery) | 4 | All P3 |

**Phase 3 Gate:** DRC legal review, payroll calculation audit (external), golden file coverage, load test at 10x design capacity.

---

### Phase 4: Payments + Finance (Weeks 13–16)

**Goal:** Payment orchestration, provider adapters, transaction ledger, reconciliation, finance foundation. Full payroll-to-payout flow operational.

| Track | Task ID | Title | Days | Dependencies |
|-------|---------|-------|------|-------------|
| Backend | P4-BE-01 | Payment provider adapter interface + mock provider | 5 | Phase 3 |
| Backend | P4-BE-02 | Payment orchestration (state machine, idempotency) | 5 | P4-BE-01 |
| Backend | P4-BE-03 | Payment batch management (from payroll or manual) | 4 | P4-BE-02 |
| Backend | P4-BE-04 | Reconciliation engine (daily matching, exception workflow) | 5 | P4-BE-02 |
| Backend | P4-BE-05 | Webhook handler (provider callbacks, signature verification) | 3 | P4-BE-02 |
| Backend | P4-BE-06 | Finance: chart of accounts + journal posting | 4 | P3-BE-09 |
| Backend | P4-BE-07 | Finance: trial balance, balance sheet, income statement | 4 | P4-BE-06 |
| Backend | P4-BE-08 | Payment retry + dead-letter queue | 3 | P4-BE-02 |
| Backend | P4-BE-09 | Full payroll-to-payout integration | 5 | P4-BE-03, P3-BE-10 |
| Frontend | P4-FE-01 | Payment batch management UI | 4 | P4-BE-03 |
| Frontend | P4-FE-02 | Reconciliation dashboard | 3 | P4-BE-04 |
| Frontend | P4-FE-03 | Finance: chart of accounts + journal entries | 4 | P4-BE-06 |
| Frontend | P4-FE-04 | Finance: trial balance + reports | 3 | P4-BE-07 |
| Testing | P4-T-01 | Ledger integrity tests (double-entry, concurrent posting) | 5 | P4-BE-01 |
| Testing | P4-T-02 | Reconciliation exception tests | 4 | P4-BE-04 |
| Testing | P4-T-03 | E2E payroll → payout test | 5 | P4-BE-09 |
| Testing | P4-T-04 | Full regression + 7-day staging soak | 5 | All P4 |

**Phase 4 Gate:** External audit, penetration test, 7-day staging soak, DR restore test, go-live decision.

---

## Cross-Cutting Concerns

### CI/CD
- Branch protection with required checks: fmt, clippy, test, audit, deny, build, contract, docker/Trivy
- Secrets via GitHub Actions OIDC → External Secrets Operator → AWS Secrets Manager
- SBOM generation, cosign image signing, admission verification
- ArgoCD blue/green promotion, one-click rollback
- Feature-flag-controlled release of regulated domains until gates pass

### Observability
- OpenTelemetry tracing end-to-end (OTLP collector)
- Structured JSON logs, PII/financial data redaction
- Prometheus/AMP metrics + Grafana/AMG dashboards per SLO
- Alert rules for SLO breach and error-budget burn
- Audit hash-chain continuity monitor
- Reconciliation drift monitor

### Security
- Tenant-isolation integration suite (read/write/injection/direct-DB) on every PR
- WAF + API quotas + rate limiting at edge
- External penetration test before Phase 4 gate
- Supply chain: cargo-audit, cargo-deny, Trivy, gitleaks, cosign

### Testing Strategy
- Unit tests: ≥80% line coverage
- Integration tests on critical paths
- Golden calculation tests for DRC payroll (exact outputs, including rounding)
- Ledger double-entry invariant tests (concurrency + append-only)
- Cross-tenant isolation suite on every PR
- Load test at 10x design load + 7-day soak before launch

---

## Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|------------|
| R1 | DRC payroll rules change mid-build | HIGH | HIGH | Versioned config, compliance review is hard gate, golden tests |
| R2 | Rust hiring/onboarding slows delivery | MEDIUM | HIGH | Phase 1 onboarding guide, pair programming |
| R3 | Cross-tenant data leakage via RLS misconfiguration | LOW | HIGH | Fail-closed isolation suite in CI, periodic audit |
| R4 | DRC payment provider integration delays | MEDIUM | HIGH | Provider adapter abstraction, sandbox + stub providers |
| R5 | Dual-currency CDF/USD rounding errors | MEDIUM | HIGH | `rust_decimal` everywhere, explicit rounding policy, golden tests |
| R6 | Keycloak production hardening slips | MEDIUM | MEDIUM | Dedicated Identity Engineer, Phase 1 gate criteria |
| R7 | External audit surfaces late blocker | MEDIUM | HIGH | Internal pre-audit in Phase 4 week 1, publish evidence early |
| R8 | Modular monolith shared blast radius | MEDIUM | MEDIUM | Single CI migration check, health checks, ADR review |
| R9 | Scope creep on UX delays phase | MEDIUM | MEDIUM | Definition of Ready/Done checklists, feature flags |

---

## Resource Allocation

| Role | Count |
|------|-------|
| Backend Engineers | 4 |
| Frontend Engineers | 1 |
| DevOps/SRE | 1 |
| QA Engineers | 2 |
| Security Engineer | 1 |
| Technical Writer | 1 |
| **Total** | **10** |

---

## Conditions Before Implementation Begins

1. **Restore green build:** Add sqlx/redis to `services/common/Cargo.toml`, fix unused imports, make `cargo check --workspace` pass
2. **Adopt money policy:** `rust_decimal` + NUMERIC(19,4) everywhere, clippy guard banning floats for money
3. **Wire RLS:** `set_config('aos.current_tenant_id', ...)` per-request, add WITH CHECK policies, fix organizations policy
4. **Commission DRC payroll regulatory review** (IPR rates, CNSS rates, SMIG, filing obligations)
5. **User confirmation** to begin Phase 1 implementation

---

## Next Step

Begin **Phase 1, Task P1-BE-01**: Fix workspace compilation by adding `sqlx` and `redis` to `services/common/Cargo.toml` and fixing all unused imports.

Then proceed incrementally through Phase 1, completing each track in parallel where dependencies allow.
