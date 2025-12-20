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

#[derive(Deserialize)]
pub struct CreatePasswordRequest {
    pub password: String,
}

#[derive(Deserialize)]
pub struct MemberLoginRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateMemberRequest {
    pub name: String,
    pub username: String,
    pub temporary_password: String,
}

#[derive(Deserialize)]
pub struct ChangeTemporaryPasswordRequest {
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

#[derive(Serialize)]
pub struct PasswordStatus {
    pub has_password: bool,
    pub name: String,
    pub member_id: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Member {
    pub id: String,
    pub username: Option<String>,
    pub name: String,
    pub approved: bool,
    pub has_password: bool,
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
        message: id,
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

/// Crée un nouveau membre directement (pour admin)
pub async fn create_member(
    pool: &SqlitePool,
    req: CreateMemberRequest,
) -> Result<ApiResponse, StatusCode> {
    // Vérifier que le username n'existe pas déjà
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM members WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
        return Ok(ApiResponse {
            success: false,
            message: "Ce nom d'utilisateur est déjà pris".to_string(),
        });
    }

    // Générer un ID unique
    let id = Uuid::new_v4().to_string();
    
    // Hasher le mot de passe temporaire
    let password_hash = hash(&req.temporary_password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Générer une clé publique par défaut (l'utilisateur pourra la changer)
    let default_public_key = format!("temp_key_{}", id);

    // Insérer le membre (approuvé d'office)
    sqlx::query(
        "INSERT INTO members (id, username, name, password_hash, public_key, approved, has_password) 
         VALUES (?, ?, ?, ?, ?, 1, 1)"
    )
    .bind(&id)
    .bind(&req.username)
    .bind(&req.name)
    .bind(&password_hash)
    .bind(&default_public_key)
    .execute(pool)
    .await
    .map_err(|e| {
        println!("Erreur création membre: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(ApiResponse {
        success: true,
        message: format!("Membre créé avec succès. ID: {}, Username: {}, Mot de passe temporaire: {}", 
            id, req.username, req.temporary_password),
    })
}

pub async fn get_members(pool: &SqlitePool) -> Result<Vec<Member>, StatusCode> {
    let rows = sqlx::query_as::<_, Member>(
        "SELECT id, username, name, approved, has_password, joined_at FROM members ORDER BY joined_at DESC"
    )
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
            Ok(res) => {
                Ok(Json(ApiResponse {
                    success: true,
                    message: format!("Demande envoyée. Votre ID: {}", res.message),
                }))
            },
            Err(e) => Err(e),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn create_member_handler(
    State(state): State<SharedState>,
    Json(payload): Json<CreateMemberRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    create_member(&state.db, payload).await.map(Json)
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

pub async fn member_login_handler(
    State(state): State<SharedState>,
    Json(payload): Json<MemberLoginRequest>,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Json<ApiResponse>), StatusCode> {
    // Chercher par username ou member_id
    let row: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, password_hash FROM members 
         WHERE (id = ? OR username = ?) AND approved = 1"
    )
    .bind(&payload.identifier)
    .bind(&payload.identifier)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let (member_id, member_name, stored_hash_opt) = row.unwrap();
    
    // Vérifier si le membre a un mot de passe défini
    match stored_hash_opt {
        Some(stored_hash) if !stored_hash.is_empty() => {
            if !verify(&payload.password, &stored_hash).unwrap_or(false) {
                return Err(StatusCode::UNAUTHORIZED);
            }
        },
        _ => return Err(StatusCode::UNAUTHORIZED),
    }

    let session_token = create_session(&state.db, &member_id).await?;

    let cookie = format!(
        "nook_session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=2592000",
        session_token
    );

    Ok((
        AppendHeaders([(SET_COOKIE, cookie)]),
        Json(ApiResponse {
            success: true,
            message: format!("Connecté en tant que {}", member_name),
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

// === Handler de création de mot de passe (ancien système) ===

pub async fn create_password_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
    Json(payload): Json<CreatePasswordRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    // Extraire le cookie de session
    let session_token = match get_cookie(&headers, "nook_session") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Valider la session et récupérer l'ID du membre
    let (member_id, _) = validate_session_and_get_user(&state.db, &session_token).await?;

    // Hasher le mot de passe
    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Mettre à jour le membre dans la base de données
    sqlx::query(
        "UPDATE members SET password_hash = ?, has_password = 1 WHERE id = ?"
    )
    .bind(&password_hash)
    .bind(&member_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Mot de passe créé avec succès".to_string(),
    }))
}

// === Handler de vérification de statut de mot de passe ===

pub async fn check_password_status_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> Result<Json<PasswordStatus>, StatusCode> {
    // Extraire le cookie de session
    let session_token = match get_cookie(&headers, "nook_session") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Valider la session et récupérer les infos du membre
    let (member_id, member_name) = validate_session_and_get_user(&state.db, &session_token).await?;

    // Vérifier si le membre a un mot de passe
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT password_hash FROM members WHERE id = ?"
    )
    .bind(&member_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let has_password = match row {
        Some((Some(hash),)) => !hash.is_empty(),
        _ => false,
    };

    Ok(Json(PasswordStatus {
        has_password,
        name: member_name,
        member_id,
    }))
}

// === Handler de vérification de changement de mot de passe (nouveau système) ===

pub async fn check_password_change_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> Result<Json<FirstLoginCheck>, StatusCode> {
    let session_token = match get_cookie(&headers, "nook_session") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let (member_id, _) = validate_session_and_get_user(&state.db, &session_token).await?;

    // Vérifier si le mot de passe est temporaire (par défaut "changeme123")
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT password_hash FROM members WHERE id = ?"
    )
    .bind(&member_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let needs_password_change = match row {
        Some((hash,)) => {
            // Si le hash correspond à "changeme123" (mot de passe temporaire par défaut)
            verify("changeme123", &hash).unwrap_or(false)
        },
        None => false,
    };

    Ok(Json(FirstLoginCheck {
        needs_password_change,
    }))
}

// === Handler de changement de mot de passe temporaire ===

pub async fn change_temporary_password_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
    Json(payload): Json<ChangeTemporaryPasswordRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let session_token = match get_cookie(&headers, "nook_session") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let (member_id, _) = validate_session_and_get_user(&state.db, &session_token).await?;

    // Récupérer le hash actuel
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT password_hash FROM members WHERE id = ?"
    )
    .bind(&member_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let current_hash = row.unwrap().0;

    // Vérifier l'ancien mot de passe
    if !verify(&payload.current_password, &current_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Hasher le nouveau mot de passe
    let new_hash = hash(&payload.new_password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Mettre à jour
    sqlx::query("UPDATE members SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(&member_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Mot de passe changé avec succès".to_string(),
    }))
}

// === Handler de validation de session ===

pub async fn validate_session_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> Result<Json<SessionInfo>, StatusCode> {
    let session_token = match get_cookie(&headers, "nook_session") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match validate_session_and_get_user(&state.db, &session_token).await {
        Ok((member_id, member_name)) => {
            Ok(Json(SessionInfo {
                member_id,
                member_name,
            }))
        },
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

// === Handler de validation admin ===

pub async fn admin_validate_handler(
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let admin_token = match get_cookie(&headers, "nook_admin") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match validate_admin_session(&state.db, &admin_token).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            message: "Session admin valide".to_string(),
        })),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

// === Utilitaires cookies ===

fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get_all(axum::http::header::COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(|cookie| cookie.trim())
        .find(|cookie| cookie.starts_with(&format!("{}=", name)))
        .and_then(|cookie| cookie.split('=').nth(1).map(|s| s.to_string()))
}

// === Vérification première connexion admin ===

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

// === Changement de mot de passe admin ===

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
