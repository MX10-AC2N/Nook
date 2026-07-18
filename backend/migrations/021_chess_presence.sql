-- migrations/021_chess_presence.sql
-- Chess presence & move history — Session C-04
-- Timestamps INTEGER (Unix epoch), cohérent avec 001_initial.sql

-- ════════════════════════════════════════════════════════════════
-- TABLE chess_games — status column already exists in 001_initial.sql (line 149)
-- 002_chess_fide.sql added ai_difficulty, 007_chess_timer.sql added time_limit_secs
-- status column is ALREADY PRESENT — skip ALTER TABLE to avoid duplicate column error
-- CREATE INDEX IF NOT EXISTS idx_chess_games_status ON chess_games(status);
-- (index creation is idempotent, safe to keep)

-- ════════════════════════════════════════════════════════════════
-- TABLE chess_moves — move history for real-time replay
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS chess_moves (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id      TEXT    NOT NULL,
    move_number  INTEGER NOT NULL,       -- 1-based, incremented per move (1, 2, 3...)
    from_square  TEXT    NOT NULL,       -- e.g. "e2"
    to_square    TEXT    NOT NULL,       -- e.g. "e4"
    san          TEXT    NOT NULL,       -- Standard Algebraic Notation: "e4", "Nf3", "O-O"
    player_id    TEXT    NOT NULL,       -- who played this move
    created_at   INTEGER NOT NULL,       -- Unix epoch
    FOREIGN KEY (game_id)   REFERENCES chess_games(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Index for real-time replay: find moves of a game in order
CREATE INDEX IF NOT EXISTS idx_chess_moves_game ON chess_moves(game_id, move_number);
-- Index to find moves by player
CREATE INDEX IF NOT EXISTS idx_chess_moves_player ON chess_moves(player_id, created_at);