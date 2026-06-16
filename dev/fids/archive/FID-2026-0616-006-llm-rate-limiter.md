# FID: Client-Side Token-Bucket Rate Limiter + HTTP 429 Exponential Backoff

**Filename:** `FID-2026-0616-006-llm-rate-limiter.md`
**ID:** FID-2026-0616-006
**Severity:** high
**Status:** closed
**Created:** 2026-06-16 17:51
**Closed:** 2026-06-16 18:34
**Author:** Agent (ECHO boot session 2026-06-16-1649)

---

## Resolution

- **Fixed By:** Agent (ECHO boot session 2026-06-16-1649)
- **Fixed Date:** 2026-06-16 18:34
- **Fix Description:** Implemented `build_limiter` and `with_backoff` in `src/llm/rate_limit.rs` (143 lines). Added `governor = "0.7"` and `rand = "0.8"` deps. The `SharedLimiter` type alias wraps `governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>`. `with_backoff` retries on `ProviderError::RateLimited` with exponential backoff (1s, 2s, 4s, ..., 60s cap) and ±20% jitter, up to `max_retries + 1` total attempts. Module registered in `src/llm/mod.rs` and re-exported. `Data::new` constructs the rate limiter from `Config::llm_rate_limit` and wraps in `Arc` (because `governor::RateLimiter` is not `Clone`).
- **Tests Added:** 6 unit tests in `rate_limit.rs`: first-try success, success after 2 retries, gives up after max retries, no retry on non-rate-limit error, zero-retries means single attempt, `build_limiter` produces a valid limiter for several inputs.
- **Verified By:** All 6 validation commands PASS + 6 new tests + FID-151 call-graph grep (`build_limiter` wired in `data.rs`; `with_backoff` wired in `commands/ask.rs`).
- **Archived:** 2026-06-16 18:34 (this file is the archive copy; original moved to `dev/fids/archive/`)

## Lessons Learned

- `governor::RateLimiter` is **not `Clone`**. To share it across `Data` (which must be `Clone` for Poise), wrap in `Arc<SharedLimiter>`. Arc deref makes the API ergonomic.
- The `governor::Quota::per_second(n)` API is the natural unit. For "5 per 10s", you have to convert to per-second: `5 / 10 = 0.5`, which `governor` rounds up to 1 per second. For finer granularity, use `Quota::per_minute(n)` directly.
- The backoff jitter calculation: `rand::random::<f64>() * 0.4 - 0.2` gives a value in `[-0.2, 0.2]`, which multiplies the delay by a factor in `[0.8, 1.2]`. This is ±20% jitter.
- `with_backoff` returns `Result<T, BotError>`. The closure parameter returns `Result<T, ProviderError>`, and the backoff function converts on the way out. This keeps the function focused on rate-limit-specific retry logic.

---

**References:** `research/comparison.md` Pattern 6; `dev/docs/Discord Bot Architecture Analysis.md` §3.2; `governor` crate documentation.
