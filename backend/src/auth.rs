use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct JoinRequest {
    pub name: String,
    pub public_key: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

pub async fn create_invite(pool: &SqlitePool) -> Result<String, StatusCode> {
    let token = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO invites (token) VALUES (?)")
        .bind(&token)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(token)
}

pub async fn handle_join(
    pool: &SqlitePool,
    token: String,
    req: JoinRequest,
) -> Result<ApiResponse, StatusCode> {
    // Vérifier token
    let row = sqlx::query("SELECT token FROM invites WHERE token = ?")
        .bind(&token)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Supprimer le token
    sqlx::query("DELETE FROM invites WHERE token = ?")
        .bind(&token)
        .execute(pool)
        .await
        .ok();

    // Créer membre en attente
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO members (id, name, public_key, approved) VALUES (?, ?, ?, 0)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.public_key)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ApiResponse {
        success: true,
        message: "Demande envoyée à l’administrateur".to_string(),
    })
}