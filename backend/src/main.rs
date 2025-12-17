mod auth;
mod db;
mod upload;
mod webrtc;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service, patch, post},
    Json,
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

// Fallback SPA
async fn spa_fallback() -> impl IntoResponse {
    match tokio::fs::read_to_string("/app/static/index.html").await {
        Ok(html) => Html(html),
        Err(_) => Html("<h1>Erreur : index.html introuvable</h1>".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // Création dossiers
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();

    println!("Démarrage de Nook v2.0");

    // Token admin
    let token_path = "/app/data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(token_path, &token).expect("Failed to create admin.token");
        println!("Nouveau token admin généré : {}", token);
    } else {
        println!("Token admin chargé depuis /app/data/admin.token");
    }

    // Init DB
    let app_state = db::init_db().await;
    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_sessions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };

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

        // Assets
        .nest_service("/_app", get_service(ServeDir::new("/app/static/_app")))
        .nest_service("/static", get_service(ServeDir::new("/app/static")))
        .nest_service("/uploads", get_service(ServeDir::new("/app/data/uploads")))

        // Fallback SPA
        .fallback(get(spa_fallback))

        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Nook prêt sur http://{}", addr);
    println!("Static files : /app/static");
    println!("Uploads : /app/data/uploads");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// WS handler
use axum::extract::ws::WebSocketUpgrade;
use futures_util::{SinkExt, StreamExt};

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let (mut sender, mut receiver) = socket.split();
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.into_text() {
                let _ = sender.send(axum::extract::ws::Message::Text(text)).await;
            }
        }
    })
}

// GIF proxy
use urlencoding::encode;

async fn gif_proxy(Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    if let Some(q) = params.get("q") {
        let url = format!(
            "https://g.tenor.com/v1/search?q={}&key=LIVDSRZULELA&limit=8",
            encode(q)
        );
        let resp = reqwest::get(&url)
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
        let json: Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        Ok(Json(json))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
