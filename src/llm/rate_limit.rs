//! Token-bucket rate limiter and HTTP 429 exponential backoff for LLM calls.
//!
//! Per doc §3.2, OpenRouter and upstream LLM providers enforce rate limits.
//! A naive client can either exceed limits (HTTP 429 → may get IP-banned)
//! or crash silently. This module provides two pieces of defense:
//!
//! 1. [`build_limiter`] — a `governor::RateLimiter` that blocks local calls
//!    until a token is available, preventing the app from sending requests
//!    faster than the configured rate.
//! 2. [`with_backoff`] — a wrapper that retries on `ProviderError::RateLimited`,
//!    parsing the `Retry-After` header value and applying exponential backoff
//!    with ±20% jitter.

use std::future::Future;
use std::time::Duration;

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};

use crate::{error::BotError, llm::provider::ProviderError};

/// Default direct rate limiter (in-memory state, not keyed). Suitable for
/// a single-process bot. Future v2: swap to a keyed limiter for per-user
/// rate limits, or a distributed limiter if we ever shard.
pub type SharedLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Maximum backoff delay. Per doc §3.2, the backoff routine should be
/// "progressively increasing the delay between retry attempts... until
/// the request succeeds or a maximum threshold is reached". 60s is a
/// reasonable cap that still respects provider rate limits.
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// Build a rate limiter from a quota of `max_requests` per `per_seconds`.
///
/// The quota is converted to "per_second" form internally because
/// `governor::Quota::per_second` is the natural unit. A request to
/// `build_limiter(5, 1)` produces 5 requests per second. To get "5 per
/// 10 seconds", pass `build_limiter(5, 10)` and the quota becomes
/// `5 / 10 = 0.5` per second, which `governor` rounds up to 1 per
/// second. For finer granularity, use [`Quota::per_minute`] directly.
pub fn build_limiter(max_requests: u32, per_seconds: u32) -> SharedLimiter {
    let per_second = (max_requests / per_seconds.max(1)).max(1);
    let n = std::num::NonZeroU32::new(per_second).unwrap_or(std::num::NonZeroU32::new(1).unwrap());
    RateLimiter::direct(Quota::per_second(n))
}

/// Run an async operation with exponential backoff on rate-limit errors.
///
/// Retries up to `max_retries` times on `ProviderError::RateLimited`. The
/// delay starts at 1s and doubles each retry (1s, 2s, 4s, 8s, ...) up
/// to [`MAX_BACKOFF`]. Each delay is multiplied by a random factor in
/// `0.8..1.2` (±20% jitter) to prevent thundering-herd.
///
/// Non-rate-limit errors are returned immediately without retry. This
/// keeps the backoff logic scoped to the specific failure mode it
/// mitigates.
pub async fn with_backoff<F, Fut, T>(max_retries: u32, f: F) -> Result<T, BotError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, ProviderError>>,
{
    let mut delay = Duration::from_secs(1);
    let mut last_err: Option<ProviderError> = None;

    for _ in 0..=max_retries {
        match f().await {
            Ok(t) => return Ok(t),
            Err(e @ ProviderError::RateLimited(retry_after)) => {
                last_err = Some(e);
                let jitter: f64 = rand::random::<f64>() * 0.4 - 0.2; // ±20%
                let actual = delay
                    .mul_f64(1.0 + jitter)
                    .max(Duration::from_secs(retry_after));
                tracing::debug!(?actual, "rate limited; sleeping with backoff+jitter");
                tokio::time::sleep(actual).await;
                delay = std::cmp::min(delay * 2, MAX_BACKOFF);
            }
            Err(other) => return Err(other.into()),
        }
    }

    Err(last_err.expect("loop ran at least once").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn with_backoff_succeeds_on_first_try() {
        let result: Result<i32, BotError> = with_backoff(3, || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn with_backoff_succeeds_after_two_retries() {
        let attempts = AtomicU32::new(0);
        let result: Result<i32, BotError> = with_backoff(3, || {
            let count = attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(ProviderError::RateLimited(0))
                } else {
                    Ok(99)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 99);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn with_backoff_gives_up_after_max_retries() {
        let attempts = AtomicU32::new(0);
        let result: Result<i32, BotError> = with_backoff(2, || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err(ProviderError::RateLimited(0)) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // initial + 2 retries
    }

    #[tokio::test]
    async fn with_backoff_does_not_retry_on_non_rate_limit_error() {
        let attempts = AtomicU32::new(0);
        let result: Result<i32, BotError> = with_backoff(3, || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err(ProviderError::Unavailable("nope".to_string())) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1); // no retries
    }

    #[tokio::test]
    async fn with_backoff_zero_retries_means_single_attempt() {
        let attempts = AtomicU32::new(0);
        let result: Result<i32, BotError> = with_backoff(0, || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err(ProviderError::RateLimited(0)) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn build_limiter_creates_valid_limiter() {
        let _limiter = build_limiter(5, 1);
        let _limiter2 = build_limiter(20, 10);
        let _limiter3 = build_limiter(1, 1);
    }
}
