// backend/src/main.rs

mod auth;
mod db;
mod upload;
mod webrtc;

use axum::{
    extract::{OriginalUri, Query, WebSocketUpgrade},
    http::{HeaderMap, HeaderName, StatusCode, SET_COOKIE},
    response::IntoResponse,
    routing::{delete, get, get_service, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use http::request::Request;
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::GovernorError;
use tower_http::services::ServeDir;
use uuid::Uuid;
use chrono::Utc;
use rand::rngs::OsRng;
use urlencoding::encode;

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_broadcasts:
        std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, tokio::sync::mpsc::Sender<String>>>>,
}

// Key Extractor basé sur l'URI (fonctionne avec CasaOS et tout reverse proxy)
#[derive(Clone)]
pub struct UriKeyExtractor;

impl KeyExtractor for UriKeyExtractor {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        Ok(req.uri().path().to_string())
    }
}

// Fonction utilitaire pour lire un cookie
fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .map(|c| c.trim())
                .find(|c| c.starts_with(&format!("{}=", name)))
                .and_then(|c| c.split_once('='))
                .map(|(_, value)| value.trim().to_string())
        })
}

// Fonction pour créer l'admin par défaut si nécessaire
async fn ensure_admin_exists(db: &sqlx::SqlitePool) {
    let admin_exists: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM users WHERE role = 'admin' LIMIT 1"
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    if admin_exists.is_none() {
        println!("Aucun administrateur trouvé. Création de l'admin par défaut...");

        let admin_id = Uuid::new_v4().to_string();
        let default_username = "admin";
        let default_password = "admin123!";

        use argon2::{Argon2, PasswordHasher};
        use argon2::password_hash::SaltString;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = argon2
            .hash_password(default_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let created_at = Utc::now().to_rfc3339();

        let _ = sqlx::query(
            "INSERT INTO users (id, username, password, name, role, approved, needs_password_change, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&admin_id)
        .bind(default_username)
        .bind(&hashed_password)
        .bind("Administrateur")
        .bind("admin")
        .bind(true)
        .bind(true)
        .bind(&created_at)
        .execute(db)
        .await;

        println!("Admin par défaut créé avec succès!");
        println!("Username: {}", default_username);
        println!("Mot de passe temporaire: {}", default_password);
        println!("ATTENTION: Ces identifiants devront être modifiés à la première connexion!");
    } else {
        println!("Un administrateur existe déjà dans la base de données.");
    }
}

// Fallback SPA sécurisé : ne sert pas index.html sur les routes /api/*
async fn spa_fallback(original_uri: OriginalUri) -> impl IntoResponse {
    let path = original_uri.0.path();

    if path.starts_with("/api/") {
        return (StatusCode::NOT_FOUND, "API route not found").into_response();
    }

    match tokio::fs::read_to_string("/app/static/index.html").await {
        Ok(html) => (StatusCode::OK, html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "<h1>Erreur : index.html introuvable</h1>".to_string(),
        ).into_response(),
    }
}

#[tokio::main]
async fn main() {
    // Création dossiers
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();
    println!("Démarrage de Nook v2.0");

    // Init DB
    let app_state = db::init_db().await;

    // Créer l'admin par défaut si nécessaire
    ensure_admin_exists(&app_state.db).await;

    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_broadcasts: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };

    // Rate limiting : 5 tentatives max toutes les 15 minutes (900 secondes)
    // UriKeyExtractor fonctionne avec CasaOS et tout reverse proxy
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(900))
            .burst_size(5)
            .key_extractor(UriKeyExtractor)
            .finish()
            .unwrap(),
    );

    let app = Router::new()
        // Login avec rate limiting
        .route(
            "/api/login",
            post(auth::login_json_handler).layer(GovernorLayer {
                config: governor_conf.clone(),
            }),
        )
        // Autres endpoints JSON modernes
        .route("/api/validate-session", get(auth::validate_session_handler))
        .route("/api/user-info", get(auth::user_info_handler))
        .route("/api/register", post(auth::register_json_handler))
        .route("/api/change-password", post(auth::change_password_json_handler))
        .route("/api/logout", post(auth::logout_json_handler))
        .route("/api/first-setup", post(auth::first_setup_handler))

        // Nouvelles routes admin JSON
        .route("/api/pending-users-json", get(auth::pending_users_json_handler))
        .route("/api/all-users-json", get(auth::all_users_json_handler))
        .route("/api/generate-invite", post(auth::generate_invite_handler))

        // Routes upload et autres
        .route("/api/upload", post(upload::upload_handler))
        .route(
            "/api/upload/:conversation_id/:sender_id/:message_type",
            post(upload::upload_chat_file),
        )
        .route("/api/upload/:id", get(upload::get_upload))
        .route("/api/upload/:id", delete(upload::delete_upload))
        .route("/api/gifs", get(gif_proxy))
        .route("/api/webrtc/offer", post(webrtc::handle_offer))
        .route("/api/webrtc/answer", get(webrtc::handle_answer))
        .route("/ws", get(ws_handler))

        // Assets
        .nest_service("/_app", get_service(ServeDir::new("/app/static/_app")))
        .nest_service("/static", get_service(ServeDir::new("/app/static")))
        .nest_service("/uploads", get_service(ServeDir::new("/app/data/uploads")))

        // Fallback SPA (avec guard /api/*)
        .fallback(get(spa_fallback))
        .with_state(Arc::new(shared_state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Nook prêt sur http://{}", addr);
    println!("Static files : /app/static");
    println!("Uploads : /app/data/uploads");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// WS handler
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
async fn gif_proxy(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
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
