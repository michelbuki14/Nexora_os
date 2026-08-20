//! Nexora OS Payroll Service.
//!
//! DRC (Democratic Republic of Congo) compliant payroll engine:
//! - IPR (Impôt Professionnel sur les Revenus) progressive tax
//! - CNSS (Caisse Nationale de Sécurité Sociale) contributions with ceiling
//! - SMIG (Salaire Minimum Interprofessionnel Garanti) compliance checks
//! - Payroll run lifecycle: draft -> review -> approved -> locked
//! - Payslip generation (append-only `payroll_items` ledger) + PDF rendering
//!
//! Schema lives in the workspace-level `migrations/003_payroll_schema.sql`,
//! applied by the `migrate` service — this crate never runs migrations itself.
//!
//! Port default: 3003 (gateway 3000, audit 3001, workforce 3002).

pub mod audit_emit;
pub mod calc;
pub mod handlers;
pub mod models;
pub mod pdf;
pub mod routes;

use nexora_common::config::{Config, S3Config};
use std::sync::Arc;

/// Shared handler state for the payroll service.
///
/// The database connection is NOT held here: every request gets its
/// RLS-activated transaction through the `DbConn` extension installed by
/// `nexora_common::tenant_context::rls_middleware`.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub s3_cfg: S3Config,
}
