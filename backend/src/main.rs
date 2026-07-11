// main.rs – Axum 0.8 + rand 0.9 compatible
// CORS dynamique : origines lues depuis .env (ALLOWED_ORIGINS + PUBLIC_SITE_URL)
// Cookie adaptatif : SameSite=None;Secure (HTTPS/WAN) ou SameSite=Lax (HTTP/LAN)
// Session 36 — SEC-02: rate limiter keyed par IP (KeyedRateLimiter)
// Session 38 — SEC-02: quota configurable via RATE_LIMIT_PER_MIN (défaut 60)
//            — Suppression base_inject_middleware (inutile avec SvelteKit adapter-static)

use axum::{
    body::Body,
    extract::{ConnectInfo, DefaultBodyLimit, Path, Query},
    http::{
        header::CONTENT_TYPE,
        Request,
    },
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use bytes::Bytes;
use sqlx::{migrate, sqlite::SqliteConnectOptions, SqlitePool};
use std::{net::SocketAddr, net::IpAddr, path::PathBuf, str::FromStr, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    set_header::SetResponseHeaderLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use chrono::Utc;
use rand::{rng, distr::Alphanumeric, Rng};

mod admin;
mod auth;
mod chess;
mod chess_engine;
mod config;
mod db;
mod e2ee;
mod invites;
mod missed_calls;
mod polls;
mod reactions;
mod push;
mod prune;
mod upload;
mod emergency;
mod gifs_updater;
mod search;
mod presence;
mod webrtc;
mod sfu;
mod events;
mod ca;
mod analytics;

use crate::config::Config;
use crate::prune::prune_old_data;
use sfu::SfuState;
use webrtc::{FileManager, WebRtcState};

// ---------------------------------------------------------------------
// SEC-02 : Rate limiter KEYED par IP (30 req / 60s par adresse)
// Remplace le NotKeyed global qui causait des faux-positifs en CI
// et ne protégeait pas correctement contre le brute-force par IP unique.
// ---------------------------------------------------------------------
type IpRateLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;

#[derive(Clone)]
pub struct SharedState {
    pub db: SqlitePool,
    pub webrtc_state: WebRtcState,
    pub file_manager: Arc<FileManager>,
    pub sfu_state: SfuState,
    pub config: Config,
    pub presence_state: presence::PresenceState,
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
    
    // ── Fix events table schema if needed ──────────────────────────────
    // Migration 16 creates the table with start_time, but if the table
    // already exists with an old schema, we need to fix it before migrations
    fix_events_schema(&pool).await?;
    
    tracing::info!("Application des migrations de base de données...");
    migrate!("./migrations").run(&pool).await?;
    tracing::info!("✓ Migrations appliquées avec succès");

    Ok(pool)
}

// ── Fix events table: ensure start_time/end_time columns exist ───────
async fn fix_events_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if events table exists
    let table_exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='events')"
    )
    .fetch_one(pool)
    .await?;

    if !table_exists.0 {
        return Ok(()); // Table doesn't exist yet, migration 16 will create it
    }

    // Add missing columns if needed
    let missing_columns = [
        ("start_time", "INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))"),
        ("end_time", "INTEGER NOT NULL DEFAULT (strftime('%s', 'now') + 3600)"),
    ];

    for (col_name, col_def) in &missing_columns {
        let col_exists: (bool,) = sqlx::query_as(
            &format!("SELECT EXISTS(SELECT 1 FROM pragma_table_info('events') WHERE name='{}')", col_name)
        )
        .fetch_one(pool)
        .await?;

        if !col_exists.0 {
            tracing::warn!("Adding column '{}' to events table", col_name);
            let alter_sql = format!("ALTER TABLE events ADD COLUMN {} {}", col_name, col_def);
            sqlx::query(&alter_sql).execute(pool).await?;
        }
    }

    tracing::info!("✓ Events table schema verified");
    Ok(())
}

async fn check_initial_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    tracing::debug!("Vérification de la présence d'un administrateur initial...");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    tracing::info!(
        user_count = count.0,
        "Nombre d'utilisateurs dans la base de données"
    );

    let admin_id = "admin-initial-id-0000-0000-000000000001".to_string();

    if count.0 == 0 {
        tracing::warn!("⚠️  Aucun utilisateur trouvé - création de l'administrateur initial");

        // FIX C2: generer un mot de passe aleatoire au lieu d'un mot de passe statique
        // Si ADMIN_INITIAL_PASSWORD est defini (CI/testing), l'utiliser
        let random_password: String = std::env::var("ADMIN_INITIAL_PASSWORD")
            .unwrap_or_else(|_| {
                rng()
                    .sample_iter(&Alphanumeric)
                    .take(16)
                    .map(char::from)
                    .collect()
            });
        let password_hash = crate::auth::hash_password(&random_password);
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
        eprintln!("[Init] ⚠️  Changez le mot de passe des la premiere connexion !");
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
            let e2e_password = std::env::var("E2E_PASSWORD")
            .unwrap_or_else(|_| {
                eprintln!("[E2E] ATTENTION: E2E_PASSWORD non défini, utilisation d'un mot de passe aléatoire");
                uuid::Uuid::new_v4().to_string()
            });
        let e2e_hash = crate::auth::hash_password(&e2e_password);
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
                "✓ Utilisateur E2E créé et ajouté à default_global"
            );
            eprintln!("[E2E] Utilisateur e2e_ci créé et inscrit à default_global");
        } else {
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
    tracing::info!("║              🚀 NOOK - Serveur démarré                      ║");
    tracing::info!("╚════════════════════════════════════════════════════════════╝");

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
    tokio::fs::create_dir_all(&config.gifs_dir).await?;
    tracing::info!("✓ Répertoires de travail créés/vérifiés");

    tracing::info!("Connexion à la base de données SQLite...");
    let pool = init_db(&config.database_url).await?;

    tracing::info!("Vérification de l'administrateur initial...");
    check_initial_admin(&pool).await?;
    tracing::info!("✓ Base de données initialisée");

    let uploads_dir = PathBuf::from(&config.uploads_dir);
    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));

    let webrtc_state = WebRtcState::new();
    let sfu_state = SfuState::new(config.sfu_relay_capacity);
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

    // Mise à jour hebdomadaire des GIFs (GIPHY_API_KEY dans .env)
    // No-op silencieux si la clé est absente
    gifs_updater::start(config.gifs_dir.clone());
    tracing::info!("✓ Tâche de mise à jour GIFs planifiée (toutes les 7 jours)");

    let shared_state = Arc::new(SharedState {
        db: pool,
        webrtc_state,
        file_manager,
        sfu_state,
        config: config.clone(),
        presence_state: presence::PresenceState::new(),
    });

    // ============================================================
    // 🛣️ Routes publiques — SEC-02 : rate limiter par IP
    // Quota configurable via RATE_LIMIT_PER_MIN (défaut : 60)
    // En prod : 60/min bloque le brute-force sans gêner l'usage normal
    // En CI   : 60/min suffit pour 3 suites × retries sans 429
    // ============================================================
    let rate_limit: u32 = std::env::var("RATE_LIMIT_PER_MIN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let ip_limiter: Arc<IpRateLimiter> = Arc::new(
        RateLimiter::keyed(Quota::per_minute(NonZeroU32::new(rate_limit).unwrap()))
    );

    // 🔐 Auth rate limiter: stricter limits for auth endpoints (5 attempts/min per IP)
    let auth_limiter: Arc<IpRateLimiter> = Arc::new(
        RateLimiter::keyed(Quota::per_minute(NonZeroU32::new(5).unwrap()))
    );

    let auth_limiter_clone = auth_limiter.clone();
    let auth_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route_layer(middleware::from_fn(move |ConnectInfo(addr): ConnectInfo<SocketAddr>, req: Request<Body>, next: Next| {
            let lim = auth_limiter_clone.clone();
            async move {
                match lim.check_key(&addr.ip()) {
                    Ok(_) => next.run(req).await,
                    Err(_) => {
                        tracing::warn!(
                            ip = %addr.ip(),
                            path = %req.uri().path(),
                            "Auth rate limit exceeded (429) — too many login attempts"
                        );
                        axum::http::StatusCode::TOO_MANY_REQUESTS.into_response()
                    }
                }
            }
        }));

    let limiter_clone = ip_limiter.clone();
    let public_routes = Router::new()
        .route("/join", post(invites::join))
        .route("/invite/validate", get(invites::validate_invite))
        .route("/invite/accept", axum::routing::post(invites::accept_invite))
        .route("/health", get(|| async { "OK" }))
        .route("/avatar/{style}/svg", get(avatar_proxy))
        .nest("/push", push::public_router())
        .route_layer(middleware::from_fn(move |
            ConnectInfo(addr): ConnectInfo<SocketAddr>,
            req: Request<Body>,
            next: Next,
        | {
            let lim = limiter_clone.clone();
            async move {
                match lim.check_key(&addr.ip()) {
                    Ok(_) => next.run(req).await,
                    Err(_) => {
                        tracing::warn!(
                            ip = %addr.ip(),
                            path = %req.uri().path(),
                            "Rate limit dépassé (429) — IP bloquée temporairement"
                        );
                        axum::http::StatusCode::TOO_MANY_REQUESTS.into_response()
                    }
                }
            }
        }));

    tracing::info!("Routes publiques configurées (rate limit: {}/min par IP):", rate_limit);
    tracing::info!("  • POST   /auth/register (auth rate limit: 5/min)");
    tracing::info!("  • POST   /auth/login (auth rate limit: 5/min)");
    tracing::info!("  • POST   /join");
    tracing::info!("  • GET    /invite/validate");
    tracing::info!("  • POST   /invite/accept");
    tracing::info!("  • GET    /health");
    // ============================================================
    // 👑 Routes ADMIN uniquement (auth + rôle admin requis)
    // ============================================================
    let admin_routes = Router::new()
        .route("/users/pending", get(admin::pending_users))
        .route("/users", get(admin::all_users))
        .route("/users/approve", post(admin::approve_user))
        .route("/users/reject", post(admin::reject_user))
        .route("/invites", get(admin::list_invites))
        .route("/invites", post(invites::generate_invite))
        .route("/invites/delete", post(admin::delete_invite))
        .route("/analytics", get(admin::get_analytics))
        .route("/users/{id}", axum::routing::delete(admin::delete_user))
        .route("/metrics", get(admin::get_system_metrics))
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
        .route("/conversations/{id}/rename", axum::routing::patch(db::rename_conversation))
        .route("/conversations/{id}/join", post(db::join_conversation))
        .route("/conversations/{id}/messages", get(db::get_conversation_messages))
        .route("/conversations/{id}/messages", post(db::send_message))
        .route(
            "/conversations/{conv_id}/messages/{msg_id}",
            axum::routing::patch(db::edit_message).delete(db::delete_message),
        )
        .route("/upload", post(upload::upload_handler))
        .route("/upload/chat", post(upload::upload_chat_file))
        .route("/download/{file_id}", get(upload::download_file))
        .route("/user/update", post(db::update_user_profile))
        .route("/conversations/{id}/participants", get(db::get_conversation_participants))
        .route("/conversations/{id}/participants", post(db::add_conversation_participant))
        .route("/conversations/{id}/leave", post(db::leave_conversation))
        .route("/users/available", get(db::get_available_users))
        .route("/emergency", post(emergency::handle_emergency))
        .nest("/push", push::router())
        .merge(polls::polls_routes())
        .merge(chess::chess_routes())
        .merge(e2ee::e2ee_routes())
        .merge(reactions::reactions_routes())
        .merge(webrtc::webrtc_routes())
        .merge(events::events_routes())
        .merge(missed_calls::missed_calls_routes())
        .merge(search::search_routes())
        .merge(presence::presence_routes())
        .merge(analytics::analytics_routes())
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            auth::require_auth,
        ));

    tracing::info!("✓ Routes protégées + admin configurées");

    let api_router = Router::new()
        .merge(auth_routes)
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

    // SvelteKit adapter-static génère des assets avec paths absolus (/_app/...)
    // → aucun <base href> nécessaire — base_inject_middleware supprimé (session 36)
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
            CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true);

    // ConnectInfo requis pour extraire l'IP dans le rate limiter par IP (SEC-02)
    let app = Router::new()
        .nest("/api", api_router)
        // FIX upload > 7Mo : Axum par défaut limite le body à 2MB.
        // On monte à 52MB (50MB fichier + overhead multipart).
        // La validation métier (50MB max) reste dans upload.rs.
        .layer(DefaultBodyLimit::max(52 * 1024 * 1024))
        .nest_service("/files", ServeDir::new(&config.uploads_dir))
        // GIFs : volume de données en priorité, fallback sur les GIFs de l'image Docker
        .nest_service("/gifs",
            ServeDir::new(&config.gifs_dir)
                .fallback(ServeDir::new(format!("{}/gifs", config.static_dir)))
        )
        .route("/ca", get(ca::get_ca_cert))
        .route("/ca/help", get(ca::ca_help))
        .fallback_service(static_service)

        // 🛡️ Security headers middleware
        .layer(middleware::from_fn(|req: Request<Body>, next: Next| async move {
            // HSTS check must happen BEFORE consuming req
            let is_https = req.headers().get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https");
            
            let mut response = next.run(req).await;
            let headers = response.headers_mut();
            headers.insert("X-Frame-Options", "DENY".parse().unwrap());
            headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
            headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
            headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
            headers.insert("Permissions-Policy", "camera=(self), microphone=(self), geolocation=(), payment=()".parse().unwrap());
            headers.insert("Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' ws: wss:; media-src 'self' blob:; frame-ancestors 'none'".parse().unwrap());
            // HSTS uniquement sur HTTPS (nginx termine TLS et forward x-forwarded-proto: https)
            if is_https {
                headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
            }
            response
        }))
        // Cache-Control for static assets (1h for hashed assets, no-cache for HTML)
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("public, max-age=3600"),
        ))
        .layer(CompressionLayer::new())
        .layer(cors_layer)
        .with_state(shared_state)
        .into_make_service_with_connect_info::<SocketAddr>();

// --- Avatar Proxy Handler ---
async fn avatar_proxy(
    Path(style): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl axum::response::IntoResponse {
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    use reqwest::Client;

    let valid_styles = [
        "adventurer", "avataaars", "open-peeps", "notionists", "fun-emoji",
        "big-smile", "lorelei", "personas", "bottts", "initials",
    ];

    let style = if valid_styles.contains(&style.as_str()) {
        style
    } else {
        "adventurer".to_string()
    };

    let seed = params.get("seed").cloned().unwrap_or_else(|| "nook".to_string());
    let size = params.get("size").and_then(|s| s.parse::<u32>().ok()).unwrap_or(32).min(512);

    let dicebear_url = if style == "initials" {
        format!("https://api.dicebear.com/9.x/initials/svg?seed={}&size={}", urlencoding::encode(&seed), size)
    } else {
        format!("https://api.dicebear.com/9.x/{}/svg?seed={}&size={}", style, urlencoding::encode(&seed), size)
    };

    let client = Client::new();
    match client.get(&dicebear_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let content_type = resp.headers().get("content-type").cloned().unwrap_or_else(|| HeaderValue::from_static("image/svg+xml"));
            let bytes = resp.bytes().await.unwrap_or_default();
            let mut headers = HeaderMap::new();
            headers.insert("content-type", content_type);
            headers.insert("cache-control", HeaderValue::from_static("public, max-age=31536000"));
            (StatusCode::OK, headers, bytes)
        }
        Ok(resp) => {
            tracing::warn!(status = %resp.status(), "DiceBear proxy error");
            (StatusCode::BAD_GATEWAY, HeaderMap::new(), Bytes::new())
        }
        Err(e) => {
            tracing::error!(error = %e, "DiceBear proxy request failed");
            (StatusCode::BAD_GATEWAY, HeaderMap::new(), Bytes::new())
        }
    }
}
// ---

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