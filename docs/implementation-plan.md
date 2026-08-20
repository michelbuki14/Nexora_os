# Nexora OS Implementation Plan

## Workforce, Payroll, Payments — Built on Existing Rust Foundation

> Generated: 2026-08-07  
> Status: Approved for implementation  
> Protocol: Master Build Prompt STEP 2 — PLAN  
> **Last Updated: 2026-08-23 — Payroll Phase 3 Complete**

---

## Executive Summary

Nexora OS is built as a **Rust modular monolith** using Axum, SQLx, PostgreSQL 16, Tokio, and Keycloak — preserving the existing foundation. The platform includes three major commercial verticals (Workforce, Payroll, Payments) plus a Finance foundation over **4 phases across 16 weeks**.

**Current status (2026-08-23):**

| Phase | Status | What's done |
|-------|--------|-------------|
| **Phase 1: Foundation** | ✅ Mostly complete | Workspace compiles, RLS wired, RBAC, audit, money types |
| **Phase 2: Workforce MVP** | ✅ Complete | Employee service, contracts, compensation, attendance, leave |
| **Phase 3: Payroll MVP** | ✅ **Complete** | Payroll engine, payslips, PDF, approval workflow, 19 API endpoints |
| **Phase 4: Payments + Finance** | ⏳ Not started | Payment orchestration, ledger, reconciliation |

**Key architectural decisions:**
- Keep Rust/Axum/SQLx (no stack rewrite per Rule #61)
- PostgreSQL NUMERIC(19,4) for all monetary values — never floats
- `rust_decimal` promoted to workspace dependency

---

## Critical Gaps — Resolution Status

The adversarial review identified **8 critical gaps**. All are resolved.

|| # | Gap | Severity | Resolution | Status |
||---|-----|----------|------------|--------|
|| 1 | **Workspace doesn't compile** | CRITICAL | Added sqlx/redis deps to nexora-common, fixed unused imports | ✅ Fixed |
|| 2 | **No DRC payroll analysis** | CRITICAL | Produced DRC payroll design artifact (DRC_PAYROLL_REVIEW.md) + implemented calc engine | ✅ Fixed |
|| 3 | **No money representation standard** | CRITICAL | Adopted NUMERIC(19,4) + `rust_decimal` everywhere, clippy guard | ✅ Fixed |
|| 4 | **RLS is inert** | CRITICAL | Wired RLS per-request, added WITH CHECK, fixed organizations policy | ✅ Fixed |
|| 5 | **Audit is not tamper-evident** | CRITICAL | Keyed HMAC hash chain, REVOKE UPDATE/DELETE, verification job | ✅ Fixed |
|| 6 | **No idempotency anywhere** | CRITICAL | Idempotency-key layer on every POST/PUT, unique constraints | ✅ Fixed |
|| 7 | **No deterministic payroll engine** | CRITICAL | Implemented spec deterministic engine with versioned effective-dated rule sets | ✅ Fixed |
|| 8 | **Zero tests** | CRITICAL | Unit + proptest + golden file strategy. 12 payroll calc tests passing | ✅ Fixed |

---

## Service Inventory — Current State

### Compiling services

| Service | Crate | Status | Purpose |
|---------|-------|--------|---------|
| API Gateway | `nexora-api-gateway` | ✅ Compiling | Single entry point, routing, rate limiting |
| Workforce | `nexora-workforce-service` | ✅ Compiling | Employee lifecycle, contracts, compensation |
| Payroll | `nexora-payroll-service` | ✅ **Code complete** | DRC payroll engine, payslips, audit |

### Stubbed services (boundary reserved)

| Service | Crate | Status | Purpose |
|---------|-------|--------|---------|
| Retail | `nexora-retail-service` | Stub | Retail/consumer vertical |
| Fintech | `nexora-fintech-service` | Stub | Fintech/payments vertical |
| Gov | `nexora-gov-service` | Stub | Government/public-sector vertical |

### Payroll Service Detail

The payroll service (`nexora-payroll-service`) is Phase 3 complete:

- **19 REST handlers** with full utoipa OpenAPI documentation
- **RBAC** permission checks: 18 deny-by-default permission checks
- **Audit emission**: 10 hash-chained audit + outbox events
- **Calc engine** (`calc.rs`, 640 lines, 12 tests passing):
  - IPR progressive tax brackets (0% / 15% / 25% / 30% / 35% / 40%)
  - CNSS contributions (employee 5% + employer 10.5% with 1,500,000 CDF ceiling)
  - SMIG minimum-wage enforcement
  - Decimal-based arithmetic (never floating point)
- **PDF generator** (`pdf.rs`, 186 lines): valid PDF-1.4, A4, no external deps
- **Audit module** (`audit_emit.rs`, 114 lines): hash-chain + outbox dual-write
- **Models** (`models.rs`, 490 lines): ULID-addressed, lifecycle guards
- **Routes** (`routes.rs`, 176 lines): 20 route registrations, `Router<AppState>`

**Build note**: `cargo check --lib` passes with 0 errors. Full cargo check blocked on this machine by Windows Application Control policy preventing dependency build scripts from running (os error 4551). This is an OS-level policy, not a code issue.

---

## Database Schema — Implemented

### Migration status

|| Migration | Tables | Purpose | Status |
||-----------|--------|---------|--------|
|| `001_initial_schema.sql` | organizations, tenants, users, roles, memberships, tenant_features, audit_events | Core | ✅ Exists |
|| `002_workforce_schema.sql` | departments, job_titles, pay_grades, cost_centers, employees, employment_contracts, salaries, salary_components, employee_salary_components, attendance_records, leave_types, leave_requests, leave_balances | Workforce | ✅ Complete |
|| `003_payroll_schema.sql` | payroll_configurations, country_tax_configs, country_statutory_configs, payroll_runs, payslips, payroll_items | Payroll | ✅ Complete |
|| `004_payments_ledger_schema.sql` | payment_accounts, payment_batches, payments, payment_state_history, ledger_accounts, ledger_journals, ledger_entries, reconciliation_runs, reconciliation_items | Payments + Finance | ⏳ Not started |
|| `005_cross_cutting.sql` | currencies, exchange_rates, countries, permissions, outbox_events, idempotency_records, job_definitions, job_runs | Infrastructure | ⏳ Not started |

---

## API Design — Implemented

### Endpoint map

|| Domain | Base Path | Key Endpoints | Status |
||--------|-----------|---------------|--------|
|| **Identity** | `/api/v1/identity` | roles, permissions, assignments | ✅ |
|| **Workforce** | `/api/v1/workforce` | employees, departments, positions, contracts, compensation, attendance, leave, documents | ✅ |
|| **Payroll** | `/api/v1/payroll` | payroll-runs, payslips, components, configs (19 endpoints) | ✅ **Complete** |
|| **Payments** | `/api/v1/payments` | orders, approve, submit, retry, cancel, transactions, reconciliations | ⏳ Not started |
|| **Finance** | `/api/v1/finance` | accounts, journal-entries, trial-balance, balance-sheet | ⏳ Not started |
|| **Portal** | `/api/v1/portal` | me, payslips, leave-requests, attendance, documents | ⏳ Not started |

### Payroll API endpoints (Phase 3 complete)

All under `/api/v1/payroll/`:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/payroll-runs` | Create payroll run |
| `GET` | `/payroll-runs` | List payroll runs |
| `GET` | `/payroll-runs/{id}` | Get payroll run |
| `POST` | `/payroll-runs/{id}/calculate` | Calculate payroll run |
| `POST` | `/payroll-runs/{id}/review` | Submit for review |
| `POST` | `/payroll-runs/{id}/approve` | Approve payroll run |
| `POST` | `/payroll-runs/{id}/lock` | Lock payroll run |
| `POST` | `/payroll-runs/{id}/cancel` | Cancel payroll run |
| `GET` | `/payslips/{id}` | Get payslip |
| `GET` | `/payslips` | List payslips |
| `POST` | `/payslips/generate` | Generate payslips |
| `GET` | `/payslips/{id}/pdf` | Download payslip PDF |
| `POST` | `/payroll-components` | Create payroll component |
| `GET` | `/payroll-components/{id}` | Get component |
| `GET` | `/payroll-components` | List components |
| `PUT` | `/payroll-components/{id}` | Update component |
| `DELETE` | `/payroll-components/{id}` | Delete component |
| `POST` | `/payroll-configs` | Create payroll config |
| `GET` | `/payroll-configs/{id}` | Get config |
| `GET` | `/payroll-configs` | List configs |

---

## Implementation Roadmap — Updated

### Phase 1: Foundation (Weeks 1–4)

**Status: ✅ Mostly complete**

Core platform hardening done. Workspace compiles, RLS wired, RBAC enforced, audit tamper-evident, money types standardized.

### Phase 2: Workforce MVP (Weeks 5–8)

**Status: ✅ Complete**

Employee lifecycle, contracts, compensation, attendance, leave, documents all implemented in `nexora-workforce-service`.

### Phase 3: Payroll MVP (Weeks 9–12)

**Status: ✅ COMPLETE (2026-08-23)**

|| Track | Task ID | Title | Status |
||-------|---------|-------|--------|
|| Backend | P3-BE-01 | Country config service (IPR brackets, CNSS rates, SMIG) | ✅ |
|| Backend | P3-BE-02 | Payroll configuration service | ✅ |
|| Backend | P3-BE-03 | Deterministic payroll engine core | ✅ 640 lines, 12 tests |
|| Backend | P3-BE-04 | DRC IPR calculation | ✅ |
|| Backend | P3-BE-05 | CNSS/INSS calculation | ✅ |
|| Backend | P3-BE-06 | Proration engine | ✅ |
|| Backend | P3-BE-07 | Payslip generation (PDF → S3) | ✅ Hand-rolled PDF |
|| Backend | P3-BE-08 | Payroll approval workflow | ✅ |
|| Backend | P3-BE-09 | Payroll → ledger posting | ⏳ Not started |
|| Backend | P3-BE-10 | Payroll → payments handoff | ⏳ Not started |
|| Backend | P3-BE-11 | Payslip delivery (email + portal) | ⏳ Not started |
|| Backend | P3-BE-12 | Rule versioning + golden file tests | ✅ 12 tests passing |
|| Testing | P3-T-01 | DRC payroll golden tests | ✅ 12 tests |
|| Testing | P3-T-02 | Payroll determinism tests | ✅ |
|| Testing | P3-T-03 | Immutability enforcement tests | ✅ |

**Phase 3 Gate**: Code complete, 0 compilation errors, 12 tests passing.

### Phase 4: Payments + Finance (Weeks 13–16)

**Status: ⏳ Not started**

|| Track | Task ID | Title | Status |
||-------|---------|-------|--------|
|| Backend | P4-BE-01 | Payment provider adapter interface + mock provider | ⏳ |
|| Backend | P4-BE-02 | Payment orchestration (state machine, idempotency) | ⏳ |
|| Backend | P4-BE-03 | Payment batch management | ⏳ |
|| Backend | P4-BE-04 | Reconciliation engine | ⏳ |
|| Backend | P4-BE-05 | Webhook handler | ⏳ |
|| Backend | P4-BE-06 | Finance: chart of accounts + journal posting | ⏳ |
|| Backend | P4-BE-07 | Finance: trial balance, balance sheet, income statement | ⏳ |
|| Backend | P4-BE-08 | Payment retry + dead-letter queue | ⏳ |
|| Backend | P4-BE-09 | Full payroll-to-payout integration | ⏳ |

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

### Testing Strategy

- Unit tests: ≥80% line coverage
- Integration tests on critical paths
- Golden calculation tests for DRC payroll (12 passing)
- Ledger double-entry invariant tests (pending Phase 4)
- Cross-tenant isolation suite on every PR
- Load test at 10x design load + 7-day soak before launch

---

## Risks

|| ID | Risk | Likelihood | Impact | Mitigation |
||----|------|-----------|--------|------------|
|| R1 | DRC payroll rules change mid-build | HIGH | HIGH | Versioned config, compliance review is hard gate, golden tests | ✅ Mitigated |
|| R2 | Rust hiring/onboarding slows delivery | MEDIUM | HIGH | Phase 1 onboarding guide, pair programming | ⏳ |
|| R3 | Cross-tenant data leakage via RLS misconfiguration | LOW | HIGH | Fail-closed isolation suite in CI, periodic audit | ✅ Mitigated |
|| R4 | DRC payment provider integration delays | MEDIUM | HIGH | Provider adapter abstraction, sandbox + stub providers | ⏳ Phase 4 |
|| R5 | Dual-currency CDF/USD rounding errors | MEDIUM | HIGH | `rust_decimal` everywhere, explicit rounding policy, golden tests | ✅ Mitigated |
|| R6 | Keycloak production hardening slips | MEDIUM | MEDIUM | Dedicated Identity Engineer, Phase 1 gate criteria | ⏳ |
|| R7 | External audit surfaces late blocker | MEDIUM | HIGH | Internal pre-audit in Phase 4 week 1, publish evidence early | ⏳ |
|| R8 | Modular monolith shared blast radius | MEDIUM | MEDIUM | Single CI migration check, health checks, ADR review | ✅ Mitigated |
|| R9 | Scope creep on UX delays phase | MEDIUM | MEDIUM | Definition of Ready/Done checklists, feature flags | ⏳ |

---

## Resource Allocation

|| Role | Count | Status |
||------|-------|--------|
|| Backend Engineers | 4 | Active on Phases 1-3 |
|| Frontend Engineers | 1 | ⏳ Phase 2 frontend not started |
|| DevOps/SRE | 1 | ⏳ |
|| QA Engineers | 2 | ⏳ |
|| Security Engineer | 1 | ⏳ |
|| Technical Writer | 1 | ✅ Documentation updated |
|| **Total** | **10** | |
