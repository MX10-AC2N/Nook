-- migrations/001_initial.sql
-- Migration initiale : tables de base (invites, users, uploads)

CREATE TABLE IF NOT EXISTS invites (
    token TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,  -- hashed password
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    approved BOOLEAN NOT NULL DEFAULT 0,
    needs_password_change BOOLEAN NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    token TEXT,  -- pour session/auth token
    public_key TEXT,
    joined_at INTEGER DEFAULT (strftime('%s', 'now'))
    -- Ajoute d'autres colonnes si besoin (ex : avatar TEXT)
);

CREATE TABLE IF NOT EXISTS uploads (
    id TEXT PRIMARY KEY,
    file_name TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    path TEXT NOT NULL,
    timestamp INTEGER NOT NULL
);