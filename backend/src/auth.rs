// backend/src/auth.rs - Axum 0.8 + rand 0.9 + rand_core 0.6 compatible

use crate::{db::User, SharedState};
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use axum::{
    body::Body,
    extract::{Extension, State as AxumState},
    http::{header::COOKIE, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use http::header::SET_COOKIE;
// OsRng depuis rand_core 0.6 — même version qu'attend argon2/password-hash.
// rand 0.9 utilise rand_core 0.9 qui est INCOMPATIBLE avec password-hash 0.5.
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ====================== STRUCTURES JSON ======================
#[derive(Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub email: String,
    pub name: String,
    pub invite_token: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordPayload {
    pub new_password: String,
    pub user_id: Option<String>,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub name: String,
    pub role: String,
    pub approved: bool,
    pub needs_password_change: bool,
    pub avatar_style: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

// ====================== UTILITAIRES ======================
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn verify_password(password: &str, hashed: &str) -> bool {
    let parsed = match PasswordHash::new(hashed) {
        Ok(p) => p,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// Construit la valeur Set-Cookie en adaptant SameSite selon le contexte.
/// - HTTPS (WAN via reverse proxy) → SameSite=None; Secure
/// - HTTP (LAN direct)            → SameSite=Lax
///
/// La détection se fait via le header X-Forwarded-Proto injecté par Nginx.
fn build_set_cookie(user_id: &str, token: &str, is_https: bool, max_age: i64) -> String {
    if is_https {
        // WAN via Nginx Proxy Manager avec TLS
        format!(
            "auth_token={}:{}; Path=/; HttpOnly; SameSite=None; Secure; Max-Age={}",
            user_id, token, max_age
        )
    } else {
        // LAN direct en HTTP
        format!(
            "auth_token={}:{}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            user_id, token, max_age
        )
    }
}

// ====================== HANDLERS PUBLICS ======================
pub async fn register(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    // FIX M1: validation cote serveur - minimum 8 caracteres
    if payload.password.trim().len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                success: false,
                message: "Le mot de passe doit contenir au moins 8 caracteres".to_string(),
                user: None,
            }),
        )
            .into_response();
    }

    let hashed_password = hash_password(&payload.password);
    let user_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();

    // Check if invite token is valid → auto-approve
    let auto_approve = if let Some(ref token) = payload.invite_token {
        let valid: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM invite_links WHERE token = ? AND expires_at > ?)"
        )
        .bind(token)
        .bind(created_at)
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);
        if valid {
            let _ = sqlx::query("DELETE FROM invite_links WHERE token = ?")
                .bind(token)
                .execute(&state.db)
                .await;
        }
        valid
    } else {
        false
    };

    let result = sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, name, role, approved, needs_password_change, created_at)\
         VALUES (?, ?, ?, ?, ?, 'user', ?, 0, ?)",
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&hashed_password)
    .bind(&payload.name)
    .bind(if auto_approve { 1i32 } else { 0i32 })
    .bind(created_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(AuthResponse {
            success: true,
            message: if auto_approve { "Inscription réussie !".to_string() } else { "Inscription réussie ! En attente d'approbation.".to_string() },
            user: None,
        })
        .into_response(),
        Err(_) => (
            StatusCode::CONFLICT,
            Json(AuthResponse {
                success: false,
                message: "Utilisateur existe déjà".to_string(),
                user: None,
            }),
        )
            .into_response(),
    }
}

pub async fn login(
    AxumState(state): AxumState<Arc<SharedState>>,
    // On extrait les headers pour détecter HTTPS (X-Forwarded-Proto de Nginx)
    headers: axum::http::HeaderMap,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    match user {
        Some(user) if user.approved && verify_password(&payload.password, &user.password_hash) => {
            let token = Uuid::new_v4().to_string();
            let _ = sqlx::query("UPDATE users SET token = ? WHERE id = ?")
                .bind(&token)
                .bind(&user.id)
                .execute(&state.db)
                .await;

            let user_info = UserInfo {
                id: user.id.clone(),
                username: user.username,
                name: user.name.unwrap_or_default(),
                role: user.role,
                approved: user.approved,
                needs_password_change: user.needs_password_change,
                avatar_style: user.avatar_style.clone(),
            };

            // Détection HTTPS via X-Forwarded-Proto (injecté par Nginx Proxy Manager)
            let is_https = headers
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                .map(|v| v == "https")
                .unwrap_or(false);

            let cookie = build_set_cookie(&user.id, &token, is_https, 86400);

            let mut response = Json(AuthResponse {
                success: true,
                message: "Connexion réussie".to_string(),
                user: Some(user_info),
            })
            .into_response();

            response
                .headers_mut()
                .insert(SET_COOKIE, cookie.parse().unwrap());
            response
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                success: false,
                message: "Identifiants incorrects ou compte non approuvé".to_string(),
                user: None,
            }),
        )
            .into_response(),
    }
}

// ====================== HANDLERS PROTÉGÉS ======================
#[derive(Clone)]
pub struct CurrentUser(pub User);

pub async fn me(Extension(CurrentUser(user)): Extension<CurrentUser>) -> impl IntoResponse {
    let user_info = UserInfo {
        id: user.id,
        username: user.username,
        name: user.name.unwrap_or_default(),
        role: user.role,
        approved: user.approved,
        needs_password_change: user.needs_password_change,
        avatar_style: user.avatar_style,
    };

    // Note : on ne retourne PAS le token ici — il est dans le cookie HttpOnly.
    // Le frontend n'a pas besoin de le connaître pour les requêtes API.
    Json(json!({
        "authenticated": true,
        "user": user_info
    }))
    .into_response()
}

pub async fn logout(
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE users SET token = NULL WHERE id = ?")
        .bind(&user.id)
        .execute(&state.db)
        .await;

    let is_https = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "https")
        .unwrap_or(false);

    // Max-Age=0 supprime le cookie, SameSite doit correspondre à celui de login
    let cookie = if is_https {
        "auth_token=; Path=/; HttpOnly; SameSite=None; Secure; Max-Age=0".to_string()
    } else {
        "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string()
    };

    let mut response = Json(json!({"success": true})).into_response();
    response
        .headers_mut()
        .insert(SET_COOKIE, cookie.parse().unwrap());
    response
}

pub async fn change_password(
    Extension(CurrentUser(current_user)): Extension<CurrentUser>,
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    // FIX C1: seul un admin ou le proprietaire du compte peut changer le mot de passe
    let target_id = payload.user_id.unwrap_or_else(|| current_user.id.clone());
    if current_user.role != "admin" && target_id != current_user.id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Permission refusee"})),
        )
            .into_response();
    }

    // Validation minimale de la force du mot de passe (cote API egalement)
    if payload.new_password.trim().len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "message": "Le mot de passe doit contenir au moins 8 caracteres"})),
        )
            .into_response();
    }

    let hashed = hash_password(&payload.new_password);
    let new_token = Uuid::new_v4().to_string();

    let result = sqlx::query(
        "UPDATE users SET password_hash = ?, needs_password_change = 0, token = ? WHERE id = ?",
    )
    .bind(&hashed)
    .bind(&new_token)
    .bind(&target_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let is_https = headers
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                .map(|v| v == "https")
                .unwrap_or(false);

            let cookie = build_set_cookie(&target_id, &new_token, is_https, 86400);

            let mut response = (
                StatusCode::OK,
                Json(json!({"success": true, "message": "Mot de passe changé"})),
            )
                .into_response();
            response
                .headers_mut()
                .insert(SET_COOKIE, cookie.parse().unwrap());
            response
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": false, "message": "Erreur lors du changement"})),
        )
            .into_response(),
    }
}

// ====================== MIDDLEWARE AUTH ======================
pub async fn require_auth(
    AxumState(state): AxumState<Arc<SharedState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let headers = req.headers();

    if let Some(cookie) = headers.get(COOKIE) {
        if let Ok(cookie_str) = cookie.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                if let Some(token_part) = part.strip_prefix("auth_token=") {
                    let parts: Vec<&str> = token_part.split(':').collect();
                    if parts.len() == 2 {
                        let user_id = parts[0];
                        let token = parts[1];

                        let user: Option<User> = sqlx::query_as(
                            "SELECT * FROM users WHERE id = ? AND token = ? AND approved = 1 LIMIT 1",
                        )
                        .bind(user_id)
                        .bind(token)
                        .fetch_optional(&state.db)
                        .await
                        .unwrap_or(None);

                        if let Some(user) = user {
                            req.extensions_mut().insert(CurrentUser(user));
                            return next.run(req).await;
                        }
                    }
                }
            }
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"success": false, "message": "Non authentifié"})),
    )
        .into_response()
}

// ──────────────────────────────────────────────────────────────
// Middleware Admin : seul l'admin peut passer
// ──────────────────────────────────────────────────────────────
pub async fn require_admin(
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if user.role != "admin" {
        tracing::warn!(
            user_id = %user.id,
            username = %user.username,
            "Tentative d'accès admin refusée (non-admin)"
        );
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        )
            .into_response();
    }

    next.run(req).await
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_creates_hash() {
        let hash = hash_password("testpassword123");
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"), "Should use Argon2id");
    }

    #[test]
    fn test_hash_password_unique_salt() {
        let hash1 = hash_password("samepassword");
        let hash2 = hash_password("samepassword");
        assert_ne!(hash1, hash2, "Each hash should have unique salt");
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "MySecurePass2026!";
        let hash = hash_password(password);
        assert!(verify_password(password, &hash));
    }

    #[test]
    fn test_verify_password_wrong() {
        let hash = hash_password("correctpassword");
        assert!(!verify_password("wrongpassword", &hash));
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        assert!(!verify_password("anypassword", "not_a_valid_hash"));
    }

    #[test]
    fn test_build_cookie_http() {
        let cookie = build_set_cookie("user123", "token456", false, 86400);
        assert!(cookie.contains("auth_token=user123:token456"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(!cookie.contains("Secure"));
    }

    #[test]
    fn test_build_cookie_https() {
        let cookie = build_set_cookie("user123", "token456", true, 86400);
        assert!(cookie.contains("auth_token=user123:token456"));
        assert!(cookie.contains("SameSite=None"));
        assert!(cookie.contains("Secure"));
    }
}
