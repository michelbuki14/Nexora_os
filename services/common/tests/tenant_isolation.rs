//! Tenant-isolation integration tests against a disposable PostgreSQL.
//!
//! These tests verify the RLS foundation end-to-end:
//! - tenant A cannot read tenant B's rows (`tenants`, `audit_events`, `users`)
//! - cross-organization access is denied
//! - a caller-supplied tenant id in the request does not bypass isolation
//! - system context bypasses RLS *only* when explicitly enabled
//! - transaction-local tenant GUCs never leak across pooled requests
//! - audit hash-chain tampering is detected
//!
//! Each run spins up a throwaway `postgres:16-alpine` container via
//! testcontainers, applies the real migrations (as the superuser), seeds
//! fixtures as superuser (admin setup), and then connects as the non-superuser
//! `nexora_app` role created by migration 007 — the same shape as production.
//! Because `nexora_app` is neither a superuser nor a table owner, every RLS policy
//! actually applies to its queries.
//!
//! ## Windows note
//! These tests are `#[ignore]`d on Windows: testcontainers parses DOCKER_HOST with
//! `url::Url`, which re-serializes `npipe:////./pipe/docker_engine` to
//! `npipe:////pipe/docker_engine` (drops the dot), so bollard attempts a bogus UNC
//! path and container create fails with ERROR_BAD_NETPATH (53). Run on Linux/CI
//! with `cargo test -p nexora-common --test tenant_isolation -- --ignored`.
//! If Docker is unavailable (e.g. a minimal CI runner), the suite FAILS with a
//! panic ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â the integration tests are a correctness gate and MUST run against a
//! real PostgreSQL. Vacuous passes are a false-green security risk.

use axum::{
    body::Body,
    extract::{Path, Request},
    http::StatusCode,
    middleware,
    routing::get,
    Json, Router,
};
use nexora_common::audit::{compute_chain_hash, GENESIS_HASH};
use nexora_common::config::Config;
use nexora_common::tenant_context::{rls_middleware, AuthContext, DbConn, RlsState};
use nexora_common::ulid::{new_ulid, Ulid};
use nexora_common::NexoraError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use testcontainers::{core::WaitFor, runners::AsyncRunner, GenericImage};
use tokio::sync::OnceCell;

/// The compile-time migration set, identical to `nexora-migrate`.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

const SUPERUSER: &str = "nexora";
const SUPERUSER_PASSWORD: &str = "nexora_super_password";
const APP_ROLE: &str = "nexora_app";
const APP_ROLE_PASSWORD: &str = "nexora_app_dev_password";

/// Shared per-run test cluster: one postgres container for the whole suite.
struct Cluster {
    super_pool: PgPool,
    app_pool: PgPool,
    app_url: String,
    org_a_ulid: String,
    org_b_ulid: String,
    tenant_a_ulid: String,
    tenant_b_ulid: String,
    /// Integer PKs retained as seeds-of-record; assertions operate on ULIDs.
    _tenant_a_id: sqlx::types::Uuid,
    _tenant_b_id: sqlx::types::Uuid,
}

static CLUSTER: OnceCell<Option<Arc<Cluster>>> = OnceCell::const_new();

async fn cluster() -> Option<Arc<Cluster>> {
    CLUSTER
        .get_or_init(|| async {
            match Cluster::start().await {
                Ok(c) => Some(Arc::new(c)),
                Err(e) => {
                    panic!("CRITICAL: tenant-isolation integration tests require Docker/Postgres. Failed to start test container: {e:?}");
                }
            }
        })
        .await
        .clone()
}

// ---------------------------------------------------------------------------
// Cluster bootstrap
// ---------------------------------------------------------------------------

impl Cluster {
    async fn start() -> anyhow::Result<Self> {
        let container = GenericImage::new("postgres", "16-alpine")
            .with_env_var("POSTGRES_USER", SUPERUSER)
            .with_env_var("POSTGRES_PASSWORD", SUPERUSER_PASSWORD)
            .with_env_var("POSTGRES_DB", "nexora")
            .with_exposed_port(5432)
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .start()
            .await?;

        let host = container.get_host().await?.to_string();
        let port = container.get_host_port_ipv4(5432).await?;
        let base_url = format!("{host}:{port}/nexora");
        let super_url = format!("postgres://{SUPERUSER}:{SUPERUSER_PASSWORD}@{base_url}");
        let app_url = format!("postgres://{APP_ROLE}:{APP_ROLE_PASSWORD}@{base_url}");

        // The readiness log can be emitted by initdb's temporary server, so
        // poll until the real server accepts connections.
        let super_pool = connect_with_retry(&super_url).await?;

        MIGRATOR.run(&super_pool).await?;

        // Seed fixtures as superuser. This is admin setup, not tenant access ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â
        // it deliberately bypasses RLS to create the worlds the tests then
        // assert cannot be crossed.
        let org_a_ulid = new_ulid();
        let org_b_ulid = new_ulid();
        let tenant_a_ulid = new_ulid();
        let tenant_b_ulid = new_ulid();

        let (org_a_id, tenant_a_id) =
            insert_org_and_tenant(&super_pool, &org_a_ulid, &tenant_a_ulid, "Org A", "org-a")
                .await?;
        let (org_b_id, tenant_b_id) =
            insert_org_and_tenant(&super_pool, &org_b_ulid, &tenant_b_ulid, "Org B", "org-b")
                .await?;

        // A user per tenant so cross-org access can be observed.
        insert_user(&super_pool, &org_a_id, &tenant_a_id, "a@example.com").await?;
        insert_user(&super_pool, &org_b_id, &tenant_b_id, "b@example.com").await?;

        // One audit event per tenant with the shared chain primitive.
        insert_audit_event(
            &super_pool,
            &org_a_id,
            &tenant_a_id,
            "tenant.a.created",
            &compute_chain_hash(&Config::default(), &tenant_a_id, GENESIS_HASH, "{\"actor\":\"system\",\"op\":\"a\"}"),
        )
        .await?;
        insert_audit_event(
            &super_pool,
            &org_b_id,
            &tenant_b_id,
            "tenant.b.created",
            &compute_chain_hash(&Config::default(), &tenant_b_id, GENESIS_HASH, "{\"actor\":\"system\",\"op\":\"b\"}"),
        )
        .await?;

        let app_pool = connect_with_retry(&app_url).await?;

        Ok(Self {
            super_pool,
            app_pool,
            app_url,
            org_a_ulid,
            org_b_ulid,
            tenant_a_ulid,
            tenant_b_ulid,
            _tenant_a_id: tenant_a_id,
            _tenant_b_id: tenant_b_id,
        })
    }
}

async fn connect_with_retry(url: &str) -> anyhow::Result<PgPool> {
    let mut last_err = None;
    for _ in 0..30 {
        match PgPool::connect(url).await {
            Ok(pool) => {
                let _: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&pool).await?;
                return Ok(pool);
            }
            Err(e) => {
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    anyhow::bail!("postgres never became ready: {last_err:?}")
}

async fn insert_org_and_tenant(
    pool: &PgPool,
    org_ulid: &str,
    tenant_ulid: &str,
    name: &str,
    slug: &str,
) -> anyhow::Result<(sqlx::types::Uuid, sqlx::types::Uuid)> {
    let org_id: sqlx::types::Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (ulid, name, slug, tier) VALUES ($1, $2, $3, 'enterprise') RETURNING id",
    )
    .bind(org_ulid)
    .bind(name)
    .bind(slug)
    .fetch_one(pool)
    .await?;

    let tenant_id: sqlx::types::Uuid = sqlx::query_scalar(
        "INSERT INTO tenants (ulid, org_id, name, slug, tier) VALUES ($1, $2, $3, $4, 'enterprise') RETURNING id",
    )
    .bind(tenant_ulid)
    .bind(org_id)
    .bind(name)
    .bind(slug)
    .fetch_one(pool)
    .await?;

    Ok((org_id, tenant_id))
}

async fn insert_user(
    pool: &PgPool,
    org_id: &sqlx::types::Uuid,
    tenant_id: &sqlx::types::Uuid,
    email: &str,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO users (ulid, org_id, tenant_id, email) VALUES ($1, $2, $3, $4)")
        .bind(new_ulid())
        .bind(org_id)
        .bind(tenant_id)
        .bind(email)
        .execute(pool)
        .await?;
    Ok(())
}

async fn insert_audit_event(
    pool: &PgPool,
    org_id: &sqlx::types::Uuid,
    tenant_id: &sqlx::types::Uuid,
    action: &str,
    hash: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO audit_events (ulid, tenant_id, org_id, actor_type, action, result, hash)
         VALUES ($1, $2, $3, 'system', $4, 'success', $5)",
    )
    .bind(new_ulid())
    .bind(tenant_id)
    .bind(org_id)
    .bind(action)
    .bind(hash)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Request plumbing: a minimal router with the real RLS middleware
// ---------------------------------------------------------------------------

/// Inject an auth context and drive a real `rls_middleware` router, exercising
/// the exact transaction + GUC path production uses (minus JWT validation).
async fn run_probe(pool: PgPool, ctx: AuthContext, uri: &str) -> axum::response::Response {
    let rls_state = RlsState {
        pool,
        config: Arc::new(Config::default()),
    };
    let app = Router::new()
        .route("/tenants", get(probe_tenant_ulids))
        .route("/tenant/:ulid", get(probe_tenant_by_ulid))
        .route("/users", get(probe_user_emails))
        .route("/audit", get(probe_audit_actions))
        .layer(middleware::from_fn_with_state(rls_state, rls_middleware));

    let mut req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    req.extensions_mut().insert(ctx);
    tower::ServiceExt::oneshot(app, req).await.unwrap()
}

async fn probe_tenant_ulids(db: DbConn) -> Result<Json<Vec<String>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<(String,)> = sqlx::query_as("SELECT ulid FROM tenants ORDER BY ulid")
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(|(u,)| u).collect()))
}

/// Resolve a tenant by its ULID as supplied in the request path. RLS still
/// applies, so a caller can only ever resolve its own tenant.
async fn probe_tenant_by_ulid(
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<Option<String>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let found: Option<(String,)> = sqlx::query_as("SELECT ulid FROM tenants WHERE ulid = $1")
        .bind(&ulid)
        .fetch_optional(conn.as_mut())
        .await?;
    Ok(Json(found.map(|(u,)| u)))
}

async fn probe_user_emails(db: DbConn) -> Result<Json<Vec<String>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<(String,)> = sqlx::query_as("SELECT email FROM users ORDER BY email")
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(|(e,)| e).collect()))
}

async fn probe_audit_actions(db: DbConn) -> Result<Json<Vec<String>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<(String,)> = sqlx::query_as("SELECT action FROM audit_events ORDER BY action")
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(|(a,)| a).collect()))
}

async fn body_json<T: serde::de::DeserializeOwned>(resp: axum::response::Response) -> T {
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .expect("body read");
    serde_json::from_slice(&bytes).expect("json body")
}

fn auth_ctx(tenant_ulid: &str, org_ulid: &str, is_system: bool) -> AuthContext {
    AuthContext {
        tenant_id: tenant_ulid.parse().expect("valid tenant ulid"),
        org_id: org_ulid.parse().expect("valid org ulid"),
        user_id: Ulid::new(),
        keycloak_sub: "test-subject".into(),
        is_system,
        roles: if is_system {
            vec!["PLATFORM_ADMIN".into()]
        } else {
            vec!["EMPLOYEE".into()]
        },
        permissions: vec![],
    }
}

// ---------------------------------------------------------------------------
// Isolation tests
// ---------------------------------------------------------------------------

#[ignore = "requires testcontainers docker socket; broken on Windows npipe (see module docs)"]
#[tokio::test]
async fn tenant_a_cannot_read_tenant_b_rows() {
    let Some(c) = cluster().await else {
        return;
    };

    // Tenant A sees only its own tenant row.
    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_a_ulid, &c.org_a_ulid, false),
            "/tenants",
        )
        .await,
    )
    .await;
    assert!(
        seen.contains(&c.tenant_a_ulid),
        "tenant A must see itself: {seen:?}"
    );
    assert!(
        !seen.contains(&c.tenant_b_ulid),
        "tenant A must NOT see tenant B: {seen:?}"
    );

    // Tenant B sees only its own.
    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_b_ulid, &c.org_b_ulid, false),
            "/tenants",
        )
        .await,
    )
    .await;
    assert!(seen.contains(&c.tenant_b_ulid));
    assert!(!seen.contains(&c.tenant_a_ulid));
}

#[ignore = "requires testcontainers docker socket; broken on Windows npipe (see module docs)"]
#[tokio::test]
async fn caller_supplied_tenant_id_does_not_escape_isolation() {
    let Some(c) = cluster().await else {
        return;
    };

    // Even when tenant A asks for tenant B's ULID directly in the path, RLS
    // hides it. The context comes from the trusted middleware, not the request.
    let resp = run_probe(
        c.app_pool.clone(),
        auth_ctx(&c.tenant_a_ulid, &c.org_a_ulid, false),
        &format!("/tenant/{}", c.tenant_b_ulid),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resolved: Option<String> = body_json(resp).await;
    assert_eq!(
        resolved, None,
        "tenant A must not resolve tenant B's row from a path-supplied id"
    );

    // And its own id still resolves.
    let resp = run_probe(
        c.app_pool.clone(),
        auth_ctx(&c.tenant_a_ulid, &c.org_a_ulid, false),
        &format!("/tenant/{}", c.tenant_a_ulid),
    )
    .await;
    let resolved: Option<String> = body_json(resp).await;
    assert_eq!(resolved.as_deref(), Some(c.tenant_a_ulid.as_str()));
}

#[ignore = "requires testcontainers docker socket; broken on Windows npipe (see module docs)"]
#[tokio::test]
async fn cross_organization_access_is_denied() {
    let Some(c) = cluster().await else {
        return;
    };

    // Tenant A sees only its own organization's users.
    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_a_ulid, &c.org_a_ulid, false),
            "/users",
        )
        .await,
    )
    .await;
    assert_eq!(
        seen,
        vec!["a@example.com"],
        "tenant A sees only org A users"
    );

    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_b_ulid, &c.org_b_ulid, false),
            "/users",
        )
        .await,
    )
    .await;
    assert_eq!(
        seen,
        vec!["b@example.com"],
        "tenant B sees only org B users"
    );
}

#[ignore = "requires testcontainers docker socket; broken on Windows npipe (see module docs)"]
#[tokio::test]
async fn audit_events_are_isolated_per_tenant() {
    let Some(c) = cluster().await else {
        return;
    };

    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_a_ulid, &c.org_a_ulid, false),
            "/audit",
        )
        .await,
    )
    .await;
    assert_eq!(
        seen,
        vec!["tenant.a.created"],
        "tenant A sees only its own audit events"
    );

    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_b_ulid, &c.org_b_ulid, false),
            "/audit",
        )
        .await,
    )
    .await;
    assert_eq!(seen, vec!["tenant.b.created"]);
}

#[ignore = "requires testcontainers docker socket; broken on Windows npipe (see module docs)"]
#[tokio::test]
async fn system_context_bypasses_rls_only_when_explicitly_enabled() {
    let Some(c) = cluster().await else {
        return;
    };

    // Same tenant identity, but with the system flag ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ sees every tenant.
    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_a_ulid, &c.org_a_ulid, true),
            "/tenants",
        )
        .await,
    )
    .await;
    assert!(
        seen.contains(&c.tenant_a_ulid) && seen.contains(&c.tenant_b_ulid),
        "system context must see all tenants: {seen:?}"
    );

    // Without the flag ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ isolated again.
    let seen: Vec<String> = body_json(
        run_probe(
            c.app_pool.clone(),
            auth_ctx(&c.tenant_a_ulid, &c.org_a_ulid, false),
            "/tenants",
        )
        .await,
    )
    .await;
    assert_eq!(
        seen,
        vec![c.tenant_a_ulid.clone()],
        "without the system flag the same caller is isolated"
    );
}

#[ignore = "requires testcontainers docker socket; broken on Windows npipe (see module docs)"]
#[tokio::test]
async fn tenant_gucs_do_not_leak_across_pooled_requests() {
    let Some(c) = cluster().await else {
        return;
    };

    // A single-connection pool so the second acquire reuses the first
    // connection ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â the only way a session-level GUC leak would be visible.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&c.app_url)
        .await
        .expect("single-connection app pool");

    let mut conn1 = pool.acquire().await.unwrap();
    sqlx::query("BEGIN").execute(&mut *conn1).await.unwrap();
    sqlx::query("SELECT set_config('nexora.current_tenant_id', $1, true)")
        .bind(&c.tenant_a_ulid)
        .execute(&mut *conn1)
        .await
        .unwrap();
    let during: String =
        sqlx::query_scalar("SELECT current_setting('nexora.current_tenant_id', true)")
            .fetch_one(&mut *conn1)
            .await
            .unwrap();
    assert_eq!(
        during, c.tenant_a_ulid,
        "GUC visible inside the transaction"
    );
    sqlx::query("COMMIT").execute(&mut *conn1).await.unwrap();
    drop(conn1);

    // Same physical connection, next request: the GUC must be gone.
    let mut conn2 = pool.acquire().await.unwrap();
    let after: String =
        sqlx::query_scalar("SELECT current_setting('nexora.current_tenant_id', true)")
            .fetch_one(&mut *conn2)
            .await
            .unwrap();
    assert_eq!(
        after, "",
        "transaction-local GUC must not survive commit on the pooled connection"
    );
}

#[ignore = "requires testcontainers docker socket; broken on Windows npipe (see module docs)"]
#[tokio::test]
async fn audit_hash_chain_detects_tampering() {
    let Some(c) = cluster().await else {
        return;
    };

    // Dedicated tenant so this test is independent of the shared fixtures.
    let org_c_ulid = new_ulid();
    let tenant_c_ulid = new_ulid();
    let (org_c_id, tenant_c_id) =
        insert_org_and_tenant(&c.super_pool, &org_c_ulid, &tenant_c_ulid, "Org C", "org-c")
            .await
            .unwrap();

    // Build and persist a two-event chain using the shared primitive.
    let payload1 = format!(r#"{{"op":"first","tenant":"{tenant_c_ulid}"}}"#);
    let h1 = compute_chain_hash(&Config::default(), &tenant_c_id, GENESIS_HASH, &payload1);
    let payload2 = format!(r#"{{"op":"second","tenant":"{tenant_c_ulid}"}}"#);
    let h2 = compute_chain_hash(&Config::default(), &tenant_c_id, &h1, &payload2);

    insert_audit_event(&c.super_pool, &org_c_id, &tenant_c_id, "chain.first", &h1)
        .await
        .unwrap();
    insert_audit_event(&c.super_pool, &org_c_id, &tenant_c_id, "chain.second", &h2)
        .await
        .unwrap();

    assert!(
        chain_verified(&c.super_pool, tenant_c_id, &tenant_c_ulid).await,
        "untampered chain must verify"
    );

    // Tamper with the second event's stored hash (as superuser to reach it).
    sqlx::query(
        "UPDATE audit_events SET hash = $1 WHERE tenant_id = $2 AND action = 'chain.second'",
    )
    .bind("f".repeat(64))
    .bind(tenant_c_id)
    .execute(&c.super_pool)
    .await
    .unwrap();

    assert!(
        !chain_verified(&c.super_pool, tenant_c_id, &tenant_c_ulid).await,
        "tampering must be detected"
    );
}

/// Recompute the hash chain for a tenant from genesis and check every stored
/// hash. Uses the same payload format the test inserted events with.
async fn chain_verified(pool: &PgPool, tenant_id: sqlx::types::Uuid, tenant_ulid: &str) -> bool {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT action, hash FROM audit_events WHERE tenant_id = $1 ORDER BY id ASC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
    .unwrap();

    let mut prev = GENESIS_HASH.to_string();
    for (action, stored) in rows {
        let payload = format!(r#"{{"op":"{action}","tenant":"{tenant_ulid}"}}"#);
        let expected = compute_chain_hash(&Config::default(), &tenant_id, &prev, &payload);
        if expected != stored {
            return false;
        }
        prev = stored;
    }
    true
}
