-- Migration 007 — Chess timer + usernames dans GameState
-- Ajoute time_limit_secs sur chess_games (0 = illimité)

ALTER TABLE chess_games ADD COLUMN time_limit_secs INTEGER NOT NULL DEFAULT 0;
