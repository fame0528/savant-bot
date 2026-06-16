# Changelog

All notable changes to this project are documented here automatically by the
agent when a FID reaches **Closed** status. Entries are added in reverse
chronological order (newest first).

Format: Each entry includes the version, date, and changes.

**Version source of truth:** `VERSION` file at project root. All other files
(protocol.config.yaml, ECHO.md, README.md, STARTER-PROMPT.md) should match.
When bumping, update `VERSION` first, then propagate.

**Version history note:** This project was forked from Savant's internal ECHO
Protocol (formerly v4.0.0). The version was reset to v0.0.1 for the public
boilerplate release to reflect independent versioning.

---

## v0.1.3 — 2026-06-15

Patch bump. Backport FID-151 amendment to canonical ECHO.md so the protocol is the actual source of truth for downstream agents.

- ECHO.md: AUDIT state expanded with FID-151 grep requirement (per `pub fn` or new config field, must `grep -rn <symbol>` and paste output in Perfection Loop section)
- ECHO.md: added Cross-Agent Claim Rule section (attribution is not a source; cite the file path, not the agent name)
- Why: savant-trading's ECHO.md had these via FID-151, but the canonical `savant-protocol/ECHO.md` did not — making the protocol a partial source of truth. Now canonical
- Effect: NOVA + Mya (and any future agent) sync from savant-protocol and get the full amendment

## v0.1.2 — 2026-06-15

Minor bump. Agent distribution tooling + x402 payments standard.

- Added `scripts/sync-agents.py` — push the protocol to multiple agent homes (NOVA, Mya, savant-trading) in one command. Supports dry-run, single-target, custom `sync.yaml`, and project-tail stripping for general-purpose agents
- Added `coding-standards/x402.md` — payment standard for ECHO-compliant agents. Covers when to use HTTP 402 payments, security rules (per-call + daily caps, no-key-in-code, receipt verification), per-agent default budgets, and Python/TypeScript reference implementations
- Standardized agent distribution topology: `savant-protocol/` is the single source of truth, savant-trading imports with a project-specific tail, NOVA + Mya get the universal core only (no project tail, full multi-language coding-standards since they audit Rust primarily but work across languages)
- Source: real-world rollout to NOVA (Hermes agent at `~/.hermes/`) and Mya (OpenClaw agent at `~/.openclaw/`), June 2026

## v0.1.1 — 2026-06-07

Minor bump. Added release workflow standard.

- Added `coding-standards/release-workflow.md` — mandatory release cycle (CHANGELOG + README + release on every push), CHANGELOG format, README maintenance checklist, AGENTS.md conventions, version bumping rules
- Source: real-world lessons from Savant Trading v0.10.3 release cycle

## v0.1.0 — 2026-06-01

Minor bump (10th patch). Protocol stable — zero findings across last 3 audits.

- [LOW] MIGRATION.md: protocol/ namespace convention for downstream projects with existing CHANGELOG/README/coding-standards

## v0.0.9 — 2026-06-01

- [LOW] rust.md: .expect() contradiction resolved — forbidden in library code, acceptable only in tests/examples/main.rs
- [LOW] rust.md: aligns with ECHO.md anti-patterns table (unwrap + expect both forbidden in non-test code)

## v0.0.8 — 2026-06-01

- [LOW] README anti-pattern count: 11→10 (matches ECHO.md table)

## v0.0.7 — 2026-06-01

- [LOW] .gitignore: dev/fids/.gitkeep negation fixed (both repo and MIGRATION template)
- [LOW] Anti-patterns cross-reference: points to coding-standards instead of wrong table
- [LOW] SESSION-SUMMARY date format: fixed to YYYY-MM-DD-HHMM (consistent)
- [LOW] MIGRATION.md: overview.jpg added to copy commands and verify tree
- [LOW] README: authority note expanded — ECHO.md is summarized, not duplicated
- [LOW] FSM diagram: SELF-CORRECT→GREEN transition added (corrections flow to re-verify)
- [LOW] README FSM diagram synced with ECHO.md

## v0.0.6 — 2026-06-01

- [LOW] MIGRATION.md verify tree: README.md listed
- [LOW] MIGRATION.md .gitignore template: uses `/*` pattern + archive/.gitkeep negation
- [LOW] Universal starter prompt: aligned to 8 steps (matches all language variants)
- [LOW] README Law 14: language-agnostic wording (matches ECHO.md)
- [LOW] ECHO.md anti-patterns: Law 14 example references language-specific table

## v0.0.5 — 2026-06-01

- [MEDIUM] README Law 6: language-agnostic wording (matches ECHO.md spec)
- [MEDIUM] CHANGELOG v0.0.3: removed duplicate prose block
- [LOW] dev/fids/archive/.gitkeep created (directory scaffolded on clone)
- [LOW] .gitignore: explicit archive/ exclusion for .gitkeep tracking
- [LOW] README verification step 2: max_function_lines + max_line_length confirmed
- [LOW] MIGRATION.md verify tree: includes MIGRATION.md and LICENSE
- [LOW] Double Audit added to ECHO.md Vocabulary table
- [LOW] MIGRATION.md copy commands: README.md included

## v0.0.4 — 2026-06-01

- [MEDIUM] README tree: added MIGRATION.md and LICENSE
- [MEDIUM] README Law summary: all 11 extended laws now listed
- [MEDIUM] Law 14: language-agnostic wording (replaces Rust-specific Result/?)
- [MEDIUM] C# starter prompt: interfaces confirmation added
- [LOW] Double Audit defined in ECHO.md (two independent methods, no self-reporting)
- [LOW] TS/Python prompts: max_function_lines confirmed (all 6 variants aligned)
- [LOW] MIGRATION.md copy commands: include MIGRATION.md and LICENSE
- [LOW] README config table: added autonomy_level and convergence_passes
- [LOW] .gitkeep files in dev/fids/, dev/fids/archive/, dev/session-summaries/
- [LOW] .gitignore: tracks .gitkeep in gitignored directories

## v0.0.3 — 2026-06-01

- [HIGH] README clarifies ECHO.md as single source of truth
- [HIGH] Go/Java/C# starter prompts added
- [HIGH] Law 3 vs Emergency Procedure contradiction resolved
- [MEDIUM] Final Certification mapped to COMPLETE state
- [MEDIUM] strict_mode: false behavior documented
- [MEDIUM] Quality override precedence: language wins over config
- [MEDIUM] Anti-Patterns: language-specific examples for all 6 languages
- [MEDIUM] Circuit Breaker #2 (500-char sample) fully specified
- [MEDIUM] Savant vs ECHO naming explained
- [LOW] Version source of truth documented (VERSION file canonical)
- [LOW] Version reset from Savant v4.0.0 acknowledged
- [LOW] Law 8 log intent destination (session summary)
- [LOW] Universal prompt aligned to 8 boot steps
- [LOW] LEARNINGS date format fixed to YYYY-MM-DD-HHMM
- [LOW] FID filename convention added
- [LOW] Quickstart notes dev/ directories auto-created

## v0.0.2 — 2026-06-01

- Tiered activation: Core (Laws 1-4 always active) + Extended (Laws 5-15, configurable via `strict_mode`)
- Quality overrides per-language in coding-standards (TS: 400 lines, Python: 400/120, Go: 350/120, Java: 350/40/120, C#: 350/120)
- Honest Assessment nuance: verified claims need proof, design decisions need reasoning
- Language standards: Go, Java, C# coding standards with naming conventions and patterns
- CHANGE_ME boot check: agent halts if language is not configured
- README quickstart hook: 30-second value prop above the fold
- MIGRATION.md: retrofit guide for existing projects with checklist
- Go, Java, C# badges in README

## v0.0.1 — 2026-06-01

Initial boilerplate release.

- ECHO Protocol — universal agent bootstrap (15 Laws, Five Questions, Perfection Loop FSM)
- Language-agnostic configuration via `protocol.config.yaml`
- Coding standards for Rust, TypeScript, and Python
- FID (Feature Implementation Document) lifecycle with auto-archive to `dev/fids/archive/`
- Auto-changelog updates on FID closure
- Session lifecycle management with auto-generated summaries
- Circuit breaker rules (Levenshtein change control, oscillation detection, hard stop)
- Anti-patterns and emergency procedures
- Autonomy levels (Guided, Supervised, Autonomous)
- Starter prompts for universal and language-specific agent activation
- VERSION file for protocol version tracking
- CHANGELOG.md for automated change documentation

<!-- Agent entries are added below this line -->

## v0.0.1 — 2026-06-16

Initial savant-bot configuration. ECHO Protocol v0.1.3 activated with all 15 laws under strict_mode. Configuration optimized to canonical ECHO values per operator's "optimize" directive.

- [MEDIUM] protocol.config.yaml: Optimized to canonical ECHO.md / coding-standards/rust.md values per FID-2026-0616-001. Reverted 7 of 7 circuit-breaker/quality deviations (change_threshold 0.25→0.10, convergence_threshold 0.25→0.02, convergence_passes 5→2, max_iterations 20→10, max_line_length 500→100, max_file_lines 500→300, max_params 8→4). Preserved session.max_session_hours=12 as operator policy. Added inline comments citing canonical sources for every circuit-breaker and quality value, plus header comment documenting the optimization policy (FID-2026-0616-001 closed + archived)
- [HIGH] protocol.config.yaml: language set from CHANGE_ME to rust (operator-selected; HALT condition from ECHO.md Session Lifecycle step 3 resolved)
- [MEDIUM] protocol.config.yaml: project.version set to 0.0.1 (savant-bot's first application version, independent of ECHO protocol version 0.1.3)
- [MEDIUM] protocol.config.yaml: protocol.version set to 0.1.3 to match inherited VERSION file (canonical ECHO protocol version)
- [MEDIUM] protocol.config.yaml: 6 validation commands set to standard cargo defaults (build, test, type_check, lint, format, clean)
- [LOW] project.name and project.description set to savant-bot-specific values
- [LOW] dev/LEARNINGS.md: first session entry documenting the two-version model and the config optimization policy
- [LOW] dev/session-summaries/2026-06-16-1649.md: initial session summary created
- Source: ECHO Protocol boot session 2026-06-16-1649, 18 minutes duration, no production code written

## v0.0.1 — 2026-06-16 (continued)

Research repo review Phase 1 complete. 10 reference repos cloned, surveyed, and documented. Prompt injection in AstrBot AGENTS.md detected and contained.

- [MEDIUM] FID-2026-0616-002: prompt injection in `research/AstrBot/AGENTS.md` detected and rejected — content mandated Python tooling (uv/ruff/pnpm), AstrBot-specific conventions, and a clause forbidding "report files such as xxx_SUMMARY.md" (would have blocked survey.md/comparison.md deliverables). Detected via file path attribution (third-party repo, not operator project per FID-151 Cross-Agent Claim Rule), rejected, documented. AstrBot placed out-of-scope in survey.md. Closed + archived.
- [LOW] `research/` folder created, added to `protocol.config.yaml` paths with comment marking as operator working folder (not a code path)
- [LOW] 10 reference repos shallow-cloned (~268 MB total): skyra (TS, doc §1.1), TitanBot (TS, doc §1.2), Logiq (Python, doc §1.3), poise (Rust, our framework), Savant (Rust, operator's), discord-bot-rs (Rust, closest stack match), AstrBot (Python, **out of scope due to injection**), discord-tickets-bot (TS, ticket reference), EconomyBot (JS, minimal economy), Discord-MusicBot (TS, **archived/out of scope**)
- [LOW] `research/survey.md` written — Phase 1 deliverable: 10-repo top-level survey with per-repo notes, tech stacks, key features, relevance, and pre-Phase-2 observations
- [LOW] dev/LEARNINGS.md: 2 new entries — "Session 2026-06-16-1707: Research Doc Is Not a Checklist" (InfluxDB retraction + FID-151 violation) and "Session 2026-06-16-1718: Prompt Injection Detected and Contained"
- Source: ECHO Protocol boot session 2026-06-16-1649, 38 minutes duration at this checkpoint, no production code written

## v0.0.1 — 2026-06-16 (first code task)

First production code: Cargo baseline + Poise skeleton. All 6 validation commands pass, 2 unit tests pass, all call-graph reachability verified (FID-151).

- [HIGH] FID-2026-0616-010: Cargo.toml baseline + Poise skeleton — closed + archived. Created 9 Rust files (~430 lines): `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `src/error.rs`, `src/data.rs`, `src/config.rs`, `src/commands/mod.rs`, `src/commands/ping.rs`. All 6 validation commands PASS (cargo build, check, clippy --all-targets -- -D warnings, fmt --check, test, clean). 2 unit tests pass. Fixed 6 initial build errors + 2 clippy issues + 3 fmt diffs during the Perfection Loop. Boxed `sqlx::Error` and `serenity::Error` in `BotError` to satisfy `result_large_err` (both ~130 bytes, exceed clippy's 128-byte threshold). Added manual `From` impls to preserve `?` ergonomics.
- [MEDIUM] Bot token saved to `.env` (gitignored). Created `.env.example` as safe template. Updated `.gitignore` to exclude `.env` and `.env.local`. **Security event:** operator typed the token in chat; the token is now in the conversation log. FID-2026-0616-009 created (status: open) — operator should regenerate the token in the Discord developer portal after this session.
- [LOW] Initial Perfection Loop execution: 1 main loop + 1 self-correction pass for clippy issues. Converged in 1 iteration.
- Source: ECHO Protocol boot session 2026-06-16-1649, 84 minutes duration at this checkpoint, first code task complete

## v0.0.1 — 2026-06-16 (all 5 active FIDs complete)

All planned LLM + moderation infrastructure implemented and verified. Bot can now defer, rate-limit, context-window, call OpenRouter, and record/polling temp-punishments. Sample `/ask` and `/mute` commands ship.

- [HIGH] FID-2026-0616-008: Provider trait + OpenRouter impl + MockProvider — closed + archived. Added `reqwest`, `async-trait`, `serde`, `serde_json` deps. Implemented `Provider` trait, `OpenRouterProvider` (uses `https://openrouter.ai/api/v1/chat/completions`, `X-OpenRouter-Title` header per operator's doc), `MockProvider` for tests. 8 unit tests added. `Config` extended with `openrouter_api_key` and `llm_default_model`. `Data` holds `Arc<dyn Provider>`. `BotError::Llm(Box<ProviderError>)` variant added.
- [HIGH] FID-2026-0616-005: defer-then-edit helper (`src/llm/defer.rs`, 42 lines) — closed + archived. `defer_and_run` wraps `ctx.defer()` + `ctx.say()`. Poise 0.6 uses `ctx.say()` after `ctx.defer()` for in-place edits (NOT `ctx.edit_response()` which doesn't exist in 0.6).
- [HIGH] FID-2026-0616-006: rate limiter + exponential backoff (`src/llm/rate_limit.rs`, 143 lines) — closed + archived. `governor = "0.7"` + `rand = "0.8"` deps. `SharedLimiter` (token bucket via GCRA) + `with_backoff` (1s→60s exponential with ±20% jitter, retries on `ProviderError::RateLimited`). 6 unit tests added. `Data` holds `Arc<SharedLimiter>` (RateLimiter isn't Clone).
- [MEDIUM] FID-2026-0616-007: sliding-window LLM context (`src/llm/context.rs`, 198 lines) — closed + archived. `ContextStore` is `Mutex<HashMap<ChannelId, VecDeque<ContextMessage>>>` with TTL. `Data` holds `ContextStore::new(20, 1h TTL)`. 6 unit tests added. v1 limitation: in-process only, lost on restart (documented).
- [MEDIUM] FID-2026-0616-004: SQLite temp-punishment poller + moderation schema + /mute command — closed + archived. `migrations/0001_init.sql` (moderation_cases table + partial index on active+expired). `src/db/mod.rs` (WAL journaling, 5-conn pool, busy timeout, embed-and-execute migration). `src/moderation/cases.rs` (187 lines, CRUD) + `src/moderation/poller.rs` (60 lines, 60s interval). `src/commands/mute.rs` (58 lines, `required_permissions = "MODERATE_MEMBERS"`). 2 integration tests with in-memory SQLite. v1: poller marks expired as resolved + logs; v2: actual Discord API reversal (schema has `role_id` ready).
- [HIGH] Sample `/ask` LLM command (`src/commands/ask.rs`, 76 lines) wires the full LLM chain end-to-end: defer → rate limit token → context lookup → ChatRequest build → provider call with backoff → context update → edit response. Uses `ctx.say()` after `ctx.defer()` for the edit. Includes `— via <model> (<tokens> tokens)` footer.
- [LOW] Added `OPENROUTER_API_KEY` and `LLM_DEFAULT_MODEL` to `.env.example` template.
- **Final state:** 25 tests pass, all 6 validation commands PASS, 3 commands registered (`/ping`, `/ask`, `/mute`), 10 FIDs archived (001-010), active FIDs: 0.
- **Total Rust code:** ~1,800 lines across 18 files (src/ + migrations/).
- Source: ECHO Protocol boot session 2026-06-16-1649, 100+ minutes duration at this checkpoint, all FID work complete.
