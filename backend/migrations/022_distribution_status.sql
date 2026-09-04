-- Migration 022: Add distribution_status to conversation_key_recipients
-- Tracks whether a distributed group key has been claimed by the recipient
ALTER TABLE conversation_key_recipients
    ADD COLUMN distribution_status TEXT NOT NULL DEFAULT 'delivered';
