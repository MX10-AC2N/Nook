-- migrations/001_initial.sql

CREATE TABLE invites (
    token TEXT PRIMARY KEY
);

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    approved BOOLEAN NOT NULL DEFAULT 0,
    needs_password_change BOOLEAN NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    public_key TEXT,
    joined_at INTEGER DEFAULT (strftime('%s', 'now'))
    -- Ajoute d'autres colonnes si tu en as (avatar, token, etc.)
);