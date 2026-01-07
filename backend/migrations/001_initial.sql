-- migrations/001_initial.sql
-- Migration initiale complète et cohérente
-- Tables : users (avec public_key), invites (single-use, 48h expiration, created_by), uploads, etc.
-- Utilise INTEGER pour timestamps Unix (cohérent avec le code Rust : chrono::Utc::now().timestamp())

-- Table users : ajout public_key pour E2EE
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
    created_at INTEGER NOT NULL,
    public_key TEXT  -- Clé publique pour chiffrement E2EE
);

-- Index pour performances
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Table invites : single-use, expiration 48h, created_by (admin unique)
CREATE TABLE IF NOT EXISTS invites (
    id TEXT PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,
    created_by TEXT NOT NULL,  -- ID de l'admin qui a créé l'invite
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    used BOOLEAN DEFAULT 0,
    used_by TEXT,              -- ID de l'utilisateur qui a utilisé l'invite
    used_at INTEGER,
    FOREIGN KEY(created_by) REFERENCES users(id),
    FOREIGN KEY(used_by) REFERENCES users(id)
);

-- Index pour recherche par token
CREATE INDEX IF NOT EXISTS idx_invites_token ON invites(token);

-- Table uploads : inchangée mais cohérente
CREATE TABLE IF NOT EXISTS uploads (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,
    from_user_id TEXT,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    content_type TEXT,
    uploaded_at INTEGER NOT NULL,
    encrypted BOOLEAN DEFAULT 0,
    nonce TEXT,
    key_text TEXT,
    FOREIGN KEY(conversation_id) REFERENCES conversations(id),
    FOREIGN KEY(from_user_id) REFERENCES users(id)
);

-- Index pour performances (uploads)
CREATE INDEX IF NOT EXISTS idx_uploads_uploaded_at ON uploads(uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_uploads_conversation ON uploads(conversation_id);