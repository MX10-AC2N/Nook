use crate::db::{get_pool, User};
use crate::State;
use argon2::{Argon2, PasswordHash};
use axum::body::Body;
use axum::extract::{Multipart, Path, State as AxumState};
use axum::http::header::{HeaderName, HeaderMap};
use axum::http::{Request, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use futures_util::stream::BytesStream;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use std::collections::HashMap;
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
    name: String,
    role: String,
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

    let _ = sqlx::query!(
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
    .await;

    Html(
        "<script>alert('Registration successful! Please wait for admin approval.'); window.location.href='/';</script>"
            .to_string(),
    )
}

pub async fn login_handler(
    AxumState(state): AxumState<Arc<State>>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(user) => {
            if !user.approved {
                return Html(
                    "<script>alert('Your account is not approved yet.'); window.location.href='/';</script>"
                        .to_string(),
                );
            }

            if verify_password(&payload.password, &user.password) {
                let token = Uuid::new_v4().to_string();
                let cookie_value = format!("{}:{}", user.id, token);

                let _ = sqlx::query!("UPDATE users SET token = $1 WHERE id = $2", token, user.id)
                    .execute(&state.db)
                    .await;

                let user_info = UserInfo {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    name: user.name.clone(),
                    role: user.role.clone(),
                    approved: user.approved,
                    needs_password_change: user.needs_password_change,
                };

                let cookie = format!(
                    "auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600",
                    cookie_value
                );

                let mut response = if user.role == "admin" {
                    Html(include_str!("../templates/admin_dashboard.html").to_string()).into_response()
                } else {
                    Html(format!(
                        include_str!("../templates/user_welcome.html"),
                        user.name,
                        serde_json::to_string(&user_info).unwrap()
                    ))
                    .into_response()
                };

                response.headers_mut().insert(
                    axum::http::header::SET_COOKIE,
                    cookie.parse().unwrap(),
                );
                response
            } else {
                Html("<script>alert('Invalid password'); window.location.href='/';</script>".to_string())
            }
        }
        None => Html("<script>alert('User not found'); window.location.href='/';</script>".to_string()),
    }
}

pub async fn pending_users_handler(AxumState(state): AxumState<Arc<State>>) -> impl IntoResponse {
    let rows = sqlx::query_as!(
        UserInfoSqlxRow,
        "SELECT id, username, name, role, approved, needs_password_change 
         FROM users WHERE approved = false"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let users: Vec<UserInfo> = rows
        .into_iter()
        .map(|row| UserInfo {
            id: row.id,
            username: row.username,
            name: row.name,
            role: row.role,
            approved: row.approved,
            needs_password_change: row.needs_password_change,
        })
        .collect();

    Html(format!(
        include_str!("../templates/pending_users.html"),
        users
            .iter()
            .map(|user| {
                format!(
                    r#"<div class="bg-gray-800 p-4 rounded-lg flex justify-between items-center">
                        <div>
                            <p class="font-semibold">{}</p>
                            <p class="text-gray-400 text-sm">{}</p>
                        </div>
                        <button onclick="approveUser('{}')" class="bg-green-600 px-4 py-2 rounded hover:bg-green-500">Approve</button>
                    </div>"#,
                    user.name, user.username, user.id
                )
            })
            .collect::<Vec<_>>()
            .join("")
    ))
}

pub async fn all_users_handler(AxumState(state): AxumState<Arc<State>>) -> impl IntoResponse {
    let rows = sqlx::query_as!(
        UserInfoSqlxRow,
        "SELECT id, username, name, role, approved, needs_password_change 
         FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let users: Vec<UserInfo> = rows
        .into_iter()
        .map(|row| UserInfo {
            id: row.id,
            username: row.username,
            name: row.name,
            role: row.role,
            approved: row.approved,
            needs_password_change: row.needs_password_change,
        })
        .collect();

    Html(format!(
        include_str!("../templates/all_users.html"),
        users
            .iter()
            .map(|user| {
                format!(
                    r#"<tr class="border-b border-gray-700">
                        <td class="p-4">{}</td>
                        <td class="p-4">{}</td>
                        <td class="p-4">{}</td>
                        <td class="p-4">{}</td>
                    </tr>"#,
                    user.name,
                    user.username,
                    user.role,
                    if user.approved { "Approved" } else { "Pending" }
                )
            })
            .collect::<Vec<_>>()
            .join("")
    ))
}

pub async fn approve_handler(
    AxumState(state): AxumState<Arc<State>>,
    Json(payload): Json<ApprovePayload>,
) -> impl IntoResponse {
    let _ = sqlx::query!("UPDATE users SET approved = true WHERE id = $1", payload.user_id)
        .execute(&state.db)
        .await;

    Json(AuthResponse {
        success: true,
        message: "User approved successfully".to_string(),
        user: None,
    })
}

pub async fn logout_handler(
    AxumState(_state): AxumState<Arc<State>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie(&req, "auth_token");

    if let Some(cookie_value) = auth_cookie {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let _ = sqlx::query!("UPDATE users SET token = NULL WHERE id = $1", user_id)
                .execute(&_state.db)
                .await;
        }
    }

    let mut response = Response::new("Logged out successfully".to_string());
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("set-cookie"),
        "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );
    response
}

pub async fn change_password_handler(
    AxumState(state): AxumState<Arc<State>>,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.new_password);

    let _ = sqlx::query!(
        "UPDATE users SET password = $1, needs_password_change = false WHERE id = $2",
        hashed_password,
        payload.user_id
    )
    .execute(&state.db)
    .await;

    Html(
        "<script>alert('Password changed successfully!'); window.location.href='/';</script>"
            .to_string(),
    )
}

// Les autres handlers (create_conversation, upload_avatar, get_avatar) restent similaires avec query! / query_as!
// Tu peux me demander si tu veux que je les corrige aussi.

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

use axum::http::HeaderMap;

pub fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|c| c.trim().starts_with(&format!("{}=", name)))
                .and_then(|c| c.trim().split('=').nth(1))
                .map(String::from)
        })
}
