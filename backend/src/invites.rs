// backend/src/invites.rs
// Gestion des invitations — schéma corrigé (session 9)
// Colonnes : id, token, created_by, created_at, expires_at, used, used_by, used_at

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
use http::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ====================== STRUCTURES ======================

#[derive(Deserialize)]
pub struct JoinPayload {
    pub name: String,
    pub public_key: String, // Clé publique pour E2EE
}


#[derive(Deserialize)]
pub struct AcceptInvitePayload {
    pub token:    String,
    pub username: String,
    pub name:     String,
    pub password: String,
}

#[derive(Serialize)]
pub struct JoinResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

// ====================== UTILITAIRE COOKIE ======================

/// Même logique que auth.rs : adapte SameSite selon le contexte HTTP/HTTPS
fn is_https(headers: &HeaderMap) -> bool {
    headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "https")
        .unwrap_or(false)
}

fn build_set_cookie(user_id: &str, token: &str, https: bool, max_age: i64) -> String {
    if https {
        format!(
            "auth_token={}:{}; Path=/; HttpOnly; SameSite=None; Secure; Max-Age={}",
            user_id, token, max_age
        )
    } else {
        format!(
            "auth_token={}:{}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            user_id, token, max_age
        )
    }
}

// ====================== HANDLERS ======================

/// Valider un token d'invitation (sans l'utiliser)
pub async fn validate_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let token = match params.get("token") {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Token manquant" })),
            )
                .into_response();
        }
    };

    let now = Utc::now().timestamp();

    // Schéma corrigé : colonnes id, used, expires_at, created_by, token
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

            let hours_left = (expires_at - now) / 3600;
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

/// Créer un token d'invitation (ADMIN ONLY, 48h)
pub async fn generate_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extraction et vérification du cookie auth_token
    let (user_id, session_token) = match headers
        .get("cookie")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(';').find(|c| c.trim().starts_with("auth_token=")))
        .and_then(|c| {
            // splitn(2) pour ne splitter qu'une fois (au cas où le token contiendrait ':')
            c.trim().strip_prefix("auth_token=")
        })
        .and_then(|v| {
            let mut parts = v.splitn(2, ':');
            let uid = parts.next()?.to_string();
            let tok = parts.next()?.to_string();
            if uid.is_empty() || tok.is_empty() {
                None
            } else {
                Some((uid, tok))
            }
        }) {
        Some(t) => t,
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "success": false, "message": "Non authentifié" })),
            )
                .into_response();
        }
    };

    // Vérification rôle admin
    let role: Option<(String,)> =
        sqlx::query_as("SELECT role FROM users WHERE id = ? AND token = ?")
            .bind(&user_id)
            .bind(&session_token)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    if role.as_ref().map(|(r,)| r.as_str()) != Some("admin") {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "success": false, "message": "Accès refusé : admin requis" })),
        )
            .into_response();
    }

    // Génération invite
    let invite_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let expires_at = now + (48 * 3600);

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
            tracing::info!(
                invite_id = %invite_id,
                created_by = %user_id,
                expires_in_hours = 48,
                "Invitation créée"
            );
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": "Invitation créée (expire dans 48h)",
                    "invite_link": format!("/invite?token={}", token),
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Erreur génération invite");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur serveur" })),
            )
                .into_response()
        }
    }
}

/// Rejoindre via token d'invitation
pub async fn join(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
    Json(payload): Json<JoinPayload>,
) -> impl IntoResponse {
    let token = match params.get("token") {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Token manquant" })),
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
                Json(json!({ "success": false, "message": "Token invalide" })),
            )
                .into_response();
        }
    };

    if used {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Token déjà utilisé" })),
        )
            .into_response();
    }
    if now > expires_at {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Token expiré" })),
        )
            .into_response();
    }

    // Créer l'utilisateur
    let user_id = Uuid::new_v4().to_string();
    let username = payload.name.to_lowercase().replace(' ', "_");
    let temp_password = &Uuid::new_v4().to_string()[..12];
    let hashed = hash_password(temp_password);
    let now_ts = Utc::now().timestamp();

    // INSERT avec public_key (colonne ajoutée par migration 002)
    let result = sqlx::query(
        r#"INSERT INTO users (
            id, username, email, password_hash, name, role, approved,
            needs_password_change, created_at, public_key
        ) VALUES (?, ?, ?, ?, ?, 'user', 1, 1, ?, ?)"#,
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
        tracing::error!(error = %e, username = %username, "Erreur création user via invite");
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "success": false,
                "message": "Nom d'utilisateur déjà pris",
            })),
        )
            .into_response();
    }

    // Marquer l'invite comme utilisée
    sqlx::query("UPDATE invites SET used = 1, used_by = ?, used_at = ? WHERE id = ?")
        .bind(&user_id)
        .bind(now_ts)
        .bind(&invite_id)
        .execute(&state.db)
        .await
        .ok();

    // Créer la session
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
        avatar_style: Some("adventurer".to_string()),
    };

    tracing::info!(user_id = %user_id, "Nouvel utilisateur créé via invitation");

    // Cookie adaptatif LAN/WAN (fix bug #3 : point-virgule + SameSite)
    let cookie = build_set_cookie(&user_id, &session_token, is_https(&headers), 86400);

    let mut response = Json(JoinResponse {
        success: true,
        message: "Compte créé ! Définis ton mot de passe dès maintenant.".to_string(),
        user: Some(user_info),
    })
    .into_response();

    response
        .headers_mut()
        .insert(SET_COOKIE, cookie.parse().unwrap());

    response
}

/// Accepter une invitation avec username + mot de passe choisi
/// POST /api/invite/accept  { token, username, name, password }
pub async fn accept_invite(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
    Json(payload): Json<AcceptInvitePayload>,
) -> impl IntoResponse {
    let now = Utc::now().timestamp();

    // Valider le token
    let invite: Option<(String, bool, i64)> =
        sqlx::query_as("SELECT id, used, expires_at FROM invites WHERE token = ?")
            .bind(&payload.token)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    let (invite_id, used, expires_at) = match invite {
        Some(row) => row,
        None => return (StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Token invalide" }))).into_response(),
    };
    if used {
        return (StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Token déjà utilisé" }))).into_response();
    }
    if now > expires_at {
        return (StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Token expiré" }))).into_response();
    }

    // Valider username
    let username = payload.username.trim().to_lowercase().replace(' ', "_");
    if username.is_empty() || username.len() < 2 {
        return (StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Identifiant trop court" }))).into_response();
    }

    // Valider mot de passe
    if payload.password.len() < 8 {
        return (StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Mot de passe trop court (8 caractères min)" }))).into_response();
    }

    let user_id    = Uuid::new_v4().to_string();
    let hashed     = hash_password(&payload.password);
    let public_key: Option<String> = None; // sera définie par le client au premier unlockCrypto()

    let result = sqlx::query(r#"INSERT INTO users (
        id, username, email, password_hash, name, role, approved,
        needs_password_change, created_at, public_key
    ) VALUES (?, ?, ?, ?, ?, 'user', 1, 0, ?, ?)"#)
        .bind(&user_id)
        .bind(&username)
        .bind(format!("{}@nook.local", username))
        .bind(&hashed)
        .bind(payload.name.trim())
        .bind(now)
        .bind(&public_key)
        .execute(&state.db)
        .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, username = %username, "accept_invite: username pris");
        return (StatusCode::CONFLICT,
            Json(json!({ "success": false, "message": "Cet identifiant est déjà pris" }))).into_response();
    }

    // Marquer l'invite utilisée
    sqlx::query("UPDATE invites SET used = 1, used_by = ?, used_at = ? WHERE id = ?")
        .bind(&user_id).bind(now).bind(&invite_id)
        .execute(&state.db).await.ok();

    // Ajouter dans default_global
    sqlx::query(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
         VALUES ('default_global', ?, ?)"
    ).bind(&user_id).bind(now).execute(&state.db).await.ok();

    // Créer session
    let session_token = Uuid::new_v4().to_string();
    sqlx::query("UPDATE users SET token = ? WHERE id = ?")
        .bind(&session_token).bind(&user_id)
        .execute(&state.db).await.ok();

    let user_info = UserInfo {
        id: user_id.clone(),
        username: username.clone(),
        name: payload.name.trim().to_string(),
        role: "user".to_string(),
        approved: true,
        needs_password_change: false,
        avatar_style: Some("adventurer".to_string()),
    };

    tracing::info!(user_id = %user_id, username = %username, "Nouvel utilisateur via accept_invite");

    let cookie = build_set_cookie(&user_id, &session_token, is_https(&headers), 86400);
    let mut response = Json(json!({
        "success": true,
        "message": "Compte créé ! Bienvenue sur Nook.",
        "user": {
            "id": user_info.id,
            "username": user_info.username,
            "name": user_info.name,
            "role": user_info.role,
        }
    })).into_response();
    response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
    response
}
