-- Migration 023: Add group_key_version to messages table
-- Required by E2EE-IMPL (79edfc94) for Hybrid Sender Keys (Option D)
-- Fixes: [get_conversation_messages] Erreur DB: no column found for name: group_key_version

ALTER TABLE messages ADD COLUMN group_key_version INTEGER;
