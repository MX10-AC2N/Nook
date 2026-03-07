-- Migration 002 : chess_engine FIDE
-- Ajout de la colonne ai_difficulty sur chess_games
-- Les anciennes parties (plateau JSON custom 14×14) sont incompatibles avec
-- le nouveau moteur FEN. On les purge proprement.
-- ════════════════════════════════════════════════════════════════

-- Ajouter la colonne ai_difficulty (NULL = humain vs humain)
ALTER TABLE chess_games ADD COLUMN ai_difficulty TEXT;

-- Purger toutes les parties et invitations existantes
-- (l'ancien format board_state était un JSON de pièces, pas un FEN)
DELETE FROM chess_invitations;
DELETE FROM chess_games;
