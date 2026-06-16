//! LLM infrastructure for savant-bot.
//!
//! Modules are added incrementally as each FID is implemented:
//! - `provider` (FID-008): trait abstraction + OpenRouter implementation
//! - `defer` (FID-005): defer-then-edit helper for slash commands
//! - `rate_limit` (FID-006): token-bucket limiter + exponential backoff
//! - `context` (FID-007): in-process sliding-window conversation memory
//!
//! Each module is a stand-alone piece of infrastructure that the LLM
//! commands (e.g., `/ask`) compose together.

pub mod context;
pub mod defer;
pub mod provider;
pub mod rate_limit;

pub use context::{ContextMessage, ContextStore};
pub use defer::defer_and_run;
pub use provider::{
    ChatMessage, ChatRequest, ChatResponse, MockProvider, OpenRouterProvider, Provider,
    ProviderError, Usage,
};
pub use rate_limit::{build_limiter, with_backoff, SharedLimiter};
