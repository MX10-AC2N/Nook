-- migrations/002_conversations_messages.sql
-- Tables pour les conversations et messages chat

CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    name TEXT,
    is_group BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_message_at INTEGER,
    last_message_preview TEXT,
    unread_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS conversation_members (
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    joined_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY(conversation_id, user_id)
);

-- Changé de "messages" à "chat_messages" pour matcher le code upload.rs
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    content TEXT,
    message_type TEXT,
    timestamp INTEGER NOT NULL,
    file JSON,
    FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY(sender_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS message_reactions (
    message_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    emoji TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(message_id) REFERENCES chat_messages(id) ON DELETE CASCADE,  -- Changé ici aussi
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY(message_id, user_id, emoji)
);

-- Index et triggers mis à jour pour chat_messages
CREATE INDEX IF NOT EXISTS idx_conversations_last_message ON conversations(last_message_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_messages_conversation ON chat_messages(conversation_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_conversation_members_user ON conversation_members(user_id);

CREATE TRIGGER IF NOT EXISTS update_conversation_last_message
AFTER INSERT ON chat_messages
BEGIN
    UPDATE conversations
    SET 
        last_message_at = NEW.timestamp,
        last_message_preview = 
            CASE 
                WHEN NEW.message_type IS NOT NULL THEN '[' || NEW.message_type || ' message]'
                ELSE SUBSTR(NEW.content, 1, 50)
            END
    WHERE id = NEW.conversation_id;
END;

CREATE TRIGGER IF NOT EXISTS increment_unread_count
AFTER INSERT ON chat_messages
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

-- Conversation globale par défaut
INSERT OR IGNORE INTO conversations (id, name, is_group, created_at)
VALUES ('default_global', 'Groupe Global', 1, strftime('%s', 'now'));

INSERT OR IGNORE INTO conversation_members (conversation_id, user_id)
SELECT 'default_global', id 
FROM users 
WHERE approved = 1;