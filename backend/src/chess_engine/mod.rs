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
//! - `rand::seq::IndexedRandom` (rand 0.10) → `SliceRandom` (rand 0.9)
//! - let-chains edition 2024 → edition 2021 compatible
//! - Chemins de modules adaptés (crate::chess_engine::*)

pub mod attacks;
pub mod board;
pub mod game;
pub mod movegen;
pub mod pgn;
pub mod san;
pub mod types;
pub mod zobrist;
pub mod ai_engine;
pub mod evaluation;

// Re-exports principaux pour usage dans chess.rs
pub use board::Position;
pub use game::Game;
pub use movegen::{legal_moves, legal_moves_from};
pub use types::{
    ChessError, Color, Difficulty, DrawReason, GameStatus, Move, MoveFlags, PieceType, Square,
};
pub use ai_engine::{AiEngine, MinimaxAi, RandomAi, default_engine};
