// backend/src/search.rs
// Recherche de messages

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::CurrentUser;
use crate::db::MessageWithSender;
use crate::SharedState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub conversation_id: Option<String>,
    pub limit: Option<i64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Routes
// ─────────────────────────────────────────────────────────────────────────────

pub fn search_routes() -> Router<Arc<SharedState>> {
    Router::new().route("/search", get(search_messages))
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// Recherche de messages par contenu
/// GET /api/search?q=terme&conversation_id=xxx&limit=20
async fn search_messages(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<MessageWithSender>>, StatusCode> {
    // Valider la requête
    if params.q.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let search_term = format!("%{}%", params.q.trim());

    let messages = if let Some(conv_id) = &params.conversation_id {
        // Recherche dans une conversation spécifique
        // Vérifier que l'utilisateur est participant
        let is_participant: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM conversation_participants WHERE conversation_id = ? AND user_id = ?)"
        )
        .bind(conv_id)
        .bind(&user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Erreur vérification participant");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if !is_participant {
            return Err(StatusCode::FORBIDDEN);
        }

        sqlx::query_as::<_, MessageWithSender>(
            r#"SELECT m.id, m.conversation_id, m.sender_id,
                      COALESCE(u.name, u.username) as sender_name,
                      u.avatar_style as sender_avatar_style,
                      u.avatar_seed as sender_avatar_seed,
                      u.public_key as sender_public_key,
                      m.content, m.message_type, m.file_id, m.encrypted,
                      m.timestamp, m.created_at, m.edited_at
               FROM messages m
               JOIN users u ON u.id = m.sender_id
               WHERE m.conversation_id = ?
                 AND m.content LIKE ?
                 AND m.encrypted = 0
               ORDER BY m.created_at DESC
               LIMIT ?"#,
        )
        .bind(conv_id)
        .bind(&search_term)
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Erreur recherche messages");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        // Recherche dans toutes les conversations de l'utilisateur
        sqlx::query_as::<_, MessageWithSender>(
            r#"SELECT m.id, m.conversation_id, m.sender_id,
                      COALESCE(u.name, u.username) as sender_name,
                      u.avatar_style as sender_avatar_style,
                      u.avatar_seed as sender_avatar_seed,
                      u.public_key as sender_public_key,
                      m.content, m.message_type, m.file_id, m.encrypted,
                      m.timestamp, m.created_at, m.edited_at
               FROM messages m
               JOIN users u ON u.id = m.sender_id
               JOIN conversation_participants cp ON cp.conversation_id = m.conversation_id
               WHERE cp.user_id = ?
                 AND m.content LIKE ?
                 AND m.encrypted = 0
               ORDER BY m.created_at DESC
               LIMIT ?"#,
        )
        .bind(&user.id)
        .bind(&search_term)
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Erreur recherche messages");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    };

    Ok(Json(messages))
}
