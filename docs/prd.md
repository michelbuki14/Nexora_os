# AOS MVP Product Requirements Document

## Problem
African organizations operate across fragmented identity, commerce, payments, compliance, logistics, and government systems. They need reusable infrastructure that works across countries, currencies, connectivity conditions, and regulatory environments.

## Business context
AOS is infrastructure for African commerce, not a collection of disconnected vertical applications. The MVP establishes shared tenant, identity, authorization, audit, workflow, data, and observability capabilities. Vertical domains consume those primitives through stable APIs.

## MVP users
- Tenant administrators managing organizations, memberships, roles, and integrations
- Developers integrating AOS APIs
- Operations and compliance teams reviewing immutable activity
- Design partners in one selected vertical at a time

## User stories
1. As a tenant administrator, I can create an organization and manage memberships without accessing another tenant's data.
2. As a developer, I can discover versioned APIs, authenticate with OIDC, and receive consistent errors and request IDs.
3. As an auditor, I can query immutable tenant-scoped audit records and export approved records.
4. As an operator, I can see health, readiness, latency, error, and dependency status.
5. As a design partner, I can enable a vertical only after its compliance and operational gates pass.

## Acceptance criteria
- Every authenticated domain request has verified subject, tenant, authorization, request ID, and audit context.
- Tenant isolation is enforced in application code and PostgreSQL RLS; cross-tenant tests fail closed.
- All write operations define idempotency behavior, validation, authorization, audit events, and migration coverage.
- API contracts are versioned and published as OpenAPI.
- Deployment is reproducible from clean infrastructure code, with rollback and restore procedures tested.
- No regulated financial or citizen data is accepted before a documented launch gate approves it.

## Non-goals
- Simultaneous production launch of all listed industries
- Building a new identity provider or payment network
- Moving money, issuing government identity, or making regulated decisions without external compliance review
