-- Group conversation keys (versioned per rotation)
CREATE TABLE conversation_keys (
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    version         INTEGER NOT NULL,
    creator_id      TEXT NOT NULL REFERENCES users(id),
    created_at      INTEGER NOT NULL,
    PRIMARY KEY (conversation_id, version)
);

-- Per-recipient sealed group keys
CREATE TABLE conversation_key_recipients (
    conversation_id     TEXT NOT NULL,
    version             INTEGER NOT NULL,
    user_id             TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    encrypted_key       TEXT NOT NULL,           -- base64 sealed box
    user_key_version    INTEGER NOT NULL,        -- which of user's keys was used
    FOREIGN KEY (conversation_id, version) REFERENCES conversation_keys(conversation_id, version) ON DELETE CASCADE,
    PRIMARY KEY (conversation_id, version, user_id)
);