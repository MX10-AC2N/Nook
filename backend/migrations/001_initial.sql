-- migrations/001_initial.sql
-- Migration initiale cohérente avec :
-- - Le code de main.rs (init_db() : tables users, conversations, conversation_members, messages, invites)
-- - upload.rs (INSERT INTO uploads avec colonnes précises pour fichiers chiffrés)
-- Tous les timestamps sont en INTEGER (Unix timestamp), comme dans ton code chrono::Utc::now().timestamp()

-- Table users (exactement comme dans main.rs)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    approved INTEGER NOT NULL DEFAULT 0,
    needs_password_change INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

-- Indexes pour performances
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Table conversations (exactement comme dans main.rs)
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Table conversation_members (nom exact de main.rs, pas conversation_participants)
CREATE TABLE IF NOT EXISTS conversation_members (
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (conversation_id, user_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Table messages (exactement comme dans main.rs)
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Index pour récupération des messages par conversation (ordre chrono)
CREATE INDEX IF NOT EXISTS idx_messages_conversation_created_at ON messages(conversation_id, created_at DESC);

-- Table invites (exactement comme dans main.rs)
CREATE TABLE IF NOT EXISTS invites (
    code TEXT PRIMARY KEY,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER,
    max_uses INTEGER,
    current_uses INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Table uploads (exactement les colonnes utilisées dans upload.rs)
CREATE TABLE IF NOT EXISTS uploads (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    from_user_id TEXT,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    content_type TEXT,
    uploaded_at INTEGER NOT NULL,
    encrypted INTEGER DEFAULT 0,  -- BOOLEAN en SQLite = INTEGER
    nonce TEXT,
    key_text TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (from_user_id) REFERENCES users(id)
);

-- Indexes utiles pour uploads (performances + pruning futur)
CREATE INDEX IF NOT EXISTS idx_uploads_uploaded_at ON uploads(uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_uploads_conversation ON uploads(conversation_id);