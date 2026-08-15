# Nexora OS

Africa Operating System is an infrastructure-first, API-first, AI-native platform for African commerce and public/private-sector operations. It is designed as a **modular monolith MVP** with explicit domain boundaries and a service-extraction path.

## Current implementation status

This repository is the production foundation in progress. The current verticals are domain boundaries, not a claim that all workflows are production-ready:

- Core: identity integration, tenant/organization context, authorization boundary, audit boundary, observability primitives
- Commerce boundary: catalog, inventory, orders, payments integration points
- Finance boundary: ledger, KYC/KYB, reconciliation and compliance integration points
- Government boundary: verification, permits, public-service workflows and retention controls

Financial movement, citizen identity, and regulated workflows remain disabled until their compliance, reconciliation, threat-model, external-integration, and operational gates pass.

## Technology

- Rust, Axum, Tokio, SQLx
- Aurora PostgreSQL, Redis, OpenSearch, S3
- Keycloak on EKS for OIDC and enterprise identity
- Terraform, Docker, Kubernetes, Helm, GitHub Actions, ArgoCD
- OpenTelemetry, structured tracing, CloudWatch/Prometheus/Grafana

## Local development

Prerequisites: Rust stable, Docker Desktop, Docker Compose.

```bash
docker compose up -d
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo run -p nexora-api-gateway
```

Gateway endpoints:

- `GET /health` — service health
- `GET /health/live` — liveness probe
- `GET /health/ready` — readiness probe
- `GET /api/v1` — API metadata
- `GET /api/v1/<domain>/status` — domain boundary status

## Production deployment

Production deployment is intentionally gated. Review [docs/production-readiness.md](docs/production-readiness.md), [docs/security-review.md](docs/security-review.md), and [docs/deployment.md](docs/deployment.md) before applying infrastructure. Never use local development credentials in AWS.

## Constitution alignment

The design follows the Nexora OS Master Constitution: modular monolith first, reusable platform capabilities, API-first boundaries, cloud-native delivery, security/observability/documentation first, and service extraction only when measured load or ownership boundaries justify it.
