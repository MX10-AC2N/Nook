// backend/src/upload.rs — Session 36
// Corrections depuis session 34 :
//   - download_file déchiffre le fichier avant de le servir
//   - upload_chat_file conserve le vrai content_type
//   - URL canonique : /api/download/{id} (route déchiffrante)
// Session 36 — SEC-04 : validation magic bytes
//   - Refuse les fichiers dont les magic bytes ne correspondent pas au content_type déclaré
//   - Bloque .html/.php/.exe déguisés en image/jpeg etc.
//   - Permissif pour les types non reconnus (documents, audio, vidéo)

use crate::{
    auth::CurrentUser,
    webrtc::{decrypt_file_from_storage, encrypt_file_for_storage},
    SharedState,
};
use axum::{
    body::Body,
    extract::{Multipart, Path, State as AxumState},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json as AxumJson,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ====================== STRUCTURES ======================

#[allow(dead_code)]
#[derive(serde::Serialize)]
pub struct UploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub file_size: usize,
    pub uploaded_at: i64,
    pub encrypted: bool,
    pub url: String,
}

// ====================== UTILITAIRES ======================

/// Devine le MIME type depuis l'extension du fichier.
fn guess_content_type(filename: &str) -> String {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png"          => "image/png",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        "svg"          => "image/svg+xml",
        "mp4"          => "video/mp4",
        "webm"         => "video/webm",
        "mov"          => "video/quicktime",
        "mp3"          => "audio/mpeg",
        "ogg"          => "audio/ogg",
        "wav"          => "audio/wav",
        "m4a"          => "audio/mp4",
        "pdf"          => "application/pdf",
        "txt"          => "text/plain; charset=utf-8",
        _              => "application/octet-stream",
    }
    .to_string()
}

fn is_viewable_inline(content_type: &str) -> bool {
    content_type.starts_with("image/")
        || content_type.starts_with("video/")
        || content_type.starts_with("audio/")
        || content_type == "application/pdf"
}

// ─────────────────────────────────────────────────────────────────────────────
// SEC-04 : Validation par magic bytes
//
// Vérifie que les premiers octets du fichier correspondent au content_type déclaré
// par le client. Un fichier .html ou .php renommé en .jpg sera rejeté.
//
// Stratégie : liste blanche des types courants ; les types non listés passent
// (permissif pour les documents, archives, audio/vidéo exotiques).
// ─────────────────────────────────────────────────────────────────────────────
fn validate_magic_bytes(data: &[u8], content_type: &str) -> Result<(), &'static str> {
    if data.len() < 4 {
        return Ok(()); // trop court pour vérifier — accepté
    }
    let magic = &data[..data.len().min(16)];

    // Normaliser : ignorer les paramètres (ex: "image/jpeg; charset=...")
    let ct_base = content_type.split(';').next().unwrap_or(content_type).trim();

    match ct_base {
        "image/jpeg" if !magic.starts_with(&[0xFF, 0xD8, 0xFF]) => {
            return Err("Fichier invalide : magic bytes JPEG attendus");
        }
        "image/png" if !magic.starts_with(&[0x89, 0x50, 0x4E, 0x47]) => {
            return Err("Fichier invalide : magic bytes PNG attendus");
        }
        "image/gif" if !magic.starts_with(b"GIF8") => {
            return Err("Fichier invalide : magic bytes GIF attendus");
        }
        "image/webp" if !(magic.len() >= 12 && magic.starts_with(b"RIFF") && magic.len() >= 12 && &magic[8..12] == b"WEBP") => {
            return Err("Fichier invalide : magic bytes WebP attendus");
        }
        "application/pdf" if !magic.starts_with(b"%PDF") => {
            return Err("Fichier invalide : magic bytes PDF attendus");
        }
        "image/svg+xml" => {
            // SVG = XML → commence par <?xml ou <svg (après éventuel BOM)
            let text = std::str::from_utf8(&data[..data.len().min(64)]).unwrap_or("");
            let trimmed = text.trim_start_matches('\u{FEFF}').trim(); // strip BOM
            if !trimmed.starts_with("<?xml") && !trimmed.starts_with("<svg") {
                return Err("Fichier invalide : contenu SVG/XML attendu");
            }
        }
        // Types non vérifiés : vidéo, audio, texte, octet-stream → permissif
        _ => {}
    }

    Ok(())
}

// ====================== HANDLERS (protégés) ======================

pub async fn upload_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
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
            let content_type = field
                .content_type()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty() && s != "application/octet-stream")
                .unwrap_or_else(|| guess_content_type(&filename));

            let mut data_vec = Vec::new();
            while let Some(chunk) = field.chunk().await.transpose() {
                match chunk {
                    Ok(bytes) => data_vec.extend_from_slice(&bytes),
                    Err(e) => {
                        eprintln!("[Upload] Erreur lecture chunk: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR,
                                AxumJson(json!({"error": "Failed to read file"})));
                    }
                }
            }
            form_data.data = Some(data_vec);
            form_data.file_name = filename;
            form_data.content_type = content_type;
        }
    }

    let data = match form_data.validate() {
        Ok(d) => d,
        Err(e) => return (StatusCode::BAD_REQUEST, AxumJson(json!({"error": e}))),
    };

    let file_id = Uuid::new_v4().to_string();
    let file_ext = std::path::Path::new(&data.file_name)
        .extension().and_then(|e| e.to_str()).unwrap_or("bin");
    let stored_filename = format!("{}.{}", file_id, file_ext);
    let storage_path = state.file_manager.get_uploads_dir().join(&stored_filename);

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
        .bind(&file_id).bind(&data.conversation_id).bind(&user.id)
        .bind(&data.file_name).bind(storage_path.to_str().unwrap_or(""))
        .bind(data.data.len() as i64).bind(&data.content_type)
        .bind(now).bind(true).bind(&nonce_b64).bind(&key_b64)
        .execute(&state.db).await
    {
        eprintln!("[Upload] Erreur DB: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Database error"})));
    }

    state.file_manager.register_file(&file_id, storage_path).await;

    let is_image = data.content_type.starts_with("image/");
    (StatusCode::OK, AxumJson(json!({
        "status":       "uploaded",
        "file_id":      file_id,
        "file_name":    data.file_name,
        "file_size":    data.data.len(),
        "content_type": data.content_type,
        "is_image":     is_image,
        "uploaded_at":  now,
        "encrypted":    true,
        "url": format!("/api/download/{}", file_id)
    })))
}

pub async fn upload_chat_file(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
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
            // Conserver le vrai content_type pour que le frontend sache si c'est une image
            let content_type = field
                .content_type()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty() && s != "application/octet-stream")
                .unwrap_or_else(|| guess_content_type(&filename));

            let mut data_vec = Vec::new();
            while let Some(chunk) = field.chunk().await.transpose() {
                match chunk {
                    Ok(bytes) => data_vec.extend_from_slice(&bytes),
                    Err(e) => {
                        eprintln!("[Upload Chat] Erreur lecture chunk: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR,
                                AxumJson(json!({"error": "Failed to read file"})));
                    }
                }
            }
            form_data.data = Some(data_vec);
            form_data.file_name = filename;
            form_data.content_type = content_type;
        }
    }

    let data = match form_data.validate() {
        Ok(d) => d,
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
        .bind(&file_id).bind(&data.conversation_id).bind(&user.id)
        .bind(&data.file_name).bind(storage_path.to_str().unwrap_or(""))
        .bind(data.data.len() as i64).bind(&data.content_type)
        .bind(now).bind(true).bind(&nonce_b64).bind(&key_b64)
        .execute(&state.db).await
    {
        eprintln!("[Upload Chat] Erreur DB: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"error": "Database error"})));
    }

    state.file_manager.register_file(&file_id, storage_path).await;

    let is_image = data.content_type.starts_with("image/");
    (StatusCode::OK, AxumJson(json!({
        "status":       "uploaded",
        "file_id":      file_id,
        "file_name":    data.file_name,
        "file_size":    data.data.len(),
        "content_type": data.content_type,
        "is_image":     is_image,
        "uploaded_at":  now,
        "encrypted":    true,
        "url": format!("/api/download/{}", file_id)
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
        // SEC-04 : validation magic bytes
        validate_magic_bytes(&data, &self.content_type)
            .map_err(|e| e.to_string())?;

        Ok(ValidatedUploadData {
            conversation_id: self.conversation_id,
            file_name: self.file_name,
            content_type: self.content_type,
            data,
        })
    }
}

// ====================== DOWNLOAD ======================
// GET /api/download/{file_id}
// - Déchiffre le fichier (XChaCha20-Poly1305) avant envoi
// - Content-Disposition: inline pour images/vidéos/audio/pdf
// - Content-Disposition: attachment pour les autres types
// - Protégé par require_auth

pub async fn download_file(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
    Path(file_id): Path<String>,
) -> Response {
    #[derive(sqlx::FromRow)]
    struct FileRow {
        file_name: String,
        file_path: String,
        content_type: String,
        encrypted: bool,
        nonce: Option<String>,
        key_text: Option<String>,
    }

    let row = match sqlx::query_as::<_, FileRow>(
        "SELECT file_name, file_path, content_type, encrypted, nonce, key_text FROM uploads WHERE id = ?",
    )
    .bind(&file_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND,
            AxumJson(json!({"error": "Fichier introuvable"}))).into_response(),
        Err(e) => {
            tracing::error!(file_id = %file_id, err = %e, "Erreur DB download");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let raw_bytes = match tokio::fs::read(&row.file_path).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(path = %row.file_path, err = %e, "Fichier absent du disque");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let plaintext = if row.encrypted {
        match (row.nonce.as_deref(), row.key_text.as_deref()) {
            (Some(nonce), Some(key)) => {
                match decrypt_file_from_storage(&raw_bytes, nonce, key) {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::error!(file_id = %file_id, err = %e, "Déchiffrement échoué");
                        return (StatusCode::INTERNAL_SERVER_ERROR,
                            AxumJson(json!({"error": "Déchiffrement impossible"}))).into_response();
                    }
                }
            }
            _ => {
                tracing::warn!(file_id = %file_id, "Fichier chiffré mais nonce/key absents — servi brut");
                raw_bytes
            }
        }
    } else {
        raw_bytes
    };

    let content_type = if row.content_type.is_empty()
        || row.content_type == "application/octet-stream"
    {
        guess_content_type(&row.file_name)
    } else {
        row.content_type.clone()
    };

    let safe_name = row.file_name.replace('"', "\\\"");
    let disposition = if is_viewable_inline(&content_type) {
        format!("inline; filename=\"{safe_name}\"")
    } else {
        format!("attachment; filename=\"{safe_name}\"")
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &content_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CONTENT_LENGTH, plaintext.len())
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(plaintext))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
