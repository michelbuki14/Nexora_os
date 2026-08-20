# Nexora OS Security Review — Foundation Baseline

## Scope

This review covers the Rust workspace foundation, local stack, tenant schema, API gateway boundaries, and planned AWS delivery controls.

## Required controls

|| Control | Implementation / gate | Status |
||---|---|---|**
|| Authentication | Keycloak OIDC, JWKS validation, RS256 allowlist, MFA/SSO configured per realm | Foundation boundary; production configuration required |
|| Authorization | Tenant context + role/permission claims; deny-by-default domain handlers | ✅ Implemented — 18 RBAC checks in payroll service alone |
|| Tenant isolation | PostgreSQL RLS plus application tenant context | ✅ Schema migration included; integration tests required |
|| Secrets | AWS Secrets Manager/IRSA; no production secrets in repository | Required before AWS apply |
|| Encryption | TLS at ingress and service boundaries; KMS for Aurora/S3/Secrets | Required in Terraform |
|| Audit | Append-only audit table, actor/resource/request context, hash chain design | ✅ Implemented — hash-chain + outbox in payroll and workforce services |
|| API security | Request ID, timeout, CORS restriction, validation, rate limiting, OpenAPI | ✅ Gateway baseline; utoipa OpenAPI on all endpoints |
|| Supply chain | cargo-audit, cargo-deny, SBOM, Trivy, signed images | CI workflow included; branch protections required |
|| Infrastructure | Terraform least privilege, private subnets, WAF, CloudTrail, GuardDuty, Security Hub | Required before production |
|| Data protection | Retention, deletion, export, data residency per country | Product and legal approval required |

## Threats and mitigations

- **Cross-tenant data access:** RLS, tenant context middleware, authorization tests, no cross-domain direct SQL. ✅ Mitigated — 2 integration tests.
- **Token substitution/replay:** issuer/audience/algorithm checks, short expiry, JWKS cache refresh, Keycloak session controls.
- **Credential exposure:** secret scanning, OIDC federation, Secrets Manager, no static AWS keys in CI.
- **Abuse and denial of service:** WAF, API Gateway quotas, service timeouts, bounded body sizes, autoscaling, circuit breakers.
- **Audit tampering:** append-only privileges, restricted service role, hash continuity checks, immutable export storage. ✅ Implemented — hash-chain + outbox in payroll.
- **Supply-chain compromise:** pinned action versions, lockfiles, dependency audit, SBOM, image signing and admission verification.
- **Payroll calculation error:** deterministic engine, golden file tests (12 passing), versioned rules, external audit before launch.

## Payroll-specific security considerations

The payroll service handles sensitive financial and personal data. Key controls:

1. **Immutability**: Once `payroll_runs.status = 'locked'`, `REVOKE UPDATE, DELETE` on `payroll_items` and `payslips`. Corrections require new payroll runs (adjustment, reversal, off-cycle).
2. **Audit trail**: Every payroll run stores `ruleset_version` used. Hash-chained audit events for all mutations.
3. **PDF payslips**: Generated as valid PDF-1.4, stored in S3 with tenant-scoped access. No external PDF library dependencies.
4. **DRC regulatory compliance**: IPR, CNSS, SMIG calculations use Decimal arithmetic (never floating point), with 12 unit tests covering all bracket scenarios.

## Launch blockers

1. External penetration test and remediation.
2. Verified tenant-isolation integration tests, including malicious SQL and authorization paths. ✅ 2 tests implemented.
3. Production Keycloak hardening, backup/restore, key rotation, and SSO/MFA tests.
4. AWS IAM review, CloudTrail/GuardDuty/Security Hub enabled, and incident response runbook exercised.
5. Financial and government modules remain disabled until regulatory counsel, reconciliation, retention, and data-residency controls are approved.
6. **Payroll legal review**: DRC IPR brackets, CNSS rates, and SMIG values must be confirmed with DGI/CNSS for 2025 before payroll service can process real data.
