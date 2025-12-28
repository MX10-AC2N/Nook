mod auth;
mod db;
mod upload;
mod webrtc;
mod cleanup;
mod emergency;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service, post},
    Router,
    middleware::{self, Next},
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_broadcasts: Arc<RwLock<HashMap<String, broadcast::Sender<webrtc::CallSignal>>>>,
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
        "SELECT u.id FROM sessions s
         JOIN users u ON s.user_id = u.id
         WHERE s.token = ? AND u.role = 'admin'"
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
        "SELECT u.id FROM sessions s
         JOIN users u ON s.user_id = u.id
         WHERE s.token = ? AND u.role = 'member' AND u.approved = 1"
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

// SPA fallback handler
async fn spa_fallback() -> impl IntoResponse {
    Html(include_str!("../../static/index.html"))
}

#[tokio::main]
async fn main() {
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();
    println!("Démarrage de Nook v3.0 - Système simplifié");

    let app_state = db::init_db().await;
    
    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_broadcasts: Arc::new(RwLock::new(HashMap::new())),
    };

    // Démarrer la tâche de nettoyage en arrière-plan
    tokio::spawn(cleanup::start_cleanup_task("/app/data/uploads"));

    let public_routes = Router::new()
        .route("/api/register", post(auth::register_handler))
        .route("/api/login", post(auth::login_handler))
        .route("/api/validate-session", get(auth::validate_session_handler))
        .route("/api/logout", post(auth::logout_handler));

    let user_routes = Router::new()
        .route("/api/change-password", post(auth::change_password_handler))
        .route("/api/upload-media", post(upload::handle_upload_media))
        .route("/api/upload", post(upload::handle_upload))
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            user_middleware,
        ));

    let admin_routes = Router::new()
        .route("/api/admin/pending-users", get(auth::pending_users_handler))
        .route("/api/admin/all-users", get(auth::all_users_handler))
        .route("/api/admin/approve-user", post(auth::approve_user_handler))
        .route("/api/admin/emergency", post(emergency::handle_emergency))
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            admin_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(user_routes)
        .merge(admin_routes)
        .route("/ws/call", get(webrtc::call_ws_handler))
        .nest_service("/_app", get_service(ServeDir::new("/app/static/_app")))
        .nest_service("/static", get_service(ServeDir::new("/app/static")))
        .nest_service("/uploads", get_service(ServeDir::new("/app/data/uploads")))
        .fallback(get(spa_fallback))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Nook prêt sur http://{}", addr);
    println!("Système de nettoyage activé (fichiers > 7 jours)");
    
    // Utiliser `axum::Server` correctement typé
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service());
    
    if let Err(e) = server.await {
        eprintln!("Erreur du serveur: {}", e);
    }
}
