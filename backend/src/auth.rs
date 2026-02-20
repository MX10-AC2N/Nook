// backend/src/auth.rs - Authentification avec validation admin

use crate::{db::User, SharedState};
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use axum::{
    extract::State as AxumState,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use http::header::SET_COOKIE;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// Structures JSON

#[derive(Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub email: String, // Ajouté pour table
    pub name: String,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ChangePasswordPayload {
    pub new_password: String,
    pub user_id: Option<String>, // Optionnel pour first-setup admin
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub name: String,
    pub role: String,
    pub approved: bool,
    pub needs_password_change: bool,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

// Utilitaires

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
        Err(_) => {
            eprintln!("[AUTH] Format de hachage invalide");
            return false;
        }
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|c| {
                c.trim()
                    .starts_with(&format!("{}=", name))
                    .then(|| c.split('=').nth(1).unwrap_or("").to_string())
            })
        })
}

// Handlers (gardés tels quels, ajustés pour colonnes)

pub async fn register(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.password);

    let user_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();

    let result = sqlx::query::<sqlx::Sqlite>(
        "INSERT INTO users (id, username, email, password_hash, name, role, approved, needs_password_change, created_at)
         VALUES (?, ?, ?, ?, ?, 'user', 0, 0, ?)"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&hashed_password)
    .bind(&payload.name)
    .bind(created_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(AuthResponse {
            success: true,
            message: "Inscription réussie! En attente d'approbation.".to_string(),
            user: None,
        })
        .into_response(),
        Err(_e) => (
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
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    // Exécuter la requête et stocker le résultat dans user_result
    let user_result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await;

    // Afficher le résultat de la requête
    eprintln!("[LOGIN] Requête utilisateur résultat: {:?}", user_result);

    // Gérer le résultat de la requête
    let user = match user_result {
        Ok(Some(u)) => Some(u),
        Ok(None) => None,
        Err(e) => {
            eprintln!("[LOGIN] Erreur SQL: {}", e);
            None
        }
    };

    match user {
        Some(user) => {
            eprintln!(
                "[LOGIN] Utilisateur trouvé : id={}, approved={}, role={}",
                user.id, user.approved, user.role
            );
            if !user.approved {
                eprintln!("[LOGIN] Échec : Compte non approuvé");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthResponse {
                        success: false,
                        message: "Compte en attente d'approbation".to_string(),
                        user: None,
                    }),
                )
                    .into_response();
            }

            eprintln!("[LOGIN] Vérification du mot de passe...");
            if verify_password(&payload.password, &user.password_hash) {
                eprintln!("[LOGIN] Mot de passe correct.");
                let token = Uuid::new_v4().to_string();
                eprintln!("[LOGIN] Génération du token : {}", token);
                let _ = sqlx::query::<sqlx::Sqlite>("UPDATE users SET token = ? WHERE id = ?")
                    .bind(&token)
                    .bind(&user.id)
                    .execute(&state.db)
                    .await;

                let user_info = UserInfo {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    name: user.name.unwrap_or_default(),
                    role: user.role,
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
                    format!(
                        "auth_token={}:{}, Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
                        user.id, token
                    )
                    .parse()
                    .unwrap(),
                );
                eprintln!("[LOGIN] Connexion réussie, cookie défini");
                response
            } else {
                eprintln!("[LOGIN] Échec : Mot de passe incorrect.");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthResponse {
                        success: false,
                        message: "Identifiants incorrects".to_string(),
                        user: None,
                    }),
                )
                    .into_response()
            }
        }
        None => {
            eprintln!("[LOGIN] Échec : Utilisateur non trouvé.");
            (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    success: false,
                    message: "Identifiants incorrects".to_string(),
                    user: None,
                }),
            )
                .into_response()
        }
    }
}

pub async fn me(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(cookie) = get_cookie(&headers, "auth_token") {
        let parts: Vec<&str> = cookie.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let token = parts[1];

            let user: Option<User> =
                sqlx::query_as("SELECT * FROM users WHERE id = ? AND token = ?")
                    .bind(user_id)
                    .bind(token)
                    .fetch_optional(&state.db)
                    .await
                    .ok()
                    .flatten();

            if let Some(user) = user {
                let user_info = UserInfo {
                    id: user.id,
                    username: user.username,
                    name: user.name.unwrap_or_default(),
                    role: user.role,
                    approved: user.approved,
                    needs_password_change: user.needs_password_change,
                };
                return Json(json!({
                    "authenticated": true,
                    "user": user_info
                }))
                .into_response();
            }
        }
    }
    Json(json!({
        "authenticated": false,
        "user": null
    }))
    .into_response()
}

pub async fn logout(
    headers: HeaderMap,
    AxumState(state): AxumState<Arc<SharedState>>,
) -> impl IntoResponse {
    if let Some(cookie) = get_cookie(&headers, "auth_token") {
        let parts: Vec<&str> = cookie.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let _ = sqlx::query::<sqlx::Sqlite>("UPDATE users SET token = NULL WHERE id = ?")
                .bind(user_id)
                .execute(&state.db)
                .await;
        }
    }

    let mut response = Json(json!({"success": true})).into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"
            .parse()
            .unwrap(),
    );
    response
}

// =====  mise en place de change_password. =====
pub async fn change_password(
    AxumState(state): AxumState<Arc<SharedState>>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    // Récupérer l'utilisateur courant depuis le cookie
    let current_user_id_opt: Option<String> =
        if let Some(cookie) = get_cookie(&headers, "auth_token") {
            let parts = cookie.split(':').collect::<Vec<&str>>();
            if parts.len() == 2 {
                Some(parts[0].to_string())
            } else {
                None
            }
        } else {
            None
        };

    // Vérifier que l'utilisateur est connecté
    let current_user_id = match current_user_id_opt {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"success": false, "message": "Non authentifié"})),
            )
                .into_response();
        }
    };

    // Déterminer l'ID cible (soit celui du payload, soit l'utilisateur courant)
    let target_id = if let Some(ref user_id) = payload.user_id {
        user_id.as_str()
    } else {
        current_user_id.as_str()
    };

    // Hasher le nouveau mot de passe
    let hashed = hash_password(&payload.new_password);

    // GÉNÉRER UN NOUVEAU TOKEN pour la session
    let new_token = Uuid::new_v4().to_string();

    // Mettre à jour dans la DB: mot de passe ET token
    let result = sqlx::query::<sqlx::Sqlite>(
        "UPDATE users SET password_hash = ?, needs_password_change = 0, token = ? WHERE id = ?",
    )
    .bind(&hashed)
    .bind(&new_token)
    .bind(target_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            // Mettre à jour le cookie avec le nouveau token
            let mut response = (
                StatusCode::OK,
                Json(json!({"success": true, "message": "Mot de passe changé"})),
            )
                .into_response();

            response.headers_mut().insert(
                SET_COOKIE,
                format!(
                    "auth_token={}:{}, Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
                    target_id, new_token
                )
                .parse()
                .unwrap(),
            );
            response
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": false, "message": "Erreur DB"})),
        )
            .into_response(),
    }
}

// =====================================================================
// MIDDLEWARE AUTH GLOBAL
// =====================================================================
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Request,
};
use std::sync::Arc;
use crate::SharedState;   // on va le rendre pub juste après

/// Middleware qui protège toutes les routes sensibles
pub async fn require_auth(
    State(state): State<Arc<SharedState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = req.headers();

    if let Some(cookie) = get_cookie(headers, "auth_token") {
        let parts: Vec<&str> = cookie.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let token = parts[1];

            // Même requête que dans ton me()
            let user: Option<crate::db::User> = sqlx::query_as(
                "SELECT * FROM users WHERE id = ? AND token = ? AND approved = 1"
            )
            .bind(user_id)
            .bind(token)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

            if user.is_some() {
                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

// Tu peux garder les handlers admin (pending_users, invites, etc.) si tu les routes dans main.rs plus tard.
