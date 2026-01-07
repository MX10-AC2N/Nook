// backend/src/admin.rs - Gestion admin : utilisateurs + invitations

use crate::{db::User, SharedState, auth::get_cookie};
use axum::{
    extract::State as AxumState,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// Structures pour réponses JSON
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

#[derive(Serialize)]
pub struct InvitesResponse {
    pub invites: Vec<InviteInfo>,
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

// Utilitaire : Récupère l'utilisateur courant + vérifie admin
async fn get_admin_user(state: &Arc<SharedState>, headers: &HeaderMap) -> Result<User, (StatusCode, Json<serde_json::Value>)> {
    let current_user: Option<User> = if let Some(cookie) = get_cookie(headers, "auth_token") {
        let parts = cookie.split(':').collect::<Vec<&str>>();
        if parts.len() == 2 {
            let user_id = parts[0];
            let token = parts[1];

            sqlx::query_as("SELECT * FROM users WHERE id = ? AND token = ?")
                .bind(user_id)
                .bind(token)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten()
        } else {
            None
        }
    } else {
        None
    };

    match current_user {
        Some(u) if u.role == "admin" => Ok(u),
        _ => Err((StatusCode::FORBIDDEN, Json(json!({"success": false, "message": "Accès refusé : admin requis"})))),
    }
}

// GET /api/pending-users-json
pub async fn pending_users(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> Result<Json<UsersResponse>, (StatusCode, Json<serde_json::Value>)> {
    get_admin_user(&state, &headers).await?;  // Guard admin

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

// GET /api/all-users-json
pub async fn all_users(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> Result<Json<UsersResponse>, (StatusCode, Json<serde_json::Value>)> {
    get_admin_user(&state, &headers).await?;  // Guard admin

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

// POST /api/approve { user_id }
#[derive(Deserialize)]
pub struct ApprovePayload {
    pub user_id: String,
}

pub async fn approve_user(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
    Json(payload): Json<ApprovePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    get_admin_user(&state, &headers).await?;  // Guard admin

    let result = sqlx::query("UPDATE users SET approved = 1 WHERE id = ?")
        .bind(&payload.user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 1 => Ok(Json(json!({"success": true}))),
        _ => Err((StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Utilisateur non trouvé"})))),
    }
}

// GET /api/list-invites
pub async fn list_invites(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> Result<Json<InvitesResponse>, (StatusCode, Json<serde_json::Value>)> {
    get_admin_user(&state, &headers).await?;  // Guard admin

    let invites: Vec<InviteInfo> = sqlx::query_as(
        "SELECT id, token, created_at, expires_at, used, used_by, used_at FROM invites ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Erreur DB"}))))?;

    Ok(Json(InvitesResponse { invites }))
}

// POST /api/delete-invite { id }
#[derive(Deserialize)]
pub struct DeleteInvitePayload {
    pub id: String,
}

pub async fn delete_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
    Json(payload): Json<DeleteInvitePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    get_admin_user(&state, &headers).await?;  // Guard admin

    // Optionnel : empêcher suppression si used, mais frontend désactive déjà

    let result = sqlx::query("DELETE FROM invites WHERE id = ?")
        .bind(&payload.id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 1 => Ok(Json(json!({"success": true}))),
        _ => Err((StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Invitation non trouvée"})))),
    }
}

