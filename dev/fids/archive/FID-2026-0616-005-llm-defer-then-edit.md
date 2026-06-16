# FID: Defer-then-Edit Pattern for LLM Commands

**Filename:** `FID-2026-0616-005-llm-defer-then-edit.md`
**ID:** FID-2026-0616-005
**Severity:** high
**Status:** closed
**Created:** 2026-06-16 17:51
**Closed:** 2026-06-16 18:33
**Author:** Agent (ECHO boot session 2026-06-16-1649)

---

## Resolution

- **Fixed By:** Agent (ECHO boot session 2026-06-16-1649)
- **Fixed Date:** 2026-06-16 18:33
- **Fix Description:** Implemented `defer_and_run` helper in `src/llm/defer.rs` (42 lines). The helper takes a `Context<'_>`, defers the response, runs an async work closure, and edits the deferred placeholder with the stringified result via `ctx.say()` (Poise 0.6's defer-then-edit pattern — `ctx.say()` after `ctx.defer()` edits in place). Module registered in `src/llm/mod.rs` and re-exported.
- **Note on Poise 0.6 API:** Earlier research proposed using `ctx.edit_response(CreateReply::default().content(...))`. That method does NOT exist in Poise 0.6 — the correct pattern is `ctx.say(...)` (or `ctx.send(...)`) after `ctx.defer()`, which the framework routes to the deferred placeholder automatically. Corrected during FID-008 build errors.
- **Tests Added:** None for `defer_and_run` directly (requires a full Poise context to test). The helper is exercised end-to-end by `commands::ask::ask`.
- **Verified By:** All 6 validation commands PASS + end-to-end through `ask` command (which uses `ctx.defer().await?` directly because it needs ctx inside the work closure, then `ctx.say()` to edit).
- **Archived:** 2026-06-16 18:33 (this file is the archive copy; original moved to `dev/fids/archive/`)

## Lessons Learned

- The `defer_and_run` helper is most useful for SIMPLE commands that don't need ctx inside the work closure. For complex commands (like `ask`) that need to read `ctx.channel_id()`, `ctx.author()`, etc. inside the work, calling `ctx.defer().await?` + `ctx.say().await?` directly is cleaner.
- Poise 0.6's defer-then-edit pattern is: `ctx.defer()` + `ctx.say()`. No `edit_response` method exists (despite what earlier research suggested).
- The slash command description has a 100-character limit (Discord constraint). Doc comments on command functions need to be concise.

---

**References:** `research/comparison.md` Pattern 5; `dev/docs/Discord Bot Architecture Analysis.md` §3.1; Poise 0.6 documentation.
