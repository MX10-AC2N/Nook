use crate::db::{User, AppState};
use crate::SharedState;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use axum::body::Body;
use axum::extract::State as AxumState;
use axum::http::header::{HeaderMap, HeaderName, SET_COOKIE};
use axum::http::Request;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

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
                let response = Json(AuthResponse {
                    success: false,
                    message: "Votre compte est en attente d'approbation.".to_string(),
                    user: None,
                });
                return (StatusCode::UNAUTHORIZED, response).into_response();
            }
            
            if verify_password(&payload.password, &user.password) {
                let token = Uuid::new_v4().to_string();
                let cookie_value = format!("{}:{}", user.id, token);
                
                let _ = sqlx::query(
                    "UPDATE users SET token = ? WHERE id = ?"
                )
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
                }).into_response();

                response.headers_mut().insert(
                    SET_COOKIE,
                    format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600", cookie_value)
                        .parse()
                        .unwrap(),
                );
                response
            } else {
                let response = Json(AuthResponse {
                    success: false,
                    message: "Nom d'utilisateur ou mot de passe incorrect.".to_string(),
                    user: None,
                });
                (StatusCode::UNAUTHORIZED, response).into_response()
            }
        }
        None => {
            let response = Json(AuthResponse {
                success: false,
                message: "Nom d'utilisateur ou mot de passe incorrect.".to_string(),
                user: None,
            });
            (StatusCode::UNAUTHORIZED, response).into_response()
        }
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
        }).into_response(),
        Err(e) => {
            let response = Json(AuthResponse {
                success: false,
                message: format!("Erreur lors de l'inscription: {}", e),
                user: None,
            });
            (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
        }
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
                        }).into_response()
                    }
                    None => {
                        let mut response = Response::new("Session invalide".to_string());
                        response.headers_mut().insert(
                            HeaderName::from_static("set-cookie"),
                            "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
                        );
                        (StatusCode::UNAUTHORIZED, response).into_response()
                    }
                }
            } else {
                let response = Json(SessionResponse {
                    authenticated: false,
                    user: None,
                });
                (StatusCode::UNAUTHORIZED, response).into_response()
            }
        }
        None => {
            let response = Json(SessionResponse {
                authenticated: false,
                user: None,
            });
            (StatusCode::UNAUTHORIZED, response).into_response()
        }
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
                        }).into_response()
                    }
                    None => {
                        let response = Json(UserInfoResponse { user: None });
                        (StatusCode::UNAUTHORIZED, response).into_response()
                    }
                }
            } else {
                let response = Json(UserInfoResponse { user: None });
                (StatusCode::UNAUTHORIZED, response).into_response()
            }
        }
        None => {
            let response = Json(UserInfoResponse { user: None });
            (StatusCode::UNAUTHORIZED, response).into_response()
        }
    }
}

pub async fn change_password_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie_from_user_id(&payload.user_id, &state.db).await;
    
    if auth_cookie.is_none() {
        let response = Json(AuthResponse {
            success: false,
            message: "Session invalide".to_string(),
            user: None,
        });
        return (StatusCode::UNAUTHORIZED, response).into_response();
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
        }).into_response(),
        Err(e) => {
            let response = Json(AuthResponse {
                success: false,
                message: format!("Erreur lors du changement de mot de passe: {}", e),
                user: None,
            });
            (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
        }
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
            let _ = sqlx::query(
                "UPDATE users SET token = NULL WHERE id = ?"
            )
            .bind(user_id)
            .execute(&state.db)
            .await;
        }
    }

    let mut response = Json(AuthResponse {
        success: true,
        message: "Déconnexion réussie".to_string(),
        user: None,
    }).into_response();

    response.headers_mut().insert(
        HeaderName::from_static("set-cookie"),
        "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );
    response
}

// ============ Page de configuration initiale pour le premier admin ============

pub async fn first_setup_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let user_id = payload.user_id.clone();
    
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password, name, role, approved, needs_password_change, created_at, token, public_key, joined_at FROM users WHERE id = ? AND needs_password_change = true"
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(_) => {
            let hashed_password = hash_password(&payload.new_password);

            let _ = sqlx::query(
                "UPDATE users SET password = ?, needs_password_change = false WHERE id = ?"
            )
            .bind(&hashed_password)
            .bind(&user_id)
            .execute(&state.db)
            .await;

            Html::<Body>(r#"
            <!DOCTYPE html>
            <html lang="fr">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Configuration terminée - Nook</title>
                <style>
                    body { font-family: Arial, sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background-color: #f5f5f5; }
                    .container { text-align: center; padding: 2rem; background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
                    h1 { color: #333; }
                    .success { color: #28a745; }
                </style>
            </head>
            <body>
                <div class="container">
                    <h1 class="success">Configuration terminée !</h1>
                    <p>Votre mot de passe a été mis à jour avec succès.</p>
                    <p>Vous allez être redirigé vers la page de connexion...</p>
                    <script>
                        setTimeout(() => { window.location.href = '/login'; }, 2000);
                    </script>
                </div>
            </body>
            </html>
            "#.to_string()).into_response()
        }
        None => Html::<Body>(r#"
        <!DOCTYPE html>
        <html lang="fr">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Erreur - Nook</title>
            <style>
                body { font-family: Arial, sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background-color: #f5f5f5; }
                .container { text-align: center; padding: 2rem; background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
                h1 { color: #dc3545; }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Erreur</h1>
                <p>Session invalide ou expirée.</p>
                <a href="/login">Retour à la connexion</a>
            </div>
        </body>
        </html>
        "#.to_string()).into_response()
    }
}

// ============ Handlers HTML legacy (pour compatibilité) ============

pub async fn register_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.password);
    
    let user_id = Uuid::new_v4().to_string();
    
    let _ = sqlx::query(
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

    Html::<Body>("Inscription réussie! En attente d'approbation de l'administrateur.".into()).into_response()
}

pub async fn login_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<LoginPayload>,
) -> Response<Body> {
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
                return Html::<Body>("Votre compte est en attente d'approbation.".into()).into_response();
            }
            
            if verify_password(&payload.password, &user.password) {
                let token = Uuid::new_v4().to_string();
                let cookie_value = format!("{}:{}", user.id, token);
                
                let _ = sqlx::query(
                    "UPDATE users SET token = ? WHERE id = ?"
                )
                .bind(&token)
                .bind(&user.id)
                .execute(&state.db)
                .await;

                let user_name = user.name.clone().unwrap_or_else(|| "Utilisateur".to_string());
                let user_role = user.role.clone().unwrap_or_else(|| "user".to_string());
                
                let user_info = UserInfo {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    name: user.name.unwrap_or_default(),
                    role: user.role.unwrap_or_else(|| "user".to_string()),
                    approved: user.approved,
                    needs_password_change: user.needs_password_change,
                };

                let mut response = if user_role == "admin" {
                    Html(r#"
                    <!DOCTYPE html>
                    <html lang="fr">
                    <head>
                        <meta charset="UTF-8">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
                        <title>Admin - Nook</title>
                    </head>
                    <body>
                        <h1>Admin Dashboard</h1>
                        <p>Bienvenue, Admin !</p>
                        <a href="/pending_users">Utilisateurs en attente</a><br>
                        <a href="/all_users">Tous les utilisateurs</a>
                    </body>
                    </html>
                    "#.into()).into_response()
                } else {
                    Html(format!(r#"
                    <!DOCTYPE html>
                    <html lang="fr">
                    <head>
                        <meta charset="UTF-8">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
                        <title>Bienvenue - Nook</title>
                    </head>
                    <body>
                        <h1>Bienvenue, {} !</h1>
                        <p>Ceci est votre tableau de bord utilisateur.</p>
                        <a href="/chat">Ouvrir le chat</a>
                        <script>
                            localStorage.setItem('user_info', '{}');
                        </script>
                    </body>
                    </html>
                    "#,
                        user_name,
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
                Html::<Body>("Nom d'utilisateur ou mot de passe incorrect.".into()).into_response()
            }
        }
        None => Html::<Body>("Nom d'utilisateur ou mot de passe incorrect.".into()).into_response(),
    }
}

pub async fn pending_users_handler(AxumState(state): AxumState<Arc<SharedState>>) -> impl IntoResponse {
    let rows: Vec<UserInfoSqlxRow> = sqlx::query_as(
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
        role: r.role.unwrap_or_else(|| "user".to_string()),
        approved: r.approved,
        needs_password_change: r.needs_password_change,
    }).collect();

    Html::<Body>(format!(r#"
    <!DOCTYPE html>
    <html lang="fr">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Utilisateurs en attente</title>
    </head>
    <body>
        <h1>Utilisateurs en attente d'approbation</h1>
        <ul>
            {}
        </ul>
        <a href="/">← Retour</a>
        <script>
            function approveUser(userId) {{
                fetch('/api/approve', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ user_id: userId }}),
                }})
                .then(response => response.json())
                .then(data => {{
                    if (data.success) {{
                        alert(data.message);
                        window.location.reload();
                    }} else {{
                        alert('Erreur: ' + data.message);
                    }}
                }})
                .catch(error => console.error('Error:', error));
            }}
        </script>
    </body>
    </html>
    "#,
        users.iter().map(|u| format!(
            r#"
            <li>
                <p>Nom: {}</p>
                <p>Username: {}</p>
                <button onclick="approveUser('{}')">Approuver</button>
            </li>
            "#,
            u.name, u.username, u.id
        )).collect::<Vec<String>>().join("")
    )).into_response()
}

pub async fn all_users_handler(AxumState(state): AxumState<Arc<SharedState>>) -> impl IntoResponse {
    let rows: Vec<UserInfoSqlxRow> = sqlx::query_as(
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
        role: r.role.unwrap_or_else(|| "user".to_string()),
        approved: r.approved,
        needs_password_change: r.needs_password_change,
    }).collect();

    Html::<Body>(format!(r#"
    <!DOCTYPE html>
    <html lang="fr">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Tous les utilisateurs</title>
    </head>
    <body>
        <h1>Tous les utilisateurs</h1>
        <table>
            <thead>
                <tr>
                    <th>Nom</th>
                    <th>Username</th>
                    <th>Rôle</th>
                    <th>Statut</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
        <a href="/">← Retour</a>
    </body>
    </html>
    "#,
        users.iter().map(|u| format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            u.name, u.username, u.role, if u.approved { "Approuvé" } else { "En attente" }
        )).collect::<Vec<String>>().join("")
    )).into_response()
}

pub async fn approve_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ApprovePayload>,
) -> impl IntoResponse {
    let _ = sqlx::query(
        "UPDATE users SET approved = true WHERE id = ?"
    )
    .bind(&payload.user_id)
    .execute(&state.db)
    .await;

    Json(AuthResponse {
        success: true,
        message: "Utilisateur approuvé avec succès".into(),
        user: None,
    })
}

pub async fn logout_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie(req.headers(), "auth_token");
    if let Some(cookie_value) = auth_cookie {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let _ = sqlx::query(
                "UPDATE users SET token = NULL WHERE id = ?"
            )
            .bind(user_id)
            .execute(&state.db)
            .await;
        }
    }

    let mut response = Response::new("Déconnexion réussie".to_string());
    response.headers_mut().insert(
        HeaderName::from_static("set-cookie"),
        "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );
    response
}

pub async fn change_password_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let hashed_password = hash_password(&payload.new_password);

    let _ = sqlx::query(
        "UPDATE users SET password = ?, needs_password_change = false WHERE id = ?"
    )
    .bind(&hashed_password)
    .bind(&payload.user_id)
    .execute(&state.db)
    .await;

    Html::<Body>("Mot de passe changé avec succès !".into()).into_response()
}

pub fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .map(|c| c.trim())
                .find(|c| c.starts_with(&format!("{} = ", name)))
                .and_then(|c| c.split_once('='))
                .map(|v| v.to_string())
        })
}

// Handlers d'invitation (legacy)
#[allow(dead_code)]
pub async fn invite_handler() -> impl IntoResponse {
    Html::<Body>("Fonction d'invitation à implémenter".into()).into_response()
}

#[allow(dead_code)]
pub async fn join_handler() -> impl IntoResponse {
    Html::<Body>("Fonction de rejoindre à implémenter".into()).into_response()
}

#[allow(dead_code)]
pub async fn members_handler() -> impl IntoResponse {
    Html::<Body>("Fonction membres à implémenter".into()).into_response()
}
