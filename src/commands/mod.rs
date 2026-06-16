//! Command modules.
//!
//! Per FID-2026-0616-003, the layout is `src/commands/<feature>.rs` with
//! one file per feature. Each file contains one or more `#[poise::command]`
//! functions. Features that grow large (e.g., moderation with cases,
//! temp actions, audit log) graduate to `src/commands/<feature>/mod.rs`
//! + submodules.
//!
//! The `all()` function returns the complete list of commands for Poise
//! framework registration. New commands are added by:
//! 1. Creating `src/commands/<feature>.rs` with a `pub fn <command>()` that
//!    returns the `#[poise::command]`-annotated function
//! 2. Adding `pub mod <feature>;` to this file
//! 3. Adding `<feature>::<command>()` to the `all()` return vector

pub mod ask;
pub mod mute;
pub mod ping;

/// All commands registered with the Poise framework.
///
/// Add new commands here as `vec![ping::ping(), ask::ask(), mute::mute(), ...]`.
/// This list is the single source of truth for command registration —
/// adding a command to a `#[poise::command]` attribute alone is not enough
/// (Poise requires explicit registration).
pub fn all() -> Vec<poise::Command<crate::Data, crate::error::BotError>> {
    vec![ping::ping(), ask::ask(), mute::mute()]
}
