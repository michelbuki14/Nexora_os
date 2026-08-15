# ADR 0002: Rust and Axum for Core Services

## Status
Accepted

## Context
Nexora OS requires high-throughput APIs, predictable memory behavior, strong compile-time guarantees, safe concurrency, and long-term maintainability. The constitution permits TypeScript, Go, Rust, Python, and SQL; the user selected Rust for deployment.

## Decision
Use Rust stable with Axum/Tokio for HTTP services and SQLx for PostgreSQL. Use explicit ports/adapters for external providers, serde for contracts, and utoipa for OpenAPI.

## Trade-offs
- **Benefits:** memory safety, low runtime overhead, reliable concurrency, compile-time SQL checks, strong type-level domain models.
- **Costs:** higher hiring/onboarding bar, longer initial implementation time, fewer off-the-shelf business integrations than Node.js.
- **Alternative rejected:** Node/Express is faster for early hiring, but does not provide Rust's memory-safety and predictable resource profile. Go remains a valid future service-extraction language.

## Guardrails
No unsafe Rust without security review. Dependencies require audit/license checks. Domain modules must remain independently testable and must not access another domain's storage directly.
