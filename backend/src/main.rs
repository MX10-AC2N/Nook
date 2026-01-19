// main.rs – Point d'entrée du serveur Nook Backend
// -------------------------------------------------
// Middleware conservé et amélioré : injection dynamique du <base href="…/" />
// - Trailing slash ajouté (best practice pour SvelteKit)
// - Self-closing tag
// - Fallback safe si Host absent
// - Gestion x-forwarded-proto pour prod HTTPS
// - Compression à la volée (évite tout conflit avec precompressed files)

use axum::{
    body::{to_bytes, Body},
    extract::Host,
    http::{header, HeaderMap, Request},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use sqlx::{migrate, SqlitePool};
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

// ---------------------------------------------------------------------
// Modules de l'application
// ---------------------------------------------------------------------
mod admin;
mod auth;
mod db;
mod invites;
mod prune;
mod upload;
mod webrtc;

use crate::prune::prune_old_data;
use webrtc::{FileManager, WebRtcState};

// ---------------------------------------------------------------------
// Structure d'état partagé
// ---------------------------------------------------------------------
#[derive(Clone)]
pub struct SharedState {
    pub db: SqlitePool,
    pub webrtc_state: WebRtcState,
    pub file_manager: Arc<FileManager>,
}

// ---------------------------------------------------------------------
// 1️⃣ Middleware – injection dynamique du <base> (conservé et renforcé)
// ---------------------------------------------------------------------
async fn base_inject_middleware(
    Host(host): Host,
    headers: HeaderMap,
    req: Request<Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Scheme (http ou https via proxy)
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");

    // Host avec port inclus (ex: 192.168.1.10:6300 ou mon-domaine.com)
    let host_str = if host.is_empty() {
        "localhost:6300".to_string()
    } else {
        host
    };

    // Base URL complète avec trailing slash
    let base_url = format!("{}://{}/", scheme, host_str);

    // Tag à injecter (self-closing, moderne)
    let replacement = format!("<base href=\"{}\" />", base_url);

    let mut resp = next.run(req).await;

    // Ne modifier que les réponses HTML
    if let Some(ct) = resp.headers().get(header::CONTENT_TYPE) {
        if ct
            .to_str()
            .ok()
            .map_or(false, |s| s.starts_with("text/html"))
        {
            let (parts, body) = resp.into_parts();
            let bytes = to_bytes(body, 10_000_000)
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

            let mut body_str = String::from_utf8_lossy(&bytes).into_owned();

            // Remplacement du placeholder
            if body_str.contains("<base-placeholder/>") {
                body_str = body_str.replace("<base-placeholder/>", &replacement);
            }
            // Si pas de placeholder → on laisse tel quel (fail-safe)

            let mut new_resp = Response::from_parts(parts, Body::from(body_str.as_bytes()));

            // Recalcul Content-Length
            if let Ok(len_str) = body_str.len().to_string().parse() {
                new_resp
                    .headers_mut()
                    .insert(header::CONTENT_LENGTH, len_str);
            }

            return Ok(new_resp.into_response());
        }
    }

    Ok(resp)
}

// ---------------------------------------------------------------------
// 2️⃣ Initialisation de la base de données (avec migrations sqlx)
// ---------------------------------------------------------------------
async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = "sqlite:/app/data/nook.db";
    let pool = SqlitePool::connect(db_path).await?;

    migrate!("./migrations").run(&pool).await?;

    eprintln!("[DB] Migrations appliquées avec succès");

    Ok(pool)
}

// ---------------------------------------------------------------------
// 3️⃣ Création de l'administrateur initial
// ---------------------------------------------------------------------
async fn check_initial_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if user_count.0 == 0 {
        let admin_id = "admin-initial-id-0000-0000-000000000001".to_string();
        let default_password = "changeme2026";
        let password_hash = crate::auth::hash_password(default_password);
        let now_ts = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, email, password_hash,
                name, role, approved, needs_password_change, created_at
            )
            VALUES (?, ?, ?, ?, ?, 'admin', 1, 1, ?)
            "#,
        )
        .bind(&admin_id)
        .bind("admin")
        .bind("admin@nook.local")
        .bind(&password_hash)
        .bind("Administrateur Initial")
        .bind(now_ts)
        .execute(pool)
        .await?;

        eprintln!(
            "[Init] Admin initial créé (ID: {}). Change username/password au premier login !",
            admin_id
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------
// 4️⃣ Point d'entrée principal
// ---------------------------------------------------------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    tokio::fs::create_dir_all("/app/data").await?;
    tokio::fs::create_dir_all("/app/data/uploads").await?;

    let db_file_path = std::path::Path::new("/app/data/nook.db");
    if !db_file_path.exists() {
        eprintln!("[Info] Création du fichier de base de données...");
        tokio::fs::File::create(db_file_path).await?;
    }

    let pool = init_db().await?;
    check_initial_admin(&pool).await?;

    let uploads_dir = PathBuf::from("/app/data/uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(&uploads_dir)?;
    }

    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));
    let webrtc_state = WebRtcState::new();

    let fm_clone = (*file_manager).clone();
    tokio::spawn(async move {
        fm_clone.start_cleanup_task().await;
    });

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        if let Err(e) = prune_old_data(&pool_clone).await {
            eprintln!("[Prune] Échec du pruning initial : {}", e);
        }
        loop {
            if let Err(e) = prune_old_data(&pool_clone).await {
                eprintln!("[Prune] Échec du pruning périodique : {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(24 * 3600)).await;
        }
    });

    let shared_state = Arc::new(SharedState {
        db: pool.clone(),
        webrtc_state: webrtc_state.clone(),
        file_manager: file_manager.clone(),
    });

    let api_router = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/change-password", post(auth::change_password))
        .route("/join", post(invites::join))
        .route("/invite/validate", get(invites::validate_invite))
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
        .route("/generate-invite", post(invites::generate_invite))
        .route("/delete-invite", post(admin::delete_invite))
        .route("/health", get(|| async { "OK" }))
        .merge(webrtc::webrtc_routes())
        .with_state(shared_state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let static_path = "/app/static";
    eprintln!("[Static] Servir les fichiers frontend depuis : {}", static_path);

    // Pas de precompressed → on compresse dynamiquement après le middleware
    let static_service = ServeDir::new(static_path)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(format!("{static_path}/index.html")));

    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/files", ServeDir::new("/app/data/uploads"))
        .fallback_service(static_service)
        // Middleware en dernier pour qu'il voit toutes les réponses
        .layer(middleware::from_fn(base_inject_middleware))
        // Compression après modification du HTML
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    eprintln!("[Serveur] Démarrage sur http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
