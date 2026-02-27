// db.rs - Structures et handlers DB avec Extension<CurrentUser>

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::CurrentUser;

// === STRUCTURES DE DONNÉES ===

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub name: Option<String>,
    pub role: String,
    pub approved: bool,
    pub needs_password_change: bool,
    pub token: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: String,
    pub name: Option<String>,
    pub is_group: bool,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
}

/// Message brut en base (sans sender_name) — utilisé uniquement pour send_message
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub message_type: String,
    pub file_id: Option<String>,
    pub encrypted: bool,
    pub timestamp: i64,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

/// Message enrichi avec le nom de l'expéditeur (JOIN users)
/// Retourné par GET /conversations/{id}/messages
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageWithSender {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String,        // COALESCE(users.name, users.username)
    pub content: String,
    pub message_type: String,
    pub file_id: Option<String>,
    pub encrypted: bool,
    pub timestamp: i64,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Upload {
    pub id: String,
    pub conversation_id: Option<String>,
    pub from_user_id: Option<String>,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub content_type: Option<String>,
    pub uploaded_at: i64,
    pub encrypted: bool,
    pub nonce: Option<String>,
    pub key_text: Option<String>,
}

// === REQUEST STRUCTURES ===

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub name: Option<String>,
    pub is_group: bool,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub encrypted: bool,
}

#[derive(Debug, Deserialize)]
pub struct MessageQueryParams {
    pub limit: Option<i64>,
    pub before: Option<i64>,
}

// === HANDLERS ===

pub async fn create_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, StatusCode> {
    let id  = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO conversations (id, name, is_group, created_at, created_by, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.is_group)
    .bind(now)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query(
        "INSERT INTO conversation_participants (conversation_id, user_id, joined_at)
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Conversation {
        id,
        name: req.name,
        is_group: req.is_group,
        created_at: now,
        created_by: user.id,
        updated_at: now,
    }))
}

pub async fn get_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
) -> Result<Json<Conversation>, StatusCode> {
    let conv = sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(conv))
}

pub async fn get_user_conversations(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<Vec<Conversation>>, StatusCode> {
    let convs = sqlx::query_as::<_, Conversation>(
        "SELECT c.* FROM conversations c
         INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
         WHERE cp.user_id = ?
         ORDER BY c.updated_at DESC",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(convs))
}

pub async fn join_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<StatusCode, StatusCode> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn send_message(
    State(state): State<Arc<crate::SharedState>>,
    Path(conversation_id): Path<String>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id  = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    // Vérifier que la conversation existe
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM conversations WHERE id = ?"
    )
    .bind(&conversation_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if exists.map(|(c,)| c).unwrap_or(0) == 0 {
        eprintln!("[send_message] Conversation '{}' introuvable", conversation_id);
        return Err(StatusCode::NOT_FOUND);
    }

    sqlx::query(
        "INSERT INTO messages
            (id, conversation_id, sender_id, content, encrypted, timestamp, created_at, message_type)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(&user.id)
    .bind(&req.content)
    .bind(req.encrypted)
    .bind(now)
    .bind(now)
    .bind("text")
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("[send_message] Erreur DB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Retourner le message enrichi (avec sender_name) pour cohérence frontend
    let sender_name: String = sqlx::query_as::<_, (String,)>(
        "SELECT COALESCE(name, username) FROM users WHERE id = ?"
    )
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .map(|(n,)| n)
    .unwrap_or_else(|| user.username.clone());

    Ok(Json(serde_json::json!({
        "id": id,
        "conversation_id": conversation_id,
        "sender_id": user.id,
        "sender_name": sender_name,
        "content": req.content,
        "message_type": "text",
        "file_id": null,
        "encrypted": req.encrypted,
        "timestamp": now,
        "created_at": now,
        "edited_at": null
    })))
}

pub async fn get_conversation_messages(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
    Query(params): Query<MessageQueryParams>,
) -> Result<Json<Vec<MessageWithSender>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);

    // JOIN sur users pour récupérer sender_name en une requête
    // ORDER BY ASC : les plus anciens en premier (ordre d'affichage naturel)
    let messages = if let Some(before) = params.before {
        sqlx::query_as::<_, MessageWithSender>(
            "SELECT
                m.id, m.conversation_id, m.sender_id,
                COALESCE(u.name, u.username) AS sender_name,
                m.content, m.message_type, m.file_id,
                m.encrypted, m.timestamp, m.created_at, m.edited_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.sender_id
             WHERE m.conversation_id = ? AND m.created_at < ?
             ORDER BY m.created_at ASC
             LIMIT ?",
        )
        .bind(&id)
        .bind(before)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, MessageWithSender>(
            "SELECT
                m.id, m.conversation_id, m.sender_id,
                COALESCE(u.name, u.username) AS sender_name,
                m.content, m.message_type, m.file_id,
                m.encrypted, m.timestamp, m.created_at, m.edited_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.sender_id
             WHERE m.conversation_id = ?
             ORDER BY m.created_at ASC
             LIMIT ?",
        )
        .bind(&id)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| {
        eprintln!("[get_conversation_messages] Erreur DB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(messages))
}

// === PROFIL UTILISATEUR ===

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub async fn update_user_profile(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<UpdateProfileRequest>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;

    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return (StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Le nom ne peut pas être vide" })));
        }
        if sqlx::query("UPDATE users SET name = ? WHERE id = ?")
            .bind(name.trim()).bind(&user.id)
            .execute(&state.db).await.is_err()
        {
            return (StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur de mise à jour" })));
        }
    }

    if let Some(ref email) = req.email {
        if sqlx::query("UPDATE users SET email = ? WHERE id = ?")
            .bind(email.trim()).bind(&user.id)
            .execute(&state.db).await.is_err()
        {
            return (StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur de mise à jour de l'email" })));
        }
    }

    (StatusCode::OK, Json(json!({ "success": true, "message": "Profil mis à jour" })))
}

// === ÉVÉNEMENTS (CALENDRIER) ===

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub date: String,
    pub time: Option<String>,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub date: String,
    pub time: Option<String>,
    pub description: Option<String>,
}

pub async fn get_events(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(_)): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events ORDER BY date ASC, time ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| { eprintln!("[events] GET error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "events": events })))
}

pub async fn create_event(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreateEventRequest>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;

    if req.title.trim().is_empty() || req.date.trim().is_empty() {
        return (StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Titre et date requis" })));
    }

    let id  = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    match sqlx::query(
        "INSERT INTO events (id, title, date, time, description, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(req.title.trim()).bind(req.date.trim())
    .bind(&req.time).bind(&req.description).bind(&user.id).bind(now)
    .execute(&state.db).await
    {
        Ok(_)  => (StatusCode::OK, Json(json!({ "success": true, "id": id }))),
        Err(e) => {
            eprintln!("[events] INSERT error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur création" })))
        }
    }
}

pub async fn delete_event(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;

    let row: Option<(String,)> = sqlx::query_as("SELECT created_by FROM events WHERE id = ?")
        .bind(&id).fetch_optional(&state.db).await.ok().flatten();

    match row {
        None => return (StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "message": "Événement introuvable" }))),
        Some((created_by,)) if created_by != user.id && user.role != "admin" =>
            return (StatusCode::FORBIDDEN,
                Json(json!({ "success": false, "message": "Accès refusé" }))),
        _ => {}
    }

    match sqlx::query("DELETE FROM events WHERE id = ?").bind(&id).execute(&state.db).await {
        Ok(_)  => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": "Erreur suppression" }))),
    }
}
