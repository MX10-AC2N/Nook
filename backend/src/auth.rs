use crate::db::{get_pool, User};
use crate::State;
use argon2::{Argon2, PasswordHash};
use axum::body::Body;
use axum::extract::{Multipart, Path, State as AxumState};
use axum::http::header::{HeaderMap, HeaderName, SET_COOKIE};
use axum::http::{Request, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use futures_util::stream::BytesStream;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

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

#[derive(sqlx::FromRow)]
struct UserInfoSqlxRow {
    id: String,
    username: String,
    name: Option<String>,               // ← Option car potentiellement NULLABLE
    role: Option<String>,               // ← Option car DEFAULT mais peut être NULL
    approved: bool,
    needs_password_change: bool,
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

pub async fn register_handler(
    AxumState(state): AxumState<Arc<State>>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.password);
    let user_id = Uuid::new_v4().to_string();

    let _ = query!(
        "INSERT INTO users (id, username, password, name, role, approved, needs_password_change)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        user_id,
        payload.username,
        hashed_password,
        payload.name,
        "user".to_string(),
        false,
        true
    )
    .execute(&state.db)
    .await
    .ok();  // .ok() pour ignorer les erreurs DB en dev

    Html(
        "<script>alert('Inscription réussie ! En attente d\\'approbation par l\\'admin.'); window.location.href='/';</script>".into()
    )
}

pub async fn login_handler(
    AxumState(state): AxumState<Arc<State>>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let user: Option<User> = query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    // ... (le reste de login_handler reste identique)
}

pub async fn pending_users_handler(AxumState(state): AxumState<Arc<State>>) -> impl IntoResponse {
    let rows = query_as!(
        UserInfoSqlxRow,
        "SELECT id, username, name, role, approved, needs_password_change FROM users WHERE approved = false"
    )
    .fetch_all(&state.db)
    .await
    .ok()
    .unwrap_or_default();

    let users: Vec<UserInfo> = rows.into_iter().map(|r| UserInfo {
        id: r.id,
        username: r.username,
        name: r.name.unwrap_or_default(),
        role: r.role.unwrap_or("user".to_string()),
        approved: r.approved,
        needs_password_change: r.needs_password_change,
    }).collect();

    // ... (le reste HTML reste identique)
}

pub async fn all_users_handler(AxumState(state): AxumState<Arc<State>>) -> impl IntoResponse {
    let rows = query_as!(
        UserInfoSqlxRow,
        "SELECT id, username, name, role, approved, needs_password_change FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .ok()
    .unwrap_or_default();

    let users: Vec<UserInfo> = rows.into_iter().map(|r| UserInfo {
        id: r.id,
        username: r.username,
        name: r.name.unwrap_or_default(),
        role: r.role.unwrap_or("user".to_string()),
        approved: r.approved,
        needs_password_change: r.needs_password_change,
    }).collect();

    // ... (le reste HTML reste identique)
}

// Le reste du fichier (approve_handler, logout_handler, change_password_handler, get_cookie, hash/verify) reste **exactement identique** à ta version actuelle.

fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();
    PasswordHash::generate(argon2, password)
        .unwrap()
        .to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash).unwrap();
    argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
}
