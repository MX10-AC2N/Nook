-- migrations/015_missed_calls.sql
-- Enregistre les appels manqués (déclinés ou non décrochés)

CREATE TABLE IF NOT EXISTS missed_calls (
    id              TEXT    PRIMARY KEY,
    conversation_id TEXT    NOT NULL,
    caller_id       TEXT    NOT NULL,
    callee_id       TEXT    NOT NULL,
    call_type       TEXT    NOT NULL DEFAULT 'audio',  -- 'audio' | 'video'
    status          TEXT    NOT NULL DEFAULT 'missed',  -- 'missed' | 'declined'
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (caller_id)       REFERENCES users(id),
    FOREIGN KEY (callee_id)       REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_missed_calls_conversation ON missed_calls(conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_missed_calls_callee ON missed_calls(callee_id, created_at DESC);
