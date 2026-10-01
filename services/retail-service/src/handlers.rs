//! Retail service handlers — real SQLx operations under the request's RLS context.

use crate::models::{
    CreateOrderRequest, CreateProductRequest, InventoryRow, InventoryResponse, OrderItemRequest,
    OrderRow, OrderResponse, ProductRow, ProductResponse,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use nexora_common::{ulid::new_ulid, AuthContext, DbConn, NexoraError};
use tracing::debug;

const PRODUCT_SELECT: &str = r#"
    SELECT ulid, name, sku, price, currency, is_active, created_at
    FROM products
"#;

const ORDER_SELECT: &str = r#"
    SELECT ulid, customer_ulid, channel, total, status, created_at
    FROM orders
"#;

const INVENTORY_SELECT: &str = r#"
    SELECT product_ulid, sku, available, reserved, warehouse
    FROM inventory
"#;

/// Create a new product.
#[utoipa::path(
    post,
    path = "/api/v1/products",
    tag = "Retail",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created", body = ProductResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_product(
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, NexoraError> {
    validate_product_request(&req)?;
    let mut conn = db.acquire().await?;
    let product_ulid = new_ulid();
    sqlx::query(
        r#"
        INSERT INTO products (ulid, tenant_id, org_id, name, sku, price, currency, is_active)
        VALUES (
            $1,
            (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            $2, $3, $4, $5, true
        )
        "#,
    )
    .bind(&product_ulid)
    .bind(&req.name)
    .bind(&req.sku)
    .bind(&req.price)
    .bind(if req.currency.is_empty() {
        "CDF".to_string()
    } else {
        req.currency.clone()
    })
    .execute(conn.as_mut())
    .await?;

    debug!(product = %product_ulid, user = %auth.user_id, "product created");
    Ok((
        StatusCode::CREATED,
        Json(ProductResponse {
            product_id: product_ulid,
            name: req.name,
            sku: req.sku,
            price: req.price,
            currency: if req.currency.is_empty() {
                "CDF".to_string()
            } else {
                req.currency
            },
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
    ))
}

/// List products visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/products",
    tag = "Retail",
    responses(
        (status = 200, description = "Products visible to caller", body = [ProductResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_products(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<ProductResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<ProductRow> = sqlx::query_as(&format!("{PRODUCT_SELECT}\nORDER BY created_at DESC"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(ProductResponse::from).collect()))
}

/// Get a single product by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    tag = "Retail",
    params(("id" = String, Path, description = "Product ULID")),
    responses(
        (status = 200, description = "Product", body = ProductResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
        (status = 404, description = "Not found", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_product(
    _auth: AuthContext,
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<ProductResponse>, NexoraError> {
    let mut conn = db.acquire().await?;
    let row: Option<ProductRow> = sqlx::query_as(&format!("{PRODUCT_SELECT}\nWHERE ulid = $1"))
        .bind(&ulid)
        .fetch_optional(conn.as_mut())
        .await?;
    let row = row.ok_or_else(|| NexoraError::NotFound(format!("product {ulid} not found")))?;
    Ok(Json(row.into()))
}

/// Create a new order.
#[utoipa::path(
    post,
    path = "/api/v1/orders",
    tag = "Retail",
    request_body = CreateOrderRequest,
    responses(
        (status = 201, description = "Order created", body = OrderResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_order(
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateOrderRequest>,
) -> Result<impl IntoResponse, NexoraError> {
    validate_order_request(&req)?;
    let mut conn = db.acquire().await?;
    let order_ulid = new_ulid();

    // Resolve prices from the catalog and compute the total atomically.
    let mut total = sqlx::types::BigDecimal::from(0);
    for item in &req.items {
        let price: Option<sqlx::types::BigDecimal> =
            sqlx::query_scalar("SELECT price FROM products WHERE ulid = $1")
                .bind(&item.product_id)
                .fetch_optional(conn.as_mut())
                .await?;
        let price = price.ok_or_else(|| {
            NexoraError::NotFound(format!("product {} not found", item.product_id))
        })?;
        let qty = sqlx::types::BigDecimal::from(item.quantity);
        total += price * qty;
    }

    sqlx::query(
        r#"
        INSERT INTO orders (ulid, tenant_id, org_id, customer_ulid, channel, total, status)
        VALUES (
            $1,
            (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            $2, $3, $4, 'pending'
        )
        "#,
    )
    .bind(&order_ulid)
    .bind(auth.user_id.to_string())
    .bind(&req.channel)
    .bind(total.to_string())
    .execute(conn.as_mut())
    .await?;

    debug!(order = %order_ulid, user = %auth.user_id, "order created");
    Ok((
        StatusCode::CREATED,
        Json(OrderResponse {
            order_id: order_ulid,
            customer_id: auth.user_id.to_string(),
            channel: req.channel,
            total: total.to_string(),
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
    ))
}

/// List orders visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/orders",
    tag = "Retail",
    responses(
        (status = 200, description = "Orders visible to caller", body = [OrderResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_orders(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<OrderResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<OrderRow> = sqlx::query_as(&format!("{ORDER_SELECT}\nORDER BY created_at DESC LIMIT 100"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(OrderResponse::from).collect()))
}

/// Get a single order by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/orders/{id}",
    tag = "Retail",
    params(("id" = String, Path, description = "Order ULID")),
    responses(
        (status = 200, description = "Order", body = OrderResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
        (status = 404, description = "Not found", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_order(
    _auth: AuthContext,
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<OrderResponse>, NexoraError> {
    let mut conn = db.acquire().await?;
    let row: Option<OrderRow> = sqlx::query_as(&format!("{ORDER_SELECT}\nWHERE ulid = $1"))
        .bind(&ulid)
        .fetch_optional(conn.as_mut())
        .await?;
    let row = row.ok_or_else(|| NexoraError::NotFound(format!("order {ulid} not found")))?;
    Ok(Json(row.into()))
}

/// List inventory visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/inventory",
    tag = "Retail",
    responses(
        (status = 200, description = "Inventory visible to caller", body = [InventoryResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_inventory(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<InventoryResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<InventoryRow> = sqlx::query_as(&format!("{INVENTORY_SELECT}\nORDER BY sku"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(InventoryResponse::from).collect()))
}

fn validate_product_request(req: &CreateProductRequest) -> Result<(), NexoraError> {
    if req.name.trim().is_empty() {
        return Err(NexoraError::Validation("name is required".into()));
    }
    if req.sku.trim().is_empty() {
        return Err(NexoraError::Validation("sku is required".into()));
    }
    if req.price.parse::<sqlx::types::BigDecimal>().is_err() {
        return Err(NexoraError::Validation("price is not a valid decimal".into()));
    }
    Ok(())
}

fn validate_order_request(req: &CreateOrderRequest) -> Result<(), NexoraError> {
    if req.items.is_empty() {
        return Err(NexoraError::Validation("order must contain at least one item".into()));
    }
    for OrderItemRequest { product_id, quantity } in &req.items {
        if product_id.trim().is_empty() {
            return Err(NexoraError::Validation("item product_id is required".into()));
        }
        if *quantity <= 0 {
            return Err(NexoraError::Validation("item quantity must be positive".into()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_validation() {
        assert!(validate_product_request(&CreateProductRequest {
            name: "Widget".into(),
            sku: "W-1".into(),
            price: "9.99".into(),
            currency: "CDF".into(),
        })
        .is_ok());
        assert!(validate_product_request(&CreateProductRequest {
            name: "Widget".into(),
            sku: "W-1".into(),
            price: "free".into(),
            currency: "CDF".into(),
        })
        .is_err());
    }

    #[test]
    fn order_validation() {
        assert!(validate_order_request(&CreateOrderRequest {
            items: vec![OrderItemRequest {
                product_id: "01J".into(),
                quantity: 2,
            }],
            channel: "web".into(),
        })
        .is_ok());
        assert!(validate_order_request(&CreateOrderRequest {
            items: vec![],
            channel: "web".into(),
        })
        .is_err());
    }
}
