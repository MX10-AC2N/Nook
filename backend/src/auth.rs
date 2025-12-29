use crate::db::{get_pool, User};
use crate::State;
use argon2::{Argon2, PasswordHash};
use axum::body::Body;
use axum::extract::{Multipart, Path, State as AxumState};
use axum::http::header::{HeaderMap, HeaderName, SET_COOKIE};
use axum::http::{Request, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
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
    name: Option<String>,
    role: Option<String>,
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
    .ok();

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

    match user {
        Some(user) => {
            if !user.approved {
                return Html(
                    "<script>alert('Votre compte n\\'est pas encore approuvé.'); window.location.href='/';</script>".into()
                );
            }

            if verify_password(&payload.password, &user.password) {
                let token = Uuid::new_v4().to_string();
                let cookie_value = format!("{}:{}", user.id, token);

                let _ = query!("UPDATE users SET token = $1 WHERE id = $2", token, user.id)
                    .execute(&state.db)
                    .await
                    .ok();

                let user_info = UserInfo {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    name: user.name.clone(),
                    role: user.role.clone(),
                    approved: user.approved,
                    needs_password_change: user.needs_password_change,
                };

                let mut response = if user.role == "admin" {
                    Html(r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Admin - Nook</title>
<script src="https://cdn.tailwindcss.com"></script></head>
<body class="bg-gray-900 text-white"><div class="container mx-auto p-8">
<h1 class="text-4xl font-bold text-purple-400 mb-8">Admin Dashboard</h1>
<div class="grid md:grid-cols-2 gap-6">
<a href="/pending_users" class="bg-purple-800 p-8 rounded-lg hover:bg-purple-700">Pending Users</a>
<a href="/all_users" class="bg-indigo-800 p-8 rounded-lg hover:bg-indigo-700">All Users</a>
</div></div></body></html>"#.into())
                } else {
                    Html(format!(
                        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Bienvenue - Nook</title>
<script src="https://cdn.tailwindcss.com"></script></head>
<body class="bg-gray-900 text-white flex items-center justify-center min-h-screen">
<div class="text-center">
<h1 class="text-4xl font-bold text-purple-400 mb-4">Bienvenue, {} !</h1>
<a href="/chat" class="bg-purple-600 px-8 py-4 rounded-lg hover:bg-purple-500">Ouvrir le chat</a>
</div>
<script>
    document.cookie = "auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600";
    localStorage.setItem('user', '{}');
</script>
</body></html>"#,
                        user.name,
                        cookie_value,
                        serde_json::to_string(&user_info).unwrap()
                    )).into_response()
                };

                response.headers_mut().insert(
                    SET_COOKIE,
                    format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600", cookie_value)
                        .parse()
                        .unwrap(),
                );

                response
            } else {
                Html("<script>alert('Mot de passe incorrect'); window.location.href='/';</script>".into())
            }
        }
        None => Html("<script>alert('Utilisateur non trouvé'); window.location.href='/';</script>".into()),
    }
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

    Html(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Utilisateurs en attente</title>
<script src="https://cdn.tailwindcss.com"></script></head>
<body class="bg-gray-900 text-white min-h-screen p-8">
<h1 class="text-3xl font-bold text-purple-400 mb-8">Utilisateurs en attente d'approbation</h1>
<div class="space-y-4">
{}
</div>
<a href="/" class="text-purple-400 mt-8 inline-block">← Retour</a>
<script>
async function approveUser(id) {{
    const res = await fetch('/api/approve', {{ method: 'POST', headers: {{'Content-Type': 'application/json'}}, body: JSON.stringify({{user_id: id}}) }});
    if ((await res.json()).success) location.reload();
}}
</script>
</body></html>"#,
        users.iter().map(|u| format!(
            r#"<div class="bg-gray-800 p-6 rounded-lg flex justify-between items-center">
                <div><p class="font-bold">{}</p><p class="text-gray-400">{}</p></div>
                <button onclick="approveUser('{}')" class="bg-green-600 px-6 py-3 rounded hover:bg-green-500">Approuver</button>
            </div>"#,
            u.name, u.username, u.id
        )).collect::<Vec<_>>().join("")
    ))
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

    Html(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Tous les utilisateurs</title>
<script src="https://cdn.tailwindcss.com"></script></head>
<body class="bg-gray-900 text-white min-h-screen p-8">
<h1 class="text-3xl font-bold text-purple-400 mb-8">Tous les utilisateurs</h1>
<table class="w-full bg-gray-800 rounded-lg">
<thead><tr class="border-b border-gray-700"><th class="p-4 text-left">Nom</th><th class="p-4 text-left">Username</th><th class="p-4 text-left">Rôle</th><th class="p-4 text-left">Statut</th></tr></thead>
<tbody>{}</tbody>
</table>
<a href="/" class="text-purple-400 mt-8 inline-block">← Retour</a>
</body></html>"#,
        users.iter().map(|u| format!(
            "<tr class=\"border-b border-gray-700\"><td class=\"p-4\">{}</td><td class=\"p-4\">{}</td><td class=\"p-4\">{}</td><td class=\"p-4\">{}</td></tr>",
            u.name, u.username, u.role, if u.approved { "Approuvé" } else { "En attente" }
        )).collect::<Vec<_>>().join("")
    ))
}

pub async fn approve_handler(
    AxumState(state): AxumState<Arc<State>>,
    Json(payload): Json<ApprovePayload>,
) -> impl IntoResponse {
    let _ = query!("UPDATE users SET approved = true WHERE id = $1", payload.user_id)
        .execute(&state.db)
        .await
        .ok();

    Json(AuthResponse {
        success: true,
        message: "Utilisateur approuvé avec succès".into(),
        user: None,
    })
}

pub async fn logout_handler(
    AxumState(state): AxumState<Arc<State>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie(req.headers(), "auth_token");

    if let Some(cookie_value) = auth_cookie {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let _ = query!("UPDATE users SET token = NULL WHERE id = $1", user_id)
                .execute(&state.db)
                .await
                .ok();
        }
    }

    let mut response = Response::new("Déconnexion réussie".into());
    response.headers_mut().insert(
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

    let _ = query!(
        "UPDATE users SET password = $1, needs_password_change = false WHERE id = $2",
        hashed_password,
        payload.user_id
    )
    .execute(&state.db)
    .await
    .ok();

    Html("<script>alert('Mot de passe changé avec succès !'); window.location.href='/';</script>".into())
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
                .and_then(|c| c.splitn(2, '=').nth(1))
                .map(|v| v.to_string())
        })
}

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
