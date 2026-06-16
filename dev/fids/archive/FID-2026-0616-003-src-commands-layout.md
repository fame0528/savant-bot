# FID: Adopt `src/commands/<feature>.rs` Per-Feature Layout

**Filename:** `FID-2026-0616-003-src-commands-layout.md`
**ID:** FID-2026-0616-003
**Severity:** low
**Status:** closed
**Created:** 2026-06-16 17:51
**Closed:** 2026-06-16 18:13
**Author:** Agent (ECHO boot session 2026-06-16-1649)

---

## Summary

Adopt the discord-bot-rs pattern for command organization: `src/commands/<feature>.rs` with one file per feature, each containing one or more `#[poise::command]` functions. Graduate to `src/commands/<feature>/mod.rs` for large features.

## Resolution

- **Fixed By:** Agent (ECHO boot session 2026-06-16-1649)
- **Fixed Date:** 2026-06-16 18:13 (with FID-010 implementation, 2026-06-16 18:05)
- **Fix Description:** Layout adopted in the FID-010 skeleton. The skeleton uses `src/commands/mod.rs` (registry) + `src/commands/ping.rs` (one feature, one file). The `all()` function in `commands/mod.rs` is the single source of truth for command registration. New features will follow the same pattern (one `<feature>.rs` file per feature, registered in `all()`).
- **Verified By:** Skeleton built and lints clean (per FID-010 verification). Subsequent features (LLM `/ask`, moderation `/mute`) will follow the same pattern as they are implemented.
- **Commit/PR:** N/A (not a git repo)
- **Archived:** 2026-06-16 18:13 (this file is the archive copy; original moved to `dev/fids/archive/`)

## Lessons Learned

- The `all()` function in `commands/mod.rs` is the single source of truth for command registration. Adding a `#[poise::command]` to a function is NOT enough — the function must also be listed in `all()` for the framework to discover it. This is documented in the module's doc-comment.

---

**References:** `research/comparison.md` Pattern 1; `research/survey.md` per-repo notes; Poise documentation; FID-2026-0616-010 (skeleton, where the layout was adopted).
