-- Migration 016: Create or replace events table with correct schema
-- DROP TABLE first to ensure we have the correct schema (fixes old/broken tables)
DROP TABLE IF EXISTS events;

CREATE TABLE events (
    id TEXT PRIMARY KEY,
    creator_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    start_time INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    end_time INTEGER NOT NULL DEFAULT (strftime('%s', 'now') + 3600),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes (idempotent)
CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time);
CREATE INDEX IF NOT EXISTS idx_events_end_time ON events(end_time);
CREATE INDEX IF NOT EXISTS idx_events_creator_id ON events(creator_id);
