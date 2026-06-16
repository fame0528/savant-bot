//! Database connection pool and migration runner.
//!
//! Per FID-2026-0616-004, we use SQLite with WAL journaling for
//! crash-resilient temp-punishment persistence. The pool is initialized
//! once at startup in `Data::new` and shared across the bot via `Data`.

use std::str::FromStr;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::BotError;

/// Connect to the SQLite database at `database_url` and return a pool.
///
/// Uses WAL journaling for better concurrent-read performance and
/// crash recovery. `create_if_missing` lets a fresh deployment start
/// without manual setup. `busy_timeout` prevents "database is locked"
/// errors under concurrent access.
pub async fn connect(database_url: &str) -> Result<SqlitePool, BotError> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

/// Run all SQL migrations from the `migrations/` directory.
///
/// For v1 we use a simple single-file approach: the `0001_init.sql` file
/// is embedded at compile time via `include_str!` and executed on
/// startup. Future v2: use `sqlx::migrate!` for versioned migrations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), BotError> {
    let sql = include_str!("../../migrations/0001_init.sql");
    sqlx::query(sql).execute(pool).await?;
    Ok(())
}
