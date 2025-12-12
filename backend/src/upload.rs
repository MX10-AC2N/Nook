use axum::{extract::Multipart, http::StatusCode};
use std::fs;
use uuid::Uuid;

pub async fn handle_upload(mut multipart: Multipart) -> Result<String, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        // Récupérer le nom AVANT de consommer le champ avec .bytes()
        let name = field.name().unwrap_or("file").to_string();

        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        if data.len() as u64 > 50 * 1024 * 1024 {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }

        let ext = if name.contains('.') {
            name.split('.').last().unwrap_or("bin")
        } else {
            "bin"
        };

        fs::create_dir_all("data/uploads").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let filename = format!("{}.{}", Uuid::new_v4(), ext);
        let path = format!("data/uploads/{}", filename);

        fs::write(&path, data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(format!("/uploads/{}", filename));
    }

    Err(StatusCode::BAD_REQUEST)
}

