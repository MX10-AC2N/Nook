// main.rs – Axum 0.8 + rand 0.9 compatible
// CORS dynamique : origines lues depuis .env (ALLOWED_ORIGINS + PUBLIC_SITE_URL)
// Cookie adaptatif : SameSite=None;Secure (HTTPS/WAN) ou SameSite=Lax (HTTP/LAN)

use axum::{
    body::{to_bytes, Body},
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
use sqlx::{migrate, sqlite::SqliteConnectOptions, SqlitePool};
use std::{net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
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
// SharedState
// ---------------------------------------------------------------------
#[derive(Clone)]
pub struct SharedState {
    pub db: SqlitePool,
    pub webrtc_state: WebRtcState,
    pub file_manager: Arc<FileManager>,
}

// ---------------------------------------------------------------------
// Middleware base inject — injecte <base href> dynamiquement
// Fonctionne aussi bien en LAN (http://192.168.x.x) qu'en WAN (https://...)
// ---------------------------------------------------------------------
async fn base_inject_middleware(
    headers: HeaderMap,
    req: Request<Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Nginx Proxy Manager injecte X-Forwarded-Proto = "https"
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");

    let host_str = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3000")
        .to_string();

    let base_url = format!("{}://{}/", scheme, host_str);
    let replacement = format!("<base href=\"{}\" />", base_url);

    let resp = next.run(req).await;

    if let Some(ct) = resp.headers().get(CONTENT_TYPE) {
        if ct.to_str().is_ok_and(|s| s.starts_with("text/html")) {
            let (parts, body) = resp.into_parts();
            let bytes = to_bytes(body, 10_000_000)
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
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
    // SqliteConnectOptions avec create_if_missing(true) crée le fichier si besoin
    // (SqlitePool::connect() refuse d'ouvrir un fichier inexistant → SQLITE_CANTOPEN code 14)
    let opts = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    migrate!("./migrations").run(&pool).await?;
    eprintln!("[DB] Migrations appliquées");
    Ok(pool)
}

async fn check_initial_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        let admin_id = "admin-initial-id-0000-0000-000000000001".to_string();
        let password_hash = crate::auth::hash_password("changeme2026");
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"INSERT INTO users (id, username, email, password_hash, name, role, approved, needs_password_change, created_at)
               VALUES (?, ?, ?, ?, ?, 'admin', 1, 1, ?)"#,
        )
        .bind(&admin_id)
        .bind("admin")
        .bind("admin@nook.local")
        .bind(&password_hash)
        .bind("Administrateur Initial")
        .bind(now)
        .execute(pool)
        .await?;
        eprintln!("[Init] Admin initial créé — identifiants : admin / changeme2026");
        eprintln!("[Init] ⚠️  Changez le mot de passe dès la première connexion !");
    }

    // Support E2E_SETUP=1 (CI Playwright)
    if std::env::var("E2E_SETUP").as_deref() == Ok("1") {
        let e2e_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = 'e2e_ci'")
                .fetch_one(pool)
                .await?;

        if e2e_count.0 == 0 {
            let e2e_id = uuid::Uuid::new_v4().to_string();
            let e2e_hash = crate::auth::hash_password("E2eTest123!");
            let now = Utc::now().timestamp();
            sqlx::query(
                r#"INSERT INTO users (id, username, email, password_hash, name, role, approved, needs_password_change, created_at)
                   VALUES (?, 'e2e_ci', 'e2e@nook.local', ?, 'E2E Test User', 'user', 1, 0, ?)"#,
            )
            .bind(&e2e_id)
            .bind(&e2e_hash)
            .bind(now)
            .execute(pool)
            .await?;
            eprintln!("[E2E] Utilisateur e2e_ci créé pour les tests Playwright");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------
// MAIN
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
    tokio::spawn(async move {
        fm_clone.start_cleanup_task().await;
    });

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        loop {
            let _ = prune_old_data(&pool_clone).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(24 * 3600)).await;
        }
    });

    let shared_state = Arc::new(SharedState {
        db: pool,
        webrtc_state,
        file_manager,
    });

    let public_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/join", post(invites::join))
        .route("/invite/validate", get(invites::validate_invite))
        .route("/generate-invite", post(invites::generate_invite))
        .route("/health", get(|| async { "OK" }));

    let protected_routes = Router::new()
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/change-password", post(auth::change_password))
        .route("/conversations", get(db::get_user_conversations))
        .route("/conversations", post(db::create_conversation))
        .route("/conversations/{id}", get(db::get_conversation))
        .route("/conversations/{id}/join", post(db::join_conversation))
        .route(
            "/conversations/{id}/messages",
            get(db::get_conversation_messages),
        )
        .route(
            "/conversations/{id}/messages",
            post(db::send_message),
        )
        .route("/upload", post(upload::upload_handler))
        .route("/upload/chat", post(upload::upload_chat_file))
        .route("/pending-users-json", get(admin::pending_users))
        .route("/all-users-json", get(admin::all_users))
        .route("/approve", post(admin::approve_user))
        .route("/list-invites", get(admin::list_invites))
        .route("/delete-invite", post(admin::delete_invite))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            auth::require_auth,
        ));

    let api_router = Router::new().merge(public_routes).merge(protected_routes);

    let static_service = ServeDir::new(&config.static_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(format!("{}/index.html", config.static_dir)));

    // -----------------------------------------------------------------------
    // CORS dynamique — origines lues depuis Config (PUBLIC_SITE_URL + ALLOWED_ORIGINS)
    //
    // ⚠️  Règle HTTP stricte :
    //   allow_credentials(true) est INCOMPATIBLE avec les wildcards (*).
    //   On doit lister explicitement chaque origine, méthode et header.
    //
    // LAN  : http://192.168.x.x:6300      → ajouté via PUBLIC_SITE_URL ou ALLOWED_ORIGINS
    // WAN  : https://nook.mondomaine.com  → ajouté via ALLOWED_ORIGINS
    // -----------------------------------------------------------------------
    let allowed_origins: Vec<axum::http::HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    eprintln!("[CORS] Origines autorisées :");
    for o in &config.allowed_origins {
        eprintln!("         - {}", o);
    }

    let cors_layer = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true);

    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/files", ServeDir::new(&config.uploads_dir))
        .merge(webrtc::webrtc_routes())
        .fallback_service(static_service)
        .layer(middleware::from_fn(base_inject_middleware))
        .layer(CompressionLayer::new())
        .layer(cors_layer)
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    eprintln!(
        "[🚀] Nook démarré sur http://0.0.0.0:{} (axum 0.8 + rand 0.9)",
        config.port
    );
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
