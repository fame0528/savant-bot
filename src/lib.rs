//! savant-bot library entry point.
//!
//! This module exposes the public API: configuration loading, error types,
//! shared data, command modules, and the `run_bot` function that boots the
//! Discord gateway connection.
//!
//! The binary entry point is `src/main.rs`; the library surface exists so
//! that future integration tests can call `run_bot` directly without spawning
//! a subprocess.

pub mod commands;
pub mod config;
pub mod data;
pub mod db;
pub mod error;
pub mod llm;
pub mod moderation;

use poise::serenity_prelude as serenity;

use crate::{config::Config, data::Data, error::BotError};

/// Public type alias for command contexts. Re-exports Poise's `Context`
/// with our concrete `Data` and `BotError` types so command files can
/// write `use crate::Context;` instead of the longer form.
pub type Context<'a> = poise::Context<'a, Data, BotError>;

/// Public type alias for the bot's error type. Re-exports `BotError` so
/// command files can write `Result<(), Error>` instead of the longer form.
pub type Error = BotError;

/// Build the Poise framework and start the Discord gateway connection.
///
/// Blocks until the gateway connection ends (e.g. fatal error, Ctrl+C, or
/// process termination). All errors are returned via `BotError`.
pub async fn run_bot(config: Config, data: Data) -> Result<(), BotError> {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(config.command_prefix.clone()),
                ..Default::default()
            },
            on_error: |error| Box::pin(async move { error::on_framework_error(error).await }),
            ..Default::default()
        })
        .setup(|_ctx, _ready, framework| {
            Box::pin(async move {
                let count = framework.options().commands.len();
                tracing::info!("{} commands registered; bot is online", count);
                Ok(data)
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MODERATION;

    let mut client = serenity::ClientBuilder::new(&config.discord_token, intents)
        .framework(framework)
        .await?;

    tracing::info!("starting Discord gateway connection");
    client.start().await?;
    Ok(())
}
