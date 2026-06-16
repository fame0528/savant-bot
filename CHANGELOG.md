# Changelog

All notable changes to this project are documented here.

Format: Each entry includes the version, date, and changes.

**Version source of truth:** Two distinct version sources:
- **App version** (0.0.x): `Cargo.toml` `version` + `protocol.config.yaml` `project.version`. Bump on app releases.
- **ECHO Protocol version** (0.1.x): `VERSION` file + `protocol.config.yaml` `protocol.version`. Reflects upstream protocol conformity; do not bump for app releases.

When bumping the app version, update `Cargo.toml` first, then propagate to `protocol.config.yaml` `project.version`, `CHANGELOG.md`, and `README.md`. Leave `VERSION` unchanged unless the upstream protocol version itself is changing.

**Version history note:** This project was forked from Savant's internal ECHO
Protocol (formerly v4.0.0). The version was reset to v0.0.1 for the public
release to reflect independent versioning.

---

## v0.0.2 — 2026-06-16 (BUGFIX: default LLM model corrected)

**SAVANT-BOT v0.0.2** — patch release. Ships the openrouter/auto → openrouter/free default-model fix from v0.0.1's corrected interpretation of the operator's original URL `https://openrouter.ai/openrouter/free` (where `openrouter/free` is the literal model slug, not a navigation hint). No code or API changes — only default value, documentation, and version bumps. Wire-compatible with v0.0.1.

### Fixed

- **Default LLM model** (`src/config.rs`): `llm_default_model` default corrected from `"openrouter/auto"` to `"openrouter/free"`. The earlier default was a wrong-interpretation bug from the prior session (commit `b5882d0`). Users who set `LLM_DEFAULT_MODEL` explicitly are unaffected; users relying on the default now get OpenRouter's free-model routing slug.
- **README.md intro line:** Updated to reference `openrouter/free` (was `openrouter/auto`). Was previously inconsistent with the corrected default.

### Audit

- **Verified unchanged:** all `src/*.rs` source files (zero API changes), `.env.example` template (default lives in `src/config.rs`), migrations (zero schema changes), ECHO Protocol conformity (still v0.1.3; this release bumps app version only).
- **Validated:** all 6 ECHO validation commands PASS; 25/25 unit tests pass (no test changes; the test fixture in `src/llm/provider.rs` was updated at commit `b5882d0` to match the corrected default).
- **FID-151 N/A:** no new `pub fn` introduced this release. The release commit touches only string version fields; call-graph reachability from FIDs 003-010 still applies unchanged.
- **Spec consistency:** `Cargo.toml` v0.0.2 ↔ `protocol.config.yaml` `project.version` v0.0.2 ↔ `CHANGELOG.md` (this entry) ↔ `README.md` title v0.0.2. Protocol version unchanged: `VERSION` file = `protocol.config.yaml` `protocol.version` = "0.1.3".

---

## v0.0.1 — 2026-06-16 (INITIAL PUBLIC RELEASE)

**SAVANT-BOT v0.0.1** — first public release. Discord bot, Rust-native, built under [SAVANT-PROTOCOL](https://github.com/fame0528/savant-protocol) (ECHO) v0.1.3. Sister project to [SAVANT-TRADING](https://github.com/fame0528/savant-trading) under the [SAVANT](https://github.com/fame0528/Savant) umbrella.

### Highlights

- **Rust-native Discord bot** built on Poise 0.6 + Serenity 0.12 + Tokio
- **3 commands shipped:** `/ping` (smoke test), `/ask` (LLM chat with full infrastructure chain), `/mute` (moderation case recording, requires `MODERATE_MEMBERS`)
- **LLM features** (OpenRouter-backed): defer-then-edit (3s → 15min deadline), token-bucket rate limiting (governor GCRA, 5 req / 0.5s default), exponential backoff on HTTP 429 (1s → 60s with ±20% jitter), sliding-window conversation context (20 messages / channel / 1h TTL)
- **Moderation features:** SQLite-backed `moderation_cases` with WAL journaling, 60-second background poller for restart-survival
- **Engineering discipline:** SAVANT-PROTOCOL v0.1.3 active with all 15 laws under `strict_mode: true`; zero-warning clippy build; FID lifecycle (10 archived, 0 active); session summaries; cross-session `LEARNINGS.md`
- **25 unit tests passing;** all 6 validation commands PASS
- **~1,800 lines of Rust** across 18 files in `src/` + 1 SQL migration

### Implementation (10 FIDs)

| FID | Title | Severity | Tests |
|-----|-------|----------|-------|
| `FID-2026-0616-010` | Cargo baseline + Poise skeleton (foundation) | high | 2 |
| `FID-2026-0616-008` | Provider trait + OpenRouter + MockProvider | high | 8 |
| `FID-2026-0616-005` | Defer-then-edit helper | high | 0* |
| `FID-2026-0616-006` | Rate limiter + exponential backoff | high | 6 |
| `FID-2026-0616-004` | SQLite poller + `/mute` command | medium | 2 |
| `FID-2026-0616-007` | Sliding-window LLM context | medium | 6 |
| `FID-2026-0616-003` | `src/commands/` layout | low | 0 |
| `FID-2026-0616-002` | AstrBot prompt-injection contained | medium | 0 |
| `FID-2026-0616-001` | Config optimization to canonical ECHO values | medium | 0 |
| (sample) | `/ask` command (wires full LLM chain) | high | 0 |

\* FID-005's helper is exercised end-to-end by `/ask`. Full FID bodies in `dev/fids/archive/`.

### Release Prep (2026-06-16)

- [LOW] **README rewritten with SAVANT-BOT branding** per operator's clarification. **SAVANT** = brand + core project. **SAVANT-PROTOCOL** = rule set (ECHO). **SAVANT-BOT** = this Discord bot. **SAVANT-TRADING** = sister project. Mirrors SAVANT and SAVANT-TRADING README style (black/cyan badges, all-caps project name, `**Savant** &bull; 2026` closing).
- [LOW] **Custom banner** at `img/banner.png` (0.23 MB) per operator.
- [LOW] **SAVANT logo** at `img/savant.png` (3.30 MB, downloaded from `fame0528/Savant`).
- [LOW] **Git repo initialized** with main branch. Initial commit `d965766` contains 63 files.
- [LOW] **.env.example expanded** with `OPENROUTER_API_KEY` and `LLM_DEFAULT_MODEL`.
- [LOW] **`.gitignore` updated** for ECHO runtime artifacts, secrets, and research working folder.
- [SECURITY] **Prompt-injection contained** (FID-002): AstrBot's `AGENTS.md` was detected when reading `research/AstrBot/AGENTS.md` and treated as third-party reference material, not operator commands.

### Configuration (verified at release)

| Field | Value | Source |
|-------|-------|--------|
| `project.version` | `0.0.1` | `Cargo.toml`, `protocol.config.yaml` |
| `protocol.version` | `0.1.3` | matches `VERSION` file |
| `language` | `rust` | config |
| `quality.max_file_lines` | `300` | `coding-standards/rust.md` |
| `quality.max_function_lines` | `50` | `coding-standards/rust.md` |
| `quality.max_line_length` | `100` | `coding-standards/rust.md` |
| `perfection_loop.max_iterations` | `10` | ECHO.md Rule #5 |
| `perfection_loop.change_threshold` | `0.10` | ECHO.md Rule #1 |
| `perfection_loop.convergence_threshold` | `0.02` | ECHO.md Rule #3 |
| `session.autonomy_level` | `3` (Autonomous) | ECHO default |

### Known Limitations (v0.0.1 → v1.0)

- `/mute` records the case but does NOT call the Discord API to actually apply the mute (schema has `role_id` ready for v2)
- LLM context is in-process only; lost on bot restart (SQLite persistence is v1.1)
- Process-wide rate limiter (per-user keyed limiting is v1.1)
- No streaming SSE for long LLM responses (v2)
- Future: integrate with [SAVANT](https://github.com/fame0528/Savant)'s memory substrate (CortexaDB), soul/personality system, A2A delegation — see README Roadmap

### Audit Results (release-time)

- All 6 validation commands PASS: `cargo build`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test` (25/25), `cargo clean`
- Zero `unwrap()` / `expect()` in non-test code
- Zero `TODO` / `FIXME` / `todo!` / `unimplemented!` in `src/`
- Zero secrets in tracked files (`.env` properly gitignored)
- All `pub fn` wired to consumers (FID-151 call-graph reachability verified)

---

### Inherited ECHO Protocol History

The following entries document the [SAVANT-PROTOCOL](https://github.com/fame0528/savant-protocol) v0.1.x / v0.0.x releases that this project conforms to. Protocol-specific changes live in the upstream repo.

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
