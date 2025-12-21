-- backend/migrations/002_conversations_messages.sql
-- Migration pour ajouter les tables de conversations, messages et membres

-- Table des conversations (1:1 et groupes)
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    name TEXT,                    -- NULL pour conversations 1:1, nom pour groupes
    is_group BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_message_at INTEGER,
    last_message_preview TEXT,
    unread_count INTEGER DEFAULT 0
);

-- Table des membres de conversations
CREATE TABLE IF NOT EXISTS conversation_members (
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    joined_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY(conversation_id, user_id)
);

-- Table des messages (texte, GIFs, audio, vidéo)
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT,                -- Contenu chiffré (texte ou metadata)
    encrypted_keys TEXT,         -- JSON: { "user_id": "base64_chiffré", ... }
    nonce TEXT,                  -- Base64 encoded nonce
    media_type TEXT,             -- NULL, 'gif', 'audio', 'video'
    media_url TEXT,              -- Chemin vers le fichier uploadé
    duration INTEGER,            -- Durée en secondes pour audio/vidéo
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY(sender_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Table pour les réactions aux messages
CREATE TABLE IF NOT EXISTS message_reactions (
    message_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    emoji TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY(message_id, user_id, emoji)
);

-- Index pour performance
CREATE INDEX IF NOT EXISTS idx_conversations_last_message ON conversations(last_message_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_conversation_members_user ON conversation_members(user_id);

-- Trigger pour mettre à jour last_message_at automatiquement
CREATE TRIGGER IF NOT EXISTS update_conversation_last_message
AFTER INSERT ON messages
BEGIN
    UPDATE conversations
    SET 
        last_message_at = NEW.timestamp,
        last_message_preview = 
            CASE 
                WHEN NEW.media_type IS NOT NULL THEN '[' || NEW.media_type || ' message]'
                ELSE SUBSTR(NEW.content, 1, 50)
            END
    WHERE id = NEW.conversation_id;
END;

-- Trigger pour incrémenter unread_count
CREATE TRIGGER IF NOT EXISTS increment_unread_count
AFTER INSERT ON messages
BEGIN
    UPDATE conversations
    SET unread_count = unread_count + 1
    WHERE id = NEW.conversation_id
    AND EXISTS (
        SELECT 1 FROM conversation_members cm
        WHERE cm.conversation_id = NEW.conversation_id
        AND cm.user_id != NEW.sender_id
    );
END;

-- Insertion d'une conversation par défaut pour les utilisateurs existants
-- (Facultatif - peut être géré par l'application)
INSERT OR IGNORE INTO conversations (id, name, is_group, created_at)
VALUES ('default_global', 'Groupe Global', 1, strftime('%s', 'now'));

-- Ajout de tous les utilisateurs approuvés à la conversation globale
INSERT OR IGNORE INTO conversation_members (conversation_id, user_id)
SELECT 'default_global', id 
FROM users 
WHERE approved = 1;