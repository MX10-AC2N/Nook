-- migrations/001_initial.sql
-- Tables de base : invites, users, uploads (cohérent avec main.rs, db.rs, auth.rs)

CREATE TABLE IF NOT EXISTS invites (
    token TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    used BOOLEAN DEFAULT 0
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT,
    role TEXT DEFAULT 'user',
    approved BOOLEAN DEFAULT 0,
    needs_password_change BOOLEAN DEFAULT 0,
    token TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS uploads (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    from_user_id TEXT,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    content_type TEXT,
    uploaded_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    encrypted BOOLEAN DEFAULT 0,
    nonce TEXT,
    key_text TEXT,
    FOREIGN KEY(conversation_id) REFERENCES conversations(id),
    FOREIGN KEY(from_user_id) REFERENCES users(id)
);

-- Index pour performances
CREATE INDEX IF NOT EXISTS idx_uploads_uploaded_at ON uploads(uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_uploads_conversation ON uploads(conversation_id);