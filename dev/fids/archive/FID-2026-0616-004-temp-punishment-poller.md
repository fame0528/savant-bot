# FID: SQLite Background Polling for Temporary Punishments

**Filename:** `FID-2026-0616-004-temp-punishment-poller.md`
**ID:** FID-2026-0616-004
**Severity:** medium
**Status:** closed
**Created:** 2026-06-16 17:51
**Closed:** 2026-06-16 18:36
**Author:** Agent (ECHO boot session 2026-06-16-1649)

---

## Resolution

- **Fixed By:** Agent (ECHO boot session 2026-06-16-1649)
- **Fixed Date:** 2026-06-16 18:36
- **Fix Description:** Implemented SQLite-based temp-punishment tracking. Created:
  - `migrations/0001_init.sql` — `moderation_cases` table with composite index on (status, expires_at) for the poller's hot query
  - `src/db/mod.rs` — `connect()` (WAL journaling, 5-connection pool, busy timeout) and `run_migrations()` (embeds + executes the SQL)
  - `src/moderation/mod.rs` — module root re-exporting public API
  - `src/moderation/cases.rs` (187 lines) — `create_case`, `find_active_expired`, `mark_resolved`, plus `ModerationCase` and `NewCase` types
  - `src/moderation/poller.rs` (60 lines) — `start_poller(pool) -> JoinHandle<()>` background task that polls every 60s, plus `poll_once` for testability
  - `src/commands/mute.rs` (58 lines) — `/mute` command using `required_permissions = "MODERATE_MEMBERS"`, inserts a case with `expires_at` set
  - Updated `Data` to include `db: SqlitePool` (initialized in `Data::new` via `db::connect` + `db::run_migrations`)
  - Updated `main.rs` to start the poller: `_poller = savant_bot::moderation::poller::start_poller(data.db.clone())`
  - Updated `lib.rs` to register `db` and `moderation` modules
- **v1 scope (deferred to v2):** The poller marks expired cases as `resolved` and logs a structured event. The actual Discord API reversal (remove Muted role, lift timeout, unban) is a v2 follow-up. The schema has `role_id` ready for it.
- **Tests Added:** 2 integration tests in `cases.rs` using in-memory SQLite: `create_then_find_active_expired` (3 cases inserted, only the active+expired one found), `mark_resolved_idempotent` (second mark is a no-op).
- **Verified By:** All 6 validation commands PASS + 2 new tests + FID-151 call-graph grep (`db::connect/run_migrations` wired in `data.rs`; `moderation::create_case` wired in `commands/mute.rs`; `start_poller` wired in `main.rs`).
- **Archived:** 2026-06-16 18:36 (this file is the archive copy; original moved to `dev/fids/archive/`)

## Lessons Learned

- SQLite WAL journaling is the right default for embedded DBs — better concurrent-read performance and crash recovery than the default rollback journal.
- The composite index `idx_moderation_cases_active_expires ON (status, expires_at) WHERE status = 'active'` is a **partial index**. It only indexes active rows, keeping the index small while making the poller's "active + expired" query fast.
- The `sqlx::Error::Configuration` constructor takes `Box<dyn Error + Send + Sync>`, NOT `Box<str>`. My initial attempt to use `Box::leak(...).into()` for an error string was over-engineered — `?` with a manual `From<sqlx::Error> for BotError` impl is the right pattern.
- For in-memory SQLite tests, `sqlx::SqlitePoolOptions::new().max_connections(1).connect_with(SqliteConnectOptions::new().filename(":memory:"))` is the standard pattern.
- The poller's `JoinHandle` is intentionally dropped (stored in `_poller`) so the task lives for the lifetime of the process. For graceful shutdown, the handle could be stored in `Data` and aborted on signal.

---

**References:** `research/comparison.md` Pattern 2; `dev/docs/Discord Bot Architecture Analysis.md` §2.1.2; Logiq's `discord.ext.tasks` strategy (the Python precedent for SQLite polling).
