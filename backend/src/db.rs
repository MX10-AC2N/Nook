// db.rs - Structures et utilitaires de base de données

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

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

// === FONCTIONS UTILITAIRES ===

pub async fn create_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Extension(user_id): Extension<String>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    sqlx::query::<sqlx::Sqlite>(
        "INSERT INTO conversations (id, name, is_group, created_at, created_by, updated_at) 
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.is_group)
    .bind(now)
    .bind(&user_id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query::<sqlx::Sqlite>(
        "INSERT INTO conversation_participants (conversation_id, user_id, joined_at) 
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user_id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Conversation {
        id,
        name: req.name,
        is_group: req.is_group,
        created_at: now,
        created_by: user_id,
        updated_at: now,
    }))
}

pub async fn get_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
) -> Result<Json<Conversation>, StatusCode> {
    let conversation =
        sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(conversation))
}

pub async fn get_user_conversations(
    State(state): State<Arc<crate::SharedState>>,
    Extension(user_id): Extension<String>,
) -> Result<Json<Vec<Conversation>>, StatusCode> {
    let conversations = sqlx::query_as::<_, Conversation>(
        "SELECT c.* FROM conversations c 
         INNER JOIN conversation_participants cp ON c.id = cp.conversation_id 
         WHERE cp.user_id = ?
         ORDER BY c.updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(conversations))
}

pub async fn join_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<StatusCode, StatusCode> {
    let now = chrono::Utc::now().timestamp();

    sqlx::query::<sqlx::Sqlite>(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at) 
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user_id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query::<sqlx::Sqlite>("UPDATE conversations SET updated_at = ? WHERE id = ?")
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
    Extension(user_id): Extension<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    sqlx::query::<sqlx::Sqlite>(
        "INSERT INTO messages (id, conversation_id, sender_id, content, encrypted, timestamp, created_at, message_type) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(&user_id)
    .bind(&req.content)
    .bind(req.encrypted)
    .bind(now)
    .bind(now)
    .bind("text")
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query::<sqlx::Sqlite>("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Message {
        id,
        conversation_id,
        sender_id: user_id,
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
