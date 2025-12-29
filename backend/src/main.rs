mod auth;
mod cleanup;
mod db;
mod emergency;
mod upload;
mod webrtc;

use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::{get, get_service, post},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::Sender;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_broadcasts: Arc<RwLock<HashMap<String, Arc<RwLock<Sender<String>>>>>>,
}

// Middleware admin
async fn admin_middleware(
    headers: axum::http::HeaderMap,
    State(state): State<SharedState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let token = auth::get_cookie(&headers, "nook_admin");
    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT u.id FROM users u WHERE u.token = ? AND u.role = 'admin'",
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

// Middleware utilisateur
async fn user_middleware(
    headers: axum::http::HeaderMap,
    State(state): State<SharedState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let token = auth::get_cookie(&headers, "nook_session");
    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT u.id FROM users u WHERE u.token = ? AND u.approved = 1",
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

// Fallback SPA : sert l'index.html pour toutes les routes non-API
async fn spa_fallback() -> impl IntoResponse {
    Html(include_str!("../../../frontend/index.html"))
}

#[tokio::main]
async fn main() {
    // Création des dossiers nécessaires
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();

    println!("🚀 Démarrage de Nook v3.0");

    // Initialisation DB
    let app_state = db::init_db().await;

    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_broadcasts: Arc::new(RwLock::new(HashMap::new())),
    };

    // Tâche de nettoyage en arrière-plan
    tokio::spawn(cleanup::start_cleanup_task("/app/data/uploads".to_string()));

    // Routes publiques
    let public_routes = Router::new()
        .route("/api/register", post(auth::register_handler))
        .route("/api/login", post(auth::login_handler))
        .route("/api/logout", post(auth::logout_handler));

    // Routes utilisateur authentifié
    let user_routes = Router::new()
        .route("/api/change-password", post(auth::change_password_handler))
        .route("/api/upload-media", post(upload::handle_upload_media))
        .route("/api/upload", post(upload::handle_upload))
        .route("/api/upload-chat/:conversation_id/:sender_id/:message_type", post(upload::upload_chat_file))
        .route("/uploads/:id", get(upload::get_upload))
        .route("/delete-upload/:id", post(upload::delete_upload))
        .layer(middleware::from_fn_with_state(shared_state.clone(), user_middleware));

    // Routes admin
    let admin_routes = Router::new()
        .route("/pending_users", get(auth::pending_users_handler))
        .route("/all_users", get(auth::all_users_handler))
        .route("/api/approve", post(auth::approve_handler))
        .route("/api/emergency", post(emergency::handle_emergency))
        .layer(middleware::from_fn_with_state(shared_state.clone(), admin_middleware));

    // Application finale
    let app = Router::new()
        .merge(public_routes)
        .merge(user_routes)
        .merge(admin_routes)
        .route("/ws/call", get(webrtc::ws_handler))
        // Servir tous les fichiers statiques du build frontend
        .fallback_service(ServeDir::new("frontend"))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🌐 Serveur Nook prêt sur http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
