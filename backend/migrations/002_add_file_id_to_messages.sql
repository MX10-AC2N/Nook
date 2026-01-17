-- migrations/002_add_file_id_to_messages.sql
-- Migration pour ajouter la liaison entre messages et uploads (pièces jointes)
-- Ajoute une colonne file_id optionnelle dans messages
-- Safe : ALTER TABLE IF NOT EXISTS (SQLite supporte ADD COLUMN)

ALTER TABLE messages ADD COLUMN file_id TEXT
    REFERENCES uploads(id) ON DELETE SET NULL;

-- Index pour performances (recherche messages avec fichier + pruning orphelins)
CREATE INDEX IF NOT EXISTS idx_messages_file_id ON messages(file_id);

-- Optionnel : index composé pour récupération messages par conversation (si beaucoup de fichiers)
-- CREATE INDEX IF NOT EXISTS idx_messages_conversation_file ON messages(conversation_id, file_id);