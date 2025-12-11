-- migrations/001_initial.sql
CREATE TABLE invites (
    token TEXT PRIMARY KEY
);

CREATE TABLE members (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    public_key TEXT NOT NULL,
    approved BOOLEAN NOT NULL DEFAULT 0,
    joined_at DATETIME DEFAULT CURRENT_TIMESTAMP
);