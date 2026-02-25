-- migrations/001_initial.sql
-- Source de vérité unique — Session 9 (réécriture complète)
-- Alignée sur : db.rs, auth.rs, invites.rs, admin.rs, upload.rs, chess.rs
-- Tous les timestamps : INTEGER (Unix timestamp via chrono::Utc::now().timestamp())

-- ════════════════════════════════════════════════════════════════
-- TABLE users
-- Colonnes vérifiées contre : struct User (db.rs) + auth.rs + invites.rs
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS users (
    id                    TEXT    PRIMARY KEY,
    username              TEXT    UNIQUE NOT NULL,
    email                 TEXT    UNIQUE NOT NULL,
    password_hash         TEXT    NOT NULL,
    name                  TEXT,                           -- Option<String> dans le struct
    role                  TEXT    NOT NULL DEFAULT 'user',
    approved              INTEGER NOT NULL DEFAULT 0,     -- bool mappé en INTEGER
    needs_password_change INTEGER NOT NULL DEFAULT 0,     -- bool mappé en INTEGER
    token                 TEXT,                           -- session token (NULL = déconnecté)
    public_key            TEXT,                           -- clé publique E2EE (invites.rs::join)
    created_at            INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email    ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_token    ON users(token);  -- require_auth : WHERE token = ?

-- ════════════════════════════════════════════════════════════════
-- TABLE conversations
-- Colonnes vérifiées contre : struct Conversation (db.rs) + create_conversation
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS conversations (
    id         TEXT    PRIMARY KEY,
    name       TEXT,                        -- Option<String>
    is_group   INTEGER NOT NULL DEFAULT 1,  -- bool : 1 = groupe, 0 = DM
    created_by TEXT    NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,            -- mis à jour à chaque nouveau message
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at DESC);

-- ════════════════════════════════════════════════════════════════
-- TABLE conversation_participants
-- Nom exact utilisé dans db.rs et prune.rs
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS conversation_participants (
    conversation_id TEXT    NOT NULL,
    user_id         TEXT    NOT NULL,
    joined_at       INTEGER NOT NULL,
    PRIMARY KEY (conversation_id, user_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id)         REFERENCES users(id)
);

-- ════════════════════════════════════════════════════════════════
-- TABLE messages
-- Colonnes vérifiées contre : struct Message (db.rs) + send_message
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS messages (
    id              TEXT    PRIMARY KEY,
    conversation_id TEXT    NOT NULL,
    sender_id       TEXT    NOT NULL,
    content         TEXT    NOT NULL,
    message_type    TEXT    NOT NULL DEFAULT 'text',
    file_id         TEXT,                   -- Option<String> — lien vers uploads
    encrypted       INTEGER NOT NULL DEFAULT 0,
    timestamp       INTEGER NOT NULL,
    created_at      INTEGER NOT NULL,
    edited_at       INTEGER,                -- Option<i64>
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (sender_id)       REFERENCES users(id),
    FOREIGN KEY (file_id)         REFERENCES uploads(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_file_id      ON messages(file_id);

-- ════════════════════════════════════════════════════════════════
-- TABLE uploads
-- Colonnes vérifiées contre : upload.rs INSERT + struct Upload (db.rs)
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS uploads (
    id              TEXT    PRIMARY KEY,
    conversation_id TEXT,
    from_user_id    TEXT,
    file_name       TEXT    NOT NULL,
    file_path       TEXT    NOT NULL,
    file_size       INTEGER NOT NULL,
    content_type    TEXT,
    uploaded_at     INTEGER NOT NULL,
    encrypted       INTEGER          DEFAULT 0,
    nonce           TEXT,
    key_text        TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (from_user_id)    REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_uploads_uploaded_at  ON uploads(uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_uploads_conversation ON uploads(conversation_id);

-- ════════════════════════════════════════════════════════════════
-- TABLE invites
-- Colonnes vérifiées contre : invites.rs (validate, generate, join) + admin.rs
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS invites (
    id          TEXT    PRIMARY KEY,            -- UUID (generate_invite)
    token       TEXT    UNIQUE NOT NULL,        -- token partagé dans l'URL
    created_by  TEXT    NOT NULL,               -- user_id de l'admin
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL,               -- toujours 48h
    used        INTEGER NOT NULL DEFAULT 0,     -- bool : 0=disponible, 1=utilisé
    used_by     TEXT,                           -- user_id de celui qui a rejoint
    used_at     INTEGER,                        -- timestamp d'utilisation
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (used_by)    REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_invites_token      ON invites(token);
CREATE INDEX IF NOT EXISTS idx_invites_created_by ON invites(created_by);
CREATE INDEX IF NOT EXISTS idx_invites_used       ON invites(used);

-- ════════════════════════════════════════════════════════════════
-- TABLE chess_games
-- Jeu d'échecs 2/3/4 joueurs — variante 4 coins (plateau 14×14)
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS chess_games (
    id              TEXT    PRIMARY KEY,
    created_by      TEXT    NOT NULL,
    player_count    INTEGER NOT NULL DEFAULT 2,  -- 2, 3 ou 4

    -- Slots joueurs (NULL si non occupé)
    player1_id      TEXT,
    player2_id      TEXT,
    player3_id      TEXT,
    player4_id      TEXT,

    -- Couleurs assignées
    player1_color   TEXT    NOT NULL DEFAULT 'white',
    player2_color   TEXT    NOT NULL DEFAULT 'black',
    player3_color   TEXT    NOT NULL DEFAULT 'red',
    player4_color   TEXT    NOT NULL DEFAULT 'green',

    -- Tour courant (1→player_count)
    current_turn    INTEGER NOT NULL DEFAULT 1,

    -- État
    status          TEXT    NOT NULL DEFAULT 'waiting',
    -- 'waiting' | 'playing' | 'finished' | 'abandoned'

    winner_id       TEXT,

    -- JSON : tableau de ChessPiece
    board_state     TEXT    NOT NULL DEFAULT '[]',

    -- JSON : tableau de ChessMove
    move_history    TEXT    NOT NULL DEFAULT '[]',

    -- JSON : tableau d'entiers (slots éliminés)
    eliminated      TEXT    NOT NULL DEFAULT '[]',

    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,

    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (player1_id) REFERENCES users(id),
    FOREIGN KEY (player2_id) REFERENCES users(id),
    FOREIGN KEY (player3_id) REFERENCES users(id),
    FOREIGN KEY (player4_id) REFERENCES users(id),
    FOREIGN KEY (winner_id)  REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_chess_status     ON chess_games(status);
CREATE INDEX IF NOT EXISTS idx_chess_created_by ON chess_games(created_by);
CREATE INDEX IF NOT EXISTS idx_chess_updated_at ON chess_games(updated_at DESC);

-- ════════════════════════════════════════════════════════════════
-- TABLE chess_invitations
-- Invitations à rejoindre une partie (slot spécifique)
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS chess_invitations (
    id              TEXT    PRIMARY KEY,
    game_id         TEXT    NOT NULL,
    invited_user_id TEXT    NOT NULL,
    slot            INTEGER NOT NULL,           -- slot proposé (2/3/4)
    status          TEXT    NOT NULL DEFAULT 'pending',
    -- 'pending' | 'accepted' | 'declined'
    created_at      INTEGER NOT NULL,

    FOREIGN KEY (game_id)         REFERENCES chess_games(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_chess_inv_user ON chess_invitations(invited_user_id, status);
CREATE INDEX IF NOT EXISTS idx_chess_inv_game ON chess_invitations(game_id);
