use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header::SET_COOKIE, HeaderMap, HeaderName},
    response::{Json, AppendHeaders},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;
use bcrypt::{verify, hash, DEFAULT_COST};
use chrono::{Utc, Duration};
use libsodium_sys as sodium;

use crate::SharedState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct AdminApproveRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
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
pub struct SessionData {
    pub user_id: String,
    pub username: String,
    pub name: String,
    pub role: String,
}

// === Inscription (demande d'approbation) ===
pub async fn register_handler(
    State(state): State<SharedState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    // Validation
    if payload.username.len() < 3 {
        return Ok(Json(ApiResponse {
            success: false,
            message: "L'identifiant doit contenir au moins 3 caractères".to_string(),
        }));
    }
    
    if payload.password.len() < 8 {
        return Ok(Json(ApiResponse {
            success: false,
            message: "Le mot de passe doit contenir au moins 8 caractères".to_string(),
        }));
    }
    
    // Vérifier si l'username existe déjà
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if existing.is_some() {
        return Ok(Json(ApiResponse {
            success: false,
            message: "Cet identifiant est déjà pris".to_string(),
        }));
    }
    
    // Hasher le mot de passe
    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Créer l'utilisateur (non approuvé)
    let user_id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO users (id, username, name, password_hash, role, approved, needs_password_change) 
         VALUES (?, ?, ?, ?, 'member', 0, 0)"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.name)
    .bind(&password_hash)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Demande d'inscription envoyée. En attente d'approbation par l'administrateur.".to_string(),
    }))
}

// === Connexion (admin ou utilisateur) ===
pub async fn login_handler(
    State(state): State<SharedState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Json<ApiResponse>), StatusCode> {
    // Chercher l'utilisateur
    let row: Option<(String, String, String, String, bool, bool)> = sqlx::query_as(
        "SELECT id, name, password_hash, role, approved, needs_password_change 
         FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    let (user_id, name, stored_hash, role, approved, needs_password_change) = row.unwrap();
    
    // Vérifier le mot de passe
    if !verify(&payload.password, &stored_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Vérifier l'approbation pour les membres
    if role == "member" && !approved {
        return Ok((
            AppendHeaders([]),
            Json(ApiResponse {
                success: false,
                message: "Compte en attente d'approbation par l'administrateur".to_string(),
            })
        ));
    }
    
    // Créer une session
    let session_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(30);
    
    sqlx::query(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&session_token)
    .bind(&user_id)
    .bind(expires_at.format("%Y-%m-%d %H:%M:%S").to_string())
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Déterminer le cookie (admin ou user)
    let cookie_name = if role == "admin" { "nook_admin" } else { "nook_session" };
    let cookie = format!(
        "{}={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=2592000",
        cookie_name, session_token
    );
    
    // Message différent selon le besoin de changement de mot de passe
    let message = if needs_password_change {
        "Première connexion. Veuillez changer votre mot de passe.".to_string()
    } else {
        format!("Connecté en tant que {}", name)
    };
    
    Ok((
        AppendHeaders([(SET_COOKIE, cookie)]),
        Json(ApiResponse {
            success: true,
            message,
        })
    ))
}

// === Validation de session ===
pub async fn validate_session_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> Result<Json<SessionData>, StatusCode> {
    // Essayer d'abord la session admin, puis user
    let token = get_cookie(&headers, "nook_admin")
        .or_else(|| get_cookie(&headers, "nook_session"));
    
    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Nettoyer les sessions expirées
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
        .bind(&now)
        .execute(&state.db)
        .await
        .ok();
    
    // Récupérer les infos utilisateur
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT u.id, u.username, u.name, u.role 
         FROM sessions s 
         JOIN users u ON s.user_id = u.id 
         WHERE s.token = ? AND u.approved = 1"
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match row {
        Some((user_id, username, name, role)) => {
            Ok(Json(SessionData {
                user_id,
                username,
                name,
                role,
            }))
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

// === Changement de mot de passe (première connexion) ===
pub async fn change_password_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    // Récupérer le token (admin ou user)
    let token = get_cookie(&headers, "nook_admin")
        .or_else(|| get_cookie(&headers, "nook_session"));
    
    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Récupérer l'utilisateur et son hash actuel
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT u.id, u.password_hash 
         FROM sessions s 
         JOIN users u ON s.user_id = u.id 
         WHERE s.token = ?"
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    let (user_id, current_hash) = row.unwrap();
    
    // Vérifier l'ancien mot de passe
    if !verify(&payload.current_password, &current_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Hasher le nouveau mot de passe
    let new_hash = hash(&payload.new_password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Mettre à jour
    sqlx::query(
        "UPDATE users SET password_hash = ?, needs_password_change = 0 WHERE id = ?"
    )
    .bind(&new_hash)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Mot de passe changé avec succès".to_string(),
    }))
}

// === Admin: Lister les demandes d'approbation ===
pub async fn pending_users_handler(
    State(state): State<SharedState>,
) -> Result<Json<Vec<UserInfo>>, StatusCode> {
    let rows = sqlx::query_as::<_, UserInfo>(
        "SELECT id, username, name, role, approved, needs_password_change 
         FROM users WHERE approved = 0 ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(rows))
}

// === Admin: Approuver un utilisateur ===
pub async fn approve_user_handler(
    State(state): State<SharedState>,
    Json(payload): Json<AdminApproveRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    sqlx::query("UPDATE users SET approved = 1 WHERE id = ?")
        .bind(&payload.user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Utilisateur approuvé".to_string(),
    }))
}

// === Admin: Liste des utilisateurs ===
pub async fn all_users_handler(
    State(state): State<SharedState>,
) -> Result<Json<Vec<UserInfo>>, StatusCode> {
    let rows = sqlx::query_as::<_, UserInfo>(
        "SELECT id, username, name, role, approved, needs_password_change 
         FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(rows))
}

// === Déconnexion ===
pub async fn logout_handler(
    headers: HeaderMap,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Json<ApiResponse>), StatusCode> {
    // Effacer les cookies admin et user
    let admin_cookie = "nook_admin=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0".to_string();
    let user_cookie = "nook_session=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0".to_string();
    
    Ok((
        AppendHeaders([(SET_COOKIE, admin_cookie), (SET_COOKIE, user_cookie)]),
        Json(ApiResponse {
            success: true,
            message: "Déconnexion réussie".to_string(),
        })
    ))
}

// === Utilitaires ===
fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get_all(axum::http::header::COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(|cookie| cookie.trim())
        .find(|cookie| cookie.starts_with(&format!("{}=", name)))
        .and_then(|cookie| cookie.split('=').nth(1).map(|s| s.to_string()))
}

// Types SQLx pour UserInfo
#[derive(sqlx::FromRow)]
struct UserInfoRow {
    id: String,
    username: String,
    name: String,
    role: String,
    approved: bool,
    needs_password_change: bool,
}
