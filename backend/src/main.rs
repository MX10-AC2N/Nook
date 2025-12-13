mod auth;
mod db;
mod upload;
mod webrtc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, patch, post},
    Router,
};
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr};
use tower_http::services::{ServeDir, ServeFile};

// État partagé unique
#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

#[tokio::main]
async fn main() {
    std::fs::create_dir_all("data").ok();
    let token_path = "data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(token_path, token).expect("Failed to create admin.token");
    }

    let db = db::init_db().await.db;
    let webrtc_sessions = std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    let shared_state = SharedState { db, webrtc_sessions };

    let app = Router::new()
        .route("/api/invite", post(auth::invite_handler))
        .route("/api/join", post(auth::join_handler))
        .route("/api/members/:id/approve", patch(auth::approve_handler))
        .route("/api/members", get(auth::members_handler))
        .route("/api/upload", post(upload::handle_upload))
        .route("/api/gifs", get(gif_proxy))
        .route("/api/webrtc/offer", post(webrtc::handle_offer))
        .route("/api/webrtc/answer", get(webrtc::handle_answer))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/uploads", ServeDir::new("data/uploads"))
        .fallback_service(ServeFile::new("static/index.html"))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Nook v2.0 running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// --- Handlers ---
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use futures_util::StreamExt;

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| async {
        let mut socket = socket;
        while let Some(Ok(msg)) = socket.next().await {
            if let Ok(text) = msg.into_text() {
                let _ = socket.send(axum::extract::ws::Message::Text(text)).await;
            }
        }
    })
}

async fn gif_proxy(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    if let Some(q) = params.get("q") {
        let url = format!(
            "https://g.tenor.com/v1/search?q={}&key=LIVDSRZULELA&limit=8",
            urlencoding::encode(q)
        );
        let resp = reqwest::get(&url).await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        let json: Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        Ok(Json(json))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// --- API handlers avec SharedState ---
async fn invite_handler(
    State(state): State<SharedState>,
) -> Result<Json<auth::ApiResponse>, StatusCode> {
    match auth::create_invite(&state.db).await {
        Ok(token) => Ok(Json(auth::ApiResponse {
            success: true,
            message: format!("https://nook.local/join?token={}", token),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn join_handler(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<auth::JoinRequest>,
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

async fn approve_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<auth::ApiResponse>, StatusCode> {
    auth::approve_member(&state.db, id).await.map(Json)
}

async fn members_handler(
    State(state): State<SharedState>,
) -> Result<Json<Value>, StatusCode> {
    let members = auth::get_members(&state.db).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}
