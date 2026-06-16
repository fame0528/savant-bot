//! Temp-punishment poller.
//!
//! Per FID-2026-0616-004 and doc §2.1.2, a background task polls the
//! `moderation_cases` table every 60 seconds for active cases whose
//! `expires_at` is in the past, and reverses them. v1 scope: the poller
//! marks expired cases as `resolved` and logs a structured event. The
//! full Discord API reversal (remove role, lift timeout, unban) is a
//! v2 follow-up — the `role_id` column is already in the schema.
//!
//! **Why polling, not `tokio::time::sleep`?** Because volatile timers
//! lose state on restart. With a 60-second poll, restart-recovery is
//! "max 60 seconds of lag" instead of "punishment never reversed".

use std::time::Duration;

use sqlx::SqlitePool;
use tokio::time::interval;

use crate::error::BotError;

/// How often the poller scans for expired cases. 60 seconds is a
/// balance between latency (how long after expiry a punishment is
/// actually reversed) and DB load (one cheap query per minute).
pub const POLL_INTERVAL: Duration = Duration::from_secs(60);

/// Start the polling background task. Returns the `JoinHandle` so the
/// caller can cancel the task (e.g., on shutdown) if needed.
pub fn start_poller(pool: SqlitePool) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut tick = interval(POLL_INTERVAL);
        // `interval` fires immediately on the first .tick(); skip that
        // initial fire so the first poll happens at POLL_INTERVAL, not
        // at startup.
        tick.tick().await;
        loop {
            tick.tick().await;
            if let Err(e) = poll_once(&pool).await {
                tracing::error!("temp-punishment poller error: {}", e);
            }
        }
    })
}

/// One polling pass: find expired cases, log them, mark resolved.
/// Errors are returned (logged by the caller) but don't stop the loop.
pub async fn poll_once(pool: &SqlitePool) -> Result<usize, BotError> {
    let now = chrono::Utc::now();
    let expired = crate::moderation::find_active_expired(pool, now).await?;
    let count = expired.len();

    for case in expired {
        tracing::info!(
            case_id = case.id,
            guild_id = case.guild_id,
            target_id = case.target_id,
            moderator_id = case.moderator_id,
            action_type = %case.action_type,
            duration_seconds = ?case.duration_seconds,
            "reversing expired temp punishment"
        );
        // v1: just mark as resolved. v2: also call Discord API to
        // remove the role, lift the timeout, or unban, depending on
        // action_type and role_id. The schema is ready.
        crate::moderation::mark_resolved(pool, case.id).await?;
    }

    Ok(count)
}
