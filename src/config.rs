//! Configuration loaded from environment variables.
//!
//! Reads `.env` (via `dotenvy::dotenv()` in `main.rs`) and parses into a
//! validated `Config` struct. All required fields are checked at startup;
//! the bot refuses to start with missing or invalid configuration (per ECHO
//! Law 3: Verify Before Proceed).

use std::env;

use crate::error::ConfigError;

/// All runtime configuration for savant-bot.
#[derive(Debug, Clone)]
pub struct Config {
    /// Discord bot token (required, from `DISCORD_BOT_TOKEN`).
    pub discord_token: String,

    /// OpenRouter API key for LLM features (from `OPENROUTER_API_KEY`).
    /// May be empty — the bot will start and LLM commands will report a
    /// "not configured" error at call time.
    pub openrouter_api_key: String,

    /// Default model for LLM calls when no override is specified.
    /// Default: `"openrouter/auto"`.
    pub llm_default_model: String,

    /// SQLite database URL (default: `sqlite:savant-bot.db`).
    pub database_url: String,

    /// Bot display name (default: `"savant-bot"`).
    pub bot_display_name: String,

    /// Command prefix for legacy text commands (default: `"!"`).
    pub command_prefix: String,

    /// Rate-limit tuple `(max_requests, per_seconds)` for LLM calls
    /// (default: `(5, 1)` = 5 requests per second, parsed from
    /// `LLM_RATE_LIMIT="5:1"`).
    pub llm_rate_limit: (u32, u32),
}

impl Config {
    /// Load and validate configuration from process environment.
    ///
    /// Returns `ConfigError::MissingEnvVar` if `DISCORD_BOT_TOKEN` is not set.
    /// Returns `ConfigError::InvalidEnvVar` if `LLM_RATE_LIMIT` is malformed.
    pub fn from_env() -> Result<Self, ConfigError> {
        let llm_rate_limit_raw = env::var("LLM_RATE_LIMIT").unwrap_or_else(|_| "5:1".to_string());
        Ok(Self {
            discord_token: required_env("DISCORD_BOT_TOKEN")?,
            openrouter_api_key: env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            llm_default_model: env::var("LLM_DEFAULT_MODEL")
                .unwrap_or_else(|_| "openrouter/auto".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:savant-bot.db".to_string()),
            bot_display_name: env::var("BOT_DISPLAY_NAME")
                .unwrap_or_else(|_| "savant-bot".to_string()),
            command_prefix: env::var("COMMAND_PREFIX").unwrap_or_else(|_| "!".to_string()),
            llm_rate_limit: parse_rate_limit(&llm_rate_limit_raw)?,
        })
    }
}

fn required_env(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::MissingEnvVar(key.to_string()))
}

fn parse_rate_limit(raw: &str) -> Result<(u32, u32), ConfigError> {
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 2 {
        return Err(ConfigError::InvalidEnvVar(
            "LLM_RATE_LIMIT".to_string(),
            raw.to_string(),
        ));
    }
    let max = parts[0]
        .parse::<u32>()
        .map_err(|_| ConfigError::InvalidEnvVar("LLM_RATE_LIMIT".to_string(), raw.to_string()))?;
    let per = parts[1]
        .parse::<u32>()
        .map_err(|_| ConfigError::InvalidEnvVar("LLM_RATE_LIMIT".to_string(), raw.to_string()))?;
    if max == 0 || per == 0 {
        return Err(ConfigError::InvalidEnvVar(
            "LLM_RATE_LIMIT".to_string(),
            raw.to_string(),
        ));
    }
    Ok((max, per))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rate_limit_valid() {
        assert_eq!(parse_rate_limit("5:1").unwrap(), (5, 1));
        assert_eq!(parse_rate_limit("20:10").unwrap(), (20, 10));
        assert_eq!(parse_rate_limit("100:60").unwrap(), (100, 60));
    }

    #[test]
    fn parse_rate_limit_invalid_format() {
        assert!(parse_rate_limit("5").is_err());
        assert!(parse_rate_limit("5:1:1").is_err());
        assert!(parse_rate_limit("abc:def").is_err());
        assert!(parse_rate_limit("0:1").is_err());
        assert!(parse_rate_limit("5:0").is_err());
    }
}
