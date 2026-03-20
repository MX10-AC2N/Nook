-- backend/migrations/005_reactions.sql
-- Réactions aux messages — Session 35
-- Contrainte : 1 réaction par user par message (modifiable via UPSERT sur l'emoji)

CREATE TABLE IF NOT EXISTS message_reactions (
    message_id  TEXT    NOT NULL,
    user_id     TEXT    NOT NULL,
    emoji       TEXT    NOT NULL,   -- ex: "👍", "❤️", "😂", "😮", "😢", "😡"
    created_at  INTEGER NOT NULL,
    PRIMARY KEY (message_id, user_id),
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id)    REFERENCES users(id)    ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_reactions_message ON message_reactions(message_id);
