// main.rs – Point d'entrée du serveur Nook Backend
// -------------------------------------------------
// Injection dynamique du <base href="…"> via middleware (Host + X-Forwarded-Proto)
// Plus besoin de rebuild pour LAN vs prod

use axum::{
    body::{to_bytes, Body, Bytes},
    http::{header, HeaderValue, Request, Response},
    middleware::{self, Next},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use sqlx::{migrate, SqlitePool};
use std::convert::Infallible;
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{
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
// 1️⃣ Middleware – injection du <base> à la volée
// ---------------------------------------------------------------------
async fn base_inject_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let scheme = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");

    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let base_url = format!("{}://{}", scheme, host);
    let replacement = format!(r#"<base href="{}">"#, base_url);

    let resp = next.run(req).await;

    if let Some(ct) = resp.headers().get(header::CONTENT_TYPE) {
        if ct.to_str().unwrap_or("").starts_with("text/html") {
            let (parts, body) = resp.into_parts();
            let whole_body = to_bytes(body, 10_000_000)
                .await
                .unwrap_or_else(|_| Bytes::new());
            let mut body_str = String::from_utf8_lossy(&whole_body).into_owned();
            body_str = body_str.replace("<base-placeholder/>", &replacement);

            let mut new_response = Response::from_parts(parts, Body::from(body_str.clone()));
            new_response.headers_mut().insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&body_str.len().to_string()).unwrap(),
            );

            return Ok(new_response);
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

    // Applique les migrations du dossier ./migrations
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

    // Création des dossiers persistants
    tokio::fs::create_dir_all("/app/data").await?;
    tokio::fs::create_dir_all("/app/data/uploads").await?;

    // Création du fichier DB s'il n'existe pas
    let db_file_path = std::path::Path::new("/app/data/nook.db");
    if !db_file_path.exists() {
        eprintln!("[Info] Création du fichier de base de données...");
        tokio::fs::File::create(db_file_path).await?;
    }

    // Initialisation DB avec migrations + admin initial
    let pool = init_db().await?;
    check_initial_admin(&pool).await?;

    // Gestion du répertoire d'uploads
    let uploads_dir = PathBuf::from("/app/data/uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(&uploads_dir)?;
    }

    // Initialisation du FileManager & WebRTC state
    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));
    let webrtc_state = WebRtcState::new();

    // Tâche de nettoyage des fichiers expirés
    let fm_clone = (*file_manager).clone();
    tokio::spawn(async move {
        fm_clone.start_cleanup_task().await;
    });

    // Tâche de pruning périodique
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

    // État partagé
    let shared_state = Arc::new(SharedState {
        db: pool.clone(),
        webrtc_state: webrtc_state.clone(),
        file_manager: file_manager.clone(),
    });

    // Routeur API (inchangé)
    let api_router = Router::new()
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/change-password", post(auth::change_password))
        .route("/api/join", post(invites::join))
        .route("/api/invite/validate", get(invites::validate_invite))
        .route("/api/conversations", get(db::get_user_conversations))
        .route("/api/conversations", post(db::create_conversation))
        .route("/api/conversations/:id", get(db::get_conversation))
        .route("/api/conversations/:id/join", post(db::join_conversation))
        .route(
            "/api/conversations/:id/messages",
            get(db::get_conversation_messages),
        )
        .route("/api/conversations/:id/messages", post(db::send_message))
        .route("/api/upload", post(upload::upload_handler))
        .route("/api/upload/chat", post(upload::upload_chat_file))
        .route("/api/pending-users-json", get(admin::pending_users))
        .route("/api/all-users-json", get(admin::all_users))
        .route("/api/approve", post(admin::approve_user))
        .route("/api/list-invites", get(admin::list_invites))
        .route("/api/generate-invite", post(invites::generate_invite))
        .route("/api/delete-invite", post(admin::delete_invite))
        .route("/api/health", get(|| async { "OK" }))
        .merge(webrtc::webrtc_routes())
        .with_state(shared_state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Service statique SPA
    let static_path = "/app/static";
    eprintln!(
        "[Static] Servir les fichiers frontend depuis : {}",
        static_path
    );

    let static_service = ServeDir::new(static_path)
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .precompressed_br()
        .fallback(ServeFile::new(format!("{static_path}/index.html")));

    // Ajout pour l'uploads dans le chat
    let uploads_service = ServeDir::new("/app/data/uploads")
        .precompressed_gzip()
        .precompressed_br();

    // Assemblage final
    let app = Router::new()
        .layer(middleware::from_fn(base_inject_middleware))
        .nest("/", api_router)
        .fallback_service(static_service)
        .nest_service(
        "/api/files",
        ServeDir::new("/app/data/uploads")
            .precompressed_gzip()
            .precompressed_br(),
    );

    // Démarrage serveur
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    eprintln!("[Serveur] Démarrage sur http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
