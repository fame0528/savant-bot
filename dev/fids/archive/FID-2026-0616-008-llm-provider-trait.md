# FID: Provider Trait + OpenRouter Implementation

**Filename:** `FID-2026-0616-008-llm-provider-trait.md`
**ID:** FID-2026-0616-008
**Severity:** high
**Status:** closed
**Created:** 2026-06-16 17:51
**Closed:** 2026-06-16 18:32
**Author:** Agent (ECHO boot session 2026-06-16-1649)

---

## Resolution

- **Fixed By:** Agent (ECHO boot session 2026-06-16-1649)
- **Fixed Date:** 2026-06-16 18:32
- **Fix Description:** Implemented `Provider` trait (`src/llm/provider.rs`) with `OpenRouterProvider` and `MockProvider` impls. Added `reqwest`, `async-trait`, `serde`, `serde_json` deps. Updated `Config` with `openrouter_api_key` and `llm_default_model` fields. Added `Llm` variant to `BotError`. Updated `Data` to hold `Arc<dyn Provider>`. Per the operator-provided `dev/docs/openrouter-llms.md` (read 0-EOF), corrected the header name to `X-OpenRouter-Title` (was `X-Title`).
- **Tests Added:** 8 unit tests in `provider.rs` (MockProvider response queue, error return, call capture; ChatMessage/ChatRequest constructors; ProviderError::is_rate_limited; OpenRouterProvider not-configured + name).
- **Verified By:** All 6 validation commands PASS + 8 new tests + FID-151 call-graph grep (provider wired in `data.rs` via `Arc::new(OpenRouterProvider::new(...))`).
- **Archived:** 2026-06-16 18:32 (this file is the archive copy; original moved to `dev/fids/archive/`)

## Lessons Learned

- The OpenRouter doc specifies `X-OpenRouter-Title` as the header name; the older `X-Title` still works but the new name is preferred. Documented in provider.rs with a citation.
- `Box<sqlx::Error>` and `Box<serenity::Error>` boxing pattern for thiserror enums works cleanly with manual `From` impls (verified across all 3 large variants now: sqlx, serenity, ProviderError).
- The `MockProvider` with response queue + captured calls is the test pattern that scales — every consumer-side test can use a MockProvider without HTTP mocking infrastructure.

---

**References:** `dev/docs/openrouter-llms.md` (operator-provided, read 0-EOF); `research/comparison.md` Pattern 8; discord-bot-rs `src/ai/chat.rs:11` for the provider-abstraction pattern.
