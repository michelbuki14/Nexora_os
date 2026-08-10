# Production Readiness Checklist

This checklist prevents the repository from claiming production readiness before objective gates pass.

## Product and architecture
- [ ] PRD and acceptance criteria approved
- [ ] Domain boundaries and extraction criteria documented
- [ ] API contracts versioned and reviewed
- [ ] Data residency and country launch scope approved

## Security
- [ ] Threat model reviewed
- [ ] External penetration test complete
- [ ] Dependency, image, IaC, secret, and license scans pass
- [ ] Keycloak MFA/SSO, key rotation, backup/restore tested
- [ ] Tenant isolation and privilege-escalation tests pass

## Reliability
- [ ] SLOs and error budgets defined
- [ ] Load and soak tests pass against capacity target
- [ ] Timeouts, retries, circuit breakers, and idempotency verified
- [ ] Database migration rollback/recovery tested
- [ ] Aurora/S3/EKS disaster recovery tested

## Operations
- [ ] Dashboards and alerts cover every SLO
- [ ] On-call, escalation, and incident runbooks published
- [ ] Audit export, retention, and deletion workflows exercised
- [ ] Cost budgets and alerts configured

## Compliance and vertical gates
- [ ] Legal/regulatory assessment complete per launch country
- [ ] Financial movement has reconciliation and provider settlement controls
- [ ] Government/citizen data has approved retention, access, and residency controls
- [ ] Vertical feature flag remains disabled until every gate above passes

## Release decision
The technical team may recommend launch only when all mandatory controls are complete. A green build alone is not production readiness.
