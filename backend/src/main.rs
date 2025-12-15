mod auth;
mod db;
mod upload;
mod webrtc;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, patch, post},
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

#[tokio::main]
async fn main() {
    // Affichage de la bannière et de la version
    println!("=======================================");
    println!("🌿 Nook — Messagerie familiale privée");
    println!("Version: v2.0.0");
    println!("=======================================");

    // Création du dossier data
    std::fs::create_dir_all("data").ok();
    let data_path = std::fs::canonicalize("data")
        .unwrap_or_else(|_| std::path::PathBuf::from("data"))
        .to_string_lossy()
        .to_string();
    println!("📁 Dossier de données: {}", data_path);

    // Génération du token admin
    let token_path = "data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(token_path, &token).expect("❌ Échec de la création du token admin");
        println!("🔐 Token admin généré et sauvegardé dans 'data/admin.token'");
        println!("⚠️  Copiez ce token : il est nécessaire pour accéder à l'interface admin");
    } else {
        println!("✅ Token admin déjà présent");
    }

    // Initialisation de la base de données
    let app_state = db::init_db().await;
    println!("🗃️  Base de données chargée");

    // Création de l'état partagé
    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_sessions: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // Configuration du routeur
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
        .nest_service("/static", tower_http::services::ServeDir::new("/app/static"))
        .nest_service("/uploads", tower_http::services::ServeDir::new("/app/data/uploads"))
        .fallback_service(tower_http::services::ServeFile::new("/app/static/index.html"))
        .with_state(shared_state);

    // Démarrage du serveur
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Nook démarré avec succès !");
    println!("📡 Écoute sur : http://{}", addr);
    println!("💡 Accédez à l'interface : http://{}/", addr);
    println!("🔒 Interface admin : http://{}/admin", addr);
    println!("=======================================");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}