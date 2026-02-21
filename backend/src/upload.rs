// backend/src/upload.rs - Gestion des uploads avec chiffrement + CurrentUser

use crate::{auth::CurrentUser, webrtc::encrypt_file_for_storage, SharedState};
use axum::{
    extract::{Multipart, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json as AxumJson,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ====================== STRUCTURES ======================

#[derive(serde::Serialize)]
pub struct UploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub file_size: usize,
    pub uploaded_at: i64,
    pub encrypted: bool,
    pub url: String,
}

// ====================== HANDLERS (protégés) ======================

pub async fn upload_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,   // ← AUTHENTIFIÉ
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut form_data = UploadFormData::default();

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "conversation_id" {
            if let Ok(text) = field.text().await {
                form_data.conversation_id = text;
            }
        } else if name == "file" {
            let filename = field.file_name().map(|s| s.to_string()).unwrap_or_default();
            let content_type = field.content_type().map(|s| s.to_string()).unwrap_or_default();

            let mut data_vec = Vec::new();
            while let Some(chunk) = field.chunk().await.transpose() {
                match chunk {
                    Ok(bytes) => data_vec.extend_from_slice(&bytes),
                    Err(e) => {
                        eprintln!("[Upload] Erreur lecture chunk: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Failed to read file"})));
                    }
                }
            }
            form_data.data = Some(data_vec);
            form_data.file_name = filename;
            form_data.content_type = content_type;
        }
    }

    let data = match form_data.validate() {
        Ok(data) => data,
        Err(e) => return (StatusCode::BAD_REQUEST, AxumJson(json!({"error": e}))),
    };

    let file_id = Uuid::new_v4().to_string();
    let file_ext = std::path::Path::new(&data.file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let stored_filename = format!("{}.{}", file_id, file_ext);
    let storage_path = state.file_manager.get_uploads_dir().join(&stored_filename);

    // Chiffrement (toujours activé pour la sécurité)
    let (ciphertext, nonce_b64, key_b64) = encrypt_file_for_storage(&data.data);
    if let Err(e) = tokio::fs::write(&storage_path, &ciphertext).await {
        eprintln!("[Upload] Erreur écriture: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Failed to save file"})));
    }

    let now = Utc::now().timestamp();
    let query = r#"
        INSERT INTO uploads (id, conversation_id, from_user_id, file_name, file_path, file_size, content_type, uploaded_at, encrypted, nonce, key_text)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    if let Err(e) = sqlx::query(query)
        .bind(&file_id)
        .bind(&data.conversation_id)
        .bind(&user.id)                    // ← CurrentUser
        .bind(&data.file_name)
        .bind(storage_path.to_str().unwrap_or(""))
        .bind(data.data.len() as i64)
        .bind(&data.content_type)
        .bind(now)
        .bind(true)
        .bind(&nonce_b64)
        .bind(&key_b64)
        .execute(&state.db)
        .await
    {
        eprintln!("[Upload] Erreur DB: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Database error"})));
    }

    state.file_manager.register_file(&file_id, storage_path).await;

    (StatusCode::OK, AxumJson(json!({
        "status": "uploaded",
        "file_id": file_id,
        "file_name": data.file_name,
        "file_size": data.data.len(),
        "uploaded_at": now,
        "encrypted": true,
        "url": format!("/files/{}", file_id)
    })))
}

pub async fn upload_chat_file(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,   // ← CurrentUser
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut form_data = UploadFormData::default();

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "conversation_id" {
            if let Ok(text) = field.text().await {
                form_data.conversation_id = text;
            }
        } else if name == "file" {
            let filename = field.file_name().map(|s| s.to_string()).unwrap_or_default();

            let mut data_vec = Vec::new();
            while let Some(chunk) = field.chunk().await.transpose() {
                match chunk {
                    Ok(bytes) => data_vec.extend_from_slice(&bytes),
                    Err(e) => {
                        eprintln!("[Upload Chat] Erreur lecture chunk: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Failed to read file"})));
                    }
                }
            }
            form_data.data = Some(data_vec);
            form_data.file_name = filename;
            form_data.content_type = "application/octet-stream".to_string();
        }
    }

    let data = match form_data.validate() {
        Ok(data) => data,
        Err(e) => return (StatusCode::BAD_REQUEST, AxumJson(json!({"error": e}))),
    };

    let file_id = Uuid::new_v4().to_string();
    let storage_path = state.file_manager.get_uploads_dir().join(&file_id);

    let (ciphertext, nonce_b64, key_b64) = encrypt_file_for_storage(&data.data);
    if let Err(e) = tokio::fs::write(&storage_path, &ciphertext).await {
        eprintln!("[Upload Chat] Erreur écriture: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Failed to save file"})));
    }

    let now = Utc::now().timestamp();
    let query = r#"
        INSERT INTO uploads (id, conversation_id, from_user_id, file_name, file_path, file_size, content_type, uploaded_at, encrypted, nonce, key_text)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    if let Err(e) = sqlx::query(query)
        .bind(&file_id)
        .bind(&data.conversation_id)
        .bind(&user.id)                    // ← CurrentUser
        .bind(&data.file_name)
        .bind(storage_path.to_str().unwrap_or(""))
        .bind(data.data.len() as i64)
        .bind(&data.content_type)
        .bind(now)
        .bind(true)
        .bind(&nonce_b64)
        .bind(&key_b64)
        .execute(&state.db)
        .await
    {
        eprintln!("[Upload Chat] Erreur DB: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Database error"})));
    }

    state.file_manager.register_file(&file_id, storage_path).await;

    (StatusCode::OK, AxumJson(json!({
        "status": "uploaded",
        "file_id": file_id,
        "file_name": data.file_name,
        "file_size": data.data.len(),
        "uploaded_at": now,
        "encrypted": true,
        "url": format!("/files/{}", file_id)
    })))
}

// ====================== STRUCTURES INTERNES ======================

#[derive(Default)]
struct UploadFormData {
    conversation_id: String,
    file_name: String,
    content_type: String,
    data: Option<Vec<u8>>,
}

struct ValidatedUploadData {
    conversation_id: String,
    file_name: String,
    content_type: String,
    data: Vec<u8>,
}

impl UploadFormData {
    fn validate(self) -> Result<ValidatedUploadData, String> {
        let data = self.data.ok_or("Aucun fichier fourni")?;

        if data.is_empty() {
            return Err("Fichier vide".to_string());
        }
        if data.len() > 50 * 1024 * 1024 {
            return Err("Fichier trop volumineux (>50Mo)".to_string());
        }

        Ok(ValidatedUploadData {
            conversation_id: self.conversation_id,
            file_name: self.file_name,
            content_type: self.content_type,
            data,
        })
    }
}
