//! Nexora OS database migration runner.
//!
//! Owns the SQLx migrations directory and applies pending migrations in order
//! within a versioned schema table (`_sqlx_migrations`). Invoked explicitly
//! during deployment and local development — never implicitly from service
//! startup, so migrations are applied by exactly one process.
//!
//! Usage:
//!   nexora-migrate run        # apply pending migrations
//!   nexora-migrate info       # print applied/pending status
//!   nexora-migrate verify     # exit non-zero if pending migrations exist

use nexora_common::{config::Config, db, logging::init_logging};
use sqlx::migrate::{Migrate, Migrator};
use sqlx::PgPool;
use std::env;
use tracing::{error, info, warn};

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("config load failed, using env DATABASE_URL: {e}");
        Config::load().unwrap()
    });
    init_logging(&config.tracing)?;

    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("run");

    // Priority: explicit DATABASE_URL env > config.database.url
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| config.database_url());

    info!(
        command,
        db_host = extract_host(&database_url),
        "migration runner starting"
    );

    let pool = db::connect_url(
        &database_url,
        config.database.max_connections,
        &config.database,
    )
    .await
    .map_err(|e| {
        error!(error = %e, "failed to connect to database");
        anyhow::anyhow!(e)
    })?;

    match command {
        "run" => run(&pool).await,
        "info" => info_cmd(&pool).await,
        "verify" => verify(&pool).await,
        other => {
            eprintln!("unknown command: {other}\nusage: aos-migrate [run|info|verify]");
            std::process::exit(2);
        }
    }
}

async fn run(pool: &PgPool) -> anyhow::Result<()> {
    info!("applying pending migrations");
    MIGRATOR.run(pool).await?;
    info!("all migrations applied successfully");
    Ok(())
}

async fn info_cmd(pool: &PgPool) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    let applied = conn.list_applied_migrations().await?;
    let dirty = conn.dirty_version().await?;

    println!("Migrations ({} known):", MIGRATOR.iter().count());
    for m in MIGRATOR.iter() {
        let applied_m = applied.iter().find(|a| a.version == m.version);
        let status = match applied_m {
            Some(a) if a.checksum != m.checksum => "DIRTY (checksum mismatch)",
            Some(_) => "applied",
            None => "pending",
        };
        println!("  v{:04}  {}  [{}]", m.version, m.description, status);
    }

    if let Some(v) = dirty {
        warn!(
            version = v,
            "database is dirty — a previous migration failed mid-way"
        );
    }
    Ok(())
}

async fn verify(pool: &PgPool) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;
    let applied = conn.list_applied_migrations().await?;

    let pending: Vec<_> = MIGRATOR
        .iter()
        .filter(|m| !applied.iter().any(|a| a.version == m.version))
        .collect();

    if pending.is_empty() {
        info!("database is up to date (no pending migrations)");
        Ok(())
    } else {
        error!(
            count = pending.len(),
            "database has pending migrations — run `aos-migrate run`"
        );
        for m in &pending {
            error!(version = %m.version, description = %m.description, "pending");
        }
        std::process::exit(1);
    }
}

/// Best-effort host extraction for logging (never panics).
fn extract_host(url: &str) -> String {
    // postgres://[user[:pass]@]host[:port]/db
    url.split('@')
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("?")
        .to_string()
}
