-- migrations/004_polls.sql
-- Sondages familiaux — Session 27
-- Timestamps INTEGER (Unix epoch), cohérent avec 001_initial.sql

-- ════════════════════════════════════════════════════════════════
-- TABLE polls
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS polls (
    id         TEXT    PRIMARY KEY,
    question   TEXT    NOT NULL,
    created_by TEXT    NOT NULL,
    created_at INTEGER NOT NULL,
    closed_at  INTEGER,            -- NULL = ouvert
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_polls_created_at ON polls(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_polls_created_by ON polls(created_by);

-- ════════════════════════════════════════════════════════════════
-- TABLE poll_options
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS poll_options (
    id       TEXT    PRIMARY KEY,
    poll_id  TEXT    NOT NULL,
    text     TEXT    NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_poll_options_poll ON poll_options(poll_id, position);

-- ════════════════════════════════════════════════════════════════
-- TABLE poll_votes
-- 1 vote par utilisateur par sondage, modifiable via UPSERT
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS poll_votes (
    poll_id   TEXT    NOT NULL,
    user_id   TEXT    NOT NULL,
    option_id TEXT    NOT NULL,
    voted_at  INTEGER NOT NULL,
    PRIMARY KEY (poll_id, user_id),
    FOREIGN KEY (poll_id)   REFERENCES polls(id)        ON DELETE CASCADE,
    FOREIGN KEY (user_id)   REFERENCES users(id)        ON DELETE CASCADE,
    FOREIGN KEY (option_id) REFERENCES poll_options(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_poll_votes_poll   ON poll_votes(poll_id);
CREATE INDEX IF NOT EXISTS idx_poll_votes_option ON poll_votes(option_id);
