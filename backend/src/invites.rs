// backend/src/invites.rs - Gestion des invitations (génération + join, single-use, expiration 48h)

use crate::{SharedState, auth::{hash_password, UserInfo}};
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

// Handler : Créer un token d'invitation (ADMIN ONLY)
pub async fn generate_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Vérification admin (réutilise la logique de get_cookie depuis auth, mais comme get_cookie n'est plus importé, on duplique minimalement ou importe seulement ce qu'il faut)
    // Pour éviter unused, on implémente la vérification ici sans get_cookie si possible, ou importe seulement UserInfo + hash_password

    // Note : Pour simplifier et éviter unused imports, on suppose que tu as une fonction auth pour guard admin, mais ici on duplique légèrement
    let current_user: Option<(String, String)> = if let Some(cookie_header) = headers.get("cookie") {
        cookie_header.to_str().ok().and_then(|s| {
            s.split(';')
                .find_map(|c| c.trim().starts_with("auth_token=").then(|| {
                    let value = c.split('=').nth(1)?;
                    let parts: Vec<&str> = value.split(':').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                }))
        })
    } else {
        None
    };

    let (user_id, token) = match current_user {
        Some(t) => t,
        None => return (StatusCode::FORBIDDEN, Json(json!({"success": false, "message": "Non authentifié"}))).into_response(),
    };

    let user: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = ? AND token = ?")
        .bind(&user_id)
        .bind(&token)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    if user.map(|(role,)| role) != Some("admin".to_string()) {
        return (StatusCode::FORBIDDEN, Json(json!({"success": false, "message": "Accès refusé : admin requis"}))).into_response(),
    }

    let token = Uuid::new_v4().to_string();
    let invite_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let expires_at = now + (48 * 3600);

    let result = sqlx::query(
        "INSERT INTO invites (id, token, created_by, created_at, expires_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&invite_id)
    .bind(&token)
    .bind(&user_id)
    .bind(now)
    .bind(expires_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let invite_link = format!("https://ton-domaine.com/?invite={}", token);
            Json(json!({
                "success": true,
                "message": "Invitation créée (expire dans 48h)",
                "invite_link": invite_link
            })).into_response()
        }
        Err(e) => {
            eprintln!("[Invites] Erreur génération : {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"success": false, "message": "Erreur serveur"}))).into_response()
        }
    }
}

// Handler : Rejoindre via token (ton code existant, légèrement nettoyé)
pub async fn join(
    AxumState(state): AxumState<Arc<SharedState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    Json(payload): Json<JoinPayload>,
) -> impl IntoResponse {
    // Ton code join existant (inchangé, il est parfait)
    // ... (copie le code que tu as fourni)
    // Je le garde tel quel pour éviter redondance
    let token = match params.get("token") {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token manquant"}))).into_response(),
    };

    let now = Utc::now().timestamp();

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
        return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token déjà utilisé"}))).into_response(),
    }

    if now > expires_at {
        return (StatusCode::BAD_REQUEST, Json(json!({"success": false, "message": "Token expiré"}))).into_response(),
    }

    let user_id = Uuid::new_v4().to_string();
    let username = payload.name.to_lowercase().replace(" ", "_");
    let temp_password = Uuid::new_v4().to_string()[..12].to_string();
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
    .bind(format!("{}@nook.local", username))
    .bind(hashed)
    .bind(&payload.name)
    .bind(now_ts)
    .bind(&payload.public_key)
    .execute(&state.db)
    .await;

    if let Err(e) = result {
        eprintln!("[Join] Erreur création user : {}", e);
        return (StatusCode::CONFLICT, Json(json!({"success": false, "message": "Nom d'utilisateur déjà pris"}))).into_response();
    }

    sqlx::query("UPDATE invites SET used = 1, used_by = ?, used_at = ? WHERE id = ?")
        .bind(&user_id)
        .bind(now_ts)
        .bind(&invite_id)
        .execute(&state.db)
        .await
        .ok();

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

    response.headers_mut().insert(
        http::header::SET_COOKIE,
        format!("auth_token={}:{}, Path=/; HttpOnly; SameSite=Lax; Max-Age=86400", user_id, session_token)
            .parse()
            .unwrap(),
    );

    response
}
