-- Migration 024: Add reply_to_id to messages table (ADR-017 reply/quote feature)
-- Permet de référencer un message cité (Option B simple, style Signal/WhatsApp)
-- Note: SQLite ne supporte pas ALTER TABLE ADD CONSTRAINT FOREIGN KEY.
-- La validité de reply_to_id (existence + même conversation) est vérifiée
-- au niveau applicatif dans send_message, et delete_message fait un SET NULL
-- simulé (ON DELETE SET NULL) sur les messages qui citent un message supprimé.

ALTER TABLE messages ADD COLUMN reply_to_id TEXT;

CREATE INDEX IF NOT EXISTS idx_messages_reply_to ON messages(reply_to_id);
