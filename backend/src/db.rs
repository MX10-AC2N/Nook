// db.rs - Structures et utilitaires de base de données

use sqlx::{SqlitePool, FromRow, Row};
use axum::{
    extract::{Path, Query, State},
    Extension,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;

// === STRUCTURES DE DONNÉES ===

#[derive(Clone, Debug, Serialise, Deserialise, sqlx::FromRow)]
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

pub async fn get_user_by_id(pool: &SqlitePool, user_id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();
    
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(created_at)
    .execute(pool)
    .await?;
    
    Ok(User {
        id: user_id.to_string(),
        username,
        password_hash,
        name: username.clone(), // ou autre valeur par défaut
        role,
        approved: role == "admin", // ou true/false selon votre logique
        needs_password_change: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn create_conversation(pool: &SqlitePool, user_id: &str) -> Result<Conversation, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    
    sqlx::query(
        "INSERT INTO conversations (id, created_at, updated_at) VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    
    // Ajouter le créateur comme participant
    sqlx::query(
        "INSERT INTO conversation_participants (conversation_id, user_id, joined_at) VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(now)
    .execute(pool)
    .await?;
    
    Ok(Conversation {
        id,
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<Conversation>, sqlx::Error> {
    sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
        .bind(conversation_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_conversations(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<Conversation>, sqlx::Error> {
    let conversations = sqlx::query_as::<_, Conversation>(
        "SELECT c.* FROM conversations c
         INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
         WHERE cp.user_id = ?
         ORDER BY c.updated_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    
    Ok(conversations)
}

pub async fn join_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp();
    
    sqlx::query(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at) VALUES (?, ?, ?)",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(now)
    .execute(pool)
    .await?;
    
    // Mettre à jour updated_at
    sqlx::query(
        "UPDATE conversations SET updated_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(conversation_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn send_message(
    pool: &SqlitePool,
    conversation_id: &str,
    user_id: &str,
    content: Option<&str>,
    message_type: &str,
    file_id: Option<&str>,
) -> Result<Message, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();
    
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, user_id, content, message_type, file_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(user_id)
    .bind(content)
    .bind(message_type)
    .bind(file_id)
    .bind(created_at)
    .execute(pool)
    .await?;
    
    // Mettre à jour updated_at de la conversation
    sqlx::query(
        "UPDATE conversations SET updated_at = ? WHERE id = ?",
    )
    .bind(created_at)
    .bind(conversation_id)
    .execute(pool)
    .await?;
    
    Ok(Message {
        id,
        conversation_id: conversation_id.to_string(),
        user_id: user_id.to_string(),
        content: content.map(|s| s.to_string()),
        message_type: message_type.to_string(),
        file_id: file_id.map(|s| s.to_string()),
        created_at,
        edited_at: None,
    })
}

pub async fn get_conversation_messages(
    pool: &SqlitePool,
    conversation_id: &str,
    limit: Option<i64>,
    before: Option<i64>,
) -> Result<Vec<Message>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    
    let query = if let Some(before_timestamp) = before {
        "SELECT * FROM messages WHERE conversation_id = ? AND created_at < ? ORDER BY created_at DESC LIMIT ?"
    } else {
        "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT ?"
    };
    
    let mut query = sqlx::query_as::<_, Message>(query).bind(conversation_id).bind(limit);
    
    if let Some(before_timestamp) = before {
        query = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE conversation_id = ? AND created_at < ? ORDER BY created_at DESC LIMIT ?"
        ).bind(conversation_id).bind(before_timestamp).bind(limit);
    } else {
        query = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT ?"
        ).bind(conversation_id).bind(limit);
    }
    
    let messages = query.fetch_all(pool).await?;
    
    // Inverser pour avoir l'ordre chronologique
    let mut messages = messages;
    messages.reverse();
    
    Ok(messages)
}

pub async fn get_upload(pool: &SqlitePool, upload_id: &str) -> Result<Option<Upload>, sqlx::Error> {
    sqlx::query_as::<_, Upload>("SELECT * FROM uploads WHERE id = ?")
        .bind(upload_id)
        .fetch_optional(pool)
        .await
}

pub async fn delete_upload(pool: &SqlitePool, upload_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM uploads WHERE id = ?")
        .bind(upload_id)
        .execute(pool)
        .await?;
    
    Ok(())
}
