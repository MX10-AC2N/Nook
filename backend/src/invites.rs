// backend/src/invites.rs - Gestion des invitations (single-use, expiration 48h)

use crate::{db::User, SharedState, auth::{hash_password, get_cookie, UserInfo}};
use axum::{
    extract::{State as AxumState, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

#[derive(Deserialize)]
pub struct JoinPayload {
    pub name: String,
    pub public_key: String,  // Clé publique pour E2EE
}

#[derive(Serialize)]
pub struct JoinResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

#[derive(Serialize)]
pub struct InviteResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub invite_link: Option<String>,
}

// Handler : Créer un token d'invitation (ADMIN ONLY, single-use, expire en 48h)
pub async fn generate_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> impl axum::response::IntoResponse {
    // Vérification auth + role admin
    let current_user: Option<User> = if let Some(cookie) = get_cookie(&headers, "auth_token") {
        let parts: Vec<&str> = cookie.split(':').collect();
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

    let user = match current_user {
        Some(u) if u.role == "admin" => u,
        _ => return (StatusCode::FORBIDDEN, Json(json!({"success": false, "message": "Accès refusé : admin requis"}))).into_response(),
    };

    // Générer token unique
    let token = Uuid::new_v4().to_string();
    let invite_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let expires_at = now + (48 * 3600); // 48 heures en secondes

    let result = sqlx::query(
        "INSERT INTO invites (id, token, created_by, created_at, expires_at) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&invite_id)
    .bind(&token)
    .bind(&user.id)
    .bind(now)
    .bind(expires_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let invite_link = format!("https://ton-domaine.com/?invite={}", token); // Adapte ton domaine réel
            Json(InviteResponse {
                success: true,
                message: "Invitation créée (expire dans 48h)".to_string(),
                token: Some(token),
                invite_link: Some(invite_link),
            }).into_response()
        }
        Err(e) => {
            eprintln!("[Invite] Erreur création : {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"success": false, "message": "Erreur serveur"}))).into_response()
        }
    }
}

// Handler : Rejoindre via token d'invitation
pub async fn join(
    AxumState(state): AxumState<Arc<SharedState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    Json(payload): Json<JoinPayload>,
) -> impl axum::response::IntoResponse {
    let token = match params.get("token") {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token manquant"}))).into_response(),
    };

    let now = Utc::now().timestamp();

    // Récupérer l'invite
    let invite: Option<(String, bool, i64)> = sqlx::query_as(
        "SELECT id, used, expires_at FROM invites WHERE token = ?"
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let (invite_id, used, expires_at) = match invite {
        Some(row) => row,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token invalide"}))).into_response(),
    };

    if used {
        return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token déjà utilisé"}))).into_response();
    }

    if now > expires_at {
        return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token expiré"}))).into_response();
    }

    // Créer l'utilisateur (auto-approved, force change password)
    let user_id = Uuid::new_v4().to_string();
    let username = payload.name.to_lowercase().replace(" ", "_"); // Exemple simple
    let temp_password = Uuid::new_v4().to_string()[..12].to_string(); // Mot de passe temporaire random
    let hashed = hash_password(&temp_password);

    let now_ts = Utc::now().timestamp();

    let result = sqlx::query(
        r#"
        INSERT INTO users (
            id, username, email, password_hash, name, role, approved, 
            needs_password_change, created_at, public_key
        ) VALUES (?, ?, ?, ?, ?, 'user', 1, 1, ?, ?)
        "#
    )
    .bind(&user_id)
    .bind(&username)
    .bind(format!("{}@nook.local", username)) // Email temporaire
    .bind(hashed)
    .bind(&payload.name)
    .bind(now_ts)
    .bind(&payload.public_key)  // Stockage de la clé publique
    .execute(&state.db)
    .await;

    if let Err(e) = result {
        eprintln!("[Join] Erreur création user : {}", e);
        return (StatusCode::CONFLICT, Json(json!({"success": false, "message": "Nom d'utilisateur déjà pris"}))).into_response();
    }

    // Marquer l'invite comme utilisée
    sqlx::query("UPDATE invites SET used = 1, used_by = ?, used_at = ? WHERE id = ?")
        .bind(&user_id)
        .bind(now_ts)
        .bind(&invite_id)
        .execute(&state.db)
        .await
        .ok();

    // Login automatique (génère token + cookie)
    let session_token = Uuid::new_v4().to_string();
    sqlx::query("UPDATE users SET token = ? WHERE id = ?")
        .bind(&session_token)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .ok();

    let user_info = UserInfo {
        id: user_id.clone(),
        username,
        name: payload.name,
        role: "user".to_string(),
        approved: true,
        needs_password_change: true,
    };

    let mut response = Json(JoinResponse {
        success: true,
        message: "Compte créé ! Change ton mot de passe dès maintenant.".to_string(),
        user: Some(user_info),
    }).into_response();

    // Set cookie HttpOnly
    response.headers_mut().insert(
        http::header::SET_COOKIE,
        format!("auth_token={}:{}, Path=/; HttpOnly; SameSite=Lax; Max-Age=86400", user_id, session_token)
            .parse()
            .unwrap(),
    );

    response
}
