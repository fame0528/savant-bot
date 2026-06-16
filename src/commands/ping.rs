//! `ping` command — the smoke test for a working bot.
//!
//! This is the simplest possible command and serves three purposes:
//! 1. Verifies the bot is connected and responding to Discord events
//! 2. Reports the WebSocket gateway latency in milliseconds
//! 3. Serves as a template for new commands (see the `#[poise::command]`
//!    macro usage and the `Context<'_>` parameter pattern)

use crate::{Context, Error};

/// Respond with "Pong!" and the bot's current gateway latency.
///
/// This is a slash command AND a prefix command. The macro
/// `#[poise::command(slash_command, prefix_command)]` enables both. To
/// restrict to one or the other, use just one of the attributes.
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency_ms = ctx.ping().await.as_millis();
    let response = format!("Pong! Gateway latency: {}ms", latency_ms);
    ctx.say(response).await?;
    Ok(())
}
