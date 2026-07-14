-- migrations/021_chess_presence.sql
-- Chess presence & move history — Session C-04
-- Timestamps INTEGER (Unix epoch), cohérent avec 001_initial.sql

-- ════════════════════════════════════════════════════════════════
-- TABLE chess_games — ajouter status (waiting, active, finished, aborted)
-- ════════════════════════════════════════════════════════════════
-- Note: chess_games existe déjà depuis 001_initial.sql et a été modifié par
-- 002_chess_fide.sql (ai_difficulty) et 007_chess_timer.sql (time_limit_secs)
-- On ajoute simplement la colonne status ici.
ALTER TABLE chess_games ADD COLUMN status TEXT NOT NULL DEFAULT 'waiting';
-- 'waiting' | 'active' | 'finished' | 'aborted'

CREATE INDEX IF NOT EXISTS idx_chess_games_status ON chess_games(status);

-- ════════════════════════════════════════════════════════════════
-- TABLE chess_moves — historique des coups pour replay temps réel
-- ════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS chess_moves (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id      TEXT    NOT NULL,
    move_number  INTEGER NOT NULL,       -- 1-based, incrémenté par coup (1, 2, 3...)
    from_square  TEXT    NOT NULL,       -- ex: "e2"
    to_square    TEXT    NOT NULL,       -- ex: "e4"
    san          TEXT    NOT NULL,       -- Standard Algebraic Notation: "e4", "Nf3", "O-O"
    player_id    TEXT    NOT NULL,       -- qui a joué ce coup
    created_at   INTEGER NOT NULL,       -- Unix epoch
    FOREIGN KEY (game_id)   REFERENCES chess_games(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Index pour replay temps réel : chercher les coups d'une partie dans l'ordre
CREATE INDEX IF NOT EXISTS idx_chess_moves_game ON chess_moves(game_id, move_number);
-- Index pour retrouver les coups d'un joueur
CREATE INDEX IF NOT EXISTS idx_chess_moves_player ON chess_moves(player_id, created_at);