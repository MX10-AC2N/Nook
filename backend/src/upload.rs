// backend/src/upload.rs

use crate::db::Upload;
use crate::SharedState;
use crate::webrtc::{decrypt_file_from_storage, encrypt_file_for_storage, FileManager};
use axum::{
    extract::{Path, State as AxumState},
    http::{header::CONTENT_DISPOSITION, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub id: String,
    pub file_name: String,
    pub content_type: String,
    pub size: i64,
    pub path: String,
    pub encrypted: bool,
    pub nonce: Option<String>,
    pub key: Option<String>,
}

lazy_static::lazy_static! {
    static ref FILE_MANAGER: Arc<FileManager> = Arc::new(FileManager::new(PathBuf::from("/app/data/uploads")));
}

// Structure simple pour recevoir les données multipart
struct FormData {
    file_name: Option<String>,
    content_type: Option<String>,
    data: Option<Vec<u8>>,
    sender_id: Option<String>,
}

pub async fn upload_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    let mut form_data = FormData {
        file_name: None,
        content_type: None,
        data: None,
        sender_id: None,
    };

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            
            let mut data_vec = Vec::new();
            let mut stream = field.stream();
            while let Some(chunk) = stream.next().await {
                let bytes = match chunk {
                    Ok(bytes) => bytes,
                    Err(_) => break,
                };
                data_vec.extend_from_slice(&bytes);
            }
            
            form_data.file_name = Some(file_name);
            form_data.content_type = Some(content_type);
            form_data.data = Some(data_vec);
        } else if name == "sender_id" {
            if let Ok(text) = field.text().await {
                form_data.sender_id = Some(text);
            }
        }
    }

    let (file_name, data, content_type) = match (form_data.file_name, form_data.data, form_data.content_type) {
        (Some(name), Some(data), Some(ct)) => (name, data, ct),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No file provided" })),
            ).into_response();
        }
    };

    let sender_id = form_data.sender_id.unwrap_or_else(|| "anonymous".to_string());
    let id = Uuid::new_v4().to_string();

    let (encrypted_path, nonce, key) = if data.len() > 50 * 1024 * 1024 {
        // Fichier > 50Mo : P2P uniquement, pas de stockage serveur
        (format!("/app/data/uploads/{}_p2p_{}", id, file_name), None, None)
    } else {
        // Fichier < 50Mo : chiffrement et stockage
        let (ciphertext, nonce_b64, key_b64) = encrypt_file_for_storage(&data);
        
        let encrypted_path = format!("/app/data/uploads/{}_{}", id, file_name);
        
        let mut file = File::create(&encrypted_path).await.unwrap();
        file.write_all(&ciphertext).await.unwrap();
        
        FILE_MANAGER.register_file(&id, PathBuf::from(&encrypted_path));
        
        (encrypted_path, Some(nonce_b64), Some(key_b64))
    };

    let timestamp = Utc::now().timestamp();

    let _ = sqlx::query(
        "INSERT INTO uploads (id, file_name, content_type, size, path, sender_id, timestamp, encrypted, nonce, key_text)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&file_name)
    .bind(&content_type)
    .bind(data.len() as i64)
    .bind(&encrypted_path)
    .bind(&sender_id)
    .bind(timestamp)
    .bind(nonce.is_some())
    .bind(nonce.as_deref())
    .bind(key.as_deref())
    .execute(&state.db)
    .await;

    Json(json!({
        "success": true,
        "id": id,
        "file_name": file_name,
        "content_type": content_type,
        "size": data.len(),
        "path": encrypted_path,
        "encrypted": nonce.is_some(),
        "nonce": nonce,
        "key": key
    })).into_response()
}

pub async fn upload_chat_file(
    AxumState(state): AxumState<Arc<SharedState>>,
    Path((conversation_id, sender_id, message_type)): Path<(String, String, String)>,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    let mut form_data = FormData {
        file_name: None,
        content_type: None,
        data: None,
        sender_id: None,
    };

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            
            let mut data_vec = Vec::new();
            let mut stream = field.stream();
            while let Some(chunk) = stream.next().await {
                let bytes = match chunk {
                    Ok(bytes) => bytes,
                    Err(_) => break,
                };
                data_vec.extend_from_slice(&bytes);
            }
            
            form_data.file_name = Some(file_name);
            form_data.content_type = Some(content_type);
            form_data.data = Some(data_vec);
        }
    }

    let (file_name, data, content_type) = match (form_data.file_name, form_data.data, form_data.content_type) {
        (Some(name), Some(data), Some(ct)) => (name, data, ct),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No file provided" })),
            ).into_response();
        }
    };

    let id = Uuid::new_v4().to_string();

    let (encrypted_path, nonce, key) = if data.len() > 50 * 1024 * 1024 {
        // Fichier > 50Mo : P2P uniquement
        (format!("/app/data/uploads/{}_p2p_{}", id, file_name), None, None)
    } else {
        // Fichier < 50Mo : chiffrement et stockage
        let (ciphertext, nonce_b64, key_b64) = encrypt_file_for_storage(&data);
        
        let encrypted_path = format!("/app/data/uploads/{}_{}", id, file_name);
        
        let mut file = File::create(&encrypted_path).await.unwrap();
        file.write_all(&ciphertext).await.unwrap();
        
        FILE_MANAGER.register_file(&id, PathBuf::from(&encrypted_path));
        
        (encrypted_path, Some(nonce_b64), Some(key_b64))
    };

    let timestamp = Utc::now().timestamp();

    let _ = sqlx::query(
        "INSERT INTO uploads (id, file_name, content_type, size, path, sender_id, timestamp, encrypted, nonce, key_text)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&file_name)
    .bind(&content_type)
    .bind(data.len() as i64)
    .bind(&encrypted_path)
    .bind(&sender_id)
    .bind(timestamp)
    .bind(nonce.is_some())
    .bind(nonce.as_deref())
    .bind(key.as_deref())
    .execute(&state.db)
    .await;

    Json(json!({
        "success": true,
        "id": id,
        "file_name": file_name,
        "content_type": content_type,
        "size": data.len(),
        "path": encrypted_path,
        "encrypted": nonce.is_some(),
        "nonce": nonce,
        "key": key
    })).into_response()
}

pub async fn get_upload(
    AxumState(state): AxumState<Arc<SharedState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let upload: Option<Upload> = sqlx::query_as(
        "SELECT id, file_name, content_type, size, path, sender_id, timestamp, encrypted, nonce, key_text FROM uploads WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match upload {
        Some(upload) => {
            let path = PathBuf::from(&upload.path);
            
            // Vérifier si le fichier existe
            if !path.exists() {
                return (StatusCode::NOT_FOUND, Json(json!({ "error": "File not found" }))).into_response();
            }

            // Si chiffré, déchiffrer à la volée
            if upload.encrypted {
                match tokio::fs::read(&path).await {
                    Ok(ciphertext) => {
                        if let (Some(nonce), Some(key)) = (&upload.nonce, &upload.key_text) {
                            match decrypt_file_from_storage(&ciphertext, nonce, key) {
                                Ok(data) => {
                                    let headers = [
                                        (CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", upload.file_name)),
                                    ];
                                    return (StatusCode::OK, headers, data).into_response();
                                }
                                Err(e) => {
                                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Decryption failed: {}", e) }))).into_response();
                                }
                            }
                        } else {
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Missing encryption keys" }))).into_response();
                        }
                    }
                    Err(e) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to read file: {}", e) }))).into_response();
                    }
                }
            }

            // Fichier non chiffré : servir directement
            match tokio::fs::read(&path).await {
                Ok(data) => {
                    let headers = [
                        (CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", upload.file_name)),
                    ];
                    (StatusCode::OK, headers, data).into_response()
                }
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to read file: {}", e) }))).into_response()
                }
            }
        }
        None => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "Upload not found" }))).into_response()
        }
    }
}

pub async fn delete_upload(
    AxumState(state): AxumState<Arc<SharedState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let upload: Option<Upload> = sqlx::query_as(
        "SELECT id, file_name, content_type, size, path, sender_id, timestamp FROM uploads WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match upload {
        Some(upload) => {
            let path = PathBuf::from(&upload.path);
            
            // Supprimer le fichier physique
            if path.exists() {
                let _ = tokio::fs::remove_file(&path).await;
            }

            // Supprimer de la base de données
            let _ = sqlx::query("DELETE FROM uploads WHERE id = ?")
                .bind(&id)
                .execute(&state.db)
                .await;

            Json(json!({ "success": true, "message": "File deleted" })).into_response()
        }
        None => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "Upload not found" }))).into_response()
        }
    }
}
