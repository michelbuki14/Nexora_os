//! Audit service route definitions and middleware composition.
//!
//! The public health router comes from [`nexora_common::health::health_router`].
//! The protected `/api/v1/audit/*` routes are assembled here with the
//! middleware order: **auth first** (extracts JWT → `AuthContext`), then
//! **RLS** (reads `AuthContext`, sets GUCs, stores `DbConn`).
//!
//! Axum composes `.layer()` calls bottom-to-top: the last layer added is the
//! *outermost* and runs first. `auth_middleware` is therefore added last so it
//! populates the request extensions that `rls_middleware` reads.

use crate::handlers::{create_audit_event, get_audit_event, list_audit_events, verify_hash_chain};
use nexora_common::{auth_middleware, tenant_context::rls_middleware, AuthState, RlsState};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

/// Protected audit routes with auth + RLS middleware applied.
///
/// The returned router has no state bound yet — the caller pins `AppState`
/// onto it (handlers extract `DbConn`/`AuthContext` from request extensions,
/// not state, so any `S` works).
pub fn protected_audit_routes(rls_state: RlsState, auth_state: AuthState) -> Router<()> {
    Router::new()
        // Create an audit event (append-only).
        .route("/events", post(create_audit_event))
        // List events (paginated, filtered).
        .route("/events", get(list_audit_events))
        // Get single event by serial ID.
        .route("/events/:id", get(get_audit_event))
        // Verify hash-chain integrity up to a given event.
        .route("/events/verify/:id", get(verify_hash_chain))
        // RLS middleware is innermost: it runs after auth has set AuthContext.
        .layer(middleware::from_fn_with_state(rls_state, rls_middleware))
        // Auth middleware is outermost (runs first), so AuthContext is in the
        // extensions by the time RLS middleware reads it.
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
}
