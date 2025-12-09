use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::db::AppState;

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub url: String,
}

pub async fn handle_upload(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    tokio::fs::create_dir_all("data/uploads")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Ok(Some(field)) = multipart.next_field().await {
        let data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let path = format!("data/uploads/{}", uuid::Uuid::new_v4());
        let mut file = File::create(&path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(UploadResponse {
            success: true,
            url: path.replace("data/", "/uploads/"),
        }));
    }

    Err(StatusCode::BAD_REQUEST)
}