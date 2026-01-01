use crate::db::User;
use crate::SharedState;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use axum::body::Body;
use axum::extract::State as AxumState;
use axum::http::header::{HeaderMap, HeaderName, SET_COOKIE};
use axum::http::Request;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use axum::http::StatusCode;

// ==================== STRUCTURES ====================

#[derive(Serialize, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginJsonPayload {
    pub member_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterJsonPayload {
    pub username: String,
    pub password: String,
    pub name: String,
    pub invite_token: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordPayload {
    pub user_id: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordJsonPayload {
    pub user_id: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct FirstSetupPayload {
    pub user_id: String,
    pub new_username: String,
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
pub struct UserInfoResponse {
    pub id: String,
    pub name: String,
    pub username: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionResponse {
    pub member_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApprovePayload {
    pub user_id: String,
}

// ==================== FONCTION UTILITAIRE ====================

fn get_cookie_from_headers(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .map(|c| c.trim())
                .find(|c| c.starts_with(&format!("{}= ", name)))
                .and_then(|c| c.split_once('='))
                .map(|(v, _)| v.to_string())
        })
}

// ==================== ENDPOINTS JSON ====================

pub async fn login_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<LoginJsonPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password, name, role, approved, needs_password_change FROM users WHERE id = ?"
    )
    .bind(&payload.member_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(user) => {
            if !user.approved {
                return (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                    success: false,
                    message: "Compte en attente d'approbation".to_string(),
                    user: None,
                }));
            }

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
                role: user.role.clone().unwrap_or_else(|| "user".to_string()),
                approved: user.approved,
                needs_password_change: user.needs_password_change,
            };

            let mut response = (StatusCode::OK, Json(AuthResponse {
                success: true,
                message: "Connexion réussie".to_string(),
                user: Some(user_info),
            }));

            response.1.headers_mut().insert(
                SET_COOKIE,
                format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600", cookie_value)
                    .parse()
                    .unwrap(),
            );
            response
        }
        None => (StatusCode::UNAUTHORIZED, Json(AuthResponse {
            success: false,
            message: "Utilisateur non trouvé".to_string(),
            user: None,
        })),
    }
}

pub async fn validate_session_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie_from_headers(req.headers(), "auth_token");
    
    if let Some(cookie_value) = auth_cookie {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];
            let token = parts[1];

            let user: Option<User> = sqlx::query_as(
                "SELECT id, approved FROM users WHERE id = ? AND token = ?"
            )
            .bind(user_id)
            .bind(token)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

            if let Some(u) = user {
                if u.approved {
                    return (StatusCode::OK, Json(SessionResponse { member_id: u.id }));
                }
            }
        }
    }

    (StatusCode::UNAUTHORIZED, Json(()))
}

pub async fn user_info_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie_from_headers(req.headers(), "auth_token");
    
    if let Some(cookie_value) = auth_cookie {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            let user_id = parts[0];

            let user: Option<User> = sqlx::query_as(
                "SELECT id, username, name, role, approved FROM users WHERE id = ?"
            )
            .bind(user_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

            if let Some(u) = user {
                return (StatusCode::OK, Json(UserInfoResponse {
                    id: u.id,
                    name: u.name.unwrap_or_default(),
                    username: u.username,
                    role: u.role.clone().unwrap_or_else(|| "user".to_string()),
                }));
            }
        }
    }

    (StatusCode::UNAUTHORIZED, Json(()))
}

pub async fn register_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<RegisterJsonPayload>,
) -> impl IntoResponse {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO users (id, username, password, name, role, approved, needs_password_change, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&hashed_password)
    .bind(&payload.name)
    .bind("user")
    .bind(false)
    .bind(true)
    .bind(&created_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(AuthResponse {
            success: true,
            message: "Inscription réussie! En attente d'approbation.".to_string(),
            user: None,
        })),
        Err(_) => (StatusCode::BAD_REQUEST, Json(AuthResponse {
            success: false,
            message: "Erreur lors de l'inscription. Identifiant peut-être déjà utilisé.".to_string(),
            user: None,
        })),
    }
}

pub async fn change_password_json_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ChangePasswordJsonPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as(
        "SELECT password FROM users WHERE id = ?"
    )
    .bind(&payload.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(user) => {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let hashed_password = argon2
                .hash_password(payload.new_password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            let _ = sqlx::query(
                "UPDATE users SET password = ?, needs_password_change = false WHERE id = ?"
            )
            .bind(&hashed_password)
            .bind(&payload.user_id)
            .execute(&state.db)
            .await;

            (StatusCode::OK, Json(AuthResponse {
                success: true,
                message: "Mot de passe changé avec succès!".to_string(),
                user: None,
            }))
        }
        None => (StatusCode::UNAUTHORIZED, Json(AuthResponse {
            success: false,
            message: "Utilisateur non trouvé".to_string(),
            user: None,
        })),
    }
}

pub async fn first_setup_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<FirstSetupPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, needs_password_change FROM users WHERE id = ? AND needs_password_change = true"
    )
    .bind(&payload.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(_) => {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let hashed_password = argon2
                .hash_password(payload.new_password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            let result = sqlx::query(
                "UPDATE users SET username = ?, password = ?, needs_password_change = false WHERE id = ?"
            )
            .bind(&payload.new_username)
            .bind(&hashed_password)
            .bind(&payload.user_id)
            .execute(&state.db)
            .await;

            match result {
                Ok(_) => (StatusCode::OK, Json(AuthResponse {
                    success: true,
                    message: "Configuration terminée avec succès!".to_string(),
                    user: None,
                })),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse {
                    success: false,
                    message: "Erreur lors de la mise à jour.".to_string(),
                    user: None,
                })),
            }
        }
        None => (StatusCode::BAD_REQUEST, Json(AuthResponse {
            success: false,
            message: "Utilisateur non trouvé ou configuration déjà effectuée.".to_string(),
            user: None,
        })),
    }
}

// ==================== ANCIENNES ROUTES HTML ====================

pub async fn register_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let _ = sqlx::query(
        "INSERT INTO users (id, username, password, name, role, approved, needs_password_change, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&hashed_password)
    .bind(&payload.name)
    .bind("user")
    .bind(false)
    .bind(true)
    .bind(&created_at)
    .execute(&state.db)
    .await;

    (StatusCode::OK, "Inscription réussie! En attente d'approbation de l'administrateur.")
}

pub async fn login_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password, name, role, approved, needs_password_change FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match user {
        Some(user) => {
            if !user.approved {
                return (StatusCode::FORBIDDEN, "Votre compte est en attente d'approbation.");
            }

            let parsed_hash = PasswordHash::new(&user.password).unwrap();
            if Argon2::default()
                .verify_password(payload.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                let token = Uuid::new_v4().to_string();
                let cookie_value = format!("{}:{}", user.id, token);

                let _ = sqlx::query("UPDATE users SET token = ? WHERE id = ?")
                    .bind(&token)
                    .bind(&user.id)
                    .execute(&state.db)
                    .await;

                let user_name = user.name.clone().unwrap_or_else(|| "Utilisateur".to_string());
                let user_role = user.role.clone().unwrap_or_else(|| "user".to_string());
                let needs_change = user.needs_password_change;

                let user_info = UserInfo {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    name: user.name.unwrap_or_default(),
                    role: user.role.unwrap_or_else(|| "user".to_string()),
                    approved: user.approved,
                    needs_password_change: needs_change,
                };

                let html_content = if needs_change {
                    format!(r#"
                    <!DOCTYPE html>
                    <html lang="fr">
                    <head>
                        <meta charset="UTF-8">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
                        <title>Configuration initiale - Nook</title>
                        <style>
                            body {{ font-family: Arial, sans-serif; max-width: 400px; margin: 50px auto; padding: 20px; background: #f5f5f5; }}
                            .container {{ background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                            h1 {{ color: #333; margin-bottom: 20px; text-align: center; }}
                            .warning {{ background: #fff3cd; border: 1px solid #ffc107; padding: 15px; border-radius: 5px; margin-bottom: 20px; color: #856404; }}
                            label {{ display: block; margin-bottom: 5px; color: #555; font-weight: bold; }}
                            input {{ width: 100%; padding: 10px; margin-bottom: 15px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; }}
                            button {{ width: 100%; padding: 12px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; }}
                            button:hover {{ background: #45a049; }}
                            #message {{ margin-top: 15px; padding: 10px; border-radius: 4px; display: none; }}
                            .error {{ background: #f8d7da; color: #721c24; }}
                            .success {{ background: #d4edda; color: #155724; }}
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <h1>🌱 Nook</h1>
                            <div class="warning">
                                <strong>⚠️ Première connexion</strong><br>
                                Vous devez définir un nouveau nom d'utilisateur et mot de passe pour sécuriser votre compte.
                            </div>
                            <form id="setupForm">
                                <input type="hidden" id="userId" value="{}">
                                <label for="newUsername">Nouvel identifiant</label>
                                <input type="text" id="newUsername" required minlength="3" placeholder="Votre nouveau username">
                                <label for="newPassword">Nouveau mot de passe</label>
                                <input type="password" id="newPassword" required minlength="8" placeholder="Minimum 8 caractères">
                                <label for="confirmPassword">Confirmer le mot de passe</label>
                                <input type="password" id="confirmPassword" required minlength="8" placeholder="Répétez le mot de passe">
                                <button type="submit">Enregistrer</button>
                            </form>
                            <div id="message"></div>
                        </div>
                        <script>
                            document.getElementById('setupForm').addEventListener('submit', async (e) {{
                                e.preventDefault();
                                const userId = document.getElementById('userId').value;
                                const newUsername = document.getElementById('newUsername').value;
                                const newPassword = document.getElementById('newPassword').value;
                                const confirmPassword = document.getElementById('confirmPassword').value;
                                const messageDiv = document.getElementById('message');
                                if (newPassword !== confirmPassword) {{
                                    messageDiv.textContent = 'Les mots de passe ne correspondent pas.'; messageDiv.className = 'error'; messageDiv.style.display = 'block'; return;
                                }}
                                if (newPassword.length < 8) {{
                                    messageDiv.textContent = 'Le mot de passe doit contenir au moins 8 caractères.'; messageDiv.className = 'error'; messageDiv.style.display = 'block'; return;
                                }}
                                try {{
                                    const response = await fetch('/api/first-setup', {{
                                        method: 'POST', headers: {{ 'Content-Type': 'application/json' }},
                                        body: JSON.stringify({{ user_id: userId, new_username: newUsername, new_password: newPassword }})
                                    }});
                                    const data = await response.json();
                                    if (data.success) {{
                                        messageDiv.textContent = 'Configuration terminée ! Redirection...'; messageDiv.className = 'success'; messageDiv.style.display = 'block';
                                        setTimeout(() => {{ window.location.href = '/'; }}, 1500);
                                    }} else {{
                                        messageDiv.textContent = data.message || 'Erreur lors de la configuration.'; messageDiv.className = 'error'; messageDiv.style.display = 'block';
                                    }}
                                }} catch (error) {{
                                    messageDiv.textContent = 'Erreur de connexion au serveur.'; messageDiv.className = 'error'; messageDiv.style.display = 'block';
                                }}
                            }});
                        </script>
                    </body>
                    </html>
                    "#, user.id)
                } else if user_role == "admin" {
                    format!(r#"
                    <!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><title>Admin - Nook</title></head>
                    <body style="font-family: Arial, sans-serif; margin: 40px;">
                        <h1>👑 Administration</h1>
                        <p>Bienvenue, Admin !</p>
                        <a href="/pending_users">Utilisateurs en attente</a><br>
                        <a href="/all_users">Tous les utilisateurs</a>
                    </body></html>
                    "#)
                } else {
                    format!(r#"
                    <!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><title>Bienvenue - Nook</title></head>
                    <body style="font-family: Arial, sans-serif; margin: 40px;">
                        <h1>Bienvenue, {} !</h1>
                        <p>Ceci est votre tableau de bord.</p>
                        <a href="/chat">Ouvrir le chat</a>
                        <script>localStorage.setItem('user_info', '{}');</script>
                    </body></html>
                    "#, user_name, serde_json::to_string(&user_info).unwrap())
                };

                let mut response = (StatusCode::OK, axum::response::Html(html_content));
                response.1.headers_mut().insert(
                    SET_COOKIE,
                    format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600", cookie_value)
                        .parse()
                        .unwrap(),
                );
                response
            } else {
                (StatusCode::UNAUTHORIZED, "Nom d'utilisateur ou mot de passe incorrect.")
            }
        }
        None => (StatusCode::UNAUTHORIZED, "Nom d'utilisateur ou mot de passe incorrect."),
    }
}

pub async fn change_password_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(payload.new_password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let _ = sqlx::query("UPDATE users SET password = ?, needs_password_change = false WHERE id = ?")
        .bind(&hashed_password)
        .bind(&payload.user_id)
        .execute(&state.db)
        .await;

    (StatusCode::OK, "Mot de passe changé avec succès !")
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

    let html = format!(r#"
    <!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><title>Utilisateurs en attente</title></head>
    <body style="font-family: Arial, sans-serif; margin: 40px;">
        <h1>Utilisateurs en attente d'approbation</h1>
        <ul style="list-style: none; padding: 0;">
            {}
        </ul>
        <a href="/">← Retour</a>
        <script>
            function approveUser(userId) {{
                fetch('/api/approve', {{ method: 'POST', headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ user_id: userId }}) }})
                    .then(response => response.json()).then(data => {{
                        if (data.success) {{ alert(data.message); window.location.reload(); }}
                        else {{ alert('Erreur: ' + data.message); }}
                    }}).catch(error => console.error('Error:', error));
            }}
        </script>
    </body></html>
    "#,
        users.iter().map(|u| format!(
            r#"<li style="border: 1px solid #ddd; padding: 15px; margin: 10px 0; border-radius: 4px;">
                <p><strong>{}</strong> (@{})</p>
                <button onclick="approveUser('{}')" style="padding: 8px 15px; cursor: pointer;">Approuver</button>
            </li>"#,
            u.name, u.username, u.id
        )).collect::<Vec<String>>().join("")
    );

    (StatusCode::OK, axum::response::Html(html))
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

    let html = format!(r#"
    <!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><title>Tous les utilisateurs</title></head>
    <body style="font-family: Arial, sans-serif; margin: 40px;">
        <h1>Tous les utilisateurs</h1>
        <table style="border-collapse: collapse; width: 100%;">
            <thead><tr><th style="border: 1px solid #ddd; padding: 10px; background: #f5f5f5;">Nom</th>
                <th style="border: 1px solid #ddd; padding: 10px; background: #f5f5f5;">Identifiant</th>
                <th style="border: 1px solid #ddd; padding: 10px; background: #f5f5f5;">Rôle</th>
                <th style="border: 1px solid #ddd; padding: 10px; background: #f5f5f5;">Statut</th></tr></thead>
            <tbody>{}</tbody>
        </table>
        <p><a href="/">← Retour</a></p>
    </body></html>
    "#,
        users.iter().map(|u| format!(
            "<tr><td style='border: 1px solid #ddd; padding: 10px;'>{}</td><td style='border: 1px solid #ddd; padding: 10px;'>@{}</td><td style='border: 1px solid #ddd; padding: 10px;'>{}</td><td style='border: 1px solid #ddd; padding: 10px;'>{}</td></tr>",
            u.name, u.username, u.role, if u.approved { "Approuvé" } else { "En attente" }
        )).collect::<Vec<String>>().join("")
    );

    (StatusCode::OK, axum::response::Html(html))
}

pub async fn approve_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Json(payload): Json<ApprovePayload>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE users SET approved = true WHERE id = ?")
        .bind(&payload.user_id)
        .execute(&state.db)
        .await;

    (StatusCode::OK, Json(AuthResponse {
        success: true,
        message: "Utilisateur approuvé avec succès".into(),
        user: None,
    }))
}

pub async fn logout_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let auth_cookie = get_cookie_from_headers(req.headers(), "auth_token");
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

    let mut response = (StatusCode::OK, "Déconnexion réussie");
    response.1.headers_mut().insert(
        HeaderName::from_static("set-cookie"),
        "auth_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );
    response
}
