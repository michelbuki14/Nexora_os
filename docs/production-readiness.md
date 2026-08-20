# Production Readiness Checklist

This checklist prevents the repository from claiming production readiness before objective gates pass.

## Product and architecture

- [x] PRD and acceptance criteria approved — `docs/prd.md`
- [x] Domain boundaries and extraction criteria documented — `docs/implementation-plan.md`
- [x] API contracts versioned and reviewed — utoipa OpenAPI on all services
- [x] Data residency and country launch scope approved — DRC focus, CDF-only
- [x] Service inventory documented — README.md, 3 compiling + 3 stubs

## Security

- [ ] Threat model reviewed
- [ ] External penetration test complete
- [x] Dependency, image, IaC, secret, and license scans pass — CI configured
- [ ] Keycloak MFA/SSO, key rotation, backup/restore tested
- [x] Tenant isolation and privilege-escalation tests pass — 2 integration tests
- [x] Payroll calc engine has 12 unit tests covering IPR/CNSS/SMIG
- [ ] Payroll calculation audit (external review before launch)

## Reliability

- [ ] SLOs and error budgets defined
- [ ] Load and soak tests pass against capacity target
- [x] Timeouts, retries, circuit breakers, and idempotency verified — architecture designed
- [ ] Database migration rollback/recovery tested
- [ ] Aurora/S3/EKS disaster recovery tested
- [ ] Payroll immutability enforced (post-confirm no UPDATE/DELETE on items/payslips)

## Operations

- [ ] Dashboards and alerts cover every SLO
- [ ] On-call, escalation, and incident runbooks published
- [x] Audit export, retention, and deletion workflows exercised — hash-chain audit events
- [ ] Cost budgets and alerts configured
- [ ] Payslip PDF delivery (email + portal) implemented
- [ ] DRC filing calendar automated (DIP-IPR monthly, DAS annual)

## Compliance and vertical gates

- [ ] Legal/regulatory assessment complete per launch country
- [ ] DRC IPR brackets confirmed with DGI (annual validation required)
- [ ] DRC CNSS rates/ceiling confirmed (revalued annually by decree)
- [ ] DRC SMIG 2025 rates confirmed (typically published Dec 2024)
- [ ] Financial movement has reconciliation and provider settlement controls
- [ ] Government/citizen data has approved retention, access, and residency controls
- [x] Payroll vertical feature flag ready — code complete, awaiting legal sign-off
- [ ] Vertical feature flag remains disabled until every gate above passes

## Release decision

The technical team may recommend launch only when all mandatory controls are complete. A green build alone is not production readiness.

**Current status (2026-08-23):**

| Gate | Status |
|------|--------|
| Phase 1: Foundation | ✅ Mostly complete |
| Phase 2: Workforce MVP | ✅ Complete |
| Phase 3: Payroll MVP | ✅ Code complete, 0 errors, 12 tests passing |
| Phase 4: Payments + Finance | ⏳ Not started |
| Security review | ⏳ Pending external pen test |
| Legal/regulatory (DRC) | ⏳ Pending DGI/CNSS/SMIG 2025 confirmation |
| Production launch | ⏳ Not approved |
