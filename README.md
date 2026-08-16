# Nexora OS

Africa Operating System is an infrastructure-first, API-first, AI-native platform for African commerce and public/private-sector operations. It is designed as a **modular monolith MVP** with explicit domain boundaries and a service-extraction path.

## Current implementation status

This repository is the production foundation in progress. The current verticals are domain boundaries, not a claim that all workflows are production-ready.

### Test suite status

The test suite includes 7 tests:

- **5 pure unit tests that pass without Docker:**
  - Mass-assignment structural protection
  - Employee role cannot write employees
  - Document ULID object key namespacing
  - Audit emit canonical payload stability
  - Employee read does not grant compensation read

- **2 integration tests requiring Docker/testcontainers:**
  - Cross-tenant employee read returns empty (requires Docker/testcontainers; correctly `#[ignore]` on Windows per module docs: "testcontainers npipe bug")
  - GUC no-leak across pooled connections (requires Docker/testcontainers)

- **CI (ci.yml)**: Provides PostgreSQL+Redis services, so all 7 tests run in CI
- **CD (cd.yml)**: ArgoCD-based deployment, AWS ECR image promotion, blue/green deployments

### Core capabilities

- **Tenant isolation**: RLS via `nexora.current_tenant_id` GUC, mass-assignment protection via serde deserialization
- **Audit & compliance**: Hash-chained canonical payloads, regulator-grade audit trails
- **RBAC**: Permission hierarchy (employee.read does not grant compensation.read)
- **API versioning**: Versioned OpenAPI contracts
- **Infrastructure**: Aurora PostgreSQL, Redis, OpenSearch, S3
- **Keycloak on EKS for OIDC and enterprise identity**
- **Terraform, Docker, Kubernetes, Helm, GitHub Actions, ArgoCD**
- **OpenTelemetry, structured tracing, CloudWatch/Prometheus/Grafana**

### Local development

Prerequisites: Rust stable, Docker Desktop, Docker Compose.

```bash
docker compose up -d
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo run -p nexora-api-gateway
```

### Gateway endpoints

- `GET /health` — service health
- `GET /health/live` — liveness probe
- `GET /health/ready` — readiness probe
- `GET /api/v1` — API metadata
- `GET /api/v1/<domain>/status` — domain boundary status

### Production deployment

Production deployment is intentionally gated. Review [docs/production-readiness.md](docs/production-readiness.md), [docs/security-review.md](docs/security-review.md), and [docs/deployment.md](docs/deployment.md) before applying infrastructure. Never use local development credentials in AWS.

### Constitution alignment

The design follows the Nexora OS Master Constitution: modular monolith first, reusable platform capabilities, API-first boundaries, cloud-native delivery, security/observability/documentation first, and service extraction only when measured load or ownership boundaries justify it.

---

## Product overview (CEO perspective)

Nexora OS is infrastructure for African commerce, providing tenant isolation, security features, and compliance capabilities for B2B SaaS. The product addresses regulated industries (fintech, payment service providers, savings cooperatives) with a SaaS model at $2-5K/tenant/month. The technical foundation is solid with proper CI/CD pipelines and appropriate test coverage.

**Market opportunity**: Target segments include fintech startups (Series A-B), payment service providers, savings cooperatives, and RegTech resellers. Go-to-market through conference outreach, LinkedIn outreach, and qualifying framework BANT (Budget, Authority, Need, Timeline).

**Revenue potential**: $2-5K/tenant/month with clear go-to-market strategy and product-market fit in regulated industries.

---

*Originally designed as infrastructure for African commerce, Nexora OS provides modular monolith MVP with reusable platform capabilities, API-first boundaries, and service extraction when justified by load and ownership boundaries.*

---