// backend/src/chess.rs
// Module jeu d'échecs multi-joueurs (2, 3 ou 4 joueurs)
// Variante "4 coins" : plateau 14×14 avec coins 4×4 retirés
//
// Routes :
//   POST   /api/chess/create          Créer une partie
//   GET    /api/chess/list            Lister les parties ouvertes
//   GET    /api/chess/{id}            État d'une partie
//   POST   /api/chess/{id}/join       Rejoindre une partie
//   POST   /api/chess/{id}/move       Jouer un coup
//   POST   /api/chess/{id}/resign     Abandonner
//   POST   /api/chess/{id}/invite     Inviter un joueur à un slot
//   GET    /api/chess/invitations     Mes invitations en attente
//   POST   /api/chess/invitations/{id}/accept  Accepter
//   POST   /api/chess/invitations/{id}/decline Refuser

use crate::{auth::CurrentUser, SharedState};
use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════
// TYPES
// ════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PieceColor {
    White,
    Black,
    Red,
    Green,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessPiece {
    pub id: String,          // ex: "wK", "bQ1", "rP3"
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub row: i32,            // 0-13
    pub col: i32,            // 0-13
    pub alive: bool,
    pub moved: bool,         // pour le roque et les pions (premier coup)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessMove {
    pub player_slot: i32,    // 1/2/3/4
    pub piece_id: String,
    pub from_row: i32,
    pub from_col: i32,
    pub to_row: i32,
    pub to_col: i32,
    pub captured_piece_id: Option<String>,
    pub promotion: Option<PieceType>,  // pour la promotion du pion
    pub timestamp: i64,
}

// ════════════════════════════════════════════════════════════════
// REQUÊTES / RÉPONSES
// ════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateGameRequest {
    pub player_count: i32,   // 2, 3 ou 4
    pub name: Option<String>, // nom optionnel de la partie
}

#[derive(Deserialize)]
pub struct MakeMoveRequest {
    pub piece_id: String,
    pub to_row: i32,
    pub to_col: i32,
    pub promotion: Option<String>, // "queen"|"rook"|"bishop"|"knight"
}

#[derive(Deserialize)]
pub struct InvitePlayerRequest {
    pub user_id: String,
    pub slot: i32,  // 2, 3 ou 4
}

// ════════════════════════════════════════════════════════════════
// LOGIQUE DU PLATEAU
// ════════════════════════════════════════════════════════════════

/// Génère l'état initial du plateau selon le nombre de joueurs.
///
/// Plateau 14×14 (variante 4 coins) :
///   - Colonnes 0-3  et 10-13 retirées des rangées 0-3 et 10-13
///   - Chaque joueur occupe un coin : blanc=bas, noir=haut, rouge=gauche, vert=droite
///
/// Pour 2 joueurs : blanc (bas) et noir (haut), plateau 8×8 centré
/// Pour 3/4 joueurs : plateau 14×14 complet
pub fn initial_board_state(player_count: i32) -> Vec<ChessPiece> {
    match player_count {
        2 => initial_board_2p(),
        3 | 4 => initial_board_4p(player_count),
        _ => initial_board_2p(),
    }
}

fn initial_board_2p() -> Vec<ChessPiece> {
    let mut pieces = Vec::new();

    // Blanc — rangée 7 (pièces) et rangée 6 (pions)
    let back_pieces = [
        PieceType::Rook, PieceType::Knight, PieceType::Bishop,
        PieceType::Queen, PieceType::King,
        PieceType::Bishop, PieceType::Knight, PieceType::Rook,
    ];
    for (col, pt) in back_pieces.iter().enumerate() {
        pieces.push(ChessPiece {
            id: format!("w{}{}", piece_letter(pt), col),
            piece_type: pt.clone(),
            color: PieceColor::White,
            row: 7, col: col as i32,
            alive: true, moved: false,
        });
        pieces.push(ChessPiece {
            id: format!("wP{}", col),
            piece_type: PieceType::Pawn,
            color: PieceColor::White,
            row: 6, col: col as i32,
            alive: true, moved: false,
        });
    }

    // Noir — rangée 0 (pièces) et rangée 1 (pions)
    for (col, pt) in back_pieces.iter().enumerate() {
        pieces.push(ChessPiece {
            id: format!("b{}{}", piece_letter(pt), col),
            piece_type: pt.clone(),
            color: PieceColor::Black,
            row: 0, col: col as i32,
            alive: true, moved: false,
        });
        pieces.push(ChessPiece {
            id: format!("bP{}", col),
            piece_type: PieceType::Pawn,
            color: PieceColor::Black,
            row: 1, col: col as i32,
            alive: true, moved: false,
        });
    }

    pieces
}

/// Plateau 14×14 pour 3 ou 4 joueurs.
/// Chaque joueur a 8 pièces + 8 pions, positionnés dans leur coin.
///
/// Disposition :
///   Blanc  : coin bas-centre  (rangées 12-13, colonnes 3-10)
///   Noir   : coin haut-centre (rangées 0-1,   colonnes 3-10)
///   Rouge  : coin gauche      (colonnes 0-1,  rangées 3-10)
///   Vert   : coin droit       (colonnes 12-13, rangées 3-10)
fn initial_board_4p(player_count: i32) -> Vec<ChessPiece> {
    let mut pieces = Vec::new();

    let back_order = [
        PieceType::Rook, PieceType::Knight, PieceType::Bishop,
        PieceType::Queen, PieceType::King,
        PieceType::Bishop, PieceType::Knight, PieceType::Rook,
    ];

    // ── BLANC (bas) : row 13=pièces, row 12=pions, cols 3-10 ──────────
    for (i, pt) in back_order.iter().enumerate() {
        let col = (i as i32) + 3;
        pieces.push(ChessPiece {
            id: format!("w{}{}", piece_letter(pt), i),
            piece_type: pt.clone(), color: PieceColor::White,
            row: 13, col, alive: true, moved: false,
        });
        pieces.push(ChessPiece {
            id: format!("wP{}", i),
            piece_type: PieceType::Pawn, color: PieceColor::White,
            row: 12, col, alive: true, moved: false,
        });
    }

    // ── NOIR (haut) : row 0=pièces, row 1=pions, cols 3-10 ────────────
    for (i, pt) in back_order.iter().enumerate() {
        let col = (i as i32) + 3;
        pieces.push(ChessPiece {
            id: format!("b{}{}", piece_letter(pt), i),
            piece_type: pt.clone(), color: PieceColor::Black,
            row: 0, col, alive: true, moved: false,
        });
        pieces.push(ChessPiece {
            id: format!("bP{}", i),
            piece_type: PieceType::Pawn, color: PieceColor::Black,
            row: 1, col, alive: true, moved: false,
        });
    }

    if player_count >= 3 {
        // ── ROUGE (gauche) : col 0=pièces, col 1=pions, rows 3-10 ─────
        for (i, pt) in back_order.iter().enumerate() {
            let row = (i as i32) + 3;
            pieces.push(ChessPiece {
                id: format!("r{}{}", piece_letter(pt), i),
                piece_type: pt.clone(), color: PieceColor::Red,
                row, col: 0, alive: true, moved: false,
            });
            pieces.push(ChessPiece {
                id: format!("rP{}", i),
                piece_type: PieceType::Pawn, color: PieceColor::Red,
                row, col: 1, alive: true, moved: false,
            });
        }
    }

    if player_count >= 4 {
        // ── VERT (droite) : col 13=pièces, col 12=pions, rows 3-10 ────
        for (i, pt) in back_order.iter().enumerate() {
            let row = (i as i32) + 3;
            pieces.push(ChessPiece {
                id: format!("g{}{}", piece_letter(pt), i),
                piece_type: pt.clone(), color: PieceColor::Green,
                row, col: 13, alive: true, moved: false,
            });
            pieces.push(ChessPiece {
                id: format!("gP{}", i),
                piece_type: PieceType::Pawn, color: PieceColor::Green,
                row, col: 12, alive: true, moved: false,
            });
        }
    }

    pieces
}

fn piece_letter(pt: &PieceType) -> &'static str {
    match pt {
        PieceType::King   => "K",
        PieceType::Queen  => "Q",
        PieceType::Rook   => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        PieceType::Pawn   => "P",
    }
}

/// Retourne la couleur assignée à un slot (1-4)
pub fn color_for_slot(slot: i32) -> Option<&'static str> {
    match slot {
        1 => Some("white"),
        2 => Some("black"),
        3 => Some("red"),
        4 => Some("green"),
        _ => None,
    }
}

/// Retourne le slot suivant (en tenant compte des joueurs éliminés et du nombre de joueurs)
pub fn next_turn(current: i32, player_count: i32, eliminated: &[i32]) -> i32 {
    let mut next = current % player_count + 1;
    let mut safety = 0;
    while eliminated.contains(&next) {
        next = next % player_count + 1;
        safety += 1;
        if safety > player_count {
            break; // tous éliminés — ne devrait pas arriver
        }
    }
    next
}

// ════════════════════════════════════════════════════════════════
// VALIDATION DES COUPS
// ════════════════════════════════════════════════════════════════

/// Valide si un coup est légal.
/// Pour l'instant : validation des règles de base (déplacement + capture).
/// En production : ajouter échec, mat, roque, en passant, promotion.
pub fn validate_move(
    pieces: &[ChessPiece],
    piece_id: &str,
    to_row: i32,
    to_col: i32,
    player_count: i32,
) -> Result<Option<String>, String> {
    let piece = pieces
        .iter()
        .find(|p| p.id == piece_id && p.alive)
        .ok_or("Pièce introuvable ou morte")?;

    // Vérifier que la destination est dans les limites du plateau
    let board_size = if player_count == 2 { 8 } else { 14 };
    if to_row < 0 || to_col < 0 || to_row >= board_size || to_col >= board_size {
        return Err("Destination hors plateau".to_string());
    }

    // Pour plateau 4 joueurs : vérifier que la destination n'est pas dans un coin retiré
    if player_count > 2 {
        let in_top_left     = to_row < 4 && to_col < 4;
        let in_top_right    = to_row < 4 && to_col > 9;
        let in_bottom_left  = to_row > 9 && to_col < 4;
        let in_bottom_right = to_row > 9 && to_col > 9;
        if in_top_left || in_top_right || in_bottom_left || in_bottom_right {
            return Err("Destination dans une zone hors-jeu".to_string());
        }
    }

    // Vérifier qu'on ne capture pas ses propres pièces
    let target = pieces.iter().find(|p| p.row == to_row && p.col == to_col && p.alive);
    if let Some(t) = target {
        if t.color == piece.color {
            return Err("Impossible de capturer sa propre pièce".to_string());
        }
    }

    // Vérifier le mouvement selon le type de pièce
    let dr = to_row - piece.row;
    let dc = to_col - piece.col;

    let is_valid = match piece.piece_type {
        PieceType::King => {
            dr.abs() <= 1 && dc.abs() <= 1 && (dr != 0 || dc != 0)
        }
        PieceType::Queen => {
            is_straight(piece, to_row, to_col, pieces)
                || is_diagonal(piece, to_row, to_col, pieces)
        }
        PieceType::Rook => is_straight(piece, to_row, to_col, pieces),
        PieceType::Bishop => is_diagonal(piece, to_row, to_col, pieces),
        PieceType::Knight => {
            (dr.abs() == 2 && dc.abs() == 1) || (dr.abs() == 1 && dc.abs() == 2)
        }
        PieceType::Pawn => {
            validate_pawn_move(piece, to_row, to_col, pieces, player_count)
        }
    };

    if !is_valid {
        return Err(format!("Mouvement invalide pour {:?}", piece.piece_type));
    }

    // Retourner l'ID de la pièce capturée si applicable
    let captured = target.map(|t| t.id.clone());
    Ok(captured)
}

fn is_straight(piece: &ChessPiece, to_row: i32, to_col: i32, pieces: &[ChessPiece]) -> bool {
    if piece.row != to_row && piece.col != to_col {
        return false;
    }
    // Vérifier qu'aucune pièce ne bloque le chemin
    path_clear(piece.row, piece.col, to_row, to_col, pieces)
}

fn is_diagonal(piece: &ChessPiece, to_row: i32, to_col: i32, pieces: &[ChessPiece]) -> bool {
    let dr = (to_row - piece.row).abs();
    let dc = (to_col - piece.col).abs();
    if dr != dc || dr == 0 {
        return false;
    }
    path_clear(piece.row, piece.col, to_row, to_col, pieces)
}

fn path_clear(from_row: i32, from_col: i32, to_row: i32, to_col: i32, pieces: &[ChessPiece]) -> bool {
    let dr = (to_row - from_row).signum();
    let dc = (to_col - from_col).signum();
    let mut r = from_row + dr;
    let mut c = from_col + dc;
    while (r, c) != (to_row, to_col) {
        if pieces.iter().any(|p| p.row == r && p.col == c && p.alive) {
            return false;
        }
        r += dr;
        c += dc;
    }
    true
}

fn validate_pawn_move(
    piece: &ChessPiece,
    to_row: i32,
    to_col: i32,
    pieces: &[ChessPiece],
    _player_count: i32,
) -> bool {
    let dr = to_row - piece.row;
    let dc = to_col - piece.col;

    // Direction selon la couleur
    let forward = match piece.color {
        PieceColor::White => -1,  // blanc monte (row décroît)
        PieceColor::Black => 1,   // noir descend
        PieceColor::Red   => 1,   // rouge va vers la droite (col croît)
        PieceColor::Green => -1,  // vert va vers la gauche
    };

    let is_lateral = piece.color == PieceColor::Red || piece.color == PieceColor::Green;

    let (advance, side) = if is_lateral {
        (dc, dr) // pour rouge/vert : avancer = changer col, côté = changer row
    } else {
        (dr, dc) // pour blanc/noir : avancer = changer row, côté = changer col
    };

    let target = pieces.iter().find(|p| p.row == to_row && p.col == to_col && p.alive);

    // Avance simple
    if advance == forward && side == 0 && target.is_none() {
        return true;
    }
    // Double avance (premier coup)
    if advance == forward * 2 && side == 0 && !piece.moved && target.is_none() {
        // Vérifier que la case intermédiaire est libre
        let mid_row = if is_lateral { piece.row } else { piece.row + forward };
        let mid_col = if is_lateral { piece.col + forward } else { piece.col };
        return pieces.iter().all(|p| !(p.row == mid_row && p.col == mid_col && p.alive));
    }
    // Capture en diagonale
    if advance == forward && side.abs() == 1 && target.is_some() {
        return true;
    }

    false
}

// ════════════════════════════════════════════════════════════════
// HANDLERS
// ════════════════════════════════════════════════════════════════

pub async fn create_game(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreateGameRequest>,
) -> impl IntoResponse {
    if req.player_count < 2 || req.player_count > 4 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "player_count doit être 2, 3 ou 4" })),
        )
            .into_response();
    }

    let game_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let board = initial_board_state(req.player_count);
    let board_json = serde_json::to_string(&board).unwrap();

    let result = sqlx::query(
        r#"INSERT INTO chess_games (
            id, created_by, player_count,
            player1_id, player1_color,
            status, board_state, move_history, eliminated,
            current_turn, created_at, updated_at
        ) VALUES (?, ?, ?, ?, 'white', 'waiting', ?, '[]', '[]', 1, ?, ?)"#,
    )
    .bind(&game_id)
    .bind(&user.id)
    .bind(req.player_count)
    .bind(&user.id)
    .bind(&board_json)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!(
                game_id = %game_id,
                created_by = %user.id,
                player_count = req.player_count,
                "Partie d'échecs créée"
            );
            (
                StatusCode::CREATED,
                Json(json!({
                    "success": true,
                    "game_id": game_id,
                    "message": "Partie créée, en attente des joueurs"
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Erreur création partie d'échecs");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur serveur" })),
            )
                .into_response()
        }
    }
}

pub async fn list_games(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    // Retourne les parties en attente ou en cours
    let rows: Vec<(String, String, i32, String, Option<String>, i32, i64)> = sqlx::query_as(
        r#"SELECT
            g.id, g.created_by, g.player_count, g.status,
            u.username as creator_name,
            g.current_turn, g.updated_at
           FROM chess_games g
           LEFT JOIN users u ON u.id = g.created_by
           WHERE g.status IN ('waiting', 'playing')
           ORDER BY g.updated_at DESC
           LIMIT 50"#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let games: Vec<Value> = rows
        .into_iter()
        .map(|(id, created_by, player_count, status, creator_name, current_turn, updated_at)| {
            json!({
                "id": id,
                "created_by": created_by,
                "creator_name": creator_name,
                "player_count": player_count,
                "status": status,
                "current_turn": current_turn,
                "updated_at": updated_at
            })
        })
        .collect();

    Json(json!({ "success": true, "games": games })).into_response()
}

pub async fn get_game(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    let row = sqlx::query(
        "SELECT * FROM chess_games WHERE id = ?"
    )
    .bind(&game_id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(row)) => {
            use sqlx::Row;
            let game = json!({
                "id": row.get::<String, _>("id"),
                "created_by": row.get::<String, _>("created_by"),
                "player_count": row.get::<i32, _>("player_count"),
                "player1_id": row.get::<Option<String>, _>("player1_id"),
                "player2_id": row.get::<Option<String>, _>("player2_id"),
                "player3_id": row.get::<Option<String>, _>("player3_id"),
                "player4_id": row.get::<Option<String>, _>("player4_id"),
                "player1_color": row.get::<String, _>("player1_color"),
                "player2_color": row.get::<String, _>("player2_color"),
                "player3_color": row.get::<String, _>("player3_color"),
                "player4_color": row.get::<String, _>("player4_color"),
                "current_turn": row.get::<i32, _>("current_turn"),
                "status": row.get::<String, _>("status"),
                "board_state": serde_json::from_str::<Value>(
                    &row.get::<String, _>("board_state")
                ).unwrap_or(json!([])),
                "move_history": serde_json::from_str::<Value>(
                    &row.get::<String, _>("move_history")
                ).unwrap_or(json!([])),
                "eliminated": serde_json::from_str::<Value>(
                    &row.get::<String, _>("eliminated")
                ).unwrap_or(json!([])),
                "winner_id": row.get::<Option<String>, _>("winner_id"),
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
            });
            Json(json!({ "success": true, "game": game })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "message": "Partie introuvable" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, game_id = %game_id, "Erreur get_game");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur serveur" })),
            )
                .into_response()
        }
    }
}

pub async fn join_game(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    let row = sqlx::query(
        "SELECT player_count, player1_id, player2_id, player3_id, player4_id, status FROM chess_games WHERE id = ?"
    )
    .bind(&game_id)
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "message": "Partie introuvable" }))).into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false }))).into_response(),
    };

    use sqlx::Row;
    let status: String = row.get("status");
    if status != "waiting" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "La partie n'est plus en attente" })),
        ).into_response();
    }

    let player_count: i32 = row.get("player_count");
    let p1: Option<String> = row.get("player1_id");
    let p2: Option<String> = row.get("player2_id");
    let p3: Option<String> = row.get("player3_id");
    let p4: Option<String> = row.get("player4_id");

    // Trouver le prochain slot libre
    let slot = if p2.is_none() { 2 }
               else if p3.is_none() && player_count >= 3 { 3 }
               else if p4.is_none() && player_count >= 4 { 4 }
               else {
                   return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": "Partie complète" }))).into_response();
               };

    // Vérifier que l'utilisateur ne joue pas déjà
    for p in [p1.as_ref(), p2.as_ref(), p3.as_ref(), p4.as_ref()].into_iter().flatten() {
        if *p == user.id {
            return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": "Vous participez déjà à cette partie" }))).into_response();
        }
    }

    let color = color_for_slot(slot).unwrap_or("black");
    let col = format!("player{}_id", slot);
    let col_color = format!("player{}_color", slot);

    // Compter les joueurs maintenant présents
    let now = Utc::now().timestamp();
    let players_after = [p1.as_ref(), p2.as_ref(), Some(&user.id), p3.as_ref(), p4.as_ref()]
        .iter()
        .filter(|x| x.is_some())
        .count() as i32;

    let new_status = if players_after >= player_count { "playing" } else { "waiting" };

    let query = format!(
        "UPDATE chess_games SET {} = ?, {} = ?, status = ?, updated_at = ? WHERE id = ?",
        col, col_color
    );

    sqlx::query(&query)
        .bind(&user.id)
        .bind(color)
        .bind(new_status)
        .bind(now)
        .bind(&game_id)
        .execute(&state.db)
        .await
        .ok();

    tracing::info!(game_id = %game_id, user_id = %user.id, slot = slot, "Joueur a rejoint la partie");

    Json(json!({
        "success": true,
        "slot": slot,
        "color": color,
        "status": new_status,
        "message": if new_status == "playing" { "La partie commence !" } else { "En attente des autres joueurs" }
    })).into_response()
}

pub async fn make_move(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
    Json(req): Json<MakeMoveRequest>,
) -> impl IntoResponse {
    let row = sqlx::query(
        "SELECT * FROM chess_games WHERE id = ?"
    )
    .bind(&game_id)
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "message": "Partie introuvable" }))).into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false }))).into_response(),
    };

    use sqlx::Row;
    let status: String = row.get("status");
    if status != "playing" {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": "La partie n'est pas en cours" }))).into_response();
    }

    let current_turn: i32 = row.get("current_turn");
    let player_count: i32 = row.get("player_count");

    // Déterminer le slot du joueur courant
    let user_slot = {
        let p1: Option<String> = row.get("player1_id");
        let p2: Option<String> = row.get("player2_id");
        let p3: Option<String> = row.get("player3_id");
        let p4: Option<String> = row.get("player4_id");
        match (&p1, &p2, &p3, &p4) {
            (Some(id), _, _, _) if id == &user.id => Some(1),
            (_, Some(id), _, _) if id == &user.id => Some(2),
            (_, _, Some(id), _) if id == &user.id => Some(3),
            (_, _, _, Some(id)) if id == &user.id => Some(4),
            _ => None,
        }
    };

    let player_slot = match user_slot {
        Some(s) => s,
        None => return (StatusCode::FORBIDDEN, Json(json!({ "success": false, "message": "Vous ne participez pas à cette partie" }))).into_response(),
    };

    if player_slot != current_turn {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": "Ce n'est pas votre tour" }))).into_response();
    }

    // Charger le plateau
    let board_json: String = row.get("board_state");
    let mut pieces: Vec<ChessPiece> = match serde_json::from_str(&board_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Erreur désérialisation plateau");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false }))).into_response();
        }
    };

    // Trouver la pièce
    let piece = match pieces.iter().find(|p| p.id == req.piece_id && p.alive) {
        Some(p) => p.clone(),
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": "Pièce introuvable" }))).into_response(),
    };

    // Vérifier que la pièce appartient au joueur
    let expected_color = color_for_slot(player_slot).unwrap_or("");
    let piece_color = match piece.color {
        PieceColor::White => "white",
        PieceColor::Black => "black",
        PieceColor::Red   => "red",
        PieceColor::Green => "green",
    };
    if piece_color != expected_color {
        return (StatusCode::FORBIDDEN, Json(json!({ "success": false, "message": "Cette pièce ne vous appartient pas" }))).into_response();
    }

    // Valider le coup
    let captured_id = match validate_move(&pieces, &req.piece_id, req.to_row, req.to_col, player_count) {
        Ok(c) => c,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": e }))).into_response(),
    };

    let from_row = piece.row;
    let from_col = piece.col;
    let now = Utc::now().timestamp();

    // Appliquer le coup sur le plateau
    let mut new_eliminated: Vec<i32> = {
        let elim_json: String = row.get("eliminated");
        serde_json::from_str(&elim_json).unwrap_or_default()
    };

    if let Some(ref cap_id) = captured_id {
        if let Some(cap) = pieces.iter_mut().find(|p| &p.id == cap_id) {
            cap.alive = false;
            // Si c'est un roi, le joueur est éliminé
            if cap.piece_type == PieceType::King {
                let elim_slot = match cap.color {
                    PieceColor::White => 1,
                    PieceColor::Black => 2,
                    PieceColor::Red   => 3,
                    PieceColor::Green => 4,
                };
                if !new_eliminated.contains(&elim_slot) {
                    new_eliminated.push(elim_slot);
                    tracing::info!(game_id = %game_id, eliminated_slot = elim_slot, "Joueur éliminé");
                }
            }
        }
    }

    // Déplacer la pièce
    if let Some(p) = pieces.iter_mut().find(|p| p.id == req.piece_id) {
        p.row = req.to_row;
        p.col = req.to_col;
        p.moved = true;

        // Promotion automatique en reine si pion atteint le bord
        if p.piece_type == PieceType::Pawn {
            let board_size = if player_count == 2 { 8 } else { 14 };
            let promotes = match p.color {
                PieceColor::White => p.row == 0,
                PieceColor::Black => p.row == board_size - 1,
                PieceColor::Red   => p.col == board_size - 1,
                PieceColor::Green => p.col == 0,
            };
            if promotes {
                if let Some(promo) = &req.promotion {
                    p.piece_type = match promo.as_str() {
                        "rook"   => PieceType::Rook,
                        "bishop" => PieceType::Bishop,
                        "knight" => PieceType::Knight,
                        _        => PieceType::Queen,
                    };
                } else {
                    p.piece_type = PieceType::Queen; // promotion automatique
                }
            }
        }
    }

    // Vérifier si la partie est terminée (1 roi vivant restant, ou tous éliminés sauf 1)
    let living_slots: Vec<i32> = (1..=player_count)
        .filter(|s| !new_eliminated.contains(s))
        .collect();

    let (winner_id, new_status) = if living_slots.len() == 1 {
        let winner_slot = living_slots[0];
        let winner = match winner_slot {
            1 => row.get::<Option<String>, _>("player1_id"),
            2 => row.get::<Option<String>, _>("player2_id"),
            3 => row.get::<Option<String>, _>("player3_id"),
            _ => row.get::<Option<String>, _>("player4_id"),
        };
        tracing::info!(game_id = %game_id, winner_slot = winner_slot, "Partie terminée");
        (winner, "finished")
    } else {
        (None, "playing")
    };

    // Calculer le prochain tour
    let next_turn_val = next_turn(current_turn, player_count, &new_eliminated);

    // Enregistrer le coup dans l'historique
    let chess_move = ChessMove {
        player_slot,
        piece_id: req.piece_id.clone(),
        from_row,
        from_col,
        to_row: req.to_row,
        to_col: req.to_col,
        captured_piece_id: captured_id.clone(),
        promotion: None,
        timestamp: now,
    };
    let move_json = serde_json::to_string(&chess_move).unwrap();

    let new_board_json = serde_json::to_string(&pieces).unwrap();
    let elim_json = serde_json::to_string(&new_eliminated).unwrap();

    sqlx::query(
        r#"UPDATE chess_games
           SET board_state = ?,
               move_history = json_insert(move_history, '$[#]', json(?)),
               current_turn = ?,
               status = ?,
               winner_id = ?,
               eliminated = ?,
               updated_at = ?
           WHERE id = ?"#,
    )
    .bind(&new_board_json)
    .bind(&move_json)
    .bind(next_turn_val)
    .bind(new_status)
    .bind(&winner_id)
    .bind(&elim_json)
    .bind(now)
    .bind(&game_id)
    .execute(&state.db)
    .await
    .ok();

    // Diffuser le coup via WebSocket
    let ws_message = json!({
        "type": "chess_move",
        "game_id": game_id,
        "move": {
            "piece_id": req.piece_id,
            "from_row": from_row,
            "from_col": from_col,
            "to_row": req.to_row,
            "to_col": req.to_col,
            "captured": captured_id,
        },
        "next_turn": next_turn_val,
        "status": new_status,
        "winner_id": winner_id,
        "eliminated": new_eliminated,
        "timestamp": now,
    });

    {
        let guard = state.webrtc_state.broadcasts.lock().await;
        for (_, tx) in guard.iter() {
            let _ = tx.send(ws_message.to_string());
        }
    }

    Json(json!({
        "success": true,
        "next_turn": next_turn_val,
        "status": new_status,
        "winner_id": winner_id,
        "captured": captured_id,
        "eliminated": new_eliminated,
    })).into_response()
}

pub async fn resign_game(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    let row = sqlx::query(
        "SELECT player_count, player1_id, player2_id, player3_id, player4_id, status, eliminated FROM chess_games WHERE id = ?"
    )
    .bind(&game_id)
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "success": false }))).into_response(),
    };

    use sqlx::Row;
    let status: String = row.get("status");
    if status == "finished" {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": "Partie déjà terminée" }))).into_response();
    }

    let p1: Option<String> = row.get("player1_id");
    let p2: Option<String> = row.get("player2_id");
    let p3: Option<String> = row.get("player3_id");
    let p4: Option<String> = row.get("player4_id");
    let player_count: i32 = row.get("player_count");

    let slot = match (&p1, &p2, &p3, &p4) {
        (Some(id), _, _, _) if id == &user.id => 1,
        (_, Some(id), _, _) if id == &user.id => 2,
        (_, _, Some(id), _) if id == &user.id => 3,
        (_, _, _, Some(id)) if id == &user.id => 4,
        _ => return (StatusCode::FORBIDDEN, Json(json!({ "success": false, "message": "Non participant" }))).into_response(),
    };

    let mut eliminated: Vec<i32> = {
        let j: String = row.get("eliminated");
        serde_json::from_str(&j).unwrap_or_default()
    };
    if !eliminated.contains(&slot) {
        eliminated.push(slot);
    }

    let living: Vec<i32> = (1..=player_count).filter(|s| !eliminated.contains(s)).collect();
    let (new_status, winner_id) = if living.len() == 1 {
        let ws = living[0];
        let wid = match ws {
            1 => p1.clone(), 2 => p2.clone(), 3 => p3.clone(), _ => p4.clone(),
        };
        ("finished", wid)
    } else if living.is_empty() {
        ("finished", None)
    } else {
        ("playing", None)
    };

    let now = Utc::now().timestamp();
    let elim_json = serde_json::to_string(&eliminated).unwrap();

    sqlx::query(
        "UPDATE chess_games SET eliminated = ?, status = ?, winner_id = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&elim_json)
    .bind(new_status)
    .bind(&winner_id)
    .bind(now)
    .bind(&game_id)
    .execute(&state.db)
    .await
    .ok();

    tracing::info!(game_id = %game_id, user_id = %user.id, slot = slot, "Abandon");

    Json(json!({
        "success": true,
        "status": new_status,
        "winner_id": winner_id,
        "eliminated": eliminated,
    })).into_response()
}

pub async fn invite_player(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
    Json(req): Json<InvitePlayerRequest>,
) -> impl IntoResponse {
    let created_by: Option<(String,)> = sqlx::query_as("SELECT created_by FROM chess_games WHERE id = ?")
        .bind(&game_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    match created_by {
        Some((creator,)) if creator == user.id => {}
        Some(_) => return (StatusCode::FORBIDDEN, Json(json!({ "success": false, "message": "Seul le créateur peut inviter" }))).into_response(),
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false }))).into_response(),
    }

    if req.slot < 2 || req.slot > 4 {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "message": "Slot invalide (2-4)" }))).into_response();
    }

    let inv_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO chess_invitations (id, game_id, invited_user_id, slot, status, created_at) VALUES (?, ?, ?, ?, 'pending', ?)"
    )
    .bind(&inv_id)
    .bind(&game_id)
    .bind(&req.user_id)
    .bind(req.slot)
    .bind(now)
    .execute(&state.db)
    .await
    .ok();

    Json(json!({ "success": true, "invitation_id": inv_id })).into_response()
}

pub async fn my_invitations(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    let rows: Vec<(String, String, i32, String)> = sqlx::query_as(
        "SELECT id, game_id, slot, status FROM chess_invitations WHERE invited_user_id = ? AND status = 'pending' ORDER BY created_at DESC"
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let invitations: Vec<Value> = rows.into_iter().map(|(id, game_id, slot, status)| {
        json!({ "id": id, "game_id": game_id, "slot": slot, "status": status })
    }).collect();

    Json(json!({ "success": true, "invitations": invitations })).into_response()
}

pub async fn accept_invitation(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(inv_id): Path<String>,
) -> impl IntoResponse {
    let row: Option<(String, i32)> = sqlx::query_as(
        "SELECT game_id, slot FROM chess_invitations WHERE id = ? AND invited_user_id = ? AND status = 'pending'"
    )
    .bind(&inv_id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let (game_id, slot) = match row {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "message": "Invitation introuvable" }))).into_response(),
    };

    let color = color_for_slot(slot).unwrap_or("black");
    let col = format!("player{}_id", slot);
    let col_color = format!("player{}_color", slot);
    let now = Utc::now().timestamp();

    let q = format!("UPDATE chess_games SET {} = ?, {} = ?, updated_at = ? WHERE id = ?", col, col_color);
    sqlx::query(&q)
        .bind(&user.id)
        .bind(color)
        .bind(now)
        .bind(&game_id)
        .execute(&state.db)
        .await
        .ok();

    sqlx::query("UPDATE chess_invitations SET status = 'accepted' WHERE id = ?")
        .bind(&inv_id)
        .execute(&state.db)
        .await
        .ok();

    Json(json!({ "success": true, "game_id": game_id, "slot": slot, "color": color })).into_response()
}

pub async fn decline_invitation(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(inv_id): Path<String>,
) -> impl IntoResponse {
    sqlx::query(
        "UPDATE chess_invitations SET status = 'declined' WHERE id = ? AND invited_user_id = ?"
    )
    .bind(&inv_id)
    .bind(&user.id)
    .execute(&state.db)
    .await
    .ok();

    Json(json!({ "success": true })).into_response()
}

// ════════════════════════════════════════════════════════════════
// ROUTES
// ════════════════════════════════════════════════════════════════

pub fn chess_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/chess/create",                         post(create_game))
        .route("/chess/list",                           get(list_games))
        .route("/chess/{id}",                           get(get_game))
        .route("/chess/{id}/join",                      post(join_game))
        .route("/chess/{id}/move",                      post(make_move))
        .route("/chess/{id}/resign",                    post(resign_game))
        .route("/chess/{id}/invite",                    post(invite_player))
        .route("/chess/invitations",                    get(my_invitations))
        .route("/chess/invitations/{id}/accept",        post(accept_invitation))
        .route("/chess/invitations/{id}/decline",       post(decline_invitation))
}
