mod db;
mod auth;
mod upload;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use db::{init_db, AppState};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    // Admin token
    std::fs::create_dir_all("data").ok();
    let token_path = "data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(token_path, token).expect("Failed to create admin.token");
    }

    let app_state = init_db().await;
    let app = Router::new()
        .route("/api/invite", post(invite_handler))
        .route("/api/join", post(join_handler))
        .route("/api/members/:id/approve", patch(approve_handler))
        .route("/api/members", get(members_handler))
        .route("/api/upload", post(upload::handle_upload))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/uploads", ServeDir::new("data/uploads"))
        .fallback_service(ServeFile::new("static/index.html"))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Nook v1.1 running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use futures_util::{SinkExt, StreamExt};

async fn ws_handler(ws: WebSocketUpgrade) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.next().await {
        if let Ok(text) = msg.into_text() {
            // Echo (à remplacer par broadcasting sécurisé)
            let _ = socket.send(axum::extract::ws::Message::Text(text)).await;
        }
    }
}

use serde_json::Value;

async fn invite_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<auth::ApiResponse>, axum::http::StatusCode> {
    // Vérifier admin.token (simplifié)
    match auth::create_invite(&state.db).await {
        Ok(token) => Ok(Json(auth::ApiResponse {
            success: true,
            message: format!("https://nook.local/join?token={}", token),
        })),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn join_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    axum::extract::Json(payload): axum::extract::Json<auth::JoinRequest>,
) -> Result<Json<auth::ApiResponse>, StatusCode> {
    if let Some(token) = params.get("token") {
        match auth::handle_join(&state.db, token.clone(), payload).await {
            Ok(res) => Ok(Json(res)),
            Err(e) => Err(e),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

use axum::routing::patch;

async fn approve_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<auth::ApiResponse>, StatusCode> {
    auth::approve_member(&state.db, id).await.map(Json).map_err(|e| e)
}

async fn members_handler(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let members = auth::get_members(&state.db).await.map_err(|e| e)?;
    Ok(Json(serde_json::json!({ "members": members })))
}