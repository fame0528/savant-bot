//! Moderation subsystem.
//!
//! Per FID-2026-0616-004 and doc §2.1.2, temp punishments (mute, timeout,
//! ban) must persist across restarts. Naive `tokio::time::sleep`-based
//! timers are volatile. This module provides:
//!
//! - [`cases`]: CRUD for `moderation_cases` rows
//! - [`poller`]: background task that finds expired cases and resolves them
//!
//! v1 scope: the poller marks expired cases as `resolved` and logs a
//! structured event. Full Discord API reversal (remove role, lift timeout,
//! unban) is a v2 follow-up — schema has `role_id` ready for it.

pub mod cases;
pub mod poller;

pub use cases::{create_case, find_active_expired, mark_resolved, ModerationCase, NewCase};
pub use poller::start_poller;
