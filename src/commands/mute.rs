//! `mute` command — issue a temp-mute with persistent tracking.
//!
//! Wires the moderation subsystem (FID-004): inserts a `moderation_cases`
//! row with `expires_at` set, and the background poller will reverse
//! the punishment at expiry.
//!
//! v1 scope: this command only records the case in the database. It
//! does NOT call the Discord API to actually apply the mute (remove
//! the Muted role). The schema and poller are in place for v2 to add
//! the actual API call.

use chrono::{Duration, Utc};
use poise::serenity_prelude as serenity;

use crate::{
    moderation::{self, NewCase},
    Context, Error,
};

/// Mute a user for a specified duration. Recorded for the poller.
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn mute(
    ctx: Context<'_>,
    #[description = "User to mute"] user: serenity::User,
    #[description = "Duration in minutes"] minutes: u64,
    #[description = "Reason for the mute"] reason: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let guild_id = ctx.guild_id().ok_or_else(|| {
        Error::Llm(Box::new(crate::llm::provider::ProviderError::Unavailable(
            "mute must be used inside a guild".to_string(),
        )))
    })?;
    let moderator_id = ctx.author().id.get();
    let target_id = user.id.get();
    let now = Utc::now();
    let expires_at = now + Duration::minutes(minutes as i64);

    let case_id = moderation::create_case(
        &data.db,
        NewCase {
            guild_id: guild_id.get() as i64,
            target_id: target_id as i64,
            moderator_id: moderator_id as i64,
            action_type: "MUTE",
            role_id: None, // v2: look up the Muted role
            duration_seconds: Some((minutes * 60) as i64),
            reason: reason.as_deref(),
            expires_at: Some(expires_at),
        },
    )
    .await?;

    let response = format!(
        "Muted <@{target_id}> for {minutes} minutes. Case #{case_id}. (v1: case recorded; Discord API mute is a v2 follow-up.)"
    );
    ctx.say(response).await?;
    Ok(())
}
