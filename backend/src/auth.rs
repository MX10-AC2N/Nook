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

use crate::SharedState;

#[derive(Deserialize)]
pub struct JoinRequest {
    pub name: String,
    pub public_key: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub member_id: String,
}

#[derive(Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct SessionInfo {
    pub member_id: String,
    pub member_name: String,
}

#[derive(Serialize)]
pub struct FirstLoginCheck {
    pub needs_password_change: bool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub approved: bool,
    pub joined_at: String,
}

// === Logique métier ===

pub async fn create_invite(pool: &SqlitePool) -> Result<String, StatusCode> {
    let token = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO invites (token) VALUES (?)")
        .bind(&token)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(token)
}

pub async fn handle_join(
    pool: &SqlitePool,
    token: String,
    req: JoinRequest,
) -> Result<ApiResponse, StatusCode> {
    let row = sqlx::query("SELECT token FROM invites WHERE token = ?")
        .bind(&token)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("DELETE FROM invites WHERE token = ?")
        .bind(&token)
        .execute(pool)
        .await
        .ok();

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO members (id, name, public_key, approved) VALUES (?, ?, ?, 0)")
        .bind(&id)
        .bind(&req.name)
        .bind(&req.public_key)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ApiResponse {
        success: true,
        message: "Demande envoyée à l'administrateur".to_string(),
    })
}

pub async fn approve_member(pool: &SqlitePool, id: String) -> Result<ApiResponse, StatusCode> {
    sqlx::query("UPDATE members SET approved = 1 WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ApiResponse {
        success: true,
        message: "Membre approuvé".to_string(),
    })
}

pub async fn get_members(pool: &SqlitePool) -> Result<Vec<Member>, StatusCode> {
    let rows =
        sqlx::query_as::<_, Member>("SELECT id, name, approved, joined_at FROM members ORDER BY joined_at DESC")
            .fetch_all(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows)
}

// === Sessions utilisateurs ===

pub async fn create_session(pool: &SqlitePool, member_id: &str) -> Result<String, StatusCode> {
    let token = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO sessions (token, member_id) VALUES (?, ?)")
        .bind(&token)
        .bind(member_id)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(token)
}

pub async fn validate_session_and_get_user(
    pool: &SqlitePool,
    token: &str,
) -> Result<(String, String), StatusCode> {
    let cutoff = Utc::now() - Duration::days(30);
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();
    
    sqlx::query("DELETE FROM sessions WHERE created_at < ?")
        .bind(cutoff_str)
        .execute(pool)
        .await
        .ok();

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT s.member_id, m.name FROM sessions s 
         JOIN members m ON s.member_id = m.id 
         WHERE s.token = ? AND m.approved = 1"
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((member_id, member_name)) => Ok((member_id, member_name)),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

// === Sessions administrateurs ===

pub async fn validate_admin_session(
    pool: &SqlitePool,
    token: &str,
) -> Result<String, StatusCode> {
    let cutoff = Utc::now() - Duration::days(7);
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();
    
    sqlx::query("DELETE FROM admin_sessions WHERE created_at < ?")
        .bind(cutoff_str)
        .execute(pool)
        .await
        .ok();

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT admin_id FROM admin_sessions WHERE token = ?"
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((admin_id,)) => Ok(admin_id),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn create_admin_session(pool: &SqlitePool, admin_id: &str) -> Result<String, StatusCode> {
    let token = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO admin_sessions (token, admin_id) VALUES (?, ?)")
        .bind(&token)
        .bind(admin_id)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(token)
}

// === Handlers Axum ===

pub async fn invite_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse>, StatusCode> {
    match create_invite(&state.db).await {
        Ok(token) => {
            let protocol = headers
                .get("X-Forwarded-Proto")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("http")
                .to_string();

            let host = headers
                .get("X-Forwarded-Host")
                .or_else(|| headers.get("host"))
                .and_then(|h| h.to_str().ok())
                .unwrap_or("localhost:3000")
                .to_string();

            let join_url = format!("{}://{}/join?token={}", protocol, host, token);

            Ok(Json(ApiResponse {
                success: true,
                message: join_url,
            }))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn join_handler(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<JoinRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    if let Some(token) = params.get("token") {
        match handle_join(&state.db, token.clone(), payload).await {
            Ok(res) => Ok(Json(res)),
            Err(e) => Err(e),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn approve_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse>, StatusCode> {
    approve_member(&state.db, id).await.map(Json)
}

pub async fn members_handler(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let members = get_members(&state.db).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}

pub async fn login_handler(
    State(state): State<SharedState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Json<ApiResponse>), StatusCode> {
    let row: Option<(String,)> = sqlx::query_as("SELECT id FROM members WHERE id = ? AND approved = 1")
        .bind(&payload.member_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let session_token = create_session(&state.db, &payload.member_id).await?;

    let cookie = format!(
        "nook_session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=2592000",
        session_token
    );

    Ok((
        AppendHeaders([(SET_COOKIE, cookie)]),
        Json(ApiResponse {
            success: true,
            message: "Connexion réussie".to_string(),
        })
    ))
}

pub async fn admin_login_handler(
    State(state): State<SharedState>,
    Json(payload): Json<AdminLoginRequest>,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Json<ApiResponse>), StatusCode> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT id, password_hash FROM admins WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let (admin_id, stored_hash) = row.unwrap();

    if !verify(&payload.password, &stored_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let admin_token = create_admin_session(&state.db, &admin_id).await?;

    let cookie = format!(
        "nook_admin={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        admin_token
    );

    Ok((
        AppendHeaders([(SET_COOKIE, cookie)]),
        Json(ApiResponse {
            success: true,
            message: "Connexion admin réussie".to_string(),
        })
    ))
}

pub async fn admin_logout_handler(
    State(_state): State<SharedState>,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Json<ApiResponse>), StatusCode> {
    let cookie = "nook_admin=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0".to_string();

    Ok((
        AppendHeaders([(SET_COOKIE, cookie)]),
        Json(ApiResponse {
            success: true,
            message: "Déconnexion réussie".to_string(),
        })
    ))
}

// === NOUVEAU : Vérification première connexion ===

fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get_all(axum::http::header::COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(|cookie| cookie.trim())
        .find(|cookie| cookie.starts_with(&format!("{}=", name)))
        .and_then(|cookie| cookie.split('=').nth(1).map(|s| s.to_string()))
}

pub async fn check_first_login_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> Result<Json<FirstLoginCheck>, StatusCode> {
    let admin_token = match get_cookie(&headers, "nook_admin") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT a.password_hash FROM admin_sessions s 
         JOIN admins a ON s.admin_id = a.id 
         WHERE s.token = ?"
    )
    .bind(admin_token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let hash = row.unwrap().0;
    
    let needs_password_change = verify("admin123", &hash).unwrap_or(false);

    Ok(Json(FirstLoginCheck {
        needs_password_change,
    }))
}

// === NOUVEAU : Changement de mot de passe ===

pub async fn change_password_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let admin_token = match get_cookie(&headers, "nook_admin") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT a.id, a.password_hash FROM admin_sessions s 
         JOIN admins a ON s.admin_id = a.id 
         WHERE s.token = ?"
    )
    .bind(admin_token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let (admin_id, current_hash) = row.unwrap();

    if !verify(&payload.current_password, &current_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let new_hash = hash(&payload.new_password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE admins SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(&admin_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Mot de passe changé avec succès".to_string(),
    }))
}

// === Validation de session pour le frontend ===

pub async fn validate_session_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> Result<Json<SessionInfo>, StatusCode> {
    let session_token = match get_cookie(&headers, "nook_session") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match validate_session_and_get_user(&state.db, &session_token).await {
        Ok((member_id, member_name)) => Ok(Json(SessionInfo { member_id, member_name })),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
