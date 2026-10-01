//! Fintech service handlers — real SQLx operations under the request's RLS context.
//!
//! All database access runs through the request-scoped [`DbConn`] extractor so
//! queries execute inside the tenant's RLS transaction set up by `rls_middleware`.

use crate::models::{
    CreateTransactionRequest, CreateWalletRequest, PaymentRow, PaymentResponse, TransactionRow,
    TransactionResponse, VALID_WALLET_STATUSES, WalletRow, WalletResponse,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use nexora_common::{ulid::new_ulid, AuthContext, DbConn, NexoraError};
use tracing::debug;

const WALLET_SELECT: &str = r#"
    SELECT ulid, owner_ulid, currency, balance, status, created_at
    FROM wallets
"#;

const TRANSACTION_SELECT: &str = r#"
    SELECT ulid, wallet_ulid, direction, amount, status, memo, created_at
    FROM transactions
"#;

const PAYMENT_SELECT: &str = r#"
    SELECT ulid, wallet_ulid, payee, amount, method, status, created_at
    FROM payments
"#;

/// Create a new wallet owned by the caller's tenant.
#[utoipa::path(
    post,
    path = "/api/v1/wallets",
    tag = "Fintech",
    request_body = CreateWalletRequest,
    responses(
        (status = 201, description = "Wallet created", body = WalletResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_wallet(
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateWalletRequest>,
) -> Result<impl IntoResponse, NexoraError> {
    validate_wallet_request(&req)?;
    let mut conn = db.acquire().await?;
    let wallet_ulid = new_ulid();
    sqlx::query(
        r#"
        INSERT INTO wallets (ulid, tenant_id, org_id, owner_ulid, currency, balance, status)
        VALUES (
            $1,
            (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            $2, $3, 0, $4
        )
        "#,
    )
    .bind(&wallet_ulid)
    .bind(auth.user_id.to_string())
    .bind(&req.currency)
    .bind(&req.status)
    .execute(conn.as_mut())
    .await?;

    debug!(wallet = %wallet_ulid, "wallet created");
    Ok((
        StatusCode::CREATED,
        Json(WalletResponse {
            wallet_id: wallet_ulid,
            owner_id: auth.user_id.to_string(),
            currency: req.currency,
            balance: "0".to_string(),
            status: req.status,
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
    ))
}

/// List wallets visible to the caller (RLS-scoped to the tenant).
#[utoipa::path(
    get,
    path = "/api/v1/wallets",
    tag = "Fintech",
    responses(
        (status = 200, description = "Wallets visible to caller", body = [WalletResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_wallets(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<WalletResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<WalletRow> = sqlx::query_as(&format!("{WALLET_SELECT}\nORDER BY created_at DESC"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(WalletResponse::from).collect()))
}

/// Get a single wallet by ULID (RLS-scoped; 404 if not visible).
#[utoipa::path(
    get,
    path = "/api/v1/wallets/{id}",
    tag = "Fintech",
    params(("id" = String, Path, description = "Wallet ULID")),
    responses(
        (status = 200, description = "Wallet", body = WalletResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
        (status = 404, description = "Not found", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_wallet(
    _auth: AuthContext,
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<WalletResponse>, NexoraError> {
    let mut conn = db.acquire().await?;
    let row: Option<WalletRow> = sqlx::query_as(&format!("{WALLET_SELECT}\nWHERE ulid = $1"))
        .bind(&ulid)
        .fetch_optional(conn.as_mut())
        .await?;
    let row = row.ok_or_else(|| NexoraError::NotFound(format!("wallet {ulid} not found")))?;
    Ok(Json(row.into()))
}

/// Create a new transaction against a wallet.
#[utoipa::path(
    post,
    path = "/api/v1/transactions",
    tag = "Fintech",
    request_body = CreateTransactionRequest,
    responses(
        (status = 201, description = "Transaction created", body = TransactionResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_transaction(
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<impl IntoResponse, NexoraError> {
    validate_transaction_request(&req)?;
    let mut conn = db.acquire().await?;
    let tx_ulid = new_ulid();

    // A debit must be covered by the wallet balance (optimistic guard; the DB
    // CHECK/constraint is the source of truth, but this gives a clean 400).
    if req.direction == "debit" {
        let balance: Option<sqlx::types::BigDecimal> = sqlx::query_scalar(
            "SELECT balance FROM wallets WHERE ulid = $1",
        )
        .bind(&req.wallet_id)
        .fetch_optional(conn.as_mut())
        .await?;
        let bal = balance.unwrap_or_default();
        let amt: sqlx::types::BigDecimal = req
            .amount
            .parse()
            .map_err(|_| NexoraError::Validation("amount is not a valid decimal".into()))?;
        if bal < amt {
            return Err(NexoraError::Validation("insufficient wallet balance".into()));
        }
    }

    sqlx::query(
        r#"
        INSERT INTO transactions (ulid, tenant_id, org_id, wallet_ulid, direction, amount, status, memo)
        VALUES (
            $1,
            (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            $2, $3, $4, 'settled', $5
        )
        "#,
    )
    .bind(&tx_ulid)
    .bind(&req.wallet_id)
    .bind(&req.direction)
    .bind(&req.amount)
    .bind(req.memo.clone())
    .execute(conn.as_mut())
    .await?;

    debug!(tx = %tx_ulid, user = %auth.user_id, "transaction created");
    Ok((
    StatusCode::CREATED,
    Json(TransactionResponse {
        transaction_id: tx_ulid,
        wallet_id: req.wallet_id,
        direction: req.direction,
        amount: req.amount,
        status: "settled".to_string(),
        memo: req.memo,
        created_at: chrono::Utc::now().to_rfc3339(),
    }),
    ))
}

/// List transactions visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/transactions",
    tag = "Fintech",
    responses(
        (status = 200, description = "Transactions visible to caller", body = [TransactionResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_transactions(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<TransactionResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<TransactionRow> =
        sqlx::query_as(&format!("{TRANSACTION_SELECT}\nORDER BY created_at DESC LIMIT 100"))
            .fetch_all(conn.as_mut())
            .await?;
    Ok(Json(rows.into_iter().map(TransactionResponse::from).collect()))
}

/// Get a single transaction by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/transactions/{id}",
    tag = "Fintech",
    params(("id" = String, Path, description = "Transaction ULID")),
    responses(
        (status = 200, description = "Transaction", body = TransactionResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
        (status = 404, description = "Not found", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_transaction(
    _auth: AuthContext,
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<TransactionResponse>, NexoraError> {
    let mut conn = db.acquire().await?;
    let row: Option<TransactionRow> =
        sqlx::query_as(&format!("{TRANSACTION_SELECT}\nWHERE ulid = $1"))
            .bind(&ulid)
            .fetch_optional(conn.as_mut())
            .await?;
    let row = row.ok_or_else(|| NexoraError::NotFound(format!("transaction {ulid} not found")))?;
    Ok(Json(row.into()))
}

/// List payments visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/payments",
    tag = "Fintech",
    responses(
        (status = 200, description = "Payments visible to caller", body = [PaymentResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_payments(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<PaymentResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<PaymentRow> = sqlx::query_as(&format!("{PAYMENT_SELECT}\nORDER BY created_at DESC LIMIT 100"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(PaymentResponse::from).collect()))
}

fn validate_wallet_request(req: &CreateWalletRequest) -> Result<(), NexoraError> {
    if req.currency.trim().is_empty() {
        return Err(NexoraError::Validation("currency is required".into()));
    }
    if !VALID_WALLET_STATUSES.contains(&req.status.as_str()) {
        return Err(NexoraError::Validation(format!(
            "invalid status {:?}: expected one of {VALID_WALLET_STATUSES:?}",
            req.status
        )));
    }
    Ok(())
}

fn validate_transaction_request(req: &CreateTransactionRequest) -> Result<(), NexoraError> {
    if req.wallet_id.trim().is_empty() {
        return Err(NexoraError::Validation("wallet_id is required".into()));
    }
    if !matches!(req.direction.as_str(), "credit" | "debit") {
        return Err(NexoraError::Validation(
            "direction must be 'credit' or 'debit'".into(),
        ));
    }
    if req.amount.parse::<sqlx::types::BigDecimal>().is_err() {
        return Err(NexoraError::Validation("amount is not a valid decimal".into()));
    }
    if req.amount.parse::<f64>().map(|v| v <= 0.0).unwrap_or(true) {
        return Err(NexoraError::Validation("amount must be positive".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_status_validation() {
        assert!(validate_wallet_request(&CreateWalletRequest {
            currency: "CDF".into(),
            status: "active".into(),
        })
        .is_ok());
        assert!(validate_wallet_request(&CreateWalletRequest {
            currency: "CDF".into(),
            status: "bogus".into(),
        })
        .is_err());
    }

    #[test]
    fn transaction_direction_validation() {
        assert!(validate_transaction_request(&CreateTransactionRequest {
            wallet_id: "01J".into(),
            direction: "credit".into(),
            amount: "10.50".into(),
            memo: None,
        })
        .is_ok());
        assert!(validate_transaction_request(&CreateTransactionRequest {
            wallet_id: "01J".into(),
            direction: "sideways".into(),
            amount: "10.50".into(),
            memo: None,
        })
        .is_err());
    }
}
