// backend/src/invites.rs - Gestion des invitations (génération + join, single-use, expiration 48h)

use crate::{
    auth::{hash_password, UserInfo},
    SharedState,
};
use axum::{
    extract::{Query, State as AxumState},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct JoinPayload {
    pub name: String,
    pub public_key: String, // Clé publique pour E2EE
}

#[derive(Serialize)]
pub struct JoinResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

// Handler : Valider un token d'invitation (sans l'utiliser)
pub async fn validate_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let token = match params.get("token") {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "message": "Token manquant"
                })),
            )
                .into_response();
        }
    };

    let now = Utc::now().timestamp();

    let invite: Option<(String, bool, i64, String)> =
        sqlx::query_as("SELECT id, used, expires_at, created_by FROM invites WHERE token = ?")
            .bind(token)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    match invite {
        Some((_id, used, expires_at, created_by)) => {
            if used {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "message": "Token déjà utilisé",
                        "valid": false
                    })),
                )
                    .into_response();
            }

            if now > expires_at {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "message": "Token expiré",
                        "valid": false
                    })),
                )
                    .into_response();
            }

            // Calculer le temps restant
            let remaining = expires_at - now;
            let hours_left = remaining / 3600;

            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "valid": true,
                    "message": "Token valide",
                    "hours_remaining": hours_left,
                    "created_by": created_by
                })),
            )
                .into_response()
        }
        None => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Token invalide",
                "valid": false
            })),
        )
            .into_response(),
    }
}

// Handler : Créer un token d'invitation (ADMIN ONLY, 48h expiration)
pub async fn generate_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extraction et vérification du cookie auth_token
    let (user_id, session_token) = match headers
        .get("cookie")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(';').find(|c| c.trim().starts_with("auth_token=")))
        .and_then(|c| c.split('=').nth(1))
        .and_then(|v| {
            let parts: Vec<&str> = v.split(':').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        }) {
        Some(t) => t,
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "success": false,
                    "message": "Non authentifié"
                })),
            )
                .into_response();
        }
    };

    // Vérification role admin
    let role: Option<String> = sqlx::query_as("SELECT role FROM users WHERE id = ? AND token = ?")
        .bind(&user_id)
        .bind(&session_token)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(r,): (String,)| r);

    if role != Some("admin".to_string()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "message": "Accès refusé : admin requis"
            })),
        )
            .into_response();
    }

    // Génération invite
    let token = Uuid::new_v4().to_string();
    let invite_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let expires_at = now + (48 * 3600); // 48 heures

    let result = sqlx::query(
        "INSERT INTO invites (id, token, created_by, created_at, expires_at) VALUES (?, ?, ?, ?, ?)",
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
            let invite_link = format!("/invite?token={}", token);
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": "Invitation créée (expire dans 48h)",
                    "invite_link": invite_link,
                })),
            )
                .into_response()
        }
        Err(e) => {
            eprintln!("[Invites] Erreur génération invite : {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": "Erreur serveur",
                })),
            )
                .into_response()
        }
    }
}

// Handler : Rejoindre via token d'invitation
pub async fn join(
    AxumState(state): AxumState<Arc<SharedState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    Json(payload): Json<JoinPayload>,
) -> impl IntoResponse {
    let token = match params.get("token") {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "message": "Token manquant",
                })),
            )
                .into_response();
        }
    };

    let now = Utc::now().timestamp();

    let invite: Option<(String, bool, i64)> =
        sqlx::query_as("SELECT id, used, expires_at FROM invites WHERE token = ?")
            .bind(token)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    let (invite_id, used, expires_at) = match invite {
        Some(row) => row,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "message": "Token invalide",
                })),
            )
                .into_response();
        }
    };

    if used {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Token déjà utilisé",
            })),
        )
            .into_response();
    }

    if now > expires_at {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Token expiré",
            })),
        )
            .into_response();
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
        "#,
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
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "success": false,
                "message": "Nom d'utilisateur déjà pris",
            })),
        )
            .into_response();
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
    })
    .into_response();

    response.headers_mut().insert(
        http::header::SET_COOKIE,
        format!(
            "auth_token={}:{}, Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
            user_id, session_token,
        )
        .parse()
        .unwrap(),
    );

    response
}
