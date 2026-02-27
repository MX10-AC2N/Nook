// backend/src/admin.rs - Gestion admin avec Extension<CurrentUser>
// Session 15 — FIX: approve_user insère aussi le user dans conversation_participants
//               Cause : un user inscrit via /api/auth/register n'était jamais ajouté
//               à default_global → GET /api/conversations retournait [] après approbation

use crate::{auth::CurrentUser, SharedState};
use axum::{
    extract::State as AxumState,
    http::StatusCode,
    Json,
    Extension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// ====================== STRUCTURES ======================
#[derive(Serialize)]
pub struct UsersResponse {
    pub users: Vec<SimpleUser>,
}

#[derive(Serialize)]
pub struct SimpleUser {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub role: String,
    pub approved: bool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct InviteInfo {
    pub id: String,
    pub token: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub used: bool,
    pub used_by: Option<String>,
    pub used_at: Option<i64>,
}

#[derive(Serialize)]
pub struct InvitesResponse {
    pub invites: Vec<InviteInfo>,
}

// ====================== PAYLOADS ======================
#[derive(Deserialize)]
pub struct ApprovePayload {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct DeleteInvitePayload {
    pub id: String,
}

// ====================== HANDLERS (avec CurrentUser) ======================

pub async fn pending_users(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<UsersResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let users: Vec<SimpleUser> = sqlx::query_as(
        "SELECT id, username, name, created_at, role, approved FROM users WHERE approved = 0 ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Erreur DB"}))))?
    .into_iter()
    .map(|u: (String, String, Option<String>, i64, String, bool)| SimpleUser {
        id: u.0,
        username: u.1,
        name: u.2,
        created_at: u.3,
        role: u.4,
        approved: u.5,
    })
    .collect();

    Ok(Json(UsersResponse { users }))
}

pub async fn all_users(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<UsersResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let users: Vec<SimpleUser> = sqlx::query_as(
        "SELECT id, username, name, created_at, role, approved FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Erreur DB"}))))?
    .into_iter()
    .map(|u: (String, String, Option<String>, i64, String, bool)| SimpleUser {
        id: u.0,
        username: u.1,
        name: u.2,
        created_at: u.3,
        role: u.4,
        approved: u.5,
    })
    .collect();

    Ok(Json(UsersResponse { users }))
}

pub async fn approve_user(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(payload): Json<ApprovePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let result = sqlx::query("UPDATE users SET approved = 1 WHERE id = ?")
        .bind(&payload.user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 1 => {
            // FIX session 15 : ajouter l'utilisateur approuvé à default_global
            // INSERT OR IGNORE → safe si déjà participant (re-approbation, double clic, etc.)
            let now = chrono::Utc::now().timestamp();
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
                 VALUES ('default_global', ?, ?)",
            )
            .bind(&payload.user_id)
            .bind(now)
            .execute(&state.db)
            .await;

            tracing::info!(
                user_id = %payload.user_id,
                "✓ Utilisateur approuvé et ajouté à default_global"
            );

            Ok(Json(json!({"success": true})))
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "message": "Utilisateur non trouvé"})),
        )),
    }
}

pub async fn list_invites(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<InvitesResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let invites: Vec<InviteInfo> = sqlx::query_as(
        "SELECT id, token, created_at, expires_at, used, used_by, used_at FROM invites ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Erreur DB"}))))?;

    Ok(Json(InvitesResponse { invites }))
}

pub async fn delete_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(payload): Json<DeleteInvitePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let result = sqlx::query("DELETE FROM invites WHERE id = ?")
        .bind(&payload.id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 1 => Ok(Json(json!({"success": true}))),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "message": "Invitation non trouvée"})),
        )),
    }
}
