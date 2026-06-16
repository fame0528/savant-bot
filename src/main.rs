use savant_bot::{config::Config, data::Data, error::BotError, run_bot};

#[tokio::main]
async fn main() -> Result<(), BotError> {
    // Best-effort .env load. In production, env vars come from the orchestrator.
    let _ = dotenvy::dotenv();

    // Initialize tracing subscriber. Falls back to a sensible default if RUST_LOG is unset.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,savant_bot=debug")),
        )
        .init();

    tracing::info!("savant-bot v{} starting", env!("CARGO_PKG_VERSION"));

    // Load configuration from environment.
    let config = Config::from_env().map_err(BotError::Config)?;
    tracing::debug!(?config, "configuration loaded");

    // Build shared state.
    let data = Data::new(config.clone()).await?;
    tracing::debug!("shared state initialized");

    // Start the temp-punishment poller (per FID-004).
    let _poller = savant_bot::moderation::poller::start_poller(data.db.clone());
    tracing::info!("temp-punishment poller started");

    // Run the bot (blocks until the gateway connection ends).
    run_bot(config, data).await
}
