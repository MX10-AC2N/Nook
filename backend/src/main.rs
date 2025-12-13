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
use db::{init_db, AppState};
use webrtc::WebRtcState;
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    std::fs::create_dir_all("data").ok();
    let token_path = "data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(token_path, token).expect("Failed to create admin.token");
    }

    let app_state = init_db().await;
    let webrtc_state = webrtc::WebRtcState::new();

    let app = Router::new()
        .route("/api/invite", post(invite_handler))
        .route("/api/join", post(join_handler))
        .route("/api/members/:id/approve", patch(approve_handler))
        .route("/api/members", get(members_handler))
        .route("/api/upload", post(upload::handle_upload))
        .route("/api/gifs", get(gif_proxy))
        .route("/api/webrtc/offer", post(webrtc::handle_offer))
        .route("/api/webrtc/answer", get(webrtc::handle_answer))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/uploads", ServeDir::new("data/uploads"))
        .fallback_service(ServeFile::new("static/index.html"))
        .with_state(app_state)
        .with_state(webrtc_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Nook v2.0 running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use futures_util::StreamExt;

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.next().await {
        if let Ok(text) = msg.into_text() {
            let _ = socket.send(axum::extract::ws::Message::Text(text)).await;
        }
    }
}

// --- Proxy GIF (anonyme, sans tracking) ---
async fn gif_proxy(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    if let Some(q) = params.get("q") {
        // Utilise la clé publique officielle de Tenor
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

// --- Handlers API ---
async fn invite_handler(
    State(state): State<AppState>,
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
    State(state): State<AppState>,
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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<auth::ApiResponse>, StatusCode> {
    auth::approve_member(&state.db, id).await.map(Json)
}

async fn members_handler(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let members = auth::get_members(&state.db).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}
