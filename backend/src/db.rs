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

use crate::auth::CurrentUser;   // ← IMPORTANT : on importe CurrentUser

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
    pub limit: Option<i32>,
    pub before: Option<i64>,
}

// === HANDLERS AVEC Extension<CurrentUser> ===

pub async fn create_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,   // ← CHANGÉ
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO conversations (id, name, is_group, created_at, created_by, updated_at) 
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.is_group)
    .bind(now)
    .bind(&user.id)           // ← CHANGÉ : user.id au lieu de user_id
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query(
        "INSERT INTO conversation_participants (conversation_id, user_id, joined_at) 
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)           // ← CHANGÉ
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Conversation {
        id,
        name: req.name,
        is_group: req.is_group,
        created_at: now,
        created_by: user.id,  // ← CHANGÉ
        updated_at: now,
    }))
}

pub async fn get_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
) -> Result<Json<Conversation>, StatusCode> {
    let conversation = sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(conversation))
}

pub async fn get_user_conversations(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,   // ← CHANGÉ
) -> Result<Json<Vec<Conversation>>, StatusCode> {
    let conversations = sqlx::query_as::<_, Conversation>(
        "SELECT c.* FROM conversations c 
         INNER JOIN conversation_participants cp ON c.id = cp.conversation_id 
         WHERE cp.user_id = ?
         ORDER BY c.updated_at DESC",
    )
    .bind(&user.id)   // ← CHANGÉ
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(conversations))
}

pub async fn join_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,   // ← CHANGÉ
) -> Result<StatusCode, StatusCode> {
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at) 
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)   // ← CHANGÉ
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
    Extension(CurrentUser(user)): Extension<CurrentUser>,   // ← CHANGÉ
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_id, content, encrypted, timestamp, created_at, message_type) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(&user.id)           // ← CHANGÉ
    .bind(&req.content)
    .bind(req.encrypted)
    .bind(now)
    .bind(now)
    .bind("text")
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Message {
        id,
        conversation_id,
        sender_id: user.id,   // ← CHANGÉ
        content: req.content,
        message_type: "text".to_string(),
        file_id: None,
        encrypted: req.encrypted,
        timestamp: now,
        created_at: now,
        edited_at: None,
    }))
}

pub async fn get_conversation_messages(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
    Query(params): Query<MessageQueryParams>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);

    let messages = if let Some(before) = params.before {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages 
             WHERE conversation_id = ? AND created_at < ? 
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(&id)
        .bind(before)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages 
             WHERE conversation_id = ? 
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(&id)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages))
}

// =============================================================================
// PROFIL UTILISATEUR — update_user_profile
// Route : POST /api/user/update
// =============================================================================

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
    use axum::http::StatusCode;
    use serde_json::json;

    // Mettre à jour uniquement les champs fournis
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Le nom ne peut pas être vide" })),
            );
        }
        let result = sqlx::query("UPDATE users SET name = ? WHERE id = ?")
            .bind(name.trim())
            .bind(&user.id)
            .execute(&state.db)
            .await;

        if result.is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur lors de la mise à jour" })),
            );
        }
    }

    if let Some(ref email) = req.email {
        let result = sqlx::query("UPDATE users SET email = ? WHERE id = ?")
            .bind(email.trim())
            .bind(&user.id)
            .execute(&state.db)
            .await;

        if result.is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur lors de la mise à jour de l'email" })),
            );
        }
    }

    (
        StatusCode::OK,
        Json(json!({ "success": true, "message": "Profil mis à jour avec succès" })),
    )
}

// =============================================================================
// ÉVÉNEMENTS — get_events / create_event / delete_event
// Routes : GET|POST /api/events, DELETE /api/events/{id}
// =============================================================================

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
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events ORDER BY date ASC, time ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("[Events] Erreur GET: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({ "events": events })))
}

pub async fn create_event(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreateEventRequest>,
) -> impl axum::response::IntoResponse {
    use axum::http::StatusCode;
    use serde_json::json;

    if req.title.trim().is_empty() || req.date.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Titre et date requis" })),
        );
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let result = sqlx::query(
        "INSERT INTO events (id, title, date, time, description, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(req.title.trim())
    .bind(req.date.trim())
    .bind(&req.time)
    .bind(&req.description)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "success": true, "message": "Événement créé", "id": id })),
        ),
        Err(e) => {
            eprintln!("[Events] Erreur INSERT: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur lors de la création" })),
            )
        }
    }
}

pub async fn delete_event(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> impl axum::response::IntoResponse {
    use axum::http::StatusCode;
    use serde_json::json;

    // Seul le créateur ou un admin peut supprimer
    let event: Option<(String,)> = sqlx::query_as(
        "SELECT created_by FROM events WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match event {
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "message": "Événement introuvable" }))),
        Some((created_by,)) if created_by != user.id && user.role != "admin" => {
            return (StatusCode::FORBIDDEN, Json(json!({ "success": false, "message": "Accès refusé" })));
        }
        _ => {}
    }

    let result = sqlx::query("DELETE FROM events WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "message": "Erreur suppression" }))),
    }
}
