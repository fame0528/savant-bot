//! In-process sliding-window context for LLM conversation memory.
//!
//! Per doc §3.3, LLM calls include recent channel history so the model
//! can maintain coherent conversation. This module implements a bounded
//! in-process sliding window with a TTL (time-to-live).
//!
//! **v1 limitation** (per FID-2026-0616-007): context is in-process only,
//! lost on bot restart. Documented in the FID. Future migration to
//! SQLite is straightforward — replace the internal `Mutex<HashMap>`
//! with `sqlx::query` calls.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use poise::serenity_prelude::ChannelId;

use crate::llm::provider::ChatMessage;

/// A single message in the conversation context, with metadata for
/// formatting and TTL bookkeeping.
#[derive(Debug, Clone)]
pub struct ContextMessage {
    pub user_id: u64,
    pub display_name: String,
    pub content: String,
    pub timestamp: Instant,
}

impl ContextMessage {
    /// Convert to a `ChatMessage` for inclusion in an LLM request.
    /// The display name is prefixed so the model can attribute quotes
    /// correctly when multiple users are in the context.
    pub fn to_chat_message(&self) -> ChatMessage {
        ChatMessage::user(format!("{}: {}", self.display_name, self.content))
    }
}

/// Per-channel context state. `last_active` is updated on push and read
/// (TTL enforcement); channels with no activity for `ttl` are eligible
/// for cleanup.
#[derive(Debug)]
struct ChannelContext {
    messages: VecDeque<ContextMessage>,
    last_active: Instant,
}

/// Thread-safe, in-process sliding-window context store, keyed by
/// channel ID. Cheap to clone (the inner state is wrapped in `Arc`).
#[derive(Clone, Debug)]
pub struct ContextStore {
    inner: Arc<Mutex<HashMap<ChannelId, ChannelContext>>>,
    max_messages: usize,
    ttl: Duration,
}

impl ContextStore {
    /// Build a new context store.
    ///
    /// `max_messages` is the per-channel cap. `ttl` is the idle time
    /// after which a channel's context is considered stale and reset
    /// on next read.
    pub fn new(max_messages: usize, ttl: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            max_messages,
            ttl,
        }
    }

    /// Push a message into the channel's context window. Trims the
    /// oldest messages if the window exceeds `max_messages`.
    pub fn push(&self, channel: ChannelId, msg: ContextMessage) {
        let mut map = self.inner.lock().expect("ContextStore mutex poisoned");
        let entry = map.entry(channel).or_insert_with(|| ChannelContext {
            messages: VecDeque::with_capacity(self.max_messages),
            last_active: Instant::now(),
        });
        entry.messages.push_back(msg);
        entry.last_active = Instant::now();
        while entry.messages.len() > self.max_messages {
            entry.messages.pop_front();
        }
    }

    /// Get the context for a channel, with TTL enforcement. Returns
    /// an empty `Vec` if the channel has no context, or if the context
    /// has expired (in which case the entry is also cleared).
    ///
    /// Reading also refreshes the `last_active` timestamp so an
    /// active conversation doesn't get its context cleared mid-stream.
    pub fn get(&self, channel: ChannelId) -> Vec<ContextMessage> {
        let mut map = self.inner.lock().expect("ContextStore mutex poisoned");
        match map.get_mut(&channel) {
            None => Vec::new(),
            Some(entry) if entry.last_active.elapsed() > self.ttl => {
                entry.messages.clear();
                Vec::new()
            }
            Some(entry) => {
                entry.last_active = Instant::now();
                entry.messages.iter().cloned().collect()
            }
        }
    }

    /// Remove all expired entries. Returns the number of entries removed.
    /// Called by a background cleanup task (every 5 minutes) to keep
    /// the in-memory map from growing unboundedly.
    pub fn cleanup_expired(&self) -> usize {
        let mut map = self.inner.lock().expect("ContextStore mutex poisoned");
        let before = map.len();
        map.retain(|_, entry| entry.last_active.elapsed() <= self.ttl);
        before - map.len()
    }

    /// Number of channels currently in the context store (for tests
    /// and observability).
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .expect("ContextStore mutex poisoned")
            .len()
    }

    /// `is_empty` companion required by clippy's `len_without_is_empty`
    /// lint when `len` is public. Delegates to `len`.
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(user: &str, content: &str) -> ContextMessage {
        ContextMessage {
            user_id: 0,
            display_name: user.to_string(),
            content: content.to_string(),
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn push_and_get_returns_messages() {
        let store = ContextStore::new(5, Duration::from_secs(60));
        let channel = ChannelId::new(123);
        store.push(channel, msg("alice", "hi"));
        let msgs = store.get(channel);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "hi");
    }

    #[test]
    fn max_messages_trims_oldest() {
        let store = ContextStore::new(2, Duration::from_secs(60));
        let channel = ChannelId::new(1);
        for i in 0..5 {
            store.push(channel, msg("u", &format!("m{}", i)));
        }
        let msgs = store.get(channel);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "m3");
        assert_eq!(msgs[1].content, "m4");
    }

    #[test]
    fn expired_context_returns_empty_and_clears() {
        let store = ContextStore::new(10, Duration::from_millis(1));
        let channel = ChannelId::new(1);
        store.push(channel, msg("alice", "hi"));
        std::thread::sleep(Duration::from_millis(10));
        let msgs = store.get(channel);
        assert!(msgs.is_empty());
        // After expiry, the entry is cleared; pushing again starts fresh
        store.push(channel, msg("bob", "new"));
        let msgs = store.get(channel);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "new");
    }

    #[test]
    fn cleanup_expired_removes_stale() {
        let store = ContextStore::new(10, Duration::from_millis(1));
        let ch1 = ChannelId::new(1);
        let ch2 = ChannelId::new(2);
        store.push(ch1, msg("a", "x"));
        store.push(ch2, msg("b", "y"));
        assert_eq!(store.len(), 2);
        std::thread::sleep(Duration::from_millis(10));
        let removed = store.cleanup_expired();
        assert_eq!(removed, 2);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn different_channels_have_separate_contexts() {
        let store = ContextStore::new(10, Duration::from_secs(60));
        let ch1 = ChannelId::new(1);
        let ch2 = ChannelId::new(2);
        store.push(ch1, msg("a", "in ch1"));
        let msgs1 = store.get(ch1);
        let msgs2 = store.get(ch2);
        assert_eq!(msgs1.len(), 1);
        assert_eq!(msgs2.len(), 0);
    }

    #[test]
    fn to_chat_message_includes_display_name() {
        let m = msg("Alice", "hello");
        let chat = m.to_chat_message();
        assert_eq!(chat.role, "user");
        assert_eq!(chat.content, "Alice: hello");
    }
}
