// backend/src/invites.rs

use crate::{db::User, SharedState, auth::{hash_password, UserInfo}}; // Réutilise hash_password et UserInfo
use axum::{
    extract::{State as AxumState, Query},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;

#[derive(Deserialize)]
pub struct JoinPayload {
    pub name: String,
    pub public_key: String, // Pour E2EE, stocke-la où tu veux (ex: nouvelle colonne users ou table keys)
}

#[derive(Serialize)]
pub struct JoinResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

#[derive(Deserialize)]
pub struct CreateInviteParams {
    pub token: Option<String>, // Optionnel : génère auto si absent
}

// Handler : Créer un token d'invitation (admin only)
pub async fn create_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap, // Pour vérifier auth admin
) -> impl axum::response::IntoResponse {
    // Vérifie que l'utilisateur est admin connecté (réutilise logique de me)
    // ... (copie/adapte la vérification cookie + role == "admin" depuis auth::me)

    let token = Uuid::new_v4().to_string();
    let admin_id = "ID_DE_L_ADMIN"; // Récupère depuis cookie

    let now = Utc::now().timestamp();
    sqlx::query("INSERT INTO invites (id, token, created_by, created_at) VALUES (?, ?, ?, ?)")
        .bind(Uuid::new_v4().to_string())
        .bind(&token)
        .bind(admin_id)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Json(json!({
        "success": true,
        "token": token,
        "invite_link": format!("https://ton-domaine.com/?invite={}", token) // Adapte ton domaine
    }))
}

// Handler : Rejoindre via token
pub async fn join(
    AxumState(state): AxumState<Arc<SharedState>>,
    Query(params): Query<serde_aux::QueryMap>, // Pour ?token=...
    Json(payload): Json<JoinPayload>,
) -> impl axum::response::IntoResponse {
    let token = match params.get("token") {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token manquant"}))).into_response(),
    };

    // Vérifie le token
    let invite: Option<(String, i64)> = sqlx::query_as("SELECT id, used FROM invites WHERE token = ?")
        .bind(token)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let (invite_id, used) = match invite {
        Some(row) => row,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token invalide"}))).into_response(),
    };

    if used == 1 {
        return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token déjà utilisé"}))).into_response();
    }

    // Crée l'utilisateur (auto-approved)
    let user_id = Uuid::new_v4().to_string();
    let username = payload.name.to_lowercase().replace(" ", ""); // Simple génération username
    let default_password = Uuid::new_v4().to_string()[0..12].to_string(); // Random temp password
    let hashed = hash_password(&default_password);

    let now = Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, name, role, approved, needs_password_change, created_at)
         VALUES (?, ?, ?, ?, ?, 'user', 1, 1, ?)" // approved=1, force change password
    )
    .bind(&user_id)
    .bind(&username)
    .bind(format!("{}@nook.local", username)) // Email temp
    .bind(hashed)
    .bind(&payload.name)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::CONFLICT)?;

    // Marque le token utilisé
    sqlx::query("UPDATE invites SET used = 1, used_by = ?, used_at = ? WHERE id = ?")
        .bind(&user_id)
        .bind(now)
        .bind(&invite_id)
        .execute(&state.db)
        .await
        .ok();

    // TODO : Stocke public_key (ajoute colonne users.public_key TEXT ou table séparée)

    // Login auto + cookie (comme dans login)
    // ... (génère token, set cookie, retourne user info)

    Json(JoinResponse {
        success: true,
        message: "Bienvenue ! Change ton mot de passe dès la première connexion.".to_string(),
        user: Some(UserInfo { /* rempli */ }),
    }).into_response()
}
