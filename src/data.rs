//! Shared bot state, passed to every command via `ctx.data()`.
//!
//! Holds: parsed `Config`, LLM `Provider`, rate limiter, sliding-window
//! context, and SQLite pool (for moderation). Future additions: caching
//! layer, per-guild settings cache, etc.

use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;

use crate::{
    config::Config,
    db,
    error::BotError,
    llm::{
        context::ContextStore,
        provider::{OpenRouterProvider, Provider},
        rate_limit::{build_limiter, SharedLimiter},
    },
};

/// The data struct passed to every Poise command invocation.
///
/// `Data` must be `Send + Sync` (Poise requires this). It is wrapped in
/// `Arc` internally by the framework, so we don't need to wrap fields
/// individually unless they need interior mutability.
#[derive(Clone)]
pub struct Data {
    /// Parsed configuration (shared, immutable).
    pub config: Config,

    /// LLM provider trait object (OpenRouter-backed; per FID-008).
    pub provider: Arc<dyn Provider>,

    /// Client-side rate limiter for LLM calls (per FID-006).
    /// Wrapped in `Arc` because `governor::RateLimiter` is not `Clone`.
    pub rate_limiter: Arc<SharedLimiter>,

    /// In-process sliding-window LLM context per channel (per FID-007).
    pub context: ContextStore,

    /// SQLite connection pool (per FID-004).
    pub db: SqlitePool,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("config", &self.config)
            .field("provider", &"<dyn Provider>")
            .field("rate_limiter", &"<SharedLimiter>")
            .field("context", &self.context)
            .field("db", &"<SqlitePool>")
            .finish()
    }
}

impl Data {
    /// Build the shared state at bot startup.
    ///
    /// Initializes: LLM provider, rate limiter, context store, SQLite
    /// pool, runs migrations. Returns `Data` ready to be passed to Poise.
    pub async fn new(config: Config) -> Result<Self, BotError> {
        let provider: Arc<dyn Provider> = Arc::new(OpenRouterProvider::new(
            config.openrouter_api_key.clone(),
            vec![], // no default fallback models; commands specify their own
        ));

        let rate_limiter = Arc::new(build_limiter(
            config.llm_rate_limit.0,
            config.llm_rate_limit.1,
        ));

        let context = ContextStore::new(20, Duration::from_secs(3600)); // 20 msgs, 1h TTL

        let db = db::connect(&config.database_url).await?;
        db::run_migrations(&db).await?;

        tracing::info!(
            display_name = %config.bot_display_name,
            default_model = %config.llm_default_model,
            "initializing shared state"
        );
        Ok(Self {
            config,
            provider,
            rate_limiter,
            context,
            db,
        })
    }
}
