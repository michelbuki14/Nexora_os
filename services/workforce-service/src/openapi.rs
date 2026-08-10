//! Workforce service OpenAPI schema.
//!
//! Same convention as audit-service: component schemas only, no paths, to
//! avoid cross-module macro expansion issues. The `#[utoipa::path]` attrs on
//! each handler are still emitted and can be collected by a future workspace-
//! level OpenApi merge if needed.

use crate::models::{
    CompensationFrequency, CompensationResponse, CreateCompensationRequest,
    CreateDepartmentRequest, CreateDocumentMetadataRequest, CreateEmployeeRequest,
    CreateLegalEntityRequest, CreateLocationRequest, CreatePositionRequest, CreateTeamRequest,
    DepartmentResponse, DocType, DocumentResponse, DocumentUrlResponse, EmployeeResponse,
    EmployeeStatus, EmploymentType, LegalEntityResponse, LocationResponse, PageQuery,
    PositionResponse, TeamResponse, UpdateEmployeeStatusRequest, UpdateEmploymentRequest,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        // Request types
        CreateLegalEntityRequest,
        CreateLocationRequest,
        CreateDepartmentRequest,
        CreateTeamRequest,
        CreatePositionRequest,
        CreateEmployeeRequest,
        UpdateEmployeeStatusRequest,
        UpdateEmploymentRequest,
        CreateCompensationRequest,
        CreateDocumentMetadataRequest,
        PageQuery,
        // Response types
        LegalEntityResponse,
        LocationResponse,
        DepartmentResponse,
        TeamResponse,
        PositionResponse,
        EmployeeResponse,
        CompensationResponse,
        DocumentResponse,
        DocumentUrlResponse,
        // Enums
        EmployeeStatus,
        EmploymentType,
        CompensationFrequency,
        DocType,
    )),
    tags(
        (name = "workforce", description = "Workforce management — org structure, employees, documents, lifecycle")
    ),
    info(
        title = "AOS Workforce Service API",
        version = "0.1.0",
        description = "Workforce Phase 1: org structure, employees, employment/compensation history, secure documents, tenant isolation.",
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0.html"
        )
    )
)]
#[allow(dead_code)]
pub struct WorkforceApi;
