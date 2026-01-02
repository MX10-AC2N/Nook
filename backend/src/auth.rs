use crate::db::User;
use crate::SharedState;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{PasswordHash, SaltString};
use rand::rngs::OsRng;
use axum::body::Body;
use axum::extract::State as AxumState;
use axum::http::header::{HeaderMap, HeaderName, SET_COOKIE};
use axum::http::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};

// ============ Structures de données pour les APIs JSON ============

#[derive(Serialize, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordPayload {
    pub user_id: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub name: String,
    pub role: String,
    pub approved: bool,
    pub needs_password_change: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ApprovePayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionResponse {
    pub authenticated: bool,
    pub user: Option<UserInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub user: Option<UserInfo>,
}

// ============ Fonctions utilitaires de sécurité ============

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn verify_password(password: &str, hashed_password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hashed_password).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

// ============ Handlers JSON pour le frontend moderne ============

pub async fn login_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password, name, role, approved, needs_password_change, created_at, token, public_key, joined_at FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(user) => {
            if !user.approved {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthResponse {
                        success: false,
                        message: "Votre compte est en attente d'approbation.".to_string(),
                        user: None,
                    }),
                )
                    .into_response();
            }

            if verify_password(&payload.password, &user.password) {
                let token = Uuid::new_v4().to_string();
                let cookie_value = format!("{}:{}", user.id, token);

                let _ = sqlx::query("UPDATE users SET token = ? WHERE id = ?")
                    .bind(&token)
                    .bind(&user.id)
                    .execute(&state.db)
                    .await;

                let user_info = UserInfo {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    name: user.name.unwrap_or_default(),
                    role: user.role.unwrap_or_else(|| "user".to_string()),
                    approved: user.approved,
                    needs_password_change: user.needs_password_change,
                };

                let mut response = Json(AuthResponse {
                    success: true,
                    message: "Connexion réussie".to_string(),
                    user: Some(user_info),
                })
                .into_response();

                response.headers_mut().insert(
                    SET_COOKIE,
                    format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600", cookie_value)
                        .parse()
                        .unwrap(),
                );
                response
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthResponse {
                        success: false,
                        message: "Nom d'utilisateur ou mot de passe incorrect.".to_string(),
                        user: None,
                    }),
                )
                    .into_response()
            }
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                success: false,
                message: "Nom d'utilisateur ou mot de passe incorrect.".to_string(),
                user: None,
            }),
        )
            .into_response(),
    }
}

pub async fn register_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.password);

    let user_id = Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO users (id, username, password, name, role, approved, needs_password_change)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&hashed_password)
    .bind(&payload.name)
    .bind("user")
    .bind(false)
    .bind(true)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(AuthResponse {
            success: true,
            message: "Inscription réussie! En attente d'approbation de l'administrateur.".to_string(),
            user: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
                success: false,
                message: format!("Erreur lors de l'inscription: {}", e),
                user: None,
            }),
        )
            .into_response(),
    }
}

pub async fn validate_session_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie(req.headers(), "auth_token");

    match auth_cookie {
        Some(cookie_value) => {
            let parts: Vec<&str> = cookie_value.split(':').collect();
            if parts.len() == 2 {
                let user_id = parts[0];
                let token = parts[1];

                let user: Option<User> = sqlx::query_as(
                    "SELECT id, username, password, name, role, approved, needs_password_change, created_at, token, public_key, joined_at FROM users WHERE id = ? AND token = ?"
                )
                .bind(user_id)
                .bind(token)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();

                match user {
                    Some(user) => {
                        let user_info = UserInfo {
                            id: user.id.clone(),
                            username: user.username.clone(),
                            name: user.name.unwrap_or_default(),
                            role: user.role.unwrap_or_else(|| "user".to_string()),
                            approved: user.approved,
                            needs_password_change: user.needs_password_change,
                        };
                        Json(SessionResponse {
                            authenticated: true,
                            user: Some(user_info),
                        })
                        .into_response()
                    }
                    None => {
                        let mut response = Json(SessionResponse {
                            authenticated: false,
                            user: None,
                        })
                        .into_response();
                        response.headers_mut().insert(
                            HeaderName::from_static("set-cookie"),
                            "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
                        );
                        (StatusCode::UNAUTHORIZED, response).into_response()
                    }
                }
            } else {
                (StatusCode::UNAUTHORIZED, Json(SessionResponse {
                    authenticated: false,
                    user: None,
                }))
                .into_response()
            }
        }
        None => (StatusCode::UNAUTHORIZED, Json(SessionResponse {
            authenticated: false,
            user: None,
        }))
        .into_response(),
    }
}

pub async fn user_info_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie(req.headers(), "auth_token");

    match auth_cookie {
        Some(cookie_value) => {
            let parts: Vec<&str> = cookie_value.split(':').collect();
            if parts.len() == 2 {
                let user_id = parts[0];
                let token = parts[1];

                let user: Option<User> = sqlx::query_as(
                    "SELECT id, username, password, name, role, approved, needs_password_change, created_at, token, public_key, joined_at FROM users WHERE id = ? AND token = ?"
                )
                .bind(user_id)
                .bind(token)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();

                match user {
                    Some(user) => {
                        let user_info = UserInfo {
                            id: user.id.clone(),
                            username: user.username.clone(),
                            name: user.name.unwrap_or_default(),
                            role: user.role.unwrap_or_else(|| "user".to_string()),
                            approved: user.approved,
                            needs_password_change: user.needs_password_change,
                        };
                        Json(UserInfoResponse {
                            user: Some(user_info),
                        })
                        .into_response()
                    }
                    None => (StatusCode::UNAUTHORIZED, Json(UserInfoResponse { user: None })).into_response(),
                }
            } else {
                (StatusCode::UNAUTHORIZED, Json(UserInfoResponse { user: None })).into_response()
            }
        }
        None => (StatusCode::UNAUTHORIZED, Json(UserInfoResponse { user: None })).into_response(),
    }
}

pub async fn change_password_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie_from_user_id(&payload.user_id, &state.db).await;

    if auth_cookie.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                success: false,
                message: "Session invalide".to_string(),
                user: None,
            }),
        )
            .into_response();
    }

    let hashed_password = hash_password(&payload.new_password);

    let result = sqlx::query(
        "UPDATE users SET password = ?, needs_password_change = false WHERE id = ?"
    )
    .bind(&hashed_password)
    .bind(&payload.user_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(AuthResponse {
            success: true,
            message: "Mot de passe changé avec succès".to_string(),
            user: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
                success: false,
                message: format!("Erreur lors du changement de mot de passe: {}", e),
                user: None,
            }),
        )
            .into_response(),
    }
}

async fn get_cookie_from_user_id(user_id: &str, db: &sqlx::SqlitePool) -> Option<String> {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password, name, role, approved, needs_password_change, created_at, token, public_key, joined_at FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    user.and_then(|u| u.token.map(|t| format!("{}:{}", u.id, t)))
}

pub async fn logout_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie(req.headers(), "auth_token");
    if let Some(cookie_value) = auth_cookie {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let _ = sqlx::query("UPDATE users SET token = NULL WHERE id = ?")
                .bind(user_id)
                .execute(&state.db)
                .await;
        }
    }

    let mut response = Json(AuthResponse {
        success: true,
        message: "Déconnexion réussie".to_string(),
        user: None,
    })
    .into_response();

    response.headers_mut().insert(
        HeaderName::from_static("set-cookie"),
        "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );
    response
}

// ============ First setup (premier changement admin) en JSON ============

pub async fn first_setup_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let user_id = payload.user_id.clone();

    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password, name, role, approved, needs_password_change, created_at, token, public_key, joined_at 
         FROM users 
         WHERE id = ? AND needs_password_change = true"
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(_) => {
            let hashed_password = hash_password(&payload.new_password);

            let result = sqlx::query(
                "UPDATE users SET password = ?, needs_password_change = false WHERE id = ?"
            )
            .bind(&hashed_password)
            .bind(&user_id)
            .execute(&state.db)
            .await;

            match result {
                Ok(_) => Json(AuthResponse {
                    success: true,
                    message: "Mot de passe mis à jour avec succès.".to_string(),
                    user: None,
                })
                .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthResponse {
                        success: false,
                        message: format!("Erreur lors du changement : {}", e),
                        user: None,
                    }),
                )
                    .into_response(),
            }
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                success: false,
                message: "Action non autorisée ou session invalide.".to_string(),
                user: None,
            }),
        )
            .into_response(),
    }
}

// ============ Nouvelles routes admin JSON modernes ============

pub async fn pending_users_json_handler(AxumState(state): AxumState<Arc<SharedState>>) -> impl IntoResponse {
    let rows: Vec<(String, String, Option<String>, Option<String>, bool, bool, String)> = sqlx::query_as(
        "SELECT id, username, name, role, approved, needs_password_change, created_at FROM users WHERE approved = false"
    )
    .fetch_all(&state.db)
    .await
    .ok()
    .unwrap_or_default();

    let users: Vec<UserInfo> = rows.into_iter().map(|r| UserInfo {
        id: r.0,
        username: r.1,
        name: r.2.unwrap_or_default(),
        role: r.3.unwrap_or_else(|| "user".to_string()),
        approved: r.4,
        needs_password_change: r.5,
    }).collect();

    Json(json!({ "users": users }))
}

pub async fn all_users_json_handler(AxumState(state): AxumState<Arc<SharedState>>) -> impl IntoResponse {
    let rows: Vec<(String, String, Option<String>, Option<String>, bool, bool, String)> = sqlx::query_as(
        "SELECT id, username, name, role, approved, needs_password_change, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .ok()
    .unwrap_or_default();

    let users: Vec<UserInfo> = rows.into_iter().map(|r| UserInfo {
        id: r.0,
        username: r.1,
        name: r.2.unwrap_or_default(),
        role: r.3.unwrap_or_else(|| "user".to_string()),
        approved: r.4,
        needs_password_change: r.5,
    }).collect();

    Json(json!({ "users": users }))
}

pub async fn generate_invite_handler(AxumState(state): AxumState<Arc<SharedState>>) -> impl IntoResponse {
    // Nettoyage des invites expirées
    let _ = sqlx::query("DELETE FROM invites WHERE expires_at < datetime('now')")
        .execute(&state.db)
        .await;

    let token = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let expires_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO invites (id, token, created_at, expires_at, used) 
         VALUES (?, ?, ?, ?, false)"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&token)
    .bind(&created_at)
    .bind(&expires_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let invite_link = format!("https://ton-domaine.com/join?token={}", token);
            Json(json!({
                "success": true,
                "invite_link": invite_link,
                "expires_in_days": 7
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": format!("Erreur génération invite : {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn approve_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ApprovePayload>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE users SET approved = true WHERE id = ?")
        .bind(&payload.user_id)
        .execute(&state.db)
        .await;

    Json(AuthResponse {
        success: true,
        message: "Utilisateur approuvé avec succès".to_string(),
        user: None,
    })
}

pub fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .map(|c| c.trim())
                .find(|c| c.starts_with(&format!("{}=", name)))
                .and_then(|c| c.split_once('='))
                .map(|(_, value)| value.trim().to_string())
        })
}
