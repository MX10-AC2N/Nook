mod auth;
mod db;
mod upload;
mod webrtc;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service, patch, post},
    Json, // <--- Ajout crucial ici
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{error, info};

#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

// Fallback SPA : sert index.html pour toutes les routes non-API
async fn spa_fallback() -> impl IntoResponse {
    match tokio::fs::read_to_string("/app/static/index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            error!("Impossible de lire index.html : {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erreur serveur : impossible de charger l'application",
            )
                .into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    // Logging verbeux
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Création des dossiers
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();

    info!("🚀 Démarrage de Nook v2.0");

    // Token admin
    let token_path = "/app/data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let new_token = uuid::Uuid::new_v4().to_string();
        if let Err(e) = std::fs::write(token_path, &new_token) {
            error!("Impossible d'écrire le token admin : {}", e);
            panic!("Échec critique : token admin");
        }
        info!("🆕 Nouveau token admin généré : {}", new_token);
    } else {
        info!("✅ Token admin chargé depuis /app/data/admin.token");
    }

    // Init DB
    let app_state = db::init_db().await;
    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_sessions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };

    // Router
    let app = Router::new()
        // API
        .route("/api/invite", post(auth::invite_handler))
        .route("/api/join", post(auth::join_handler))
        .route("/api/members/:id/approve", patch(auth::approve_handler))
        .route("/api/members", get(auth::members_handler))
        .route("/api/upload", post(upload::handle_upload))
        .route("/api/gifs", get(gif_proxy))
        .route("/api/webrtc/offer", post(webrtc::handle_offer))
        .route("/api/webrtc/answer", get(webrtc::handle_answer))
        .route("/ws", get(ws_handler))

        // Assets statiques
        .nest_service("/_app", get_service(ServeDir::new("/app/static/_app")))
        .nest_service("/static", get_service(ServeDir::new("/app/static")))
        .nest_service("/uploads", get_service(ServeDir::new("/app/data/uploads")))

        // Fallback SPA
        .fallback(get(spa_fallback))

        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("🌍 Nook prêt sur http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// WebSocket echo
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

// Proxy GIF Tenor
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
