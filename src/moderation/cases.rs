//! CRUD for `moderation_cases` rows.
//!
//! Used by moderation commands (e.g., `/mute`) to create temp-punishment
//! cases, and by the poller to find and resolve expired ones.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::error::BotError;

/// A row from the `moderation_cases` table. Field names match the
/// column names 1:1.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModerationCase {
    pub id: i64,
    pub guild_id: i64,
    pub target_id: i64,
    pub moderator_id: i64,
    pub action_type: String,
    pub role_id: Option<i64>,
    pub duration_seconds: Option<i64>,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
}

/// Parameters for a new moderation case. Using a builder struct (vs.
/// positional args) makes call sites self-documenting.
pub struct NewCase<'a> {
    pub guild_id: i64,
    pub target_id: i64,
    pub moderator_id: i64,
    pub action_type: &'a str,
    pub role_id: Option<i64>,
    pub duration_seconds: Option<i64>,
    pub reason: Option<&'a str>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Insert a new moderation case. Returns the inserted row's `id`.
pub async fn create_case(pool: &SqlitePool, new: NewCase<'_>) -> Result<i64, BotError> {
    let result = sqlx::query(
        "INSERT INTO moderation_cases \
         (guild_id, target_id, moderator_id, action_type, role_id, duration_seconds, reason, expires_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(new.guild_id)
    .bind(new.target_id)
    .bind(new.moderator_id)
    .bind(new.action_type)
    .bind(new.role_id)
    .bind(new.duration_seconds)
    .bind(new.reason)
    .bind(new.expires_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Find all active cases whose `expires_at` is in the past (or
/// exactly `now`). Used by the poller.
pub async fn find_active_expired(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<Vec<ModerationCase>, BotError> {
    let cases: Vec<ModerationCase> = sqlx::query_as(
        "SELECT id, guild_id, target_id, moderator_id, action_type, role_id, \
                duration_seconds, reason, timestamp, expires_at, status \
         FROM moderation_cases \
         WHERE status = 'active' AND expires_at IS NOT NULL AND expires_at <= ?1",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;

    Ok(cases)
}

/// Mark a case as resolved. Used by the poller after reversing.
pub async fn mark_resolved(pool: &SqlitePool, id: i64) -> Result<(), BotError> {
    sqlx::query("UPDATE moderation_cases SET status = 'resolved' WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(":memory:")
                    .create_if_missing(true)
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
            )
            .await
            .expect("in-memory SQLite should open");
        let sql = include_str!("../../migrations/0001_init.sql");
        sqlx::query(sql)
            .execute(&pool)
            .await
            .expect("schema should apply");
        pool
    }

    #[tokio::test]
    async fn create_then_find_active_expired() {
        let pool = test_pool().await;
        let now = Utc::now();
        let past = now - chrono::Duration::seconds(60);
        let future = now + chrono::Duration::seconds(60);

        // Case 1: expired (active, expires_at in the past)
        let id_expired = create_case(
            &pool,
            NewCase {
                guild_id: 1,
                target_id: 100,
                moderator_id: 200,
                action_type: "MUTE",
                role_id: None,
                duration_seconds: Some(60),
                reason: Some("test"),
                expires_at: Some(past),
            },
        )
        .await
        .unwrap();
        // Case 2: not yet expired (active, expires_at in the future)
        let _id_future = create_case(
            &pool,
            NewCase {
                guild_id: 1,
                target_id: 101,
                moderator_id: 200,
                action_type: "MUTE",
                role_id: None,
                duration_seconds: Some(3600),
                reason: None,
                expires_at: Some(future),
            },
        )
        .await
        .unwrap();
        // Case 3: expired but already resolved
        let id_resolved = create_case(
            &pool,
            NewCase {
                guild_id: 1,
                target_id: 102,
                moderator_id: 200,
                action_type: "MUTE",
                role_id: None,
                duration_seconds: Some(60),
                reason: None,
                expires_at: Some(past),
            },
        )
        .await
        .unwrap();
        mark_resolved(&pool, id_resolved).await.unwrap();

        let expired = find_active_expired(&pool, now).await.unwrap();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].id, id_expired);
        assert_eq!(expired[0].target_id, 100);
    }

    #[tokio::test]
    async fn mark_resolved_idempotent() {
        let pool = test_pool().await;
        let now = Utc::now();
        let id = create_case(
            &pool,
            NewCase {
                guild_id: 1,
                target_id: 100,
                moderator_id: 200,
                action_type: "MUTE",
                role_id: None,
                duration_seconds: Some(60),
                reason: None,
                expires_at: Some(now - chrono::Duration::seconds(60)),
            },
        )
        .await
        .unwrap();
        mark_resolved(&pool, id).await.unwrap();
        // Second call: should not error (UPDATE on already-resolved row is a no-op)
        mark_resolved(&pool, id).await.unwrap();
        let expired = find_active_expired(&pool, now).await.unwrap();
        assert!(expired.is_empty());
    }
}
