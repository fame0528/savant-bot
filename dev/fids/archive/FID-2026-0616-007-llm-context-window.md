# FID: In-Process Sliding-Window LLM Context

**Filename:** `FID-2026-0616-007-llm-context-window.md`
**ID:** FID-2026-0616-007
**Severity:** medium
**Status:** closed
**Created:** 2026-06-16 17:51
**Closed:** 2026-06-16 18:35
**Author:** Agent (ECHO boot session 2026-06-16-1649)

---

## Resolution

- **Fixed By:** Agent (ECHO boot session 2026-06-16-1649)
- **Fixed Date:** 2026-06-16 18:35
- **Fix Description:** Implemented `ContextStore` in `src/llm/context.rs` (198 lines). The store is a `Mutex<HashMap<ChannelId, ChannelContext>>` where `ChannelContext` is a `VecDeque<ContextMessage>` with `last_active` for TTL. Public API: `new(max_messages, ttl)`, `push(channel, msg)`, `get(channel)`, `cleanup_expired() -> usize`, plus `len()` / `is_empty()` for tests. The `ContextMessage::to_chat_message()` method formats a message as `display_name: content` for inclusion in the LLM prompt. Module registered in `src/llm/mod.rs` and re-exported. `Data::new` constructs a `ContextStore::new(20, Duration::from_secs(3600))` (20 messages per channel, 1-hour TTL).
- **Tests Added:** 6 unit tests: push and get, max_messages trims oldest, expired context returns empty and clears, cleanup_expired removes stale, different channels have separate contexts, to_chat_message includes display name.
- **Verified By:** All 6 validation commands PASS + 6 new tests + FID-151 call-graph grep (`ContextStore::new` wired in `data.rs`; `context.get/push` wired in `commands/ask.rs`).
- **Archived:** 2026-06-16 18:35 (this file is the archive copy; original moved to `dev/fids/archive/`)

## Lessons Learned

- The sliding-window context is the cheapest way to give an LLM conversation coherence. discord-bot-rs's `MAX_RELEVANT: usize = 10` is the practical upper bound; we use 20 for slightly more context, with a 1-hour TTL.
- `VecDeque` is the right data structure for sliding windows: O(1) push to back, O(1) pop from front when trimming. The standard library provides this for free.
- v1 limitation: context is in-process only, lost on bot restart. Future migration to SQLite is straightforward: replace the internal `Mutex<HashMap>` with `sqlx::query` calls. Documented in the module doc-comment.
- Clippy's `len_without_is_empty` lint fires when a public `len()` exists without `is_empty()`. Both methods are required for the standard collection interface.

---

**References:** `research/comparison.md` Pattern 7; `dev/docs/Discord Bot Architecture Analysis.md` §3.3.
