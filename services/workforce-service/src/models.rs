//! Workforce service data models.
//!
//! Convention matches `audit-service/src/models.rs`:
//!   - `*Row` structs: `FromRow` for SELECT; never serialised directly to callers.
//!   - `*Response` structs: `ToSchema + Serialize`; what callers receive.
//!   - `*Request` structs: `ToSchema + Deserialize`; what callers send.
//!   - Sensitive fields (compensation) are absent from response types unless the
//!     handler has confirmed the caller holds `employee.compensation.read`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeStatus {
    Onboarding,
    Active,
    OnLeave,
    Terminated,
}

impl EmployeeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Onboarding => "onboarding",
            Self::Active => "active",
            Self::OnLeave => "on_leave",
            Self::Terminated => "terminated",
        }
    }
}

impl std::fmt::Display for EmployeeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EmploymentType {
    Permanent,
    Contract,
    Intern,
    Consultant,
}

impl EmploymentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Permanent => "permanent",
            Self::Contract => "contract",
            Self::Intern => "intern",
            Self::Consultant => "consultant",
        }
    }
}

impl std::fmt::Display for EmploymentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompensationFrequency {
    Monthly,
    Annual,
    Hourly,
    Weekly,
}

impl CompensationFrequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Monthly => "monthly",
            Self::Annual => "annual",
            Self::Hourly => "hourly",
            Self::Weekly => "weekly",
        }
    }
}

impl std::fmt::Display for CompensationFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    Contract,
    IdDocument,
    Certificate,
    OfferLetter,
    Payslip,
    Other,
}

impl DocType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::IdDocument => "id_document",
            Self::Certificate => "certificate",
            Self::OfferLetter => "offer_letter",
            Self::Payslip => "payslip",
            Self::Other => "other",
        }
    }
}

impl std::fmt::Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Pagination (mirrors audit-service pattern)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

impl PageQuery {
    pub fn sanitized(&self) -> Self {
        Self {
            page: self.page.max(1),
            page_size: self.page_size.clamp(1, 100),
        }
    }

    pub fn offset(&self) -> i64 {
        (self.page.max(1) - 1) * self.page_size.clamp(1, 100)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

// ---------------------------------------------------------------------------
// Legal Entities
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateLegalEntityRequest {
    pub name: String,
    pub registration_number: Option<String>,
    pub country_ulid: Option<String>,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LegalEntityResponse {
    pub ulid: String,
    pub name: String,
    pub registration_number: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Locations
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateLocationRequest {
    pub legal_entity_ulid: String,
    pub name: String,
    pub code: Option<String>,
    pub address: Option<serde_json::Value>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationResponse {
    pub ulid: String,
    pub name: String,
    pub code: Option<String>,
    pub timezone: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Departments
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub code: Option<String>,
    pub parent_department_ulid: Option<String>,
    pub location_ulid: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DepartmentResponse {
    pub ulid: String,
    pub name: String,
    pub code: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Teams
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTeamRequest {
    pub department_ulid: String,
    pub name: String,
    pub code: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TeamResponse {
    pub ulid: String,
    pub name: String,
    pub code: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Positions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePositionRequest {
    pub title: String,
    pub code: Option<String>,
    pub department_ulid: Option<String>,
    pub job_grade: Option<String>,
    pub description: Option<String>,
    pub responsibilities: Option<String>,
    pub employment_type: Option<EmploymentType>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PositionResponse {
    pub ulid: String,
    pub title: String,
    pub code: Option<String>,
    pub job_grade: Option<String>,
    pub employment_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Employees
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEmployeeRequest {
    pub employee_number: String,
    pub legal_name: String,
    pub preferred_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    /// Raw national ID — hashed on write; never stored in clear.
    #[serde(skip_serializing)]
    pub national_id: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub department_ulid: Option<String>,
    pub position_ulid: Option<String>,
    pub location_ulid: Option<String>,
    pub manager_employee_ulid: Option<String>,
    pub employment_type: Option<EmploymentType>,
}

/// Response for employee reads. Compensation fields are absent; they are
/// returned only by the separate `GET /employees/{id}/compensation` endpoint
/// which gate-checks `employee.compensation.read`.
#[derive(Debug, Serialize, ToSchema)]
pub struct EmployeeResponse {
    pub ulid: String,
    pub employee_number: String,
    pub legal_name: String,
    pub preferred_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<String>,
    /// Last four characters of the national ID (for human disambiguation).
    pub national_id_last4: Option<String>,
    pub status: String,
    pub hire_date: Option<NaiveDate>,
    pub current_department_ulid: Option<String>,
    pub current_position_ulid: Option<String>,
    pub current_location_ulid: Option<String>,
    pub manager_employee_ulid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateEmployeeStatusRequest {
    pub status: EmployeeStatus,
    pub termination_date: Option<NaiveDate>,
    pub termination_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateEmploymentRequest {
    pub position_ulid: Option<String>,
    pub department_ulid: Option<String>,
    pub location_ulid: Option<String>,
    pub employment_type: Option<EmploymentType>,
    pub effective_date: NaiveDate,
    pub change_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Compensation
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCompensationRequest {
    /// Amount in minor currency units (centimes/cents).
    pub gross_amount_minor: i64,
    pub currency_code: String,
    pub frequency: CompensationFrequency,
    pub effective_date: NaiveDate,
    pub change_reason: Option<String>,
}

/// Sensitive — only returned to callers with `employee.compensation.read`.
#[derive(Debug, Serialize, ToSchema)]
pub struct CompensationResponse {
    pub ulid: String,
    pub gross_amount_minor: i64,
    pub currency_code: String,
    pub frequency: String,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDocumentMetadataRequest {
    pub doc_type: DocType,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: Option<i64>,
    pub sha256: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentResponse {
    pub ulid: String,
    pub doc_type: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: Option<i64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Returned by the presigned-URL endpoint. The URL is valid for `ttl_secs`.
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentUrlResponse {
    pub presigned_url: String,
    pub expires_in_secs: u64,
}

// ---------------------------------------------------------------------------
// Database row types (FromRow — never exposed directly)
// ---------------------------------------------------------------------------

/// Minimal employee row used for IDOR checks (does this employee belong to
/// the calling tenant's org, and can this caller see them?)
#[derive(Debug, sqlx::FromRow)]
pub struct EmployeeIdRow {
    pub id: uuid::Uuid,
    pub ulid: String,
    pub tenant_id: uuid::Uuid,
    pub manager_employee_id: Option<uuid::Uuid>,
    pub status: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EmployeeRow {
    pub id: uuid::Uuid,
    pub ulid: String,
    pub employee_number: String,
    pub legal_name: String,
    pub preferred_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<String>,
    pub national_id_last4: Option<String>,
    pub status: String,
    pub hire_date: Option<NaiveDate>,
    pub current_department_id: Option<uuid::Uuid>,
    pub current_position_id: Option<uuid::Uuid>,
    pub current_location_id: Option<uuid::Uuid>,
    pub manager_employee_id: Option<uuid::Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: uuid::Uuid,
    pub ulid: String,
    pub doc_type: String,
    pub object_key: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: Option<i64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CompensationRow {
    pub ulid: String,
    pub gross_amount_minor: i64,
    pub currency_code: String,
    pub frequency: String,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}
