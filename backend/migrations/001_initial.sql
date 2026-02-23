-- migrations/001_initial.sql
-- Source de vérité : alignée sur db.rs, auth.rs, upload.rs, main.rs
-- Tous les timestamps : INTEGER (Unix timestamp via chrono::Utc::now().timestamp())

-- ================================================================
-- TABLE users
-- Colonnes vérifiées contre : struct User (db.rs) + auth.rs + main.rs
-- ================================================================
CREATE TABLE IF NOT EXISTS users (
    id                    TEXT    PRIMARY KEY,
    username              TEXT    UNIQUE NOT NULL,
    email                 TEXT    UNIQUE NOT NULL,
    password_hash         TEXT    NOT NULL,
    name                  TEXT,                           -- Option<String> dans le struct
    role                  TEXT    NOT NULL DEFAULT 'user',
    approved              INTEGER NOT NULL DEFAULT 0,     -- bool mappé en INTEGER
    needs_password_change INTEGER NOT NULL DEFAULT 0,     -- bool mappé en INTEGER
    token                 TEXT,                           -- session token (NULL = déconnecté)
    created_at            INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email    ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_token    ON users(token); -- require_auth : WHERE token = ?

-- ================================================================
-- TABLE conversations
-- Colonnes vérifiées contre : struct Conversation (db.rs) + create_conversation
-- ================================================================
CREATE TABLE IF NOT EXISTS conversations (
    id         TEXT    PRIMARY KEY,
    name       TEXT,                        -- Option<String> (conversations sans nom explicite)
    is_group   INTEGER NOT NULL DEFAULT 1,  -- bool : 1 = groupe, 0 = DM
    created_by TEXT    NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,            -- mis à jour à chaque nouveau message
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at DESC);

-- ================================================================
-- TABLE conversation_participants
-- Nom exact utilisé dans db.rs (pas conversation_members)
-- ================================================================
CREATE TABLE IF NOT EXISTS conversation_participants (
    conversation_id TEXT    NOT NULL,
    user_id         TEXT    NOT NULL,
    joined_at       INTEGER NOT NULL,
    PRIMARY KEY (conversation_id, user_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id)         REFERENCES users(id)
);

-- ================================================================
-- TABLE messages
-- Colonnes vérifiées contre : struct Message (db.rs) + send_message + get_messages
-- ================================================================
CREATE TABLE IF NOT EXISTS messages (
    id              TEXT    PRIMARY KEY,
    conversation_id TEXT    NOT NULL,
    sender_id       TEXT    NOT NULL,       -- struct Message.sender_id
    content         TEXT    NOT NULL,
    message_type    TEXT    NOT NULL DEFAULT 'text',  -- struct Message.message_type
    file_id         TEXT,                   -- Option<String> — lien vers uploads
    encrypted       INTEGER NOT NULL DEFAULT 0,       -- bool
    timestamp       INTEGER NOT NULL,       -- struct Message.timestamp
    created_at      INTEGER NOT NULL,
    edited_at       INTEGER,                -- Option<i64>
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (sender_id)       REFERENCES users(id),
    FOREIGN KEY (file_id)         REFERENCES uploads(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_file_id      ON messages(file_id);

-- ================================================================
-- TABLE uploads
-- Colonnes vérifiées contre : upload.rs INSERT + struct UploadResponse
-- ================================================================
CREATE TABLE IF NOT EXISTS uploads (
    id              TEXT    PRIMARY KEY,
    conversation_id TEXT,
    from_user_id    TEXT,
    file_name       TEXT    NOT NULL,
    file_path       TEXT    NOT NULL,
    file_size       INTEGER NOT NULL,
    content_type    TEXT,
    uploaded_at     INTEGER NOT NULL,
    encrypted       INTEGER          DEFAULT 0,
    nonce           TEXT,
    key_text        TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (from_user_id)    REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_uploads_uploaded_at  ON uploads(uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_uploads_conversation ON uploads(conversation_id);

-- ================================================================
-- TABLE invites
-- Colonnes vérifiées contre : invites.rs + admin.rs
-- ================================================================
CREATE TABLE IF NOT EXISTS invites (
    code          TEXT    PRIMARY KEY,
    created_by    TEXT    NOT NULL,
    created_at    INTEGER NOT NULL,
    expires_at    INTEGER,
    max_uses      INTEGER,
    current_uses  INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (created_by) REFERENCES users(id)
);