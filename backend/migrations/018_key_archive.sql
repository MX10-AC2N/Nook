-- User key version history
CREATE TABLE user_key_history (
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version         INTEGER NOT NULL,
    public_key      TEXT NOT NULL,           -- base64 X25519 (32 bytes)
    encrypted_priv  TEXT NOT NULL,           -- base64(salt[16] || nonce[24] || ciphertext)
    created_at      INTEGER NOT NULL,
    revoked_at      INTEGER,                 -- NULL = current, timestamp = archived
    PRIMARY KEY (user_id, version)
);
CREATE INDEX idx_user_key_history_user ON user_key_history(user_id);

-- Track which sender key version was used per message
ALTER TABLE message_keys ADD COLUMN sender_key_version INTEGER NOT NULL DEFAULT 1;
CREATE INDEX idx_message_keys_sender_version ON message_keys(sender_key_version);