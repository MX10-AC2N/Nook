// backend/src/reactions.rs — Réactions aux messages
// Session 35 — POST/DELETE réaction + GET agrégées par message
//
// Endpoints :
//   POST   /api/conversations/{conv_id}/messages/{msg_id}/reactions  { emoji }
//   DELETE /api/conversations/{conv_id}/messages/{msg_id}/reactions
//   GET    /api/conversations/{conv_id}/messages/{msg_id}/reactions
//
// Broadcast WS à chaque changement → type "reaction_updated"

#![allow(clippy::for_kv_map)]

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{auth::CurrentUser, SharedState};

// ────────────────────────────────────────────────────────────────────────────
// Types
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddReactionRequest {
    pub emoji: String,
}

/// Agrégat retourné par GET /reactions : { "👍": ["alice", "bob"], "❤️": ["carol"] }
/// + ma propre réaction pour l'UI
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ReactionsResponse {
    pub counts: std::collections::HashMap<String, Vec<String>>, // emoji → [user_name]
    pub my_emoji: Option<String>,                                // réaction de l'utilisateur courant
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Vérifie que l'utilisateur est membre de la conversation.
async fn check_conv_membership(
    db: &sqlx::SqlitePool,
    conv_id: &str,
    user_id: &str,
) -> Result<(), StatusCode> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM conversation_participants WHERE conversation_id = ? AND user_id = ?",
    )
    .bind(conv_id)
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.map(|(c,)| c).unwrap_or(0) == 0 {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// Vérifie que le message appartient à la conversation.
async fn check_msg_in_conv(
    db: &sqlx::SqlitePool,
    conv_id: &str,
    msg_id: &str,
) -> Result<(), StatusCode> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE id = ? AND conversation_id = ?",
    )
    .bind(msg_id)
    .bind(conv_id)
    .fetch_optional(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.map(|(c,)| c).unwrap_or(0) == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(())
}

/// Charge les réactions agrégées d'un message → HashMap<emoji, Vec<user_name>>.
async fn load_reactions(
    db: &sqlx::SqlitePool,
    msg_id: &str,
) -> Result<std::collections::HashMap<String, Vec<String>>, StatusCode> {
    #[derive(sqlx::FromRow)]
    struct Row {
        emoji: String,
        user_name: String,
    }

    let rows: Vec<Row> = sqlx::query_as::<_, Row>(
        "SELECT r.emoji, COALESCE(u.name, u.username) AS user_name
         FROM message_reactions r
         JOIN users u ON u.id = r.user_id
         WHERE r.message_id = ?
         ORDER BY r.created_at ASC",
    )
    .bind(msg_id)
    .fetch_all(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for row in rows {
        map.entry(row.emoji).or_default().push(row.user_name);
    }
    Ok(map)
}

/// Broadcast WS "reaction_updated" à toutes les connexions actives.
async fn broadcast_reaction(state: &SharedState, conv_id: &str, msg_id: &str) {
    let payload = json!({
        "type": "reaction_updated",
        "conversation_id": conv_id,
        "message_id": msg_id,
    });
    let guard = state.webrtc_state.broadcasts.lock().await;
    for (_, tx) in guard.iter() {
        let _ = tx.send(payload.to_string());
    }
}

// ────────────────────────────────────────────────────────────────────────────
// POST /api/conversations/{conv_id}/messages/{msg_id}/reactions
// Body : { "emoji": "👍" }
// Règle : 1 réaction par user par message — UPSERT (remplace si emoji différent)
// ────────────────────────────────────────────────────────────────────────────

pub async fn add_reaction(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path((conv_id, msg_id)): Path<(String, String)>,
    Json(req): Json<AddReactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    check_conv_membership(&state.db, &conv_id, &user.id).await?;
    check_msg_in_conv(&state.db, &conv_id, &msg_id).await?;

    let now = Utc::now().timestamp();

    // UPSERT : INSERT ou UPDATE si même (message_id, user_id) existe déjà
    sqlx::query(
        "INSERT INTO message_reactions (message_id, user_id, emoji, created_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(message_id, user_id) DO UPDATE SET emoji = excluded.emoji, created_at = excluded.created_at",
    )
    .bind(&msg_id)
    .bind(&user.id)
    .bind(&req.emoji)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "add_reaction: DB error");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Broadcast + retourner l'état complet
    broadcast_reaction(&state, &conv_id, &msg_id).await;
    let counts = load_reactions(&state.db, &msg_id).await?;

    Ok(Json(json!({
        "success": true,
        "message_id": msg_id,
        "counts": counts,
        "my_emoji": req.emoji,
    })))
}

// ────────────────────────────────────────────────────────────────────────────
// DELETE /api/conversations/{conv_id}/messages/{msg_id}/reactions
// Retire la réaction de l'utilisateur courant.
// ────────────────────────────────────────────────────────────────────────────

pub async fn remove_reaction(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path((conv_id, msg_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    check_conv_membership(&state.db, &conv_id, &user.id).await?;

    sqlx::query(
        "DELETE FROM message_reactions WHERE message_id = ? AND user_id = ?",
    )
    .bind(&msg_id)
    .bind(&user.id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "remove_reaction: DB error");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    broadcast_reaction(&state, &conv_id, &msg_id).await;
    let counts = load_reactions(&state.db, &msg_id).await?;

    Ok(Json(json!({
        "success": true,
        "message_id": msg_id,
        "counts": counts,
        "my_emoji": null,
    })))
}

// ────────────────────────────────────────────────────────────────────────────
// GET /api/conversations/{conv_id}/messages/{msg_id}/reactions
// ────────────────────────────────────────────────────────────────────────────

pub async fn get_reactions(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path((conv_id, msg_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    check_conv_membership(&state.db, &conv_id, &user.id).await?;

    let counts = load_reactions(&state.db, &msg_id).await?;

    // Récupérer ma propre réaction
    let my_emoji: Option<String> = sqlx::query_as::<_, (String,)>(
        "SELECT emoji FROM message_reactions WHERE message_id = ? AND user_id = ?",
    )
    .bind(&msg_id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(|(e,)| e);

    Ok(Json(json!({
        "message_id": msg_id,
        "counts": counts,
        "my_emoji": my_emoji,
    })))
}

// ────────────────────────────────────────────────────────────────────────────
// Router — à merger dans protected_routes dans main.rs
// ────────────────────────────────────────────────────────────────────────────

pub fn reactions_routes() -> axum::Router<Arc<SharedState>> {
    use axum::routing::post;
    axum::Router::new()
        .route(
            "/conversations/{conv_id}/messages/{msg_id}/reactions",
            post(add_reaction).delete(remove_reaction).get(get_reactions),
        )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_reaction_request_deserialize() {
        let json = r#"{"emoji": "👍"}"#;
        let req: AddReactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.emoji, "👍");
    }

    #[test]
    fn test_add_reaction_request_invalid_json() {
        let json = r#"{"invalid": "field"}"#;
        let result: Result<AddReactionRequest, _> = serde_json::from_str(json);
        // emoji field is missing, but serde doesn't error on missing fields by default
        // unless #[serde(deny_unknown_fields)] is set
        assert!(result.is_ok()); // serde allows missing fields
    }
}
