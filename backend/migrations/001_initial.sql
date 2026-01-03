-- migrations/001_initial.sql
-- Tables de base : invites, users, uploads

CREATE TABLE IF NOT EXISTS invites (
    token TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    approved BOOLEAN NOT NULL DEFAULT 0,
    needs_password_change BOOLEAN NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    token TEXT,
    public_key TEXT,
    joined_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Table uploads avec chiffrement (champs ajoutés)
CREATE TABLE IF NOT EXISTS uploads (
    id TEXT PRIMARY KEY,
    file_name TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    path TEXT NOT NULL,
    sender_id TEXT DEFAULT 'anonymous',
    timestamp INTEGER NOT NULL,
    encrypted BOOLEAN NOT NULL DEFAULT 0,
    nonce TEXT,
    key_text TEXT
);

-- Index pour les uploads
CREATE INDEX IF NOT EXISTS idx_uploads_timestamp ON uploads(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_uploads_sender ON uploads(sender_id);
