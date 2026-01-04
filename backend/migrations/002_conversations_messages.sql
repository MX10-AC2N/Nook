-- migrations/002_conversations_messages.sql
-- Tables pour conversations et messages (renommé messages pour cohérence main.rs/db.rs)

CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS conversation_participants (
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    joined_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (conversation_id, user_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    content TEXT,
    message_type TEXT DEFAULT 'text',
    file_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    edited_at INTEGER,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES uploads(id)
);

-- Index et triggers simplifiés (cohérents avec db.rs)
CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, created_at DESC);

CREATE TRIGGER IF NOT EXISTS update_conversation_updated_at
AFTER INSERT ON messages
BEGIN
    UPDATE conversations
    SET updated_at = NEW.created_at
    WHERE id = NEW.conversation_id;
END;