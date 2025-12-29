use crate::db::{get_pool, User};
use crate::State;
use argon2::{Argon2, PasswordHash};
use axum::body::Body;
use axum::extract::{multipart::Multipart, Path, Query, State as AxumState};
use axum::http::header::SetCookie;
use axum::http::{header::HeaderName, Request, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use futures_util::stream::BytesStream;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub name: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: i64,
    pub file: Option<serde_json::Value>,
}

pub async fn register_handler(
    State(state): AxumState<Arc<State>>,
    payload: axum::Json<RegisterPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.password);
    let user_id = Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO users (id, username, password, name, role, approved, needs_password_change) VALUES (?, ?, ?, ?, ?, ?, ?)",
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
        Ok(_) => Html(
            "<script>alert('Registration successful! Please wait for admin approval.'); window.location.href='/';</script>"
                .to_string(),
        ),
        Err(e) => Html(format!(
            "<script>alert('Registration failed: {}'); window.location.href='/';</script>",
            e
        )),
    }
}

pub async fn login_handler(
    State(state): AxumState<Arc<State>>,
    payload: axum::Json<LoginPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

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

                sqlx::query("UPDATE users SET token = ? WHERE id = ?")
                    .bind(&token)
                    .bind(&user.id)
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

                let cookie = format!(
                    "auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600",
                    cookie_value
                );

                if user.role == "admin" {
                    Html(format!(
                        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Admin Dashboard - Nook</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-white min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <h1 class="text-3xl font-bold mb-8 text-purple-400">Admin Dashboard</h1>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <a href="/pending_users" class="bg-purple-800 p-6 rounded-lg hover:bg-purple-700 transition">
                <h2 class="text-xl font-semibold mb-2">Pending Users</h2>
                <p class="text-gray-300">Review and approve new user registrations</p>
            </a>
            <a href="/all_users" class="bg-indigo-800 p-6 rounded-lg hover:bg-indigo-700 transition">
                <h2 class="text-xl font-semibold mb-2">All Users</h2>
                <p class="text-gray-300">View and manage all registered users</p>
            </a>
        </div>
        <div class="mt-8">
            <a href="/" class="text-purple-400 hover:text-purple-300">← Back to Home</a>
        </div>
    </div>
</body>
</html>"#
                    )).into_response()
                } else {
                    Html(format!(
                        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Welcome - Nook</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-white min-h-screen flex items-center justify-center">
    <div class="text-center">
        <h1 class="text-4xl font-bold mb-4 text-purple-400">Welcome, {}!</h1>
        <p class="text-gray-400 mb-8">You have successfully logged in.</p>
        <a href="/chat" class="bg-purple-600 px-6 py-3 rounded-lg hover:bg-purple-500 transition">Open Chat</a>
    </div>
    <script>
        document.cookie = "auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600";
        localStorage.setItem('user', JSON.stringify({}));
    </script>
</body>
</html>"#,
                        user.name,
                        serde_json::to_string(&user_info).unwrap()
                    )).into_response()
                }
            } else {
                Html("<script>alert('Invalid password'); window.location.href='/';</script>".to_string())
            }
        }
        None => Html("<script>alert('User not found'); window.location.href='/';</script>".to_string()),
    }
}

pub async fn pending_users_handler(
    State(state): AxumState<Arc<State>>,
) -> impl IntoResponse {
    let rows: Vec<UserInfoSqlxRow> = sqlx::query_as::<_, UserInfoSqlxRow>(
        "SELECT id, username, name, role, approved, needs_password_change FROM users WHERE approved = 0"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or(Vec::new());

    let users: Vec<UserInfo> = rows.into_iter().map(|row| UserInfo {
        id: row.id,
        username: row.username,
        name: row.name,
        role: row.role,
        approved: row.approved,
        needs_password_change: row.needs_password_change,
    }).collect();

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Pending Users - Nook</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-white min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <h1 class="text-3xl font-bold mb-8 text-purple-400">Pending Users</h1>
        <div class="grid gap-4">
            {} 
        </div>
        <div class="mt-8">
            <a href="/" class="text-purple-400 hover:text-purple-300">← Back to Home</a>
        </div>
    </div>
    <script>
        async function approveUser(userId) {{
            try {{
                const response = await fetch('/api/approve', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ user_id: userId }})
                }});
                const data = await response.json();
                if (data.success) {{
                    alert('User approved successfully!');
                    window.location.reload();
                }} else {{
                    alert('Error: ' + data.message);
                }}
            }} catch (error) {{
                alert('An error occurred');
            }}
        }}
    </script>
</body>
</html>"#,
        users.iter().map(|user| format!(
            r#"<div class="bg-gray-800 p-4 rounded-lg flex justify-between items-center">
                <div>
                    <p class="font-semibold">{}</p>
                    <p class="text-gray-400 text-sm">{}</p>
                </div>
                <button onclick="approveUser('{}')" class="bg-green-600 px-4 py-2 rounded hover:bg-green-500">Approve</button>
            </div>"#,
            user.name, user.username, user.id
        )).collect::<Vec<_>>().join("")
    ))
}

pub async fn all_users_handler(
    State(state): AxumState<Arc<State>>,
) -> impl IntoResponse {
    let rows: Vec<UserInfoSqlxRow> = sqlx::query_as::<_, UserInfoSqlxRow>(
        "SELECT id, username, name, role, approved, needs_password_change FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or(Vec::new());

    let users: Vec<UserInfo> = rows.into_iter().map(|row| UserInfo {
        id: row.id,
        username: row.username,
        name: row.name,
        role: row.role,
        approved: row.approved,
        needs_password_change: row.needs_password_change,
    }).collect();

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>All Users - Nook</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-white min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <h1 class="text-3xl font-bold mb-8 text-purple-400">All Users</h1>
        <div class="overflow-x-auto">
            <table class="w-full bg-gray-800 rounded-lg">
                <thead>
                    <tr class="text-left border-b border-gray-700">
                        <th class="p-4">Name</th>
                        <th class="p-4">Username</th>
                        <th class="p-4">Role</th>
                        <th class="p-4">Status</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>
        <div class="mt-8">
            <a href="/" class="text-purple-400 hover:text-purple-300">← Back to Home</a>
        </div>
    </div>
</body>
</html>"#,
        users.iter().map(|user| format!(
            r#"<tr class="border-b border-gray-700">
                <td class="p-4">{}</td>
                <td class="p-4">{}</td>
                <td class="p-4">{}</td>
                <td class="p-4">{}</td>
            </tr>"#,
            user.name, user.username, user.role, if user.approved { "Approved" } else { "Pending" }
        )).collect::<Vec<_>>().join("")
    ))
}

pub async fn approve_handler(
    State(state): AxumState<Arc<State>>,
    payload: axum::Json<ApprovePayload>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE users SET approved = 1 WHERE id = ?")
        .bind(&payload.user_id)
        .execute(&state.db)
        .await;

    let response = AuthResponse {
        success: true,
        message: "User approved successfully".to_string(),
        user: None,
    };

    axum::Json(response)
}

pub async fn logout_handler(
    State(state): AxumState<Arc<State>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie(&req, "auth_token");

    if let Some(cookie_value) = auth_cookie {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            sqlx::query("UPDATE users SET token = NULL WHERE id = ?")
                .bind(user_id)
                .execute(&state.db)
                .await
                .ok();
        }
    }

    let mut response = Response::new("Logged out successfully".to_string());
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("set-cookie"),
        "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("set-cookie"),
        "user_data=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );

    response
}

pub async fn change_password_handler(
    State(state): AxumState<Arc<State>>,
    payload: axum::Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.new_password);

    let result = sqlx::query(
        "UPDATE users SET password = ?, needs_password_change = 0 WHERE id = ?"
    )
    .bind(&hashed_password)
    .bind(&payload.user_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Html(
            "<script>alert('Password changed successfully!'); window.location.href='/';</script>"
                .to_string(),
        ),
        Err(e) => Html(format!(
            "<script>alert('Error changing password: {}'); window.location.href='/';</script>",
            e
        )),
    }
}

pub async fn create_conversation_handler(
    State(state): AxumState<Arc<State>>,
    Path(user_id): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let name = if let Some(field) = multipart.next_field().await.unwrap() {
        field.text().await.unwrap_or_else(|_| "New Group".to_string())
    } else {
        "New Group".to_string()
    };

    let conversation_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    sqlx::query("INSERT INTO conversations (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&conversation_id)
        .bind(&name)
        .bind(&timestamp)
        .execute(&state.db)
        .await
        .ok();

    sqlx::query("INSERT INTO conversation_participants (conversation_id, user_id) VALUES (?, ?)")
        .bind(&conversation_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .ok();

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Conversation Created</title>
    <script>
        if (window.opener) {{
            window.opener.postMessage({{
                type: 'conversation_created',
                conversationId: '{}'
            }}, '*');
        }}
    </script>
</head>
<body>
    <p>Conversation created. Closing...</p>
</body>
</html>"#,
        conversation_id
    ))
}

pub async fn upload_avatar_handler(
    State(state): AxumState<Arc<State>>,
    Path(user_id): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();
        let id = Uuid::new_v4().to_string();
        let upload_dir = "avatars";
        let path = format!("{}/{}", upload_dir, id);
        std::fs::create_dir_all(upload_dir).ok();

        let mut file = File::create(&path).await.unwrap();
        file.write_all(&data).await.unwrap();

        sqlx::query("UPDATE users SET avatar = ? WHERE id = ?")
            .bind(&path)
            .bind(&user_id)
            .execute(&state.db)
            .await
            .ok();

        return Html(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Avatar Upload</title>
    <script>
        if (window.opener) {{
            window.opener.postMessage({{
                type: 'avatar_uploaded',
                path: '{}'
            }}, '*');
        }}
    </script>
</head>
<body>
    <p>Avatar uploaded. Closing...</p>
</body>
</html>"#,
            path
        ));
    }

    Html("No file uploaded".into())
}

pub async fn get_avatar(Path(id): Path<String>) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&id)
        .fetch_optional(&get_pool())
        .await
        .unwrap_or(None);

    match user.and_then(|u| u.avatar) {
        Some(path) => {
            let path = std::path::Path::new(&path);
            if path.exists() {
                let file = File::open(path).await.unwrap();
                let stream = BytesStream::from(tokio::fs::read(path).await.unwrap());
                let body = Body::from_stream(stream);
                let mut headers = HeaderMap::new();
                headers.insert(
                    "content-type",
                    "image/jpeg".parse().unwrap(),
                );
                (headers, body).into_response()
            } else {
                (StatusCode::NOT_FOUND, "Avatar not found").into_response()
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            "Avatar not found".to_string(),
        ).into_response(),
    }
}

fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::generate(argon2, password).unwrap();
    password_hash.to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(hash).unwrap();
    argon2.verify_password(password.as_bytes(), &password_hash).is_ok()
}

fn get_cookie(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get("cookie")
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|c| c.trim().starts_with(&format!("{}=", name)))
                .map(|c| c.trim().split('=').nth(1).unwrap_or("").to_string())
        })
}
