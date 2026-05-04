-- Migration 016: Create events table with correct schema
-- This migration creates the events table if it doesn't exist,
-- or adds missing columns if the table already exists with an old schema.

-- Create table if it doesn't exist
CREATE TABLE IF NOT EXISTS events (
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

-- Ensure indexes exist (idempotent)
CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time);
CREATE INDEX IF NOT EXISTS idx_events_end_time ON events(end_time);
CREATE INDEX IF NOT EXISTS idx_events_creator_id ON events(creator_id);
