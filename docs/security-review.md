# AOS Security Review — Foundation Baseline

## Scope
This review covers the Rust workspace foundation, local stack, tenant schema, API gateway boundaries, and planned AWS delivery controls.

## Required controls

| Control | Implementation / gate | Status |
|---|---|---|
| Authentication | Keycloak OIDC, JWKS validation, RS256 allowlist, MFA/SSO configured per realm | Foundation boundary; production configuration required |
| Authorization | Tenant context + role/permission claims; deny-by-default domain handlers | Foundation boundary; policy tests required |
| Tenant isolation | PostgreSQL RLS plus application tenant context | Schema migration included; integration tests required |
| Secrets | AWS Secrets Manager/IRSA; no production secrets in repository | Required before AWS apply |
| Encryption | TLS at ingress and service boundaries; KMS for Aurora/S3/Secrets | Required in Terraform |
| Audit | Append-only audit table, actor/resource/request context, hash chain design | Schema included; service implementation required |
| API security | Request ID, timeout, CORS restriction, validation, rate limiting, OpenAPI | Gateway baseline; production origin/rate policy required |
| Supply chain | cargo-audit, cargo-deny, SBOM, Trivy, signed images | CI workflow included; branch protections required |
| Infrastructure | Terraform least privilege, private subnets, WAF, CloudTrail, GuardDuty, Security Hub | Required before production |
| Data protection | Retention, deletion, export, data residency per country | Product and legal approval required |

## Threats and mitigations

- **Cross-tenant data access:** RLS, tenant context middleware, authorization tests, no cross-domain direct SQL.
- **Token substitution/replay:** issuer/audience/algorithm checks, short expiry, JWKS cache refresh, Keycloak session controls.
- **Credential exposure:** secret scanning, OIDC federation, Secrets Manager, no static AWS keys in CI.
- **Abuse and denial of service:** WAF, API Gateway quotas, service timeouts, bounded body sizes, autoscaling, circuit breakers.
- **Audit tampering:** append-only privileges, restricted service role, hash continuity checks, immutable export storage.
- **Supply-chain compromise:** pinned action versions, lockfiles, dependency audit, SBOM, image signing and admission verification.

## Launch blockers

1. External penetration test and remediation.
2. Verified tenant-isolation integration tests, including malicious SQL and authorization paths.
3. Production Keycloak hardening, backup/restore, key rotation, and SSO/MFA tests.
4. AWS IAM review, CloudTrail/GuardDuty/Security Hub enabled, and incident response runbook exercised.
5. Financial and government modules remain disabled until regulatory counsel, reconciliation, retention, and data-residency controls are approved.
