use axum::{
    body::Bytes,
    extract::{Multipart, Path},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use tokio::fs;

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50 Mo

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub url: Option<String>,
}

pub async fn handle_upload(mut multipart: Multipart) -> Result<Json<UploadResponse>, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        if data.len() as u64 > MAX_FILE_SIZE {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }

        let filename = uuid::Uuid::new_v4().to_string() + ".enc";
        let path = format!("data/uploads/{}", filename);
        fs::create_dir_all("data/uploads").await.ok();
        fs::write(&path, data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Planifier suppression dans 7 jours (à implémenter)
        return Ok(Json(UploadResponse {
            success: true,
            url: Some(format!("/uploads/{}", filename)),
        }));
    }
    Err(StatusCode::BAD_REQUEST)
}