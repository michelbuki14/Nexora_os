//! Audit service OpenAPI (Swagger) schema.
//!
//! Generated from the request/response schemas in [`crate::models`].
//! The `paths()` list is intentionally omitted to avoid macro expansion
//! issues with cross-module `#[utoipa::path]` items. Schemas are still
//! available for gateway contract generation.

use crate::models::{
    AuditEventListResponse, AuditEventResponse, AuditListQuery, CreateAuditRequest,
    HashChainVerification,
};
use utoipa::OpenApi;

/// OpenAPI document containing only component schemas (no paths).
#[derive(OpenApi)]
#[openapi(
    components(schemas(
        CreateAuditRequest,
        AuditEventResponse,
        AuditEventListResponse,
        AuditListQuery,
        HashChainVerification
    )),
    tags(
        (name = "Audit", description = "Append-only audit event management")
    ),
    info(
        title = "AOS Audit Service API",
        version = "0.1.0",
        description = "Append-only audit log for tenant actions. All endpoints require an authenticated JWT with tenant context.",
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0.html"
        )
    )
)]
#[allow(dead_code)]
pub struct AuditApi;
