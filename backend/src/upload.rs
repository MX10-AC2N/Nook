use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
    Form,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;
use std::collections::HashMap;

use crate::SharedState;

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50 Mo
const MAX_DURATION: u64 = 60 * 10; // 10 minutes

#[derive(Deserialize)]
pub struct UploadMediaForm {
    conversation_id: String,
    media_type: String, // "audio" or "video"
    duration: u64,
    encrypted_keys: String, // JSON string
    nonce: String, // JSON array of bytes
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub url: Option<String>,
    pub message_id: Option<String>,
    pub file_path: Option<String>,
    pub encrypted_keys: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn handle_upload_media(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    let mut file_data = Vec::new();
    let mut filename = String::new();
    let mut form_data = HashMap::new();

    // Process multipart form
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            // Handle file upload
            filename = Uuid::new_v4().to_string() + ".enc";
            file_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
            
            if file_data.len() as u64 > MAX_FILE_SIZE {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
        } else {
            // Handle form fields
            let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            form_data.insert(name, value);
        }
    }

    // Validate required form fields
    let conversation_id = form_data.get("conversation_id").ok_or(StatusCode::BAD_REQUEST)?;
    let media_type = form_data.get("media_type").ok_or(StatusCode::BAD_REQUEST)?;
    let duration_str = form_data.get("duration").ok_or(StatusCode::BAD_REQUEST)?;
    let encrypted_keys = form_data.get("encrypted_keys").ok_or(StatusCode::BAD_REQUEST)?;
    let nonce = form_data.get("nonce").ok_or(StatusCode::BAD_REQUEST)?;

    // Validate media type
    if media_type != "audio" && media_type != "video" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate duration
    let duration = duration_str.parse::<u64>().map_err(|_| StatusCode::BAD_REQUEST)?;
    if duration == 0 || duration > MAX_DURATION {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Save file
    let uploads_dir = "/app/data/uploads";
    fs::create_dir_all(uploads_dir).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_path = format!("{}/{}", uploads_dir, filename);
    
    fs::write(&file_path, file_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate message ID
    let message_id = Uuid::new_v4().to_string();

    // Get current timestamp
    let timestamp = chrono::Utc::now().timestamp();

    // Insert message into database
    let query = r#"
        INSERT INTO messages (
            id, conversation_id, sender_id, content, encrypted_keys, 
            nonce, media_type, media_url, duration, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    let sender_id = match sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM sessions WHERE token = ? AND expires_at > ?"
    )
    .bind(form_data.get("token").unwrap_or(&"".to_string()))
    .bind(chrono::Utc::now().timestamp())
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // Get sender name
    let sender_name = match sqlx::query_scalar::<_, String>(
        "SELECT name FROM users WHERE id = ?"
    )
    .bind(&sender_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(name)) => name,
        _ => "Unknown".to_string(),
    };

    sqlx::query(query)
        .bind(&message_id)
        .bind(conversation_id)
        .bind(&sender_id)
        .bind("") // Content is empty for media messages
        .bind(encrypted_keys)
        .bind(nonce)
        .bind(media_type)
        .bind(format!("/uploads/{}", filename))
        .bind(duration as i64)
        .bind(timestamp)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update conversation last message
    let update_conv = r#"
        UPDATE conversations 
        SET last_message_at = ?, last_message_preview = ?
        WHERE id = ?
    "#;
    
    sqlx::query(update_conv)
        .bind(timestamp)
        .bind(format!("[{} message]", media_type))
        .bind(conversation_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update unread counts for other participants
    let update_unread = r#"
        UPDATE conversations c
        SET unread_count = unread_count + 1
        WHERE c.id = ?
        AND EXISTS (
            SELECT 1 FROM conversation_members cm
            WHERE cm.conversation_id = c.id
            AND cm.user_id != ?
        )
    "#;
    
    sqlx::query(update_unread)
        .bind(conversation_id)
        .bind(&sender_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Broadcast new message via WebSocket if needed
    // (Implementation depends on your WebSocket setup)

    Ok(Json(UploadResponse {
        success: true,
        url: Some(format!("/uploads/{}", filename)),
        message_id: Some(message_id),
        file_path: Some(format!("/uploads/{}", filename)),
        encrypted_keys: Some(encrypted_keys.clone()),
        nonce: Some(nonce.clone()),
    }))
}

// Legacy handler for backward compatibility
pub async fn handle_upload(mut multipart: Multipart) -> Result<Json<UploadResponse>, StatusCode> {
    // Récupère le premier (et unique) champ
    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        if data.len() as u64 > MAX_FILE_SIZE {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }

        let filename = uuid::Uuid::new_v4().to_string() + ".enc";
        let path = format!("/app/data/uploads/{}", filename);
        fs::create_dir_all("/app/data/uploads").await.ok();
        fs::write(&path, data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(UploadResponse {
            success: true,
            url: Some(format!("/uploads/{}", filename)),
            message_id: None,
            file_path: None,
            encrypted_keys: None,
            nonce: None,
        }));
    }

    Err(StatusCode::BAD_REQUEST)
}
