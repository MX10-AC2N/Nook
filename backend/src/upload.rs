use crate::db::{get_pool, ChatMessage, MessageType, Upload};
use crate::webrtc::broadcast_message;
use crate::State;
use axum::body::Body;
use axum::extract::{Multipart, Path, State as AxumState};
use axum::http::header::{ContentDisposition, HeaderMap};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use futures_util::stream::BytesStream;
use serde_json::{json, Value};
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use sqlx::{query, query_as};

pub async fn upload_handler(
    AxumState(_state): AxumState<Arc<State>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploads: Vec<Upload> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        let allowed = ["image/", "video/", "audio/", "application/pdf"];
        if !allowed.iter().any(|&prefix| content_type.starts_with(prefix)) {
            return Html("<script>alert('Type de fichier non autorisé'); window.location.href='/';</script>".into());
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

    for upload in &uploads {
        let _ = query!(
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
        .await
        .ok();
    }

    let saved_upload = uploads.first().cloned().unwrap_or(Upload {
        id: "".to_string(),
        file_name: "aucun".to_string(),
        content_type: "".to_string(),
        size: 0,
        path: "".to_string(),
        timestamp: 0,
    });

    Html(format!(
        "<!DOCTYPE html>
<html><head><meta charset=\"UTF-8\"><title>Upload réussi</title></head>
<body style=\"text-align:center;padding:50px;font-family:sans-serif;background:#f9f9f9;\">
<h1 style=\"color:#28a745;\">Upload réussi ! ✅</h1>
<p><strong>Fichier :</strong> {}</p>
<p><strong>Taille :</strong> {} octets</p>
<p><strong>ID :</strong> {}</p>
<br>
<button onclick=\"window.location.href='/'\" style=\"padding:12px 24px;background:#28a745;color:white;border:none;border-radius:8px;cursor:pointer;font-size:16px;\">
    Retour
</button>
</body></html>",
        saved_upload.file_name, saved_upload.size, saved_upload.id
    ))
}

pub async fn upload_chat_file(
    AxumState(state): AxumState<Arc<State>>,
    Path((conversation_id, sender_id, message_type)): Path<(String, String, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let sender_name_opt: Option<String> = query!("SELECT name FROM users WHERE id = $1", sender_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .and_then(|r| r.name);

    let sender_name = match sender_name_opt {
        Some(n) => n,
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

    let _ = query!(
        "INSERT INTO chat_messages (id, conversation_id, sender_id, sender_name, content, message_type, timestamp, file)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        message.id,
        message.conversation_id,
        message.sender_id,
        message.sender_name,
        message.content,
        message.message_type.to_string(),
        message.timestamp,
        serde_json::to_string(&message.file).unwrap_or("null".to_string())
    )
    .execute(&state.db)
    .await
    .ok();

    let message_json = serde_json::to_string(&message).unwrap();
    broadcast_message(state.clone(), conversation_id, "new_message".to_string(), message_json.clone());

    Html(format!(
        "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"></head>
        <script>if(window.opener){{window.opener.postMessage({{type:'file_uploaded',message:{}}},'*');window.close();}}</script>
        <body>Fichier envoyé. Fermeture...</body></html>",
        message_json
    ))
}

pub async fn get_upload(Path(id): Path<String>) -> impl IntoResponse {
    let upload: Option<Upload> = query_as!(
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
                let data = tokio::fs::read(path).await.unwrap_or_default();
                let stream = BytesStream::from(data);
                let body = Body::from_stream(stream);

                let mut headers = HeaderMap::new();
                headers.insert("content-type", upload.content_type.parse().unwrap_or_else(|_| "application/octet-stream".parse().unwrap()));
                headers.insert("content-disposition", ContentDisposition::inline().filename(&upload.file_name).to_string().parse().unwrap());

                (headers, body).into_response()
            } else {
                (StatusCode::NOT_FOUND, "Fichier non trouvé").into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Upload non trouvé").into_response(),
    }
}

pub async fn delete_upload(
    AxumState(state): AxumState<Arc<State>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let upload: Option<Upload> = query_as!(
        Upload,
        "SELECT * FROM uploads WHERE id = $1",
        id.clone()
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some(upload) = upload {
        let path = StdPath::new(&upload.path);
        if path.exists() {
            let _ = tokio::fs::remove_file(path).await;
        }

        let _ = query!("DELETE FROM uploads WHERE id = $1", id)
            .execute(&state.db)
            .await
            .ok();

        Html("<!DOCTYPE html><html><head><script>if(window.opener){window.opener.postMessage({type:'file_deleted'},'*');window.close();}</script></head><body>Supprimé.</body></html>".into())
    } else {
        Html("Fichier non trouvé".into())
    }
}
