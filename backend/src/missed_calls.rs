// backend/src/missed_calls.rs
// Gestion des appels manqués (déclinés ou non décrochés)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Extension, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::SharedState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MissedCallWithNames {
    pub id: String,
    pub conversation_id: String,
    pub caller_id: String,
    pub caller_name: String,
    pub callee_id: String,
    pub callee_name: String,
    pub call_type: String,
    pub status: String,
    pub created_at: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Routes
// ─────────────────────────────────────────────────────────────────────────────

pub fn missed_calls_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/missed-calls", get(get_missed_calls))
        .route("/missed-calls/{conversation_id}", get(get_conversation_missed_calls))
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// Récupérer les appels manqués de l'utilisateur courant
async fn get_missed_calls(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<Vec<MissedCallWithNames>>, StatusCode> {
    let calls = sqlx::query_as::<_, MissedCallWithNames>(
        r#"SELECT mc.id, mc.conversation_id,
                  mc.caller_id,
                  COALESCE(u1.name, u1.username) as caller_name,
                  mc.callee_id,
                  COALESCE(u2.name, u2.username) as callee_name,
                  mc.call_type, mc.status, mc.created_at
           FROM missed_calls mc
           JOIN users u1 ON u1.id = mc.caller_id
           JOIN users u2 ON u2.id = mc.callee_id
           WHERE mc.callee_id = ?
           ORDER BY mc.created_at DESC
           LIMIT 50"#,
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Erreur get_missed_calls");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(calls))
}

/// Récupérer les appels manqués d'une conversation
async fn get_conversation_missed_calls(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conversation_id): Path<String>,
) -> Result<Json<Vec<MissedCallWithNames>>, StatusCode> {
    // Vérifier que l'utilisateur est participant
    let is_participant: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM conversation_participants WHERE conversation_id = ? AND user_id = ?)"
    )
    .bind(&conversation_id)
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

    let calls = sqlx::query_as::<_, MissedCallWithNames>(
        r#"SELECT mc.id, mc.conversation_id,
                  mc.caller_id,
                  COALESCE(u1.name, u1.username) as caller_name,
                  mc.callee_id,
                  COALESCE(u2.name, u2.username) as callee_name,
                  mc.call_type, mc.status, mc.created_at
           FROM missed_calls mc
           JOIN users u1 ON u1.id = mc.caller_id
           JOIN users u2 ON u2.id = mc.callee_id
           WHERE mc.conversation_id = ?
           ORDER BY mc.created_at DESC
           LIMIT 50"#,
    )
    .bind(&conversation_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Erreur get_conversation_missed_calls");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(calls))
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers (appelés depuis webrtc.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// Enregistrer un appel manqué

#[allow(dead_code)]
pub async fn record_missed_call(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
    caller_id: &str,
    callee_id: &str,
    call_type: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    sqlx::query(
        r#"INSERT INTO missed_calls (id, conversation_id, caller_id, callee_id, call_type, status, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(caller_id)
    .bind(callee_id)
    .bind(call_type)
    .bind(status)
    .bind(now)
    .execute(pool)
    .await?;

    tracing::info!(
        call_type = call_type,
        status = status,
        caller_id = caller_id,
        callee_id = callee_id,
        "Appel manqué enregistré"
    );

    Ok(())
}
