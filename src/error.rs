//! Typed error hierarchy for savant-bot.
//!
//! All fallible operations return `Result<_, BotError>`. The `#[from]` derives
//! allow `?` to convert from common error types. `on_framework_error` is the
//! central error reporter for the Poise framework.

use poise::serenity_prelude as serenity;

use crate::Data;

/// Top-level error type for all bot operations.
///
/// Both `sqlx::Error` and `serenity::Error` are large enums (~130 bytes each).
/// To keep `BotError` under clippy's 128-byte `result_large_err` threshold,
/// we box both. Manual `From` impls preserve the `?` ergonomics.
#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("configuration error: {0}")]
    Config(#[source] ConfigError),

    #[error("database error: {0}")]
    Database(#[source] Box<sqlx::Error>),

    #[error("Discord API error: {0}")]
    Serenity(#[source] Box<serenity::Error>),

    #[error("LLM provider error: {0}")]
    Llm(#[source] Box<crate::llm::provider::ProviderError>),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Manual `From<sqlx::Error>` impl: the `Database` variant holds a
/// `Box<sqlx::Error>` to keep `BotError` under clippy's 128-byte threshold.
impl From<sqlx::Error> for BotError {
    fn from(error: sqlx::Error) -> Self {
        BotError::Database(Box::new(error))
    }
}

/// Manual `From<serenity::Error>` impl for the same reason.
impl From<serenity::Error> for BotError {
    fn from(error: serenity::Error) -> Self {
        BotError::Serenity(Box::new(error))
    }
}

/// Manual `From<ProviderError>` impl for the same reason.
impl From<crate::llm::provider::ProviderError> for BotError {
    fn from(error: crate::llm::provider::ProviderError) -> Self {
        BotError::Llm(Box::new(error))
    }
}

/// Configuration-load error. Surfaced via `BotError::Config`.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("invalid value for environment variable {0}: {1}")]
    InvalidEnvVar(String, String),
}

/// Central error handler for the Poise framework.
///
/// Logs the error with `tracing` and (where possible) tells the user via the
/// affected context. Per ECHO Law 14, all error paths are handled; no error
/// is silently swallowed.
pub async fn on_framework_error(error: poise::FrameworkError<'_, Data, BotError>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            tracing::error!(command = %ctx.command().name, "command error: {}", error);
            let _ = ctx.say(format!("an error occurred: {}", error)).await;
        }
        poise::FrameworkError::ArgumentParse { error, ctx, .. } => {
            tracing::warn!("argument parse error: {}", error);
            let _ = ctx.say(format!("invalid arguments: {}", error)).await;
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            tracing::debug!("command check failed: {:?}", error);
            let _ = ctx.say("you are not allowed to run this command.").await;
        }
        poise::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            tracing::debug!("cooldown hit ({}s remaining)", remaining_cooldown.as_secs());
            let _ = ctx
                .say(format!(
                    "please wait {}s before running this command again.",
                    remaining_cooldown.as_secs()
                ))
                .await;
        }
        other => {
            tracing::error!("framework error: {:?}", other);
        }
    }
}
