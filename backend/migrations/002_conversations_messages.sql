-- migrations/002_conversations_messages.sql
-- Tables pour conversations, participants et messages
-- Cohérent avec main.rs (init_db) et db.rs

CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    name TEXT,
    is_group BOOLEAN DEFAULT 0,
    created_at INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

-- Index pour tri des conversations (par updated_at)
CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at DESC);

CREATE TABLE IF NOT EXISTS conversation_participants (
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (conversation_id, user_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT,
    message_type TEXT DEFAULT 'text',
    file_id TEXT,
    encrypted BOOLEAN DEFAULT 0,
    timestamp INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    edited_at INTEGER,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES uploads(id) ON DELETE SET NULL
);

-- Index pour performances (récupération messages par conversation, ordre chrono)
CREATE INDEX IF NOT EXISTS idx_messages_conversation_timestamp ON messages(conversation_id, timestamp DESC);

-- Trigger : met à jour updated_at de la conversation sur nouveau message
CREATE TRIGGER IF NOT EXISTS trigger_update_conversation_updated_at
AFTER INSERT ON messages
FOR EACH ROW
BEGIN
    UPDATE conversations
    SET updated_at = NEW.timestamp
    WHERE id = NEW.conversation_id;
END;

-- Trigger optionnel : met à jour updated_at sur edit message (si tu implémentes edit)
CREATE TRIGGER IF NOT EXISTS trigger_update_conversation_on_edit
AFTER UPDATE OF content, edited_at ON messages
FOR EACH ROW
WHEN NEW.edited_at IS NOT NULL
BEGIN
    UPDATE conversations
    SET updated_at = NEW.edited_at
    WHERE id = NEW.conversation_id;
END;