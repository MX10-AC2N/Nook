//! chess_engine — moteur d'échecs FIDE intégré dans Nook.
//!
//! Adapté depuis rust-chess (RumenDamyanov, MIT) :
//! - Moteur bitboard complet (attaques magiques, Zobrist)
//! - Génération de coups légaux (roque, en passant, promotions)
//! - Détection : échec, mat, pat, nulle (50 coups, répétition, matériel insuffisant)
//! - IA Minimax avec alpha-bêta, tables de transposition, iterative deepening
//! - FEN et PGN (import/export)
//!
//! Changements vs source originale :
//! - `thiserror` supprimé → Display + Error implémentés manuellement
//! - `rand::RngExt::random_range` (rand 0.10) au lieu de `SliceRandom::choose`
//! - let-chains edition 2024 → edition 2021 compatible
//! - Chemins de modules adaptés (crate::chess_engine::*)

// Bibliothèque intégrée : toutes les APIs ne sont pas encore utilisées
#![allow(dead_code)]

pub mod ai_engine;
pub mod attacks;
pub mod board;
pub mod evaluation;
pub mod game;
pub mod movegen;
pub mod pgn;
pub mod san;
pub mod types;
pub mod zobrist;

// Re-exports utilisés dans chess.rs
pub use ai_engine::{AiEngine, MinimaxAi};
pub use game::Game;
pub use types::{ChessError, Color, Difficulty, GameStatus, Move, PieceType, Square};
