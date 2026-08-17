//! Authenticated request context and RLS middleware.
//!
//! This module provides the [`AuthContext`] extractor that represents the
//! authenticated user's identity and permissions within a tenant, and the
//! [`RlsMiddleware`] that activates PostgreSQL row-level security by setting
//! the transaction GUCs `nexora.current_tenant_id` and `nexora.is_system` on every
//! request.
//!
//! The middleware MUST be applied to all routes that access RLS-protected tables.
//!
//! # How RLS activation works
//!
//! PostgreSQL GUCs are session-scoped by default, which would let one pooled
//! request leak its tenant context into the next. Instead we open an explicit
//! transaction per request and set the GUCs with `set_config(..., is_local =
//! true)`, which scopes them to that transaction. Every query the handler runs
//! on the request's [`DbConn`] therefore executes under the tenant's context,
//! and when the transaction ends the GUCs are discarded — isolation by
//! construction, with no per-request reset step that can be forgotten.

use crate::{config::Config, ulid::Ulid, NexoraError};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error};

/// A request-scoped, RLS-activated database transaction.
///
/// The RLS middleware begins one transaction per request, sets the
/// `nexora.current_tenant_id` / `nexora.is_system` GUCs on it, and stores it here in
/// request extensions so handlers reuse the *same* transaction (and thus the
/// same RLS context). The transaction is committed on a successful response and
/// rolled back on error; either way its transaction-local GUCs are discarded,
/// so a pooled connection can never carry a stale tenant context into another
/// request.
///
/// `Transaction` is not `Clone`, but request extensions require `Clone`, so the
/// transaction lives behind an `Arc<Mutex<Option<…>>>` guard. Exactly one
/// handler drains the transaction for the duration of a request, so the lock is
/// never contended semantically.
///
/// [`DbConn::acquire`] yields the live transaction; once the query is done it must
/// be handed back (it is automatically returned on drop of the inner guard).
#[derive(Clone)]
pub struct DbConn {
    inner: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
}

impl DbConn {
    /// Take the transaction out of the guard for the duration of a query.
    ///
    /// The transaction is automatically returned to the guard (and finalized by
    /// the middleware at request end) when the returned [`DbConnGuard`] is dropped.
    pub async fn acquire(&self) -> Result<DbConnGuard, NexoraError> {
        let conn = self.inner.lock().await.take().ok_or_else(|| {
            NexoraError::Internal("RLS database connection already consumed for this request".into())
        })?;
        Ok(DbConnGuard {
            conn: Some(conn),
            holder: self.inner.clone(),
        })
    }
}

/// RAII guard holding the request's RLS transaction. Dropping it puts the
/// transaction back into the shared [`DbConn`] so the middleware can finalize it
/// (commit/rollback) when the request ends.
pub struct DbConnGuard {
    conn: Option<Transaction<'static, Postgres>>,
    holder: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
}

impl std::ops::Deref for DbConnGuard {
    type Target = Transaction<'static, Postgres>;

    fn deref(&self) -> &Self::Target {
        self.conn
            .as_ref()
            .expect("DbConnGuard connection taken illegally")
    }
}

impl std::ops::DerefMut for DbConnGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn
            .as_mut()
            .expect("DbConnGuard connection taken illegally")
    }
}

impl Drop for DbConnGuard {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            // Best-effort return; if the holder is gone (request already torn
            // down), the transaction simply drops — which rolls it back.
            if let Ok(mut slot) = self.holder.try_lock() {
                slot.get_or_insert(conn);
            }
        }
    }
}

/// Authenticated user context for the current request.
///
/// Populated by `RlsMiddleware` after validating the JWT and extracting claims.
/// Used by downstream handlers for authorization and audit.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// The tenant ULID this request operates within.
    pub tenant_id: Ulid,
    /// The organization ULID (derived from tenant).
    pub org_id: Ulid,
    /// The authenticated user's ULID.
    pub user_id: Ulid,
    /// The user's Keycloak subject ID.
    pub keycloak_sub: String,
    /// Whether this request has system-level (cross-tenant) privileges.
    pub is_system: bool,
    /// Roles granted to the user for this tenant.
    pub roles: Vec<String>,
    /// Permissions granted to the user for this tenant.
    pub permissions: Vec<String>,
}

impl AuthContext {
    /// Check if the context has a specific permission.
    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions.iter().any(|p| p == perm || p == "*")
    }

    /// Check if the context has any of the given permissions.
    pub fn has_any_permission(&self, perms: &[&str]) -> bool {
        perms.iter().any(|p| self.has_permission(p))
    }

    /// Check if the context has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// State required by the RLS middleware.
#[derive(Clone)]
pub struct RlsState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

/// Middleware that activates RLS by setting the PostgreSQL GUCs.
///
/// For each request with a valid auth context:
/// 1. Begins a transaction on a pooled connection
/// 2. Executes `SELECT set_config('nexora.current_tenant_id', …, true)` (transaction-local)
/// 3. Executes `SELECT set_config('nexora.is_system', …, true)` (transaction-local)
/// 4. Stores the transaction in request extensions for handler reuse
/// 5. Runs the handler
/// 6. Commits the transaction on a successful response, rolls back otherwise
///    (transaction-local GUCs are discarded either way)
///
/// If no auth context is available (e.g., health checks), the middleware
/// skips RLS activation and passes through.
pub async fn rls_middleware(
    State(state): State<RlsState>,
    mut request: Request,
    next: Next,
) -> Result<Response, NexoraError> {
    // Extract auth context from request extensions (set by auth middleware)
    let ctx = request.extensions().get::<AuthContext>().cloned();

    let Some(ctx) = ctx else {
        // No auth context — skip RLS (e.g., health checks, public endpoints)
        debug!("no auth context; skipping RLS activation");
        return Ok(next.run(request).await);
    };

    // Begin a request-scoped transaction. The GUCs set below are local to it,
    // so they persist for the handler's queries and are discarded afterwards.
    let mut tx = state.pool.begin().await.map_err(|e| {
        error!(error = %e, "failed to begin RLS transaction");
        NexoraError::ServiceUnavailable("database connection failed".into())
    })?;

    // Set the tenant GUC for RLS using parameterized query
    let tenant_ulid = ctx.tenant_id.to_string();
    if let Err(e) = sqlx::query("SELECT set_config('nexora.current_tenant_id', $1, true)")
        .bind(&tenant_ulid)
        .execute(&mut *tx)
        .await
    {
        error!(error = %e, tenant_id = %tenant_ulid, "failed to set nexora.current_tenant_id");
        return Err(NexoraError::Internal("RLS tenant GUC setup failed".into()));
    }

    // Set the system GUC
    let is_system = ctx.is_system.to_string();
    if let Err(e) = sqlx::query("SELECT set_config('nexora.is_system', $1, true)")
        .bind(&is_system)
        .execute(&mut *tx)
        .await
    {
        error!(error = %e, is_system = ctx.is_system, "failed to set nexora.is_system");
        return Err(NexoraError::Internal("RLS system GUC setup failed".into()));
    }

    debug!(tenant_id = %tenant_ulid, is_system = ctx.is_system, "RLS GUCs activated");

    // Store the transaction in a clonable request extension so handlers can
    // reuse the transaction carrying the tenant GUCs.
    let db_conn = DbConn {
        inner: Arc::new(Mutex::new(Some(tx))),
    };
    request.extensions_mut().insert(db_conn.clone());

    // Run the handler
    let response = next.run(request).await;

    // Finalize the transaction: commit successful responses so writes persist,
    // roll back errors so a failed request leaves no partial state. In both
    // cases the transaction-local GUCs are discarded, preventing context leaks
    // across pooled connections. `take()` can be a no-op if the handler never
    // drained the connection (the transaction is still in the slot).
    let should_commit = response.status().is_success();
    let mut slot = db_conn.inner.lock().await;
    if let Some(tx) = slot.take() {
        let result = if should_commit {
            tx.commit().await
        } else {
            tx.rollback().await
        };
        if let Err(e) = result {
            error!(error = %e, should_commit, "failed to finalize RLS transaction");
        }
    }

    Ok(response)
}

/// Axum extractor for `AuthContext`.
///
/// Usage in handlers:
/// ```ignore
/// async fn my_handler(auth: AuthContext) -> impl IntoResponse { ... }
/// ```
#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthContext {
    type Rejection = NexoraError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or_else(|| NexoraError::Unauthorized("auth context not found".into()))
    }
}

/// Axum extractor for the request's RLS-scoped [`DbConn`].
///
/// Requires that [`rls_middleware`] has already run and stored a `DbConn` in
/// the request extensions. Handlers that need database access should extract
/// `DbConn` (not a raw pool) so queries run under the tenant's RLS context:
///
/// ```ignore
/// async fn list_employees(db: DbConn) -> Result<Json<Vec<Employee>>, NexoraError> {
///     let mut conn = db.acquire().await?;
///     let rows = sqlx::query_as::<_, Employee>("SELECT * FROM employees")
///         .fetch_all(&mut *conn)
///         .await?;
///     Ok(Json(rows))
/// }
/// ```
#[axum::async_trait]
impl<S> FromRequestParts<S> for DbConn {
    type Rejection = NexoraError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<DbConn>().cloned().ok_or_else(|| {
            NexoraError::Internal(
                "RLS database connection not available; is rls_middleware mounted?".into(),
            )
        })
    }
}

/// Builder for adding RLS middleware to a router.
///
/// Axum routers do not expose their state for cloning. Pass the same state to
/// this helper that will later be supplied with `with_state`.
pub fn add_rls_middleware(
    router: axum::Router<RlsState>,
    state: RlsState,
) -> axum::Router<RlsState> {
    router.layer(axum::middleware::from_fn_with_state(state, rls_middleware))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_context_permission_checks() {
        let ctx = AuthContext {
            tenant_id: Ulid::new(),
            org_id: Ulid::new(),
            user_id: Ulid::new(),
            keycloak_sub: "kc-123".into(),
            is_system: false,
            roles: vec!["HR_ADMIN".into()],
            permissions: vec!["employee.read".into(), "employee.write".into()],
        };

        assert!(ctx.has_permission("employee.read"));
        assert!(!ctx.has_permission("payroll.approve"));
        assert!(ctx.has_any_permission(&["employee.read", "payroll.approve"]));
        assert!(!ctx.has_any_permission(&["payroll.approve", "finance.post"]));
        assert!(ctx.has_role("HR_ADMIN"));
        assert!(!ctx.has_role("PAYROLL_ADMIN"));
    }

    #[test]
    fn auth_context_system_bypass() {
        let ctx = AuthContext {
            tenant_id: Ulid::new(),
            org_id: Ulid::new(),
            user_id: Ulid::new(),
            keycloak_sub: "kc-123".into(),
            is_system: true,
            roles: vec!["PLATFORM_ADMIN".into()],
            permissions: vec![],
        };

        // System context should have implicit access (handled by RLS policies)
        assert!(ctx.is_system);
    }
}
