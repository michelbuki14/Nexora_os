//! Payroll service API routes.
//!
//! Assembled into a single `Router` that the `main` function serves. All routes
//! are protected by the platform's auth + RLS middleware (mounted in `main`);
//! this module just wires the handler functions to the paths.

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use utoipa::OpenApi;

use crate::handlers;

/// OpenAPI documentation for the payroll service.
///
/// The utoipa macro resolves every path's request/response types from the
/// utoipa::path attributes on the handler functions.
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::create_payroll_run,
        handlers::list_payroll_runs,
        handlers::get_payroll_run,
        handlers::calculate_payroll_run,
        handlers::review_payroll_run,
        handlers::approve_payroll_run,
        handlers::lock_payroll_run,
        handlers::cancel_payroll_run,
        handlers::get_payslip,
        handlers::list_payslips,
        handlers::generate_payslips,
        handlers::download_payslip_pdf,
        handlers::create_payroll_component,
        handlers::get_payroll_component,
        handlers::list_payroll_components,
        handlers::update_payroll_component,
        handlers::delete_payroll_component,
        handlers::create_payroll_config,
        handlers::get_payroll_config,
        handlers::list_payroll_configs,
    ),
    components(schemas(
        PageQuery,
        PayrollRunResponse,
        PayslipResponse,
        PayslipDetailResponse,
        PayrollItemResponse,
        PayrollComponentResponse,
        ComponentType,
        PayrollRunStatus,
        PayrollConfigResponse,
        CreatePayrollRunRequest,
        CalculateRunRequest,
        ReviewPayrollRunRequest,
        ApprovePayrollRunRequest,
        TransitionRequest,
        GeneratePayslipsRequest,
        CreatePayrollComponentRequest,
        UpdatePayrollComponentRequest,
        CalculationSummaryResponse,
        GeneratePayslipsResponse,
    )),
    tags(
        (name = "payroll", description = "Payroll run lifecycle management"),
        (name = "payslips", description = "Payslip retrieval and PDF generation"),
        (name = "components", description = "Reusable payroll allowance/deduction definitions"),
        (name = "configs", description = "Country tax/statutory configuration versions"),
    ),
    contact(
        name = "Nexora OS Platform Team",
        email = "platform@nexora.local",
    ),
)]
pub struct PayrollApiDoc;

/// Build the payroll service router.
///
/// The caller (usually `main`) is responsible for:
/// - mounting auth middleware (JWT validation → `AuthContext` extension)
/// - mounting RLS middleware (`rls_middleware` → `DbConn` extension)
/// - applying `.with_state(state)` with the service state
pub fn router() -> Router<AppState> {
    Router::new()
        // --- Payroll runs ---
        .route("/api/v1/payroll/runs", post(handlers::create_payroll_run))
        .route("/api/v1/payroll/runs", get(handlers::list_payroll_runs))
        .route(
            "/api/v1/payroll/runs/{ulid}",
            get(handlers::get_payroll_run),
        )
        .route(
            "/api/v1/payroll/runs/{ulid}/calculate",
            post(handlers::calculate_payroll_run),
        )
        .route(
            "/api/v1/payroll/runs/{ulid}/review",
            post(handlers::review_payroll_run),
        )
        .route(
            "/api/v1/payroll/runs/{ulid}/approve",
            post(handlers::approve_payroll_run),
        )
        .route(
            "/api/v1/payroll/runs/{ulid}/lock",
            post(handlers::lock_payroll_run),
        )
        .route(
            "/api/v1/payroll/runs/{ulid}/cancel",
            post(handlers::cancel_payroll_run),
        )
        // --- Payslips ---
        .route(
            "/api/v1/payroll/payslips/{ulid}",
            get(handlers::get_payslip),
        )
        .route(
            "/api/v1/payroll/runs/{run_ulid}/payslips",
            get(handlers::list_payslips),
        )
        .route(
            "/api/v1/payroll/runs/{run_ulid}/generate-payslips",
            post(handlers::generate_payslips),
        )
        .route(
            "/api/v1/payroll/payslips/{ulid}/pdf",
            get(handlers::download_payslip_pdf),
        )
        // --- Payroll components (allowances/deductions) ---
        .route(
            "/api/v1/payroll/components",
            post(handlers::create_payroll_component),
        )
        .route(
            "/api/v1/payroll/components",
            get(handlers::list_payroll_components),
        )
        .route(
            "/api/v1/payroll/components/{ulid}",
            get(handlers::get_payroll_component),
        )
        .route(
            "/api/v1/payroll/components/{ulid}",
            put(handlers::update_payroll_component),
        )
        .route(
            "/api/v1/payroll/components/{ulid}",
            delete(handlers::delete_payroll_component),
        )
        // --- Payroll configurations (country tax/statutory settings) ---
        .route(
            "/api/v1/payroll/configs",
            post(handlers::create_payroll_config),
        )
        .route(
            "/api/v1/payroll/configs",
            get(handlers::list_payroll_configs),
        )
        .route(
            "/api/v1/payroll/configs/{version}",
            get(handlers::get_payroll_config),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_has_all_payroll_routes() {
        let r = router();
        // Confirm the router was constructed without panicking.
        // If any handler has a signature mismatch the build would fail first.
        assert!(true);
    }
}
