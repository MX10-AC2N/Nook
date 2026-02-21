// main.rs – VERSION FINALE (Config + middleware auth global)

use axum::{
    body::{to_bytes, Body},
    extract::Host,
    http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE},
        HeaderMap, HeaderValue, Request,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use chrono::Utc;
use sqlx::{migrate, SqlitePool};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

mod admin;
mod auth;
mod config;
mod db;
mod invites;
mod prune;
mod upload;
mod webrtc;

use crate::config::Config;
use crate::prune::prune_old_data;
use webrtc::{FileManager, WebRtcState};

// ---------------------------------------------------------------------
// SharedState (pub pour middleware)
// ---------------------------------------------------------------------
#[derive(Clone)]
pub struct SharedState {
    pub db: SqlitePool,
    pub webrtc_state: WebRtcState,
    pub file_manager: Arc<FileManager>,
}

// ---------------------------------------------------------------------
// Middleware base inject (ton code original)
// ---------------------------------------------------------------------
async fn base_inject_middleware(
    Host(host): Host,
    headers: HeaderMap,
    req: Request<Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let scheme = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()).unwrap_or("http");
    let host_str = if host.is_empty() { "localhost:3000".to_string() } else { host };
    let base_url = format!("{}://{}/", scheme, host_str);
    let replacement = format!("<base href=\"{}\" />", base_url);

    let resp = next.run(req).await;

    if let Some(ct) = resp.headers().get(CONTENT_TYPE) {
        if ct.to_str().is_ok_and(|s| s.starts_with("text/html")) {
            let (parts, body) = resp.into_parts();
            let bytes = to_bytes(body, 10_000_000).await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut body_str = String::from_utf8_lossy(&bytes).into_owned();
            if body_str.contains("<base-placeholder/>") {
                body_str = body_str.replace("<base-placeholder/>", &replacement);
            }
            let body_bytes = Bytes::from(body_str);
            let content_length = body_bytes.len();
            let mut new_resp = Response::from_parts(parts, Body::from(body_bytes));
            if let Ok(len) = HeaderValue::from_str(&content_length.to_string()) {
                new_resp.headers_mut().insert(CONTENT_LENGTH, len);
            }
            return Ok(new_resp.into_response());
        }
    }
    Ok(resp)
}

// ---------------------------------------------------------------------
// DB + Initial admin
// ---------------------------------------------------------------------
async fn init_db(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(url).await?;
    migrate!("./migrations").run(&pool).await?;
    eprintln!("[DB] Migrations OK");
    Ok(pool)
}

async fn check_initial_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // ton code original (inchangé)
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(pool).await?;
    if count.0 == 0 {
        // ... (ton insert admin initial)
        let admin_id = "admin-initial-id-0000-0000-000000000001".to_string();
        let password_hash = crate::auth::hash_password("changeme2026");
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"INSERT INTO users (id, username, email, password_hash, name, role, approved, needs_password_change, created_at)
               VALUES (?, ?, ?, ?, ?, 'admin', 1, 1, ?)"#
        )
        .bind(&admin_id).bind("admin").bind("admin@nook.local").bind(password_hash).bind("Administrateur Initial").bind(now)
        .execute(pool).await?;
        eprintln!("[Init] Admin initial créé");
    }
    Ok(())
}

// ---------------------------------------------------------------------
// MAIN FINAL
// ---------------------------------------------------------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = Config::load();

    tokio::fs::create_dir_all("/app/data").await?;
    tokio::fs::create_dir_all(&config.uploads_dir).await?;

    let pool = init_db(&config.database_url).await?;
    check_initial_admin(&pool).await?;

    let uploads_dir = PathBuf::from(&config.uploads_dir);
    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));
    let webrtc_state = WebRtcState::new();

    let fm_clone = (*file_manager).clone();
    tokio::spawn(async move { fm_clone.start_cleanup_task().await; });

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        loop {
            let _ = prune_old_data(&pool_clone).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(24 * 3600)).await;
        }
    });

    let shared_state = Arc::new(SharedState { db: pool, webrtc_state, file_manager });

    // Routes publiques
    let public_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/join", post(invites::join))
        .route("/invite/validate", get(invites::validate_invite))
        .route("/generate-invite", post(invites::generate_invite))
        .route("/health", get(|| async { "OK" }));

    // Routes protégées avec middleware global
    let protected_routes = Router::new()
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/change-password", post(auth::change_password))
        .route("/conversations", get(db::get_user_conversations))
        .route("/conversations", post(db::create_conversation))
        .route("/conversations/:id", get(db::get_conversation))
        .route("/conversations/:id/join", post(db::join_conversation))
        .route("/conversations/:id/messages", get(db::get_conversation_messages))
        .route("/conversations/:id/messages", post(db::send_message))
        .route("/upload", post(upload::upload_handler))
        .route("/upload/chat", post(upload::upload_chat_file))
        .route("/pending-users-json", get(admin::pending_users))
        .route("/all-users-json", get(admin::all_users))
        .route("/approve", post(admin::approve_user))
        .route("/list-invites", get(admin::list_invites))
        .route("/delete-invite", post(admin::delete_invite))
        .layer(middleware::from_fn_with_state(shared_state.clone(), auth::require_auth));

    let api_router = Router::new().merge(public_routes).merge(protected_routes);

    let static_service = ServeDir::new(&config.static_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(format!("{}/index.html", config.static_dir)));

    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/files", ServeDir::new(&config.uploads_dir))
        .merge(webrtc::webrtc_routes())
        .fallback_service(static_service)
        .layer(middleware::from_fn(base_inject_middleware))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any).allow_credentials(true))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    eprintln!("[🚀] Nook démarré → http://0.0.0.0:{} (middleware auth global actif)", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
