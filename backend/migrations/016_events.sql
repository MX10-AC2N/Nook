-- Migration 016: Create events table
-- This migration is a no-op because fix_events_schema() in Rust
-- already handled the schema migration before this runs.
-- We keep this migration to record that we intended to create the events table.

-- Ensure indexes exist (idempotent)
CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time);
CREATE INDEX IF NOT EXISTS idx_events_end_time ON events(end_time);
CREATE INDEX IF NOT EXISTS idx_events_creator_id ON events(creator_id);
