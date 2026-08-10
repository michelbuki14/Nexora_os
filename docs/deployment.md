# Deployment Guide

## Prerequisites
- AWS account with Organizations, billing, and SCPs configured
- Terraform >= 1.6, Helm >= 3.12, kubectl >= 1.28, ArgoCD CLI
- AWS CLI with OIDC role for GitHub Actions (no static credentials)
- Keycloak realm export and TLS certificates

## Account structure
```
management (billing, SCPs, IAM roles)
  └── shared (Route53 hosted zone, ACM certs, central logging)
  └── workload-dev
  └── workload-staging
  └── workload-prod (per region if multi-region)
```

## Terraform apply order
```bash
cd infra/environments/dev
terraform init -backend-config="bucket=aos-terraform-state-dev"
terraform apply -target module.vpc -target module.eks -target module.aurora -target module.elasticache
# verify core infra
terraform apply
```

## EKS add-ons (managed)
- VPC CNI
- CoreDNS
- kube-proxy
- EBS CSI Driver
- AWS Load Balancer Controller
- External Secrets Operator
- Cert Manager
- Prometheus (AMP) / Grafana (AMG) / X-Ray
- ArgoCD (Helm)

## ArgoCD applications
Each service has an Application manifest in `helm/<service>/argocd-application.yaml` pointing to the environment Kustomize overlay.

## Blue/Green production
- Two target groups behind ALB, ArgoCD manages the active set
- Smoke tests against preview before cutover
- One-click rollback via ArgoCD history

## Secrets
- CI: OIDC role, no keys
- Runtime: External Secrets Operator → AWS Secrets Manager → K8s secrets
- Keycloak: admin password in Secrets Manager, realm config in Git (no secrets)

## Disaster recovery
- Aurora: point-in-time restore, cross-region snapshot copy (RPO 5min, RTO 30min target)
- S3: cross-region replication, versioning
- EKS: cluster re-creation via Terraform, ArgoCD re-sync
- Tested quarterly

## Verification checklist before production apply
- [ ] All CI jobs pass on main
- [ ] PR has security review approval
- [ ] Infra plan shows no unintended destruction
- [ ] Staging deployment healthy for 24h with traffic
- [ ] DR restore tested in last 90 days
- [ ] Penetration test report accepted
- [ ] Compliance sign-off for regulated modules