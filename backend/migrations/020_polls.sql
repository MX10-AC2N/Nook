-- migrations/020_polls.sql
-- Sondages familiaux — Session C-03
-- Timestamps INTEGER (Unix epoch), cohérent avec 001_initial.sql
-- FIX: Handle existing polls table from deleted 004_polls.sql (missing conversation_id)
-- Strategy: Ensure base table exists, then ADD COLUMN conversation_id if missing.
-- Note: SQLite ALTER TABLE cannot add FK constraints. Column added nullable; FK enforced via trigger.

-- ════════════════════════════════════════════════════════════════
-- TABLE polls
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS polls (
    id              TEXT    PRIMARY KEY,
    question        TEXT    NOT NULL,
    created_by      TEXT    NOT NULL,
    created_at      INTEGER NOT NULL,
    closed_at       INTEGER,            -- NULL = ouvert
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

-- Add conversation_id column if missing (from deleted 004_polls.sql migration)
-- SQLite: ALTER TABLE ADD COLUMN does NOT support REFERENCES clause.
-- Column added nullable; FK enforced via trigger on INSERT/UPDATE.
-- Application layer enforces NOT NULL for new polls; legacy polls have NULL conversation_id.
ALTER TABLE polls ADD COLUMN conversation_id TEXT;

-- Create index for conversation_id
CREATE INDEX IF NOT EXISTS idx_polls_conversation ON polls(conversation_id);
CREATE INDEX IF NOT EXISTS idx_polls_created_by   ON polls(created_by);
CREATE INDEX IF NOT EXISTS idx_polls_created_at   ON polls(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_polls_closed_at    ON polls(closed_at);

-- Trigger to enforce conversation_id NOT NULL for new polls (application layer should enforce,
-- but trigger provides DB-level guard; legacy polls with NULL remain valid)
CREATE TRIGGER IF NOT EXISTS polls_require_conversation_id
BEFORE INSERT ON polls
FOR EACH ROW
WHEN NEW.conversation_id IS NULL
BEGIN
    SELECT RAISE(ABORT, 'conversation_id is required for new polls');
END;

-- Trigger to enforce FK to conversations(id) on INSERT/UPDATE
CREATE TRIGGER IF NOT EXISTS polls_fk_conversation_id
BEFORE INSERT ON polls
FOR EACH ROW
WHEN NEW.conversation_id IS NOT NULL AND 
     (SELECT COUNT(*) FROM conversations WHERE id = NEW.conversation_id) = 0
BEGIN
    SELECT RAISE(ABORT, 'conversation_id must reference existing conversations(id)');
END;

CREATE TRIGGER IF NOT EXISTS polls_fk_conversation_id_update
BEFORE UPDATE OF conversation_id ON polls
FOR EACH ROW
WHEN NEW.conversation_id IS NOT NULL AND 
     (SELECT COUNT(*) FROM conversations WHERE id = NEW.conversation_id) = 0
BEGIN
    SELECT RAISE(ABORT, 'conversation_id must reference existing conversations(id)');
END;

-- ════════════════════════════════════════════════════════════════
-- TABLE poll_options
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS poll_options (
    id        TEXT    PRIMARY KEY,
    poll_id   TEXT    NOT NULL,
    text      TEXT    NOT NULL,
    position  INTEGER NOT NULL DEFAULT 0,
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
CREATE INDEX IF NOT EXISTS idx_poll_votes_user   ON poll_votes(user_id);