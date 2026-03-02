// main.rs – Axum 0.8 + rand 0.9 compatible
// CORS dynamique : origines lues depuis .env (ALLOWED_ORIGINS + PUBLIC_SITE_URL)
// Cookie adaptatif : SameSite=None;Secure (HTTPS/WAN) ou SameSite=Lax (HTTP/LAN)
// Session 15 — FIX: insérer admin + e2e_ci comme conversation_participants de default_global

use axum::{
    body::{to_bytes, Body},
    http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE},
        HeaderMap, HeaderValue, Request,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
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
mod chess;
mod config;
mod db;
mod invites;
mod polls;
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
    let start_time = std::time::Instant::now();

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

    tracing::debug!(
        method = %req.method(),
        uri = %req.uri(),
        scheme = %scheme,
        host = %host_str,
        base_url = %base_url,
        "→ Traitement de la requête HTTP"
    );

    let resp = next.run(req).await;
    let elapsed = start_time.elapsed();

    if let Some(ct) = resp.headers().get(CONTENT_TYPE) {
        if ct.to_str().is_ok_and(|s| s.starts_with("text/html")) {
            tracing::debug!(content_type = %ct.to_str().unwrap_or("unknown"), "Injection du base href dans le HTML");

            let (parts, body) = resp.into_parts();
            let bytes = to_bytes(body, 10_000_000)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Erreur lors de la lecture du corps de la réponse");
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                })?;
            let mut body_str = String::from_utf8_lossy(&bytes).into_owned();
            if body_str.contains("<base-placeholder/>") {
                body_str = body_str.replace("<base-placeholder/>", &replacement);
                tracing::debug!(base_href = %replacement, "Base href injecté avec succès");
            } else {
                tracing::trace!("Aucun placeholder <base-placeholder/> trouvé dans le HTML");
            }
            let body_bytes = Bytes::from(body_str);
            let content_length = body_bytes.len();
            let mut new_resp = Response::from_parts(parts, Body::from(body_bytes));
            if let Ok(len) = HeaderValue::from_str(&content_length.to_string()) {
                new_resp.headers_mut().insert(CONTENT_LENGTH, len);
            }

            tracing::debug!(
                content_length = content_length,
                elapsed_ms = elapsed.as_millis(),
                "← Réponse HTML avec base href servie"
            );

            return Ok(new_resp.into_response());
        }
    }

    tracing::trace!(
        elapsed_ms = elapsed.as_millis(),
        "← Réponse statique servie (non-HTML)"
    );

    Ok(resp)
}

// ---------------------------------------------------------------------
// DB + Initial admin
// ---------------------------------------------------------------------
async fn init_db(url: &str) -> Result<SqlitePool, sqlx::Error> {
    tracing::info!(database_url = %url, "Initialisation de la connexion SQLite");

    let opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let pool = SqlitePool::connect_with(opts).await?;

    tracing::info!("✓ Connexion SQLite établie avec succès");

    tracing::info!("Application des migrations de base de données...");
    migrate!("./migrations").run(&pool).await?;
    tracing::info!("✓ Migrations appliquées avec succès");

    Ok(pool)
}

async fn check_initial_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    tracing::debug!("Vérification de la présence d'un administrateur initial...");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    tracing::info!(user_count = count.0, "Nombre d'utilisateurs dans la base de données");

    let admin_id = "admin-initial-id-0000-0000-000000000001".to_string();

    if count.0 == 0 {
        tracing::warn!("⚠️  Aucun utilisateur trouvé - création de l'administrateur initial");

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

        tracing::info!(
            username = "admin",
            email = "admin@nook.local",
            role = "admin",
            "✓ Administrateur initial créé avec succès"
        );
        eprintln!("[Init] Admin initial créé — identifiants : admin / changeme2026");
        eprintln!("[Init] ⚠️  Changez le mot de passe dès la première connexion !");
    } else {
        tracing::debug!("Administrateur initial déjà existant");
    }

    // ============================================================
    // 🌐 Création de la conversation globale si inexistante
    // ============================================================
    let conv_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM conversations WHERE id = 'default_global'")
            .fetch_one(pool)
            .await?;

    if conv_count.0 == 0 {
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"INSERT INTO conversations (id, name, is_group, created_by, created_at, updated_at)
               VALUES ('default_global', 'Groupe Global', 1, 'admin-initial-id-0000-0000-000000000001', ?, ?)"#,
        )
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
        tracing::info!("✓ Conversation globale 'default_global' créée");
        eprintln!("[Init] Conversation globale créée");
    }

    // ============================================================
    // 👥 Inscrire TOUS les utilisateurs approuvés à default_global
    // (INSERT OR IGNORE → idempotent, safe à chaque redémarrage)
    // Cause session 15 : e2e_ci n'était pas participant → GET /conversations retournait []
    // ============================================================
    let now = Utc::now().timestamp();
    sqlx::query(
        r#"INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
           SELECT 'default_global', id, ?
           FROM users
           WHERE approved = 1"#,
    )
    .bind(now)
    .execute(pool)
    .await?;
    tracing::info!("✓ Tous les utilisateurs approuvés inscrits à 'default_global'");

    // Support E2E_SETUP=1 (CI Playwright)
    if std::env::var("E2E_SETUP").as_deref() == Ok("1") {
        tracing::info!("Mode E2E détecté - configuration de l'utilisateur de test");

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

            // FIX session 15 : ajouter e2e_ci comme participant à default_global
            sqlx::query(
                "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
                 VALUES ('default_global', ?, ?)",
            )
            .bind(&e2e_id)
            .bind(now)
            .execute(pool)
            .await?;

            tracing::info!(
                username = "e2e_ci",
                email = "e2e@nook.local",
                role = "user",
                "✓ Utilisateur E2E créé et ajouté à default_global"
            );
            eprintln!("[E2E] Utilisateur e2e_ci créé et inscrit à default_global");
        } else {
            // e2e_ci existe déjà — s'assurer qu'il est participant (re-run CI)
            let e2e_row: (String,) =
                sqlx::query_as("SELECT id FROM users WHERE username = 'e2e_ci'")
                    .fetch_one(pool)
                    .await?;
            let now = Utc::now().timestamp();
            sqlx::query(
                "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
                 VALUES ('default_global', ?, ?)",
            )
            .bind(&e2e_row.0)
            .bind(now)
            .execute(pool)
            .await?;
            tracing::debug!("Utilisateur E2E déjà existant — participation default_global vérifiée");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------
// MAIN
// ---------------------------------------------------------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    tracing_subscriber::fmt::init();

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let pid = std::process::id();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        pid = pid,
        hostname = %hostname,
        "╔════════════════════════════════════════════════════════════╗"
    );
    tracing::info!(
        "║              🚀 NOOK - Serveur démarré                      ║"
    );
    tracing::info!(
        "╚════════════════════════════════════════════════════════════╝"
    );

    tracing::info!("Chargement de la configuration depuis les variables d'environnement...");
    let config = Config::load();

    tracing::info!(
        port = config.port,
        static_dir = %config.static_dir,
        uploads_dir = %config.uploads_dir,
        database_url = %config.database_url,
        "✓ Configuration chargée"
    );

    tracing::info!("Origines CORS autorisées:");
    for origin in &config.allowed_origins {
        tracing::info!("  • {}", origin);
    }

    tokio::fs::create_dir_all("/app/data").await?;
    tokio::fs::create_dir_all(&config.uploads_dir).await?;
    tracing::info!("✓ Répertoires de travail créés/vérifiés");

    tracing::info!("Connexion à la base de données SQLite...");
    let pool = init_db(&config.database_url).await?;

    tracing::info!("Vérification de l'administrateur initial...");
    check_initial_admin(&pool).await?;
    tracing::info!("✓ Base de données initialisée");

    let uploads_dir = PathBuf::from(&config.uploads_dir);
    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));

    let webrtc_state = WebRtcState::new();
    tracing::info!("✓ État WebRTC initialisé");

    let fm_clone = (*file_manager).clone();
    tokio::spawn(async move {
        fm_clone.start_cleanup_task().await;
    });
    tracing::info!("✓ Tâche de nettoyage des fichiers planifiée");

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        loop {
            let result = prune_old_data(&pool_clone).await;
            match result {
                Ok(_) => tracing::debug!("Nettoyage des anciennes données terminé"),
                Err(e) => tracing::error!(error = %e, "Erreur lors du nettoyage des anciennes données"),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(24 * 3600)).await;
        }
    });
    tracing::info!("✓ Tâche de suppression planifiée (toutes les 24 heures)");

    let shared_state = Arc::new(SharedState {
        db: pool,
        webrtc_state,
        file_manager,
    });

    // ============================================================
    // 🛣️ Routes publiques (aucune authentification)
    // ============================================================
    let public_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/join", post(invites::join))
        .route("/invite/validate", get(invites::validate_invite))
        .route("/health", get(|| async { "OK" }));

    tracing::info!("Routes publiques configurées:");
    tracing::info!("  • POST   /auth/register");
    tracing::info!("  • POST   /auth/login");
    tracing::info!("  • POST   /join");
    tracing::info!("  • GET    /invite/validate");
    tracing::info!("  • GET    /health");

    // ============================================================
    // 👑 Routes ADMIN uniquement (auth + rôle admin requis)
    // ============================================================
    let admin_routes = Router::new()
        .route("/users/pending", get(admin::pending_users))
        .route("/users", get(admin::all_users))
        .route("/users/approve", post(admin::approve_user))
        .route("/invites", get(admin::list_invites))
        .route("/invites", post(invites::generate_invite))
        .route("/invites/delete", post(admin::delete_invite))
        .layer(middleware::from_fn(auth::require_admin));

    // ============================================================
    // 🔐 Routes protégées (tous les utilisateurs authentifiés)
    // ============================================================
    let protected_routes = Router::new()
        .merge(admin_routes)
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
        .route("/user/update", post(db::update_user_profile))
        .route("/events", get(db::get_events))
        .route("/events", post(db::create_event))
        .route("/events/{id}", delete(db::delete_event))
        .route("/conversations/{id}/participants", get(db::get_conversation_participants))
        .route("/conversations/{id}/participants", post(db::add_conversation_participant))
        .route("/conversations/{id}/leave", post(db::leave_conversation))
        .route("/users/available", get(db::get_available_users))
        .merge(polls::polls_routes())
        .merge(chess::chess_routes())
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            auth::require_auth,
        ));

    tracing::info!("✓ Routes protégées + admin configurées");
    tracing::info!("Routes admin disponibles :");
    tracing::info!("  • GET    /users/pending");
    tracing::info!("  • GET    /users");
    tracing::info!("  • POST   /users/approve");
    tracing::info!("  • GET    /invites");
    tracing::info!("  • POST   /invites");
    tracing::info!("  • POST   /invites/delete");

    let api_router = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(|| async {
            (
                axum::http::StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({
                    "success": false,
                    "message": "Route API introuvable"
                })),
            )
        });

    tracing::info!("✓ Routeur API configuré");

    let static_service = ServeDir::new(&config.static_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(format!("{}/index.html", config.static_dir)));

    // -----------------------------------------------------------------------
    // CORS dynamique
    // ⚠️  allow_credentials(true) incompatible avec wildcards → origines explicites
    // -----------------------------------------------------------------------
    let allowed_origins: Vec<axum::http::HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("[CORS] Origines autorisées :");
    for o in &config.allowed_origins {
        tracing::info!("         • {}", o);
    }
    tracing::info!("═══════════════════════════════════════════════════════════");

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

    tracing::info!("✓ Application Axum construite avec tous les layers");

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("║  🎉 NOOK - SERVEUR PRÊT À RECEVOIR DES REQUÊTES           ║");
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!(
        address = %addr,
        port = config.port,
        "Serveur HTTP en écoute sur 0.0.0.0 (toutes les interfaces)"
    );

    eprintln!(
        "[🚀] Nook démarré sur http://0.0.0.0:{} (axum 0.8 + rand 0.9)",
        config.port
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let startup_duration = start_time.elapsed();
    tracing::info!(
        startup_time_ms = startup_duration.as_millis(),
        "Temps de démarrage total"
    );

    axum::serve(listener, app).await?;

    Ok(())
}
