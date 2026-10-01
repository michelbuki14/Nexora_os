//! Retail service request/response models and DB row mapping.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Create a new product.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateProductRequest {
    pub name: String,
    pub sku: String,
    /// Price as a string to preserve decimal precision.
    pub price: String,
    #[serde(default)]
    pub currency: String,
}

/// Product representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductResponse {
    pub product_id: String,
    pub name: String,
    pub sku: String,
    pub price: String,
    pub currency: String,
    pub is_active: bool,
    pub created_at: String,
}

/// A `products` row.
#[derive(Debug, sqlx::FromRow)]
pub struct ProductRow {
    pub ulid: String,
    pub name: String,
    pub sku: String,
    pub price: sqlx::types::BigDecimal,
    pub currency: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ProductRow> for ProductResponse {
    fn from(row: ProductRow) -> Self {
        Self {
            product_id: row.ulid,
            name: row.name,
            sku: row.sku,
            price: row.price.to_string(),
            currency: row.currency,
            is_active: row.is_active,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

/// Create a new order (line items reference product SKUs).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrderRequest {
    pub items: Vec<OrderItemRequest>,
    #[serde(default)]
    pub channel: String,
}

/// A single order line item.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderItemRequest {
    pub product_id: String,
    pub quantity: i32,
}

/// Order representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub order_id: String,
    pub customer_id: String,
    pub channel: String,
    pub total: String,
    pub status: String,
    pub created_at: String,
}

/// An `orders` row.
#[derive(Debug, sqlx::FromRow)]
pub struct OrderRow {
    pub ulid: String,
    pub customer_ulid: String,
    pub channel: String,
    pub total: sqlx::types::BigDecimal,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<OrderRow> for OrderResponse {
    fn from(row: OrderRow) -> Self {
        Self {
            order_id: row.ulid,
            customer_id: row.customer_ulid,
            channel: row.channel,
            total: row.total.to_string(),
            status: row.status,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

/// Inventory representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct InventoryResponse {
    pub product_id: String,
    pub sku: String,
    pub available: i64,
    pub reserved: i64,
    pub warehouse: String,
}

/// An `inventory` row.
#[derive(Debug, sqlx::FromRow)]
pub struct InventoryRow {
    pub product_ulid: String,
    pub sku: String,
    pub available: i64,
    pub reserved: i64,
    pub warehouse: String,
}

impl From<InventoryRow> for InventoryResponse {
    fn from(row: InventoryRow) -> Self {
        Self {
            product_id: row.product_ulid,
            sku: row.sku,
            available: row.available,
            reserved: row.reserved,
            warehouse: row.warehouse,
        }
    }
}
