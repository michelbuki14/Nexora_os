# ADR 0001: Modular Monolith with Service Extraction Path

## Status
Accepted

## Context
Nexora OS must deliver reusable platform capabilities (identity, tenancy, audit, authorization, data, workflow, observability) while supporting independent vertical domain development. Starting with distributed microservices adds operational complexity before domain boundaries are proven.

## Decision
Adopt a **modular monolith** as the MVP architecture. Domain modules share a single deployment unit (Axum server) with strict internal boundaries:
- Each domain owns its database schema (or RLS policies)
- Cross-domain calls use explicit typed interfaces, not direct database access
- Domain events are published to an internal event bus for async work
- Extraction to independent services is a documented, reversible step

## Consequences
**Positive**
- Single deployment, single database, single tracing context
- Fast local development and testing
- Refactoring across boundaries is safe with compiler support
- Clear extraction triggers (team ownership, load isolation, fault isolation, compliance isolation)

**Negative**
- All domains share the blast radius of a process crash (mitigated by health checks and fast restarts)
- Database schema changes require coordinated migrations (enforced by CI)
- Scaling is vertical until extraction (Aurora Serverless v2 handles initial load)

## Extraction criteria
A domain MAY be extracted when ALL are true:
1. Dedicated team owns the domain
2. Measured load justifies independent scaling
3. Fault-isolation requirement documented and approved
4. Compliance boundary requires independent audit scope
5. Database schema is fully owned and migration-compatible
6. Event contracts are versioned and consumers migrated

## References
- Shopify Modular Monolith
- Martin Fowler MonolithFirst
- AWS Well-Architected Monolith to Microservices