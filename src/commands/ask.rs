//! `ask` command — chat with the LLM using the full infrastructure chain.
//!
//! Wires together: defer-then-edit (FID-005), rate limiter (FID-006),
//! sliding-window context (FID-007), and provider (FID-008). This is
//! the canonical example of how LLM commands compose the `llm` module.

use std::time::Instant;

use crate::{
    llm::{
        context::ContextMessage,
        provider::{ChatMessage, ChatRequest},
        rate_limit::with_backoff,
    },
    Context, Error,
};

/// Ask the LLM a question.
#[poise::command(slash_command, prefix_command)]
pub async fn ask(
    ctx: Context<'_>,
    #[description = "Your question or prompt for the AI"] prompt: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Capture ctx-derived values before `defer()` so we can use them
    // in the work closure (the defer helper is for cases that don't
    // need ctx access inside the work — see `llm::defer::defer_and_run`).
    let channel_id = ctx.channel_id();
    let author_id = ctx.author().id.get();
    let author_name = ctx.author().name.clone();
    let prompt_for_context = prompt.clone();

    // Step 1: defer the response (3s → 15min, "thinking..." placeholder).
    ctx.defer().await?;

    // Step 2: wait for a rate-limit token.
    data.rate_limiter.until_ready().await;

    // Step 3: build the chat history. Recent channel context first
    // (per FID-007 sliding window), then the user's new prompt.
    let history = data.context.get(channel_id);
    let mut messages: Vec<ChatMessage> = history.iter().map(|m| m.to_chat_message()).collect();
    messages.push(ChatMessage::user(&prompt));

    // Step 4: build the LLM request.
    let req = ChatRequest::new(messages)
        .with_temperature(0.7)
        .with_max_tokens(1024);

    // Step 5: call the LLM with exponential backoff on 429 (FID-006).
    let resp = with_backoff(3, || async { data.provider.chat(req.clone()).await }).await?;

    // Step 6: update the sliding-window context with this turn.
    let now = Instant::now();
    data.context.push(
        channel_id,
        ContextMessage {
            user_id: author_id,
            display_name: author_name,
            content: prompt_for_context,
            timestamp: now,
        },
    );
    data.context.push(
        channel_id,
        ContextMessage {
            user_id: 0, // bot user (no real user id)
            display_name: data.config.bot_display_name.clone(),
            content: resp.content.clone(),
            timestamp: now,
        },
    );

    // Step 7: edit the deferred response with the final answer.
    // After defer(), ctx.say() edits the deferred placeholder in place.
    let body = if let Some(usage) = resp.usage {
        format!(
            "{}\n\n— _via {} ({} tokens)_",
            resp.content, resp.model, usage.total_tokens
        )
    } else {
        resp.content
    };
    ctx.say(body).await?;

    Ok(())
}
