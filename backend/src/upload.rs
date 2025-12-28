use crate::db::{get_pool, ChatMessage, MessageType, Upload, User};
use crate::webrtc::broadcast_message;
use crate::State;
use axum::body::Body;
use axum::extract::{multipart::Multipart, Path, State as AxumState};
use axum::http::header::ContentDisposition;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use futures_util::stream::BytesStream;
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use uuid::Uuid;

pub async fn upload_handler(
    State(state): AxumState<Arc<State>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploads: Vec<Upload> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        // Validate file type
        let allowed_types = ["image/", "video/", "audio/", "application/pdf"];
        if !allowed_types.iter().any(|t| content_type.starts_with(t)) {
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

        if let Err(e) = std::fs::create_dir_all(upload_dir) {
            return Html(format!(
                "<script>alert('Error creating directory: {}'); window.location.href='/';</script>",
                e
            ));
        }

        // Add timestamp and save to disk
        let timestamp = chrono::Utc::now().timestamp();
        let file_path = std::path::Path::new(&path);
        let mut file = File::create(file_path).await.unwrap();
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

    let uploads_json = serde_json::to_string(&uploads).unwrap();

    let mut saved_uploads: Vec<Upload> = Vec::new();
    let _ = sqlx::query("INSERT INTO uploads (id, file_name, content_type, size, path, timestamp) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&uploads[0].id)
        .bind(&uploads[0].file_name)
        .bind(&uploads[0].content_type)
        .bind(&uploads[0].size)
        .bind(&uploads[0].path)
        .bind(&uploads[0].timestamp)
        .execute(&get_pool())
        .await;

    let uploads_from_db: Vec<Upload> = sqlx::query_as::<_, Upload>("SELECT * FROM uploads WHERE id = ?")
        .bind(&uploads[0].id)
        .fetch_all(&get_pool())
        .await
        .unwrap();

    if let Some(upload) = uploads_from_db.first() {
        saved_uploads.push(upload.clone());
    }

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Upload Complete</title>
    <style>
        body {{ font-family: Arial, sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }}
        .container {{ background: white; padding: 40px; border-radius: 20px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25); text-align: center; max-width: 500px; width: 90%; }}
        h1 {{ color: #667eea; margin-bottom: 30px; font-size: 28px; }}
        .success-icon {{ font-size: 60px; margin-bottom: 20px; }}
        .upload-info {{ background: #f7fafc; padding: 20px; border-radius: 10px; margin: 20px 0; text-align: left; }}
        .upload-info p {{ margin: 10px 0; color: #4a5568; }}
        .upload-info strong {{ color: #667eea; }}
        .btn {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; padding: 15px 30px; border-radius: 10px; cursor: pointer; font-size: 16px; font-weight: bold; transition: transform 0.2s, box-shadow 0.2s; }}
        .btn:hover {{ transform: translateY(-2px); box-shadow: 0 10px 20px rgba(102, 126, 234, 0.4); }}
    </style>
</head>
<body>
    <div class="container">
        <div class="success-icon">✅</div>
        <h1>Upload Successful!</h1>
        <div class="upload-info">
            <p><strong>📄 File:</strong> {}</p>
            <p><strong>📊 Size:</strong> {} bytes</p>
            <p><strong>🆔 ID:</strong> {}</p>
        </div>
        <button class="btn" onclick="window.location.href='/'">Back to Home</button>
    </div>
</body>
</html>"#,
        saved_uploads[0].file_name,
        saved_uploads[0].size,
        saved_uploads[0].id
    ))
}

pub async fn upload_chat_file(
    State(state): AxumState<Arc<State>>,
    Path((conversation_id, sender_id, message_type)): Path<(String, String, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let sender_name = match sqlx::query_scalar::<_, String>(
        "SELECT name FROM users WHERE id = ?"
    )
    .bind(&sender_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(name)) => name,
        Ok(None) => return Html("User not found".into()),
        Err(e) => return Html(format!("Database error: {}", e).into()),
    };

    let uploaded_file = if let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        let data = field.bytes().await.unwrap();
        let id = Uuid::new_v4().to_string();
        let upload_dir = "uploads";
        let path = format!("{}/{}", upload_dir, id);
        std::fs::create_dir_all(upload_dir).ok();

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
        "file" => MessageType::File,
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
        file: uploaded_file,
    };

    sqlx::query("INSERT INTO chat_messages (id, conversation_id, sender_id, sender_name, content, message_type, timestamp, file) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(&message.sender_id)
        .bind(&message.sender_name)
        .bind(&message.content)
        .bind(message.message_type.to_string())
        .bind(&message.timestamp)
        .bind(serde_json::to_string(&message.file).unwrap())
        .execute(&state.db)
        .await
        .ok();

    let message_json = serde_json::to_string(&message).unwrap();

    broadcast_message(
        state.clone(),
        conversation_id.clone(),
        "new_message".to_string(),
        message_json,
    );

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>File Upload</title>
    <script>
        if (window.opener) {{
            window.opener.postMessage({{
                type: 'file_uploaded',
                message: {}
            }}, '*');
        }}
    </script>
</head>
<body>
    <p>Upload complete. Closing...</p>
</body>
</html>"#,
        serde_json::to_string(&message).unwrap()
    ))
}

pub async fn get_upload(Path(id): Path<String>) -> impl IntoResponse {
    let upload: Option<Upload> = sqlx::query_as::<_, Upload>("SELECT * FROM uploads WHERE id = ?")
        .bind(&id)
        .fetch_optional(&get_pool())
        .await
        .unwrap();

    match upload {
        Some(upload) => {
            let path = StdPath::new(&upload.path);
            if path.exists() {
                let file = File::open(path).await.unwrap();
                let stream = BytesStream::from(tokio::fs::read(path).await.unwrap());
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
                (axum::http::StatusCode::NOT_FOUND, "File not found").into_response()
            }
        }
        None => (axum::http::StatusCode::NOT_FOUND, "Upload not found").into_response(),
    }
}

pub async fn delete_upload(
    State(state): AxumState<Arc<State>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let upload_dir = "uploads";
    let upload: Option<Upload> = sqlx::query_as::<_, Upload>("SELECT * FROM uploads WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap();

    if let Some(upload) = &upload {
        let path = format!("{}/{}", upload_dir, upload.id);
        let path = StdPath::new(&path);
        if path.exists() {
            tokio::fs::remove_file(path).await.unwrap();
        }
        sqlx::query("DELETE FROM uploads WHERE id = ?")
            .bind(&id)
            .execute(&state.db)
            .await
            .ok();

        Html(format!(
            r#"<!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Delete Complete</title>
            <script>
                if (window.opener) {{
                    window.opener.postMessage({{type: 'file_deleted'}}, '*');
                }}
            </script>
        </head>
        <body>
            <p>Delete complete. Closing...</p>
        </body>
        </html>"#
        ))
    } else {
        Html("Upload not found".into())
    }
}
