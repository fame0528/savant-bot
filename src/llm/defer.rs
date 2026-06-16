//! Defer-then-edit helper for long-running slash commands.
//!
//! Per doc §3.1, the Discord gateway has a 3-second response deadline.
//! Long-running operations (LLM calls, expensive API requests) must
//! acknowledge the interaction immediately (extending the validity window
//! from 3s to 15min) and edit the response when complete.
//!
//! This helper wraps Poise's `ctx.defer()` + `ctx.edit_response()` flow
//! with a single call. For commands that need to access `ctx` (e.g.,
//! to read channel ID or author info) inside the work closure, use
//! Poise's primitives directly instead — see `commands::ask::ask` for
//! the direct pattern.

use std::future::Future;

use crate::{Context, Error};

/// Run a long async operation with the defer-then-edit pattern.
///
/// Steps:
/// 1. `ctx.defer().await?` — acknowledges the interaction, extends the
///    validity window from 3s to 15min, shows a "thinking..." placeholder
/// 2. Runs the work closure
/// 3. `ctx.edit_response(...)` — replaces the placeholder with the
///    stringified result
///
/// `T: ToString` allows most types (String, &str, custom types with
/// `Display`) to be used as the result. For more complex responses
/// (embeds, components, multi-message), use Poise's `CreateReply`
/// directly instead of this helper.
pub async fn defer_and_run<F, Fut, T>(ctx: Context<'_>, work: F) -> Result<(), Error>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
    T: ToString,
{
    ctx.defer().await?;
    let result = work().await?;
    let response_text = result.to_string();
    // After defer(), `ctx.say()` (or `ctx.send()`) edits the deferred
    // placeholder in place rather than sending a new message. This is
    // Poise 0.6's defer-then-edit pattern.
    ctx.say(response_text).await?;
    Ok(())
}
