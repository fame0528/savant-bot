-- savant-bot schema v1 (2026-06-16)
-- Per FID-2026-0616-004: temp-punishment persistence for restart-survival.

CREATE TABLE IF NOT EXISTS moderation_cases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id BIGINT NOT NULL,
    target_id BIGINT NOT NULL,
    moderator_id BIGINT NOT NULL,
    action_type TEXT NOT NULL CHECK (action_type IN ('MUTE', 'TIMEOUT', 'BAN', 'KICK', 'WARN')),
    role_id BIGINT,
    duration_seconds INTEGER,
    reason TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'resolved', 'cancelled'))
);

-- Composite index for the poller's hot query: "active cases that have expired".
CREATE INDEX IF NOT EXISTS idx_moderation_cases_active_expires
ON moderation_cases (status, expires_at)
WHERE status = 'active';
