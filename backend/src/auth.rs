use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::SharedState;

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

#[derive(Serialize, sqlx::FromRow)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub approved: bool,
}

// === Logique métier ===

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
    let row = sqlx::query("SELECT token FROM invites WHERE token = ?")
        .bind(&token)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("DELETE FROM invites WHERE token = ?")
        .bind(&token)
        .execute(pool)
        .await
        .ok();

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO members (id, name, public_key, approved) VALUES (?, ?, ?, 0)")
        .bind(&id)
        .bind(&req.name)
        .bind(&req.public_key)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ApiResponse {
        success: true,
        message: "Demande envoyée à l'administrateur".to_string(),
    })
}

pub async fn approve_member(pool: &SqlitePool, id: String) -> Result<ApiResponse, StatusCode> {
    sqlx::query("UPDATE members SET approved = 1 WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ApiResponse {
        success: true,
        message: "Membre approuvé".to_string(),
    })
}

pub async fn get_members(pool: &SqlitePool) -> Result<Vec<Member>, StatusCode> {
    let rows = sqlx::query_as::<_, Member>(
        "SELECT id, name, approved FROM members ORDER BY joined_at"
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows)
}

// === Handlers Axum ===

pub async fn invite_handler(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse>, StatusCode> {
    match create_invite(&state.db).await {
        Ok(token) => Ok(Json(ApiResponse {
            success: true,
            message: format!("https://nook.local/join?token={}", token),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn join_handler(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<JoinRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    if let Some(token) = params.get("token") {
        match handle_join(&state.db, token.clone(), payload).await {
            Ok(res) => Ok(Json(res)),
            Err(e) => Err(e),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn approve_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse>, StatusCode> {
    approve_member(&state.db, id).await.map(Json)
}

pub async fn members_handler(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let members = get_members(&state.db).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}
