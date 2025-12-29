use crate::db::{get_pool, ChatMessage, MessageType, Upload};
use crate::webrtc::broadcast_message;
use crate::State;
use axum::body::Body;
use axum::extract::{Multipart, Path, State as AxumState};
use axum::http::header::{ContentDisposition, HeaderMap};
use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;
use futures_util::stream::BytesStream;
use serde_json::{json, Value};
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use sqlx::{query, query_as, query_scalar};

pub async fn upload_handler(
    AxumState(state): AxumState<Arc<State>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploads: Vec<Upload> = Vec::new();

    while let Some(field) = multipart.next_field().await.transpose().unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        let allowed_prefixes = ["image/", "video/", "audio/", "application/pdf"];
        if !allowed_prefixes.iter().any(|prefix| content_type.starts_with(prefix)) {
            return Html(
                "<script>alert('Invalid file type'); window.location.href='/';</script>"
                    .to_string(),
            );
        }

        let data = field.bytes().await.unwrap();
        let id = Uuid::new_v4().to_string();
        let upload_dir = "uploads";
        let ext = if content_type.starts_with("image/") {
            if content_type.contains("svg") { "svg" } else { "jpg" }
        } else if content_type.starts_with("video/") {
            "mp4"
        } else if content_type.starts_with("audio/") {
            "mp3"
        } else {
            "pdf"
        };
        let path = format!("{}/{}.{}", upload_dir, id, ext);

        std::fs::create_dir_all(upload_dir).ok();

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

    // Insertion en base (gère plusieurs fichiers si besoin)
    for upload in &uploads {
        let _ = sqlx::query!(
            "INSERT INTO uploads (id, file_name, content_type, size, path, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6)",
            upload.id,
            upload.file_name,
            upload.content_type,
            upload.size,
            upload.path,
            upload.timestamp
        )
        .execute(&get_pool())
        .await;
    }

    // On prend le premier pour l'affichage (comme avant)
    let saved_upload = if let Some(first) = uploads.first() {
        sqlx::query_as!(Upload, "SELECT * FROM uploads WHERE id = $1", first.id)
            .fetch_optional(&get_pool())
            .await
            .ok()
            .flatten()
            .unwrap_or(first.clone())
    } else {
        return Html("<script>alert('No file uploaded');</script>".to_string());
    };

    Html(format!(
        include_str!("../templates/upload_success.html"),
        saved_upload.file_name,
        saved_upload.size,
        saved_upload.id
    ))
}

pub async fn upload_chat_file(
    AxumState(state): AxumState<Arc<State>>,
    Path((conversation_id, sender_id, message_type)): Path<(String, String, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let sender_name: String = match query_scalar!("SELECT name FROM users WHERE id = $1", sender_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
    {
        Some(name) => name,
        None => return Html("User not found".into()),
    };

    let uploaded_file: Option<Value> = if let Some(field) = multipart.next_field().await.transpose().unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.unwrap();

        let id = Uuid::new_v4().to_string();
        let path = format!("uploads/{}", id);

        std::fs::create_dir_all("uploads").ok();
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

    let message = ChatMessage {
        id: message_id.clone(),
        conversation_id: conversation_id.clone(),
        sender_id: sender_id.clone(),
        sender_name: sender_name.clone(),
        content: "".to_string(),
        message_type: msg_type,
        timestamp: chrono::Utc::now().timestamp(),
        file: uploaded_file.clone(),
    };

    let _ = sqlx::query!(
        r#"
        INSERT INTO chat_messages 
        (id, conversation_id, sender_id, sender_name, content, message_type, timestamp, file) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        message.id,
        message.conversation_id,
        message.sender_id,
        message.sender_name,
        message.content,
        message.message_type.to_string(),
        message.timestamp,
        serde_json::to_string(&message.file).unwrap_or_else(|_| "null".to_string())
    )
    .execute(&state.db)
    .await;

    let message_json = serde_json::to_string(&message).unwrap();
    broadcast_message(
        state.clone(),
        conversation_id.clone(),
        "new_message".to_string(),
        message_json,
    );

    Html(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>File Upload</title>
<script>
    if (window.opener) {{
        window.opener.postMessage({{ type: 'file_uploaded', message: {} }}, '*');
        window.close();
    }}
</script>
</head><body><p>Upload complete. Closing...</p></body></html>"#,
        serde_json::to_string(&message).unwrap()
    ))
}

pub async fn get_upload(Path(id): Path<String>) -> impl IntoResponse {
    let upload: Option<Upload> = sqlx::query_as!(
        Upload,
        "SELECT * FROM uploads WHERE id = $1",
        id
    )
    .fetch_optional(&get_pool())
    .await
    .ok()
    .flatten();

    match upload {
        Some(upload) => {
            let path = StdPath::new(&upload.path);
            if path.exists() {
                let data = tokio::fs::read(path).await.unwrap();
                let stream = BytesStream::from(data);
                let body = Body::from_stream(stream);

                let mut headers = HeaderMap::new();
                headers.insert(
                    "content-type",
                    upload.content_type.parse().unwrap_or("application/octet-stream".parse().unwrap()),
                );
                headers.insert(
                    "content-disposition",
                    ContentDisposition::inline()
                        .filename(&upload.file_name)
                        .to_string()
                        .parse()
                        .unwrap(),
                );

                (headers, body).into_response()
            } else {
                (StatusCode::NOT_FOUND, "File not found").into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Upload not found").into_response(),
    }
}

pub async fn delete_upload(
    AxumState(state): AxumState<Arc<State>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let upload: Option<Upload> = sqlx::query_as!(
        Upload,
        "SELECT * FROM uploads WHERE id = $1",
        id.clone()
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some(upload) = upload {
        let file_path = StdPath::new(&upload.path);
        if file_path.exists() {
            let _ = tokio::fs::remove_file(file_path).await;
        }

        let _ = sqlx::query!("DELETE FROM uploads WHERE id = $1", id)
            .execute(&state.db)
            .await;

        Html(r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Delete Complete</title>
<script>
    if (window.opener) {
        window.opener.postMessage({type: 'file_deleted'}, '*');
        window.close();
    }
</script>
</head><body><p>Delete complete. Closing...</p></body></html>"#.to_string())
    } else {
        Html("Upload not found".into())
    }
}
