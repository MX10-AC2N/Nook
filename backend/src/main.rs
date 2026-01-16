// main.rs – Point d'entrée du serveur Nook Backend
// -------------------------------------------------
// Ce fichier a été modifié pour **injecter dynamiquement** le
// `<base href="…">` (ou tout autre meta) en fonction du header
// `Host` et du header `X‑Forwarded‑Proto` (fourni par Nginx Proxy Manager).
// Aucun rebuild n’est plus nécessaire : le même conteneur fonctionne
// en LAN (http://192.168.1.192:6300) et en production (https://mon‑site.exemple.com).

use axum::{
    body::{Body, Bytes},
    extract::ConnectInfo,
    http::{header, HeaderMap, HeaderValue, Request, Response, StatusCode},
    middleware::{self, Next},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use futures::future::BoxFuture;
use sqlx::SqlitePool;
use std::{convert::Infallible, fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tower::ServiceBuilder;
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
// 1️⃣ Middleware – injection du <base> (ou meta) à la volée
// ---------------------------------------------------------------------
async fn base_inject_middleware<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response<Body>, Infallible>
where
    B: Send + 'static,
{
    // -------------------------------------------------
    // 1️⃣ Récupérer le scheme (http / https) et le host
    // -------------------------------------------------
    // NPM ajoute le header `X-Forwarded-Proto`. S’il n’est pas présent (LAN direct),
    // on suppose `http`.
    let scheme = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");

    // Le header `Host` est toujours présent (ex. 192.168.1.192:6300 ou mon-site.exemple.com)
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let base_url = format!("{}://{}", scheme, host);
    let replacement = format!(r#"<base href="{}">"#, base_url);

    // -------------------------------------------------
    // 2️⃣ Appeler le handler suivant (qui génère la réponse)
    // -------------------------------------------------
    let mut resp = next.run(req).await;

    // -------------------------------------------------
    // 3️⃣ Modifier uniquement les réponses HTML
    // -------------------------------------------------
    if let Some(ct) = resp.headers().get(header::CONTENT_TYPE) {
        if ct.to_str().unwrap_or("").starts_with("text/html") {
            // Lire le corps complet
            let whole_body = hyper::body::to_bytes(resp.body_mut())
                .await
                .unwrap_or_else(|_| Bytes::new());

            // Convertir en String, remplacer le placeholder
            let mut body_str = String::from_utf8_lossy(&whole_body).into_owned();
            body_str = body_str.replace("<base-placeholder/>", &replacement);

            // Reconstruire la réponse avec le nouveau corps
            *resp.body_mut() = Body::from(body_str.clone());

            // Mettre à jour Content‑Length (important pour HTTP/1.1)
            resp.headers_mut().insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&body_str.len().to_string()).unwrap(),
            );
        }
    }

    Ok(resp)
}

// ---------------------------------------------------------------------
// 2️⃣ Initialisation de la base de données
// ---------------------------------------------------------------------
async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = "sqlite:/app/data/nook.db";
    let pool = SqlitePool::connect(db_path).await?;

    // Création des tables (omise ici pour concision – garde ton code actuel)
    // …
    eprintln!("[DB] Base de données initialisée");
    Ok(pool)
}

// ---------------------------------------------------------------------
// 3️⃣ Création de l'administrateur initial (inchangé)
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
    // -------------------------------------------------
    // Logger
    // -------------------------------------------------
    tracing_subscriber::fmt::init();

    // -------------------------------------------------
    // Création des dossiers persistants
    // -------------------------------------------------
    tokio::fs::create_dir_all("/app/data").await?;
    tokio::fs::create_dir_all("/app/data/uploads").await?;

    // -------------------------------------------------
    // Création du fichier DB s'il n'existe pas
    // -------------------------------------------------
    let db_file_path = std::path::Path::new("/app/data/nook.db");
    if !db_file_path.exists() {
        eprintln!("[Info] Création du fichier de base de données...");
        tokio::fs::File::create(db_file_path).await?;
    }

    // -------------------------------------------------
    // Initialisation DB + admin
    // -------------------------------------------------
    let pool = init_db().await?;
    check_initial_admin(&pool).await?;

    // -------------------------------------------------
    // Gestion du répertoire d'uploads
    // -------------------------------------------------
    let uploads_dir = PathBuf::from("/app/data/uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(&uploads_dir)?;
    }

    // -------------------------------------------------
    // Initialisation du FileManager & WebRTC state
    // -------------------------------------------------
    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));
    let webrtc_state = WebRtcState::new();

    // Tâche de nettoyage des fichiers expirés
    let fm_clone = (*file_manager).clone();
    tokio::spawn(async move {
        fm_clone.start_cleanup_task().await;
    });

    // -------------------------------------------------
    // Tâche de pruning (7 jours) – tourne en arrière‑plan
    // -------------------------------------------------
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        // Petite pause au démarrage
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        // Prune immédiat (utile si le conteneur a été arrêté longtemps)
        if let Err(e) = prune_old_data(&pool_clone).await {
            eprintln!("[Prune] Échec du pruning initial : {}", e);
        }
        loop {
            if let Err(e) = prune_old_data(&pool_clone).await {
                eprintln!("[Prune] Échec du pruning périodique : {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_hours(24)).await;
        }
    });

    // -------------------------------------------------
    // Construction de l'état partagé
    // -------------------------------------------------
    let shared_state = Arc::new(SharedState {
        db: pool.clone(),
        webrtc_state: webrtc_state.clone(),
        file_manager: file_manager.clone(),
    });

    // -------------------------------------------------
    // 5️⃣ Construction du routeur API
    // -------------------------------------------------
    let api_router = Router::new()
        // Auth
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/change-password", post(auth::change_password))
        .route("/api/auth/register", post(auth::register))
        // Join
        .route("/api/join", post(invites::join))
        .route("/api/invite/validate", get(invites::validate_invite))
        .route("/api/invite/accept", post(invites::accept_invite))
        // Conversations
        .route("/api/conversations", get(db::get_user_conversations))
        .route("/api/conversations", post(db::create_conversation))
        .route("/api/conversations/:id", get(db::get_conversation))
        .route("/api/conversations/:id/join", post(db::join_conversation))
        // Messages
        .route(
            "/api/conversations/:id/messages",
            get(db::get_conversation_messages),
        )
        .route("/api/conversations/:id/messages", post(db::send_message))
        // Uploads
        .route("/api/upload", post(upload::upload_handler))
        .route("/api/upload/chat", post(upload::upload_chat_file))
        // Admin
        .route("/api/pending-users-json", get(admin::pending_users))
        .route("/api/all-users-json", get(admin::all_users))
        .route("/api/approve", post(admin::approve_user))
        .route("/api/list-invites", get(admin::list_invites))
        .route("/api/generate-invite", post(invites::generate_invite))
        .route("/api/delete-invite", post(admin::delete_invite))
        // Health‑check
        .route("/api/health", get(|| async { "OK" }))
        // WebRTC
        .merge(webrtc::webrtc_routes())
        // Partage de l'état
        .with_state(shared_state.clone())
        // CORS (global)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // -------------------------------------------------
    // 6️⃣ Service statique (frontend SPA) avec fallback
    // -------------------------------------------------
    let static_path = "/app/static";
    eprintln!(
        "[Static] Servir les fichiers frontend depuis : {}",
        static_path
    );

    let static_service = ServeDir::new(static_path)
        .append_index_html_on_directories(true) // / → /index.html
        .precompressed_gzip()
        .precompressed_br()
        .fallback(ServeFile::new(format!("{static_path}/index.html")));

    // -------------------------------------------------
    // 7️⃣ Assemblage final du router avec le middleware
    // -------------------------------------------------
    let app = Router::new()
        // Middleware qui injecte le <base> (ou meta) dynamique
        .layer(ServiceBuilder::new().layer(middleware::from_fn(base_inject_middleware)))
        // Routes API
        .nest("/", api_router)
        // Service statique (fallback SPA)
        .fallback_service(static_service);

    // -------------------------------------------------
    // 8️⃣ Démarrage du serveur HTTP
    // -------------------------------------------------
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    eprintln!("[Serveur] Démarrage sur http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
