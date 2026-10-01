//! Fintech service request/response models and DB row mapping.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Wallet status values — mirrors the `wallets.status` CHECK constraint.
pub const VALID_WALLET_STATUSES: [&str; 4] = ["active", "frozen", "closed", "pending"];

/// Create a new wallet for the caller's tenant.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateWalletRequest {
    pub currency: String,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "active".to_string()
}

/// Wallet representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct WalletResponse {
    pub wallet_id: String,
    pub owner_id: String,
    pub currency: String,
    pub balance: String,
    pub status: String,
    pub created_at: String,
}

/// A `wallets` row.
#[derive(Debug, sqlx::FromRow)]
pub struct WalletRow {
    pub ulid: String,
    pub owner_ulid: String,
    pub currency: String,
    pub balance: sqlx::types::BigDecimal,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<WalletRow> for WalletResponse {
    fn from(row: WalletRow) -> Self {
        Self {
            wallet_id: row.ulid,
            owner_id: row.owner_ulid,
            currency: row.currency,
            balance: row.balance.to_string(),
            status: row.status,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

/// Create a new transaction (debit/credit against a wallet).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub wallet_id: String,
    /// "credit" increases balance, "debit" decreases it.
    pub direction: String,
    /// Amount as a string to preserve decimal precision (BigDecimal in DB).
    pub amount: String,
    #[serde(default)]
    pub memo: Option<String>,
}

/// Transaction representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub transaction_id: String,
    pub wallet_id: String,
    pub direction: String,
    pub amount: String,
    pub status: String,
    pub memo: Option<String>,
    pub created_at: String,
}

/// A `transactions` row.
#[derive(Debug, sqlx::FromRow)]
pub struct TransactionRow {
    pub ulid: String,
    pub wallet_ulid: String,
    pub direction: String,
    pub amount: sqlx::types::BigDecimal,
    pub status: String,
    pub memo: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<TransactionRow> for TransactionResponse {
    fn from(row: TransactionRow) -> Self {
        Self {
            transaction_id: row.ulid,
            wallet_id: row.wallet_ulid,
            direction: row.direction,
            amount: row.amount.to_string(),
            status: row.status,
            memo: row.memo,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

/// Payment representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentResponse {
    pub payment_id: String,
    pub wallet_id: String,
    pub payee: String,
    pub amount: String,
    pub method: String,
    pub status: String,
    pub created_at: String,
}

/// A `payments` row.
#[derive(Debug, sqlx::FromRow)]
pub struct PaymentRow {
    pub ulid: String,
    pub wallet_ulid: String,
    pub payee: String,
    pub amount: sqlx::types::BigDecimal,
    pub method: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<PaymentRow> for PaymentResponse {
    fn from(row: PaymentRow) -> Self {
        Self {
            payment_id: row.ulid,
            wallet_id: row.wallet_ulid,
            payee: row.payee,
            amount: row.amount.to_string(),
            method: row.method,
            status: row.status,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}
