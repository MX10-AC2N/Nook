// backend/src/chess.rs
// Jeu d'échecs FIDE standard — 2 joueurs (humain vs humain ou humain vs IA)
//
// Moteur : chess_engine (adapté de rust-chess, MIT — RumenDamyanov)
// Stockage : SQLite — position en FEN, historique en JSON SAN
//
// Routes :
//   POST  /api/chess/create
//   GET   /api/chess/list
//   GET   /api/chess/{id}
//   POST  /api/chess/{id}/join
//   POST  /api/chess/{id}/move
//   POST  /api/chess/{id}/ai-move
//   POST  /api/chess/{id}/resign
//   GET   /api/chess/{id}/moves         coups légaux (?from=e2)
//   POST  /api/chess/{id}/invite
//   GET   /api/chess/invitations
//   POST  /api/chess/invitations/{id}/accept
//   POST  /api/chess/invitations/{id}/decline

use crate::chess_engine::{
    AiEngine, ChessError, Color, Difficulty, Game, GameStatus, MinimaxAi, Move as ChessMove,
    PieceType, Square,
};
use crate::{auth::CurrentUser, SharedState};
use axum::{
    extract::{Path, Query, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use rand::Rng;
use uuid::Uuid;
use tokio::time::{sleep as tokio_sleep, Duration as TokioDuration};

// ════════════════════════════════════════════════════════════════
// TYPES
// ════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateGameRequest {
    pub opponent: Option<String>,
    pub color: Option<String>,
    pub time_limit_secs: Option<i64>,
}

#[derive(Deserialize)]
pub struct MakeMoveRequest {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

#[derive(Deserialize)]
pub struct AiMoveRequest {
    pub difficulty: Option<String>,
}

#[derive(Deserialize)]
pub struct LegalMovesQuery {
    pub from: Option<String>,
}

#[derive(Deserialize)]
pub struct InvitePlayerRequest {
    pub user_id: String,
}

// ════════════════════════════════════════════════════════════════
// HELPERS
// ════════════════════════════════════════════════════════════════

fn parse_difficulty(s: &str) -> Difficulty {
    match s.to_lowercase().as_str() {
        "easy" => Difficulty::Easy,
        "medium" => Difficulty::Medium,
        "hard" => Difficulty::Hard,
        "expert" => Difficulty::Expert,
        "godlike" => Difficulty::Godlike,
        _ => Difficulty::Medium,
    }
}

fn parse_move(game: &Game, from: &str, to: &str, promo: Option<&str>) -> Result<ChessMove, String> {
    let from_sq = Square::from_algebraic(from).ok_or_else(|| format!("Case invalide : {from}"))?;
    let to_sq = Square::from_algebraic(to).ok_or_else(|| format!("Case invalide : {to}"))?;

    let promotion = match promo {
        Some("q") | Some("queen") => Some(PieceType::Queen),
        Some("r") | Some("rook") => Some(PieceType::Rook),
        Some("b") | Some("bishop") => Some(PieceType::Bishop),
        Some("n") | Some("knight") => Some(PieceType::Knight),
        None => None,
        Some(other) => return Err(format!("Promotion invalide : {other}")),
    };

    let legal = game.legal_moves_from(from_sq);
    legal
        .iter()
        .find(|mv| mv.from == from_sq && mv.to == to_sq && mv.promotion == promotion)
        .copied()
        .ok_or_else(|| format!("Coup illégal : {from}{to}"))
}

fn game_json(game: &Game) -> Value {
    let board = game.board_array();
    let legal: Vec<String> = game
        .legal_moves()
        .iter()
        .map(|mv| {
            let promo = mv
                .promotion
                .map(|p| match p {
                    PieceType::Queen => "=q",
                    PieceType::Rook => "=r",
                    PieceType::Bishop => "=b",
                    PieceType::Knight => "=n",
                    _ => "",
                })
                .unwrap_or("");
            format!("{}{}{}", mv.from, mv.to, promo)
        })
        .collect();

    json!({
        "fen": game.to_fen(),
        "board": board,
        "side_to_move": match game.side_to_move() {
            Color::White => "white",
            Color::Black => "black",
        },
        "status": game.status().as_str(),
        "legal_moves": legal,
        "move_count": game.move_history().len(),
    })
}

// Limites de temps par difficulté
fn ai_time_limit(difficulty: Difficulty) -> std::time::Duration {
    match difficulty {
        Difficulty::Harmless => std::time::Duration::from_millis(30),
        Difficulty::Easy => std::time::Duration::from_millis(100),
        Difficulty::Medium => std::time::Duration::from_millis(400),
        Difficulty::Hard => std::time::Duration::from_millis(1500),
        Difficulty::Expert => std::time::Duration::from_millis(3000),
        Difficulty::Godlike => std::time::Duration::from_millis(6000),
    }
}

// TT size par difficulté
fn ai_tt_size(difficulty: Difficulty) -> usize {
    match difficulty {
        Difficulty::Harmless => 1 << 10,
        Difficulty::Easy => 1 << 14,
        Difficulty::Medium => 1 << 16,
        Difficulty::Hard => 1 << 18,
        Difficulty::Expert => 1 << 19,
        Difficulty::Godlike => 1 << 20,
    }
}

// Calculer le délai d'affichage (sans dormir — juste retourner la durée)
fn ai_display_delay(difficulty: Difficulty) -> u64 {
    let base_ms: u64 = match difficulty {
        Difficulty::Harmless => 1000,
        Difficulty::Easy => 2500,
        Difficulty::Medium => 4000,
        Difficulty::Hard => 6000,
        Difficulty::Expert => 9000,
        Difficulty::Godlike => 12000,
    };
    // Jitter : ±30% de la valeur de base
    let jitter = (base_ms as f64 * 0.3 * (rand::rng().random::<f64>() * 2.0 - 1.0)) as i64;
    (base_ms as i64 + jitter).max(500) as u64
}

// Retourne (san, from_alg, to_alg, new_fen, game_status_str, delay_ms)
// NOTE: Le délai d'affichage est calculé mais le sleep est géré côté async
fn play_ai(mut game: Game, difficulty: Difficulty) -> Result<(String, String, String, String, String, u64), ChessError> {
    let time_limit = ai_time_limit(difficulty);
    let tt_size = ai_tt_size(difficulty);
    let ai = MinimaxAi::with_time_limit_and_tt(time_limit, tt_size);
    let mv = ai.best_move(&game, difficulty)?;
    let from_alg = mv.from.to_algebraic();
    let to_alg = mv.to.to_algebraic();

    // Calculer le délai d'affichage (sans bloquer)
    let delay_ms = ai_display_delay(difficulty);

    let san = game.make_move(mv)?;
    let new_fen = game.to_fen();
    let status_str = game.status().as_str().to_string();
    Ok((san, from_alg, to_alg, new_fen, status_str, delay_ms))
}

// ════════════════════════════════════════════════════════════════
// HANDLERS
// ════════════════════════════════════════════════════════════════

pub async fn create_game(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreateGameRequest>,
) -> impl IntoResponse {
    let game_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let creator_color = req.color.as_deref().unwrap_or("white");
    let (p1_color, p2_color) = if creator_color == "black" {
        ("black", "white")
    } else {
        ("white", "black")
    };

    let opponent = req.opponent.as_deref().unwrap_or("human");
    let is_ai = opponent != "human";

    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let (initial_fen, initial_history) = if is_ai && creator_color == "black" {
        // L'IA joue le premier coup (elle est Blancs, le joueur est Noirs).
        // OBLIGATOIRE : spawn_blocking — play_ai() est CPU-bound et bloquerait
        // le thread Tokio sinon (cause du freeze frontend S42).
        // Pas de délai d'affichage ici : on veut juste créer la partie rapidement.
        let game = Game::new();
        let difficulty = parse_difficulty(opponent);
        let ai_result = tokio::task::spawn_blocking(move || play_ai(game, difficulty)).await;
        match ai_result {
            Ok(Ok((san, from_alg, to_alg, new_fen_init, _, _))) => (
                new_fen_init,
                json!([{"san": san, "from": from_alg, "to": to_alg, "by": "ai", "color": "white"}]).to_string(),
            ),
            _ => (starting_fen.to_string(), "[]".to_string()),
        }
    } else {
        (starting_fen.to_string(), "[]".to_string())
    };

    let initial_status = if is_ai { "playing" } else { "waiting" };
    let ai_diff: Option<&str> = if is_ai { Some(opponent) } else { None };
    let time_limit = req.time_limit_secs.unwrap_or(0).max(0);

    let result = sqlx::query(
        r#"
        INSERT INTO chess_games (
            id, created_by, player_count,
            player1_id, player1_color, player2_color,
            status, board_state, move_history, eliminated,
            current_turn, ai_difficulty, time_limit_secs, created_at, updated_at
        ) VALUES (?, ?, 2, ?, ?, ?, ?, ?, ?, '[]', 1, ?, ?, ?, ?)"#,
    )
    .bind(&game_id)
    .bind(&user.id)
    .bind(&user.id)
    .bind(p1_color)
    .bind(p2_color)
    .bind(initial_status)
    .bind(&initial_fen)
    .bind(&initial_history)
    .bind(ai_diff)
    .bind(time_limit)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!(game_id = %game_id, opponent = opponent, "Partie d'échecs créée");
            (
                StatusCode::CREATED,
                Json(json!({
                    "success": true,
                    "game_id": game_id,
                    "player_color": creator_color,
                    "opponent": opponent,
                    "status": initial_status,
                    "fen": initial_fen,
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Erreur création partie");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "message": "Erreur serveur"})),
            )
                .into_response()
        }
    }
}

pub async fn list_games(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    type Row = (String, String, String, Option<String>, i64);
    let rows: Vec<Row> = sqlx::query_as(
        r#"
        SELECT g.id, g.status, g.player1_color, u.username, g.updated_at
        FROM chess_games g
        LEFT JOIN users u ON u.id = g.created_by
        WHERE g.status IN ('waiting','playing')
        ORDER BY g.updated_at DESC LIMIT 50"#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let games: Vec<Value> = rows
        .into_iter()
        .map(|(id, status, color, creator, updated_at)| {
            json!({ "id": id, "status": status, "creator_color": color,
                "creator_name": creator, "updated_at": updated_at })
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
        r#"SELECT g.*,
            u1.username AS p1_username, u1.name AS p1_name,
            u2.username AS p2_username, u2.name AS p2_name
           FROM chess_games g
           LEFT JOIN users u1 ON g.player1_id = u1.id
           LEFT JOIN users u2 ON g.player2_id = u2.id
           WHERE g.id = ?"#
    )
        .bind(&game_id)
        .fetch_optional(&state.db)
        .await;

    match row {
        Ok(Some(row)) => {
            use sqlx::Row;
            let fen: String = row.get("board_state");
            let history_raw: String = row.get("move_history");
            let history: Value = serde_json::from_str(&history_raw).unwrap_or(json!([]));

            let engine = Game::from_fen(&fen)
                .map(|g| game_json(&g))
                .unwrap_or(json!(null));

            let p1_display: Option<String> = row.get::<Option<String>, _>("p1_name")
                .or_else(|| row.get::<Option<String>, _>("p1_username"));
            let p2_display: Option<String> = row.get::<Option<String>, _>("p2_name")
                .or_else(|| row.get::<Option<String>, _>("p2_username"));

            Json(json!({ "success": true, "game": {
                "id": row.get::<String, _>("id"),
                "created_by": row.get::<String, _>("created_by"),
                "player1_id": row.get::<Option<String>, _>("player1_id"),
                "player2_id": row.get::<Option<String>, _>("player2_id"),
                "player1_name": p1_display,
                "player2_name": p2_display,
                "player1_color": row.get::<String, _>("player1_color"),
                "player2_color": row.get::<String, _>("player2_color"),
                "status": row.get::<String, _>("status"),
                "winner_id": row.get::<Option<String>, _>("winner_id"),
                "ai_difficulty": row.get::<Option<String>, _>("ai_difficulty"),
                "time_limit_secs": row.get::<i64, _>("time_limit_secs"),
                "fen": fen,
                "move_history": history,
                "engine": engine,
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
            }}))
            .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "message": "Partie introuvable" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, game_id = %game_id, "get_game");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false })),
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
        "SELECT player1_id, player2_id, player2_color, status, ai_difficulty FROM chess_games WHERE id = ?")
        .bind(&game_id).fetch_optional(&state.db).await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "message": "Partie introuvable" })),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false })),
            )
                .into_response()
        }
    };

    use sqlx::Row;
    if row.get::<String, _>("status") != "waiting" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "La partie n'est plus en attente" })),
        )
            .into_response();
    }
    let ai_diff: Option<String> = row.get("ai_difficulty");
    if ai_diff.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Partie contre IA" })),
        )
            .into_response();
    }
    let p1: Option<String> = row.get("player1_id");
    let p2: Option<String> = row.get("player2_id");
    if p2.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Partie complète" })),
        )
            .into_response();
    }
    if p1.as_deref() == Some(&user.id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Déjà participant" })),
        )
            .into_response();
    }

    let p2_color: String = row.get("player2_color");
    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE chess_games SET player2_id = ?, status = 'playing', updated_at = ? WHERE id = ?",
    )
    .bind(&user.id)
    .bind(now)
    .bind(&game_id)
    .execute(&state.db)
    .await
    .ok();

    let ws = json!({ "type": "chess_player_joined", "game_id": game_id, "player2_id": user.id });
    let guard = state.webrtc_state.broadcasts.lock().await;
    for (_, tx) in guard.iter() {
        let _ = tx.send(ws.to_string());
    }
    drop(guard);

    tracing::info!(game_id = %game_id, user_id = %user.id, "Joueur 2 rejoint");
    Json(json!({ "success": true, "color": p2_color, "status": "playing" })).into_response()
}

pub async fn make_move(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
    Json(req): Json<MakeMoveRequest>,
) -> impl IntoResponse {
    let row = sqlx::query("SELECT * FROM chess_games WHERE id = ?")
        .bind(&game_id)
        .fetch_optional(&state.db)
        .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "message": "Partie introuvable" })),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false })),
            )
                .into_response()
        }
    };

    use sqlx::Row;
    if row.get::<String, _>("status") != "playing" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Partie non en cours" })),
        )
            .into_response();
    }

    let p1_id: Option<String> = row.get("player1_id");
    let p2_id: Option<String> = row.get("player2_id");
    let p1_color: String = row.get("player1_color");
    let ai_diff: Option<String> = row.get("ai_difficulty");

    let player_color = if p1_id.as_deref() == Some(&user.id) {
        if p1_color == "white" {
            Color::White
        } else {
            Color::Black
        }
    } else if p2_id.as_deref() == Some(&user.id) {
        if p1_color == "white" {
            Color::Black
        } else {
            Color::White
        }
    } else if ai_diff.is_some() && p1_id.as_deref() == Some(&user.id) {
        if p1_color == "white" {
            Color::White
        } else {
            Color::Black
        }
    } else {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "success": false, "message": "Non participant" })),
        )
            .into_response();
    };

    let fen: String = row.get("board_state");
    let mut game = match Game::from_fen(&fen) {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "FEN invalide" })),
            )
                .into_response()
        }
    };

    if game.side_to_move() != player_color {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Pas votre tour" })),
        )
            .into_response();
    }

    let mv = match parse_move(&game, &req.from, &req.to, req.promotion.as_deref()) {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": e })),
            )
                .into_response()
        }
    };

    let san = match game.make_move(mv) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": e.to_string() })),
            )
                .into_response()
        }
    };

    let new_fen = game.to_fen();
    let game_status = game.status().clone();
    let now = Utc::now().timestamp();

    let color_str = match player_color {
        Color::White => "white",
        Color::Black => "black",
    };

    let (winner_id, db_status): (Option<String>, &str) = match &game_status {
        GameStatus::Checkmate => {
            let winner = match player_color {
                Color::White => p1_id.clone(),
                Color::Black => p2_id.clone().or(p1_id.clone()),
            };
            (winner, "finished")
        }
        GameStatus::Stalemate | GameStatus::Draw(_) => (None, "finished"),
        _ => (None, "playing"),
    };

    let mut history: Vec<Value> = {
        let raw: String = row.get("move_history");
        serde_json::from_str(&raw).unwrap_or_default()
    };
    history.push(
        json!({ "san": san, "from": req.from, "to": req.to, "by": user.id, "color": color_str }),
    );
    let new_history = serde_json::to_string(&history).unwrap();

    let next_turn = if game_status.is_game_over() {
        0
    } else {
        match game.side_to_move() {
            Color::White => 1,
            Color::Black => 2,
        }
    };

    sqlx::query(
        r#"UPDATE chess_games
        SET board_state = ?, move_history = ?, current_turn = ?,
            status = ?, winner_id = ?, updated_at = ?
        WHERE id = ?"#,
    )
    .bind(&new_fen)
    .bind(&new_history)
    .bind(next_turn)
    .bind(db_status)
    .bind(&winner_id)
    .bind(now)
    .bind(&game_id)
    .execute(&state.db)
    .await
    .ok();

    let engine = game_json(&game);
    let ws = json!({
        "type": "chess_move", "game_id": game_id,
        "move": { "san": san, "from": req.from, "to": req.to, "color": color_str },
        "fen": new_fen, "status": game_status.as_str(),
        "winner_id": winner_id, "engine": engine, "timestamp": now,
    });
    {
        let guard = state.webrtc_state.broadcasts.lock().await;
        for (_, tx) in guard.iter() {
            let _ = tx.send(ws.to_string());
        }
    }

    let move_history_val: serde_json::Value = serde_json::from_str(&new_history).unwrap_or(json!([]));
    Json(json!({
        "success": true,
        "game": {
            "id": game_id,
            "created_by": row.get::<String, _>("created_by"),
            "player1_id": row.get::<Option<String>, _>("player1_id"),
            "player2_id": row.get::<Option<String>, _>("player2_id"),
            "player1_color": row.get::<String, _>("player1_color"),
            "player2_color": row.get::<String, _>("player2_color"),
            "status": db_status,
            "winner_id": winner_id,
            "ai_difficulty": row.get::<Option<String>, _>("ai_difficulty"),
            "fen": new_fen,
            "move_history": move_history_val,
            "engine": engine,
            "created_at": row.get::<i64, _>("created_at"),
            "updated_at": now,
        }
    }))
    .into_response()
}

pub async fn ai_move(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
    body: Option<axum::extract::Json<AiMoveRequest>>,
) -> impl IntoResponse {
    let req = body.map(|b| b.0).unwrap_or(AiMoveRequest { difficulty: None });
    let row = sqlx::query("SELECT * FROM chess_games WHERE id = ?")
        .bind(&game_id)
        .fetch_optional(&state.db)
        .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "message": "Partie introuvable" })),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false })),
            )
                .into_response()
        }
    };

    use sqlx::Row;
    if row.get::<String, _>("status") != "playing" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Partie non en cours" })),
        )
            .into_response();
    }

    let p1_id: Option<String> = row.get("player1_id");
    if p1_id.as_deref() != Some(&user.id) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "success": false, "message": "Non participant" })),
        )
            .into_response();
    }

    let ai_diff_str: Option<String> = row.get("ai_difficulty");
    let difficulty = req
        .difficulty
        .as_deref()
        .or(ai_diff_str.as_deref())
        .map(parse_difficulty)
        .unwrap_or(Difficulty::Medium);

    let fen: String = row.get("board_state");
    let game = match Game::from_fen(&fen) {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "FEN invalide" })),
            )
                .into_response()
        }
    };

    let ai_color = game.side_to_move();

    // Exécuter l'IA dans spawn_blocking
    let ai_result = tokio::task::spawn_blocking(move || {
        play_ai(game, difficulty)
    }).await;

    let (san, ai_from, ai_to, new_fen_ai, new_status_ai, delay_ms) = match ai_result {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": e.to_string() })),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "IA indisponible" })),
            )
                .into_response()
        }
    };

    // CORRECTION: Appliquer le délai d'affichage de manière NON-BLOQUANTE
    tokio_sleep(TokioDuration::from_millis(delay_ms)).await;

    let new_fen = new_fen_ai;
    let game_status_str = new_status_ai;
    let now = Utc::now().timestamp();
    let ai_color_str = match ai_color {
        Color::White => "white",
        Color::Black => "black",
    };

    // FIX: In case of checkmate, the AI just won — set winner_id to the AI player
    let (winner_id, db_status): (Option<String>, &str) = match game_status_str.as_str() {
        "checkmate" => {
            // AI wins: if AI plays white, winner is player1, else player2
            let p2_id: Option<String> = row.get("player2_id");
            match ai_color {
                Color::White => (Some(row.get::<String, _>("player1_id")), "finished"),
                Color::Black => (p2_id.clone(), "finished"),
            }
        }
        "stalemate" | "draw" | "insufficient_material" | "repetition" | "fifty_moves" => (None, "finished"),
        _ => (None, "playing"),
    };

    let mut history: Vec<Value> = {
        let raw: String = row.get("move_history");
        serde_json::from_str(&raw).unwrap_or_default()
    };
    history.push(json!({ "san": san, "from": ai_from, "to": ai_to, "by": "ai", "color": ai_color_str }));
    let new_history = serde_json::to_string(&history).unwrap();
    // Alternate turns: 1=human, 2=AI for AI games
    let next_turn = if db_status == "finished" { 0 } else { 2 };

    sqlx::query(
        r#"UPDATE chess_games
        SET board_state = ?, move_history = ?, current_turn = ?,
            status = ?, winner_id = ?, updated_at = ?
        WHERE id = ?"#,
    )
    .bind(&new_fen)
    .bind(&new_history)
    .bind(next_turn)
    .bind(db_status)
    .bind(&winner_id)
    .bind(now)
    .bind(&game_id)
    .execute(&state.db)
    .await
    .ok();

    let engine = match Game::from_fen(&new_fen) {
        Ok(rebuilt) => game_json(&rebuilt),
        Err(_) => serde_json::json!(null),
    };
    let ws = json!({
        "type": "chess_ai_move", "game_id": game_id,
        "move": { "san": san, "from": ai_from, "to": ai_to, "by": "ai", "color": ai_color_str },
        "fen": new_fen, "status": game_status_str.as_str(),
        "winner_id": winner_id, "engine": engine, "timestamp": now,
    });
    {
        let guard = state.webrtc_state.broadcasts.lock().await;
        for (_, tx) in guard.iter() {
            let _ = tx.send(ws.to_string());
        }
    }

    let move_history_val: serde_json::Value = serde_json::from_str(&new_history).unwrap_or(json!([]));
    Json(json!({
        "success": true,
        "game": {
            "id": game_id,
            "created_by": row.get::<String, _>("created_by"),
            "player1_id": row.get::<Option<String>, _>("player1_id"),
            "player2_id": row.get::<Option<String>, _>("player2_id"),
            "player1_color": row.get::<String, _>("player1_color"),
            "player2_color": row.get::<String, _>("player2_color"),
            "status": game_status_str.as_str(),
            "winner_id": winner_id,
            "ai_difficulty": row.get::<Option<String>, _>("ai_difficulty"),
            "fen": new_fen,
            "move_history": move_history_val,
            "engine": engine,
            "created_at": row.get::<i64, _>("created_at"),
            "updated_at": now,
        }
    }))
    .into_response()
}

pub async fn resign_game(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    let row = sqlx::query(
        "SELECT player1_id, player2_id, status, ai_difficulty FROM chess_games WHERE id = ?",
    )
    .bind(&game_id)
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        _ => return (StatusCode::NOT_FOUND, Json(json!({ "success": false }))).into_response(),
    };

    use sqlx::Row;
    if row.get::<String, _>("status") == "finished" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Partie déjà terminée" })),
        )
            .into_response();
    }

    let p1_id: Option<String> = row.get("player1_id");
    let p2_id: Option<String> = row.get("player2_id");
    let ai_diff: Option<String> = row.get("ai_difficulty");

    let winner_id: Option<String> = if p1_id.as_deref() == Some(&user.id) {
        if ai_diff.is_some() { None } else { p2_id.clone() }
    } else if p2_id.as_deref() == Some(&user.id) {
        p1_id.clone()
    } else {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "success": false, "message": "Non participant" })),
        )
            .into_response();
    };

    let now = Utc::now().timestamp();
    if let Err(e) = sqlx::query(
        "UPDATE chess_games SET status = 'finished', winner_id = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&winner_id)
    .bind(now)
    .bind(&game_id)
    .execute(&state.db)
    .await
    {
        tracing::error!(error = %e, game_id = %game_id, "Erreur UPDATE resign");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": "Erreur serveur" })),
        )
            .into_response();
    }

    tracing::info!(game_id = %game_id, user_id = %user.id, "Abandon");
    Json(json!({ "success": true, "status": "finished", "winner_id": winner_id })).into_response()
}

pub async fn legal_moves(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
    Query(params): Query<LegalMovesQuery>,
) -> impl IntoResponse {
    let fen: Option<(String,)> = sqlx::query_as("SELECT board_state FROM chess_games WHERE id = ?")
        .bind(&game_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let (fen,) = match fen {
        Some(f) => f,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "message": "Partie introuvable" })),
            )
                .into_response()
        }
    };

    let game = match Game::from_fen(&fen) {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false })),
            )
                .into_response()
        }
    };

    let moves = if let Some(from_str) = &params.from {
        match Square::from_algebraic(from_str) {
            Some(sq) => game.legal_moves_from(sq),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "success": false, "message": "Case invalide" })),
                )
                    .into_response()
            }
        }
    } else {
        game.legal_moves()
    };

    let moves_json: Vec<String> = moves
        .iter()
        .map(|mv| {
            let promo = mv
                .promotion
                .map(|p| match p {
                    PieceType::Queen => "=q",
                    PieceType::Rook => "=r",
                    PieceType::Bishop => "=b",
                    PieceType::Knight => "=n",
                    _ => "",
                })
                .unwrap_or("");
            format!("{}{}{}", mv.from, mv.to, promo)
        })
        .collect();

    Json(moves_json).into_response()
}

// ════════════════════════════════════════════════════════════════
// INVITATIONS
// ════════════════════════════════════════════════════════════════

pub async fn invite_player(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
    Json(req): Json<InvitePlayerRequest>,
) -> impl IntoResponse {
    let created_by: Option<(String,)> =
        sqlx::query_as("SELECT created_by FROM chess_games WHERE id = ?")
            .bind(&game_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    match created_by {
        Some((creator,)) if creator == user.id => {}
        Some(_) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "success": false, "message": "Seul le créateur peut inviter" })),
            )
                .into_response()
        }
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false }))).into_response(),
    }

    let inv_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO chess_invitations (id, game_id, invited_user_id, slot, status, created_at) VALUES (?, ?, ?, 2, 'pending', ?)")
        .bind(&inv_id).bind(&game_id).bind(&req.user_id).bind(now)
        .execute(&state.db).await.ok();

    Json(json!({ "success": true, "invitation_id": inv_id })).into_response()
}

pub async fn my_invitations(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    let rows: Vec<(String, String, i32, String)> = sqlx::query_as(
        "SELECT id, game_id, slot, status FROM chess_invitations WHERE invited_user_id = ? AND status = 'pending' ORDER BY created_at DESC")
        .bind(&user.id).fetch_all(&state.db).await.unwrap_or_default();

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
        "SELECT game_id, slot FROM chess_invitations WHERE id = ? AND invited_user_id = ? AND status = 'pending'")
        .bind(&inv_id).bind(&user.id).fetch_optional(&state.db).await.ok().flatten();

    let (game_id, _slot) = match row {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "message": "Invitation introuvable" })),
            )
                .into_response()
        }
    };

    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE chess_games SET player2_id = ?, status = 'playing', updated_at = ? WHERE id = ?",
    )
    .bind(&user.id)
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

    Json(json!({ "success": true, "game_id": game_id })).into_response()
}

pub async fn decline_invitation(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(inv_id): Path<String>,
) -> impl IntoResponse {
    sqlx::query(
        "UPDATE chess_invitations SET status = 'declined' WHERE id = ? AND invited_user_id = ?",
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

// ─────────────────────────────────────────────────────────────────────────────
// PGN EXPORT
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/chess/{id}/pgn — Exporte une partie en format PGN
pub async fn export_pgn(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    // Récupérer les coups de la partie
    let moves = sqlx::query!("SELECT san FROM chess_moves WHERE game_id = ?", game_id)
        .fetch_all(&state.db)
        .await;
    
    match moves {
        Ok(rows) => {
            let mut pgn = String::new();
            
            // Header PGN
            pgn.push_str(&format!("[Result "*"]\r\n\r\n"));
            
            // Corps: numroter les coups
            for (i, row) in rows.iter().enumerate() {
                if i % 2 == 0 {
                    // Coup blanc
                    pgn.push_str(&format!("{}. {} ", (i / 2) + 1, row.san));
                } else {
                    // Coup noir
                    pgn.push_str(&format!("{} ", row.san));
                }
            }
            pgn.push_str("*\r\n");
            
            (StatusCode::OK, pgn).into_response()
        }
        Err(e) => {
            (StatusCode::NOT_FOUND, format!("Partie non trouvée: {}", e)).into_response()
        }
    }
}


pub fn chess_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/chess/create", post(create_game))
        .route("/chess/list", get(list_games))
        .route("/chess/invitations", get(my_invitations))
        .route("/chess/invitations/{id}/accept", post(accept_invitation))
        .route("/chess/invitations/{id}/decline", post(decline_invitation))
        .route("/chess/{id}", get(get_game))
        .route("/chess/{id}/join", post(join_game))
        .route("/chess/{id}/move", post(make_move))
        .route("/chess/{id}/ai-move", post(ai_move))
        .route("/chess/{id}/resign", post(resign_game))
        .route("/chess/{id}/moves", get(legal_moves))
        .route("/chess/{id}/invite", post(invite_player))
        .route("/chess/{id}/pgn", get(export_pgn))

}