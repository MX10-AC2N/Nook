use crate::db::{ChatMessage, MessageType, Upload};
use crate::webrtc::broadcast_message;
use crate::SharedState;
use axum::body::Body;
use axum::extract::{Multipart, Path, State as AxumState};
use axum::http::header::HeaderMap;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use headers::ContentDisposition;
use serde_json::{json, Value};
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use sqlx::{query, query_as};
use sqlx::pool::Pool;
use sqlx::Sqlite;

pub async fn upload_handler(
    AxumState(_state): AxumState<Arc<SharedState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploads: Vec<Upload> = Vec::new();
    
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let allowed = ["image/", "video/", "audio/", "application/pdf"];
        
        if !allowed.iter().any(|&prefix| content_type.starts_with(prefix)) {
            return Html("Type de fichier non autorisé".into());
        }
        
        let data = field.bytes().await.unwrap();
        let id = Uuid::new_v4().to_string();
        
        let ext = if content_type.starts_with("image/") {
            if content_type.contains("svg") { "svg" } else { "jpg" }
        } else if content_type.starts_with("video/") {
            "mp4"
        } else if content_type.starts_with("audio/") {
            "mp3"
        } else {
            "pdf"
        };
        
        let path = format!("uploads/{}.{}", id, ext);
        let _ = std::fs::create_dir_all("uploads");
        let timestamp = chrono::Utc::now().timestamp();
        
        let mut file = File::create(&path).await.unwrap();
        file.write_all(&data).await.unwrap();

        let upload = Upload {
            id: id.clone(),
            file_name,
            content_type,
            size: data.len() as i64,
            path: path.clone(),
            timestamp,
        };
        uploads.push(upload);
    }

    Html(format!(
        "Upload réussi !\n\nFichier : {}\nTaille : {} octets\nID : {}",
        uploads.first().map(|u| &u.file_name).unwrap_or(&"aucun".to_string()),
        uploads.first().map(|u| u.size).unwrap_or(0),
        uploads.first().map(|u| &u.id).unwrap_or(&"".to_string())
    ))
}

pub async fn upload_chat_file(
    AxumState(state): AxumState<Arc<SharedState>>,
    Path((conversation_id, sender_id, message_type)): Path<(String, String, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let pool: &sqlx::SqlitePool = &state.db;
    
    let sender_name_opt: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM users WHERE id = ?"
    )
    .bind(&sender_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let sender_name: String = match sender_name_opt {
        Some((name,)) => name,
        None => return Html("Utilisateur non trouvé".into()),
    };

    let uploaded_file: Option<Value> = if let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.unwrap();
        let id = Uuid::new_v4().to_string();
        let path = format!("uploads/{}", id);
        let _ = std::fs::create_dir_all("uploads");
        
        let mut file = File::create(&path).await.unwrap();
        file.write_all(&data).await.unwrap();

        Some(json!({
            "id": id,
            "file_name": file_name,
            "content_type": content_type,
            "size": data.len(),
            "path": path
        }))
    } else {
        None
    };

    let message_id = Uuid::new_v4().to_string();
    let msg_type = match message_type.as_str() {
        "image" => MessageType::Image,
        "video" => MessageType::Video,
        "audio" => MessageType::Audio,
        _ => MessageType::File,
    };

    let timestamp = chrono::Utc::now().timestamp();
    let file_json = serde_json::to_string(&uploaded_file).unwrap_or("null".to_string());

    let _ = sqlx::query(
        "INSERT INTO chat_messages (id, conversation_id, sender_id, sender_name, content, message_type, timestamp, file)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&message_id)
    .bind(&conversation_id)
    .bind(&sender_id)
    .bind(&sender_name)
    .bind("")
    .bind(msg_type.to_string())
    .bind(timestamp)
    .bind(&file_json)
    .execute(pool)
    .await;

    let message_json = serde_json::to_string(&json!({
        "id": message_id,
        "conversation_id": conversation_id,
        "sender_id": sender_id,
        "sender_name": sender_name,
        "content": "",
        "message_type": msg_type.to_string(),
        "timestamp": timestamp,
        "file": uploaded_file
    })).unwrap();

    broadcast_message(state.clone(), conversation_id, "new_message".to_string(), message_json.clone());

    Html("Fichier envoyé avec succès".into())
}

pub async fn get_upload(Path(id): Path<String>) -> impl IntoResponse {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:/app/data/nook.db".to_string())).await.unwrap();
    
    let upload: Option<Upload> = sqlx::query_as(
        "SELECT id, file_name, content_type, size, path, timestamp FROM uploads WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    match upload {
        Some(upload) => {
            let path = StdPath::new(&upload.path);
            if path.exists() {
                match tokio::fs::File::open(path).await {
                    Ok(file) => {
                        let mut response = axum::response::Response::new(axum::body::Body::from(file));
                        response.headers_mut().insert(
                            "Content-Type",
                            upload.content_type.parse().unwrap_or("application/octet-stream".parse().unwrap())
                        );
                        let content_disposition = ContentDisposition::attachment(upload.file_name);
                        response.headers_mut().insert(
                            "Content-Disposition",
                            content_disposition.to_string().parse().unwrap()
                        );
                        response
                    }
                    Err(_) => Html("Fichier non trouvé".into()),
                }
            } else {
                Html("Fichier non trouvé".into())
            }
        }
        None => Html("Fichier non trouvé".into()),
    }
}

pub async fn delete_upload(
    AxumState(state): AxumState<Arc<SharedState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pool: &sqlx::SqlitePool = &state.db;
    
    let upload: Option<Upload> = sqlx::query_as(
        "SELECT id, file_name, content_type, size, path, timestamp FROM uploads WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match upload {
        Some(upload) => {
            let path = StdPath::new(&upload.path);
            if path.exists() {
                let _ = tokio::fs::remove_file(path).await;
            }
            
            let _ = sqlx::query(
                "DELETE FROM uploads WHERE id = ?"
            )
            .bind(&id)
            .execute(pool)
            .await;

            Html("Fichier supprimé avec succès".into())
        }
        None => Html("Fichier non trouvé".into()),
    }
}
