//! Workforce service route configuration.
//!
//! Middleware order matches audit-service (load-bearing): auth middleware is
//! added LAST so it is outermost and populates extensions before RLS reads them.

use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use nexora_common::{
    auth_middleware::{auth_middleware, AuthState},
    config::Config,
    tenant_context::{rls_middleware, RlsState},
};

use crate::{handlers::*, AppState};

/// Returns a `Router<AppState>` — caller must call `.with_state(state)`.
pub fn workforce_router(
    rls_state: RlsState,
    auth_state: AuthState,
    config: Config,
) -> Router<AppState> {
    Router::new()
        // Legal entities
        .route("/legal-entities", post(create_legal_entity))
        .route("/legal-entities", get(list_legal_entities))
        // Locations
        .route("/locations", post(create_location))
        .route("/locations", get(list_locations))
        // Departments
        .route("/departments", post(create_department))
        .route("/departments", get(list_departments))
        // Teams
        .route("/teams", post(create_team))
        .route("/teams", get(list_teams))
        // Positions
        .route("/positions", post(create_position))
        .route("/positions", get(list_positions))
        // Employees
        .route("/employees", post(create_employee))
        .route("/employees", get(list_employees))
        .route("/employees/:employee_ulid", get(get_employee))
        .route(
            "/employees/:employee_ulid/status",
            patch(update_employee_status),
        )
        .route(
            "/employees/:employee_ulid/employment",
            patch(update_employment),
        )
        .route(
            "/employees/:employee_ulid/compensation",
            post(create_compensation),
        )
        .route(
            "/employees/:employee_ulid/compensation",
            get(get_compensation),
        )
        .route("/employees/:employee_ulid/documents", post(create_document))
        .route("/employees/:employee_ulid/documents", get(list_documents))
        .route(
            "/employees/:employee_ulid/documents/:doc_ulid/url",
            get(get_document_url),
        )
        // RLS is innermost (runs after auth set AuthContext in extensions).
        .layer(middleware::from_fn_with_state(rls_state, rls_middleware))
        // Auth is outermost (runs first).
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        // Config extension for audit emission
        .layer(axum::Extension(config))
}
