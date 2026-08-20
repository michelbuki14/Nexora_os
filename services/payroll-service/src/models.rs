//! Payroll domain models.
//!
//! Wire types are ULID-addressed (never raw UUIDs) to match the rest of the
//! platform; UUID primary keys stay internal to the database layer. Monetary
//! amounts use [`nexora_common::money::Money`] so currency travels with the
//! value and JSON never carries a float.

use chrono::{DateTime, NaiveDate, Utc};
use nexora_common::money::Money;
use nexora_common::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ---------------------------------------------------------------------------
// Lifecycle status
// ---------------------------------------------------------------------------

/// Payroll run lifecycle state.
///
/// The DB `CHECK` constraint in `003_payroll_schema.sql` allows
/// `draft | review | approved | locked`, so those strings are the wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PayrollRunStatus {
    Draft,
    Review,
    Approved,
    Locked,
    Cancelled,
}

impl PayrollRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Review => "review",
            Self::Approved => "approved",
            Self::Locked => "locked",
            Self::Cancelled => "cancelled",
        }
    }

    /// Parse a status from its wire-format string (case-insensitive).
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "review" => Ok(Self::Review),
            "approved" => Ok(Self::Approved),
            "locked" => Ok(Self::Locked),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(format!("invalid payroll run status: {other}")),
        }
    }

    /// Whether `self -> next` is a legal lifecycle transition.
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Review)
                | (Self::Review, Self::Approved)
                | (Self::Review, Self::Draft)
                | (Self::Approved, Self::Locked)
                | (Self::Approved, Self::Review)
                | (Self::Draft, Self::Cancelled)
                | (Self::Review, Self::Cancelled)
                | (Self::Approved, Self::Cancelled)
        )
    }
}

impl std::fmt::Display for PayrollRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PayrollRunStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

/// Payroll ledger line type — mirrors the `payroll_items.item_type` CHECK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PayrollItemType {
    Earning,
    Deduction,
    EmployerContribution,
    Tax,
}

impl PayrollItemType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Earning => "earning",
            Self::Deduction => "deduction",
            Self::EmployerContribution => "employer_contribution",
            Self::Tax => "tax",
        }
    }
}

/// Type of payroll component (allowance or deduction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Allowance,
    Deduction,
}

impl ComponentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowance => "allowance",
            Self::Deduction => "deduction",
        }
    }
}

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/// Create a payroll run in `draft`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePayrollRunRequest {
    /// Tax/statutory config version to pin, e.g. `"ipr-2025"`.
    pub config_version: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    /// Optional explicit employee ULID scope. `None` = every active employee.
    #[serde(default)]
    pub employee_ulids: Option<Vec<String>>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Calculate (or recalculate) a draft run.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct CalculateRunRequest {
    /// Days worked assumed for SMIG proration when no attendance data exists.
    #[serde(default)]
    pub default_days_worked: Option<u32>,
    /// Advance the run to `review` after a successful calculation.
    #[serde(default)]
    pub submit_for_review: bool,
}

/// Lifecycle transition body (review / approve / lock / reopen).
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct TransitionRequest {
    #[serde(default)]
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Responses
// ---------------------------------------------------------------------------

/// A payroll run as returned by the API.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PayrollRunResponse {
    pub ulid: String,
    pub config_version: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub status: PayrollRunStatus,
    pub total_gross: Money,
    pub total_net: Money,
    pub total_employer_cost: Money,
    pub employee_count: i32,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One payslip, with the SMIG verdict carried alongside the amounts.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PayslipResponse {
    pub ulid: String,
    pub payroll_run_ulid: String,
    pub employee_ulid: String,
    pub employee_name: String,
    pub employee_number: String,
    pub version: i32,
    pub gross_pay: Money,
    pub taxable_pay: Money,
    pub ipr_deduction: Money,
    pub cnss_employee: Money,
    pub cnss_employer: Money,
    pub net_pay: Money,
    pub employer_cost: Money,
    pub smig_compliant: bool,
    pub created_at: DateTime<Utc>,
}

/// A single ledger line of a payslip (append-only).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PayrollItemResponse {
    pub ulid: String,
    pub item_type: PayrollItemType,
    pub item_code: String,
    pub description: Option<String>,
    pub amount: Money,
    pub is_taxable: bool,
    pub is_cnssable: bool,
}

/// Payslip plus its ledger lines and the calculation audit trail.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PayslipDetailResponse {
    #[serde(flatten)]
    pub payslip: PayslipResponse,
    pub items: Vec<PayrollItemResponse>,
    pub calculation_details: serde_json::Value,
}

/// Calculation summary returned by `POST /runs/:ulid/calculate`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CalculationSummaryResponse {
    pub run: PayrollRunResponse,
    pub payslips_written: usize,
    pub smig_compliant_count: usize,
    pub smig_non_compliant_count: usize,
    /// Employee ULIDs whose net pay fell below the SMIG threshold — these
    /// require HR action before the run may be approved.
    pub smig_violations: Vec<String>,
}

/// The effective statutory configuration for a run.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PayrollConfigResponse {
    pub config_version: String,
    pub country_code: String,
    #[schema(value_type = Vec<Object>)]
    pub ipr_bands: serde_json::Value,
    #[schema(value_type = String, example = "0.0500")]
    pub cnss_employee_rate: Decimal,
    #[schema(value_type = String, example = "0.1050")]
    pub cnss_employer_rate: Decimal,
    pub cnss_ceiling: Money,
    pub smig_daily: Money,
    pub smig_monthly_26: Money,
    pub smig_monthly_30: Money,
    pub is_active: bool,
}

/// Cursor-free page envelope (matches workforce-service list responses).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Pagination query parameters.
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct PageQuery {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub page_size: Option<i64>,
    /// Optional `status` filter for run listings.
    #[serde(default)]
    pub status: Option<String>,
}

impl PageQuery {
    /// Clamp user input into a safe range (page >= 1, 1 <= page_size <= 200).
    pub fn sanitized(&self) -> SanitizedPage {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(50).clamp(1, 200);
        SanitizedPage {
            page,
            page_size,
            status: self.status.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SanitizedPage {
    pub page: i64,
    pub page_size: i64,
    pub status: Option<String>,
}

impl SanitizedPage {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

// ---------------------------------------------------------------------------
// Missing request/response types for handlers
// ---------------------------------------------------------------------------

/// Request body for reviewing a payroll run.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct ReviewPayrollRunRequest {
    #[serde(default)]
    pub notes: Option<String>,
}

/// Request body for approving a payroll run.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct ApprovePayrollRunRequest {
    /// Notes from the approver.
    #[serde(default)]
    pub approver_notes: Option<String>,
}

/// Request body for generating payslips for a payroll run.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct GeneratePayslipsRequest {
    /// Override the default days worked for SMIG calculation.
    #[serde(default)]
    pub default_days_worked: Option<u32>,
}

/// Response returned by `POST /runs/:ulid/generate-payslips`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GeneratePayslipsResponse {
    pub run_ulid: String,
    pub payslips_generated: usize,
    pub smig_compliant_count: usize,
    pub smig_non_compliant_count: usize,
    pub smig_violations: Vec<String>,
}

/// Request body for creating a payroll component (allowance/deduction).
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePayrollComponentRequest {
    pub code: String,
    pub description: Option<String>,
    pub component_type: ComponentType,
    /// Default amount in CDF (minor units).
    #[serde(default)]
    pub default_amount: Option<Decimal>,
    #[serde(default)]
    pub is_taxable: bool,
    #[serde(default)]
    pub is_cnssable: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Request body for updating a payroll component.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdatePayrollComponentRequest {
    pub code: Option<String>,
    pub description: Option<String>,
    pub component_type: Option<ComponentType>,
    #[serde(default)]
    pub default_amount: Option<Decimal>,
    #[serde(default)]
    pub is_taxable: Option<bool>,
    #[serde(default)]
    pub is_cnssable: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Response for a single payroll component.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PayrollComponentResponse {
    pub ulid: String,
    pub code: String,
    pub description: Option<String>,
    pub component_type: ComponentType,
    pub default_amount: Option<Money>,
    pub is_taxable: bool,
    pub is_cnssable: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single IPR tax bracket (progressive income tax).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IprBracket {
    /// Upper bound of taxable income (CDF minor units). `None` = top bracket (no ceiling).
    #[schema(value_type = Option<String>)]
    pub ceiling: Option<Decimal>,
    /// Marginal tax rate applied to income within this bracket (e.g. `0.05` = 5%).
    #[schema(value_type = String, example = "0.05")]
    pub rate: Decimal,
}

/// CNSS (social security) configuration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CnssConfig {
    /// Employee contribution rate (e.g. `0.05` = 5%).
    #[schema(value_type = String, example = "0.05")]
    pub employee_rate: Decimal,
    /// Employer contribution rate (e.g. `0.105` = 10.5%).
    #[schema(value_type = String, example = "0.105")]
    pub employer_rate: Decimal,
    /// Wage ceiling in CDF minor units above which CNSS is not calculated.
    pub ceiling: Decimal,
}

/// SMIG (minimum wage) configuration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SmigConfig {
    /// Daily minimum wage in CDF minor units.
    pub daily: Decimal,
    /// Monthly minimum wage based on 26 working days.
    pub monthly_26: Decimal,
    /// Monthly minimum wage based on 30 calendar days.
    pub monthly_30: Decimal,
}

/// Request body for creating a country tax configuration.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePayrollConfigRequest {
    pub country_code: String,
    pub config_version: String,
    pub ipr_brackets: Vec<IprBracket>,
    pub cnss: CnssConfig,
    pub smig: SmigConfig,
    #[serde(default)]
    pub notes: Option<String>,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allowance => write!(f, "allowance"),
            Self::Deduction => write!(f, "deduction"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_transitions_are_restricted() {
        use PayrollRunStatus::*;
        assert!(Draft.can_transition_to(Review));
        assert!(Review.can_transition_to(Approved));
        assert!(Approved.can_transition_to(Locked));
        // Reopen paths
        assert!(Review.can_transition_to(Draft));
        assert!(Approved.can_transition_to(Review));
        // Illegal jumps
        assert!(!Draft.can_transition_to(Approved));
        assert!(!Draft.can_transition_to(Locked));
        assert!(!Review.can_transition_to(Locked));
        // Locked is terminal
        assert!(!Locked.can_transition_to(Review));
        assert!(!Locked.can_transition_to(Draft));
        assert!(!Locked.can_transition_to(Approved));
    }

    #[test]
    fn status_roundtrips_through_db_strings() {
        for s in [
            PayrollRunStatus::Draft,
            PayrollRunStatus::Review,
            PayrollRunStatus::Approved,
            PayrollRunStatus::Locked,
        ] {
            assert_eq!(s.as_str().parse::<PayrollRunStatus>().unwrap(), s);
        }
        assert!("nonsense".parse::<PayrollRunStatus>().is_err());
    }

    #[test]
    fn page_query_is_clamped() {
        let q = PageQuery {
            page: Some(-5),
            page_size: Some(10_000),
            status: None,
        };
        let s = q.sanitized();
        assert_eq!(s.page, 1);
        assert_eq!(s.page_size, 200);
        assert_eq!(s.offset(), 0);
    }
}
