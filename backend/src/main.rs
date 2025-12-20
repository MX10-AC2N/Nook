mod auth;
mod db;
mod upload;
mod webrtc;

use axum::{
    extract::{Query, State},
    http::{StatusCode, HeaderMap, header},
    response::{Html, IntoResponse},
    routing::{get, get_service, post},
    Json, Router,
    middleware::{self, Next},
};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_sessions: Arc<RwLock<HashMap<String, String>>>,
    pub chat_connections: Arc<RwLock<HashMap<String, String>>>,
}

// Middleware admin
async fn admin_middleware(
    headers: HeaderMap,
    State(state): State<SharedState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let token = auth::get_cookie(&headers, "nook_admin");
    
    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Vérifier si c'est un admin
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT u.id FROM sessions s 
         JOIN users u ON s.user_id = u.id 
         WHERE s.token = ? AND u.role = 'admin'"
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    Ok(next.run(request).await)
}

// Middleware utilisateur (membre approuvé)
async fn user_middleware(
    headers: HeaderMap,
    State(state): State<SharedState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let token = auth::get_cookie(&headers, "nook_session");
    
    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Vérifier si c'est un membre approuvé
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT u.id FROM sessions s 
         JOIN users u ON s.user_id = u.id 
         WHERE s.token = ? AND u.role = 'member' AND u.approved = 1"
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if row.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() {
    // Création dossiers
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();

    println!("Démarrage de Nook v3.0 - Système simplifié");

    // Init DB
    let app_state = db::init_db().await;
    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_sessions: Arc::new(RwLock::new(HashMap::new())),
        chat_connections: Arc::new(RwLock::new(HashMap::new())),
    };

    // Routes publiques
    let public_routes = Router::new()
        .route("/api/register", post(auth::register_handler))
        .route("/api/login", post(auth::login_handler))
        .route("/api/validate-session", get(auth::validate_session_handler))
        .route("/api/logout", post(auth::logout_handler));

    // Routes utilisateur (membres approuvés)
    let user_routes = Router::new()
        .route("/api/change-password", post(auth::change_password_handler))
        .route("/api/upload", post(upload::handle_upload))
        .route("/api/webrtc/offer", post(webrtc::handle_offer))
        .route("/api/webrtc/answer", get(webrtc::handle_answer))
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            user_middleware,
        ));

    // Routes admin
    let admin_routes = Router::new()
        .route("/api/admin/pending-users", get(auth::pending_users_handler))
        .route("/api/admin/all-users", get(auth::all_users_handler))
        .route("/api/admin/approve-user", post(auth::approve_user_handler))
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            admin_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(user_routes)
        .merge(admin_routes)
        // Assets statiques
        .nest_service("/_app", get_service(ServeDir::new("/app/static/_app")))
        .nest_service("/static", get_service(ServeDir::new("/app/static")))
        .nest_service("/uploads", get_service(ServeDir::new("/app/data/uploads")))
        // Fallback SPA
        .fallback(get(spa_fallback))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Nook prêt sur http://{}", addr);
    println!("Système simplifié - Pas de localStorage, tout en base de données");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn spa_fallback() -> impl IntoResponse {
    match tokio::fs::read_to_string("/app/static/index.html").await {
        Ok(html) => Html(html),
        Err(_) => Html("<h1>Erreur : index.html introuvable</h1>".to_string()),
    }
}
