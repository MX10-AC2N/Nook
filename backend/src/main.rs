// main.rs - Point d'entrée du serveur Nook Backend

// Backend Axum pour gestion d'upload, WebRTC, et chiffrement

use axum::{
    routing::{get, post, get_service},
    Router,
    handler::Handler,
};
use std::{fs, path::PathBuf, sync::Arc, net::SocketAddr};

use sqlx::SqlitePool;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::{ServeDir, ServeFile};
use axum::http::StatusCode;

// Modules
mod db;
mod auth;
mod webrtc;
mod upload;
mod prune;

// Import des structures partagées
use webrtc::{WebRtcState, FileManager};
// Import de prune.rs
use crate::prune::prune_old_data;

/// Structure contenant l'état partagé entre les différents handlers.
#[derive(Clone)]
pub struct SharedState {
    pub db: SqlitePool,
    pub webrtc_state: WebRtcState,
    pub file_manager: Arc<FileManager>,
}

// ---------------------------------------------------------------------------
//  INITIALISATION DE LA BASE DE DONNÉES
// ---------------------------------------------------------------------------
async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = "sqlite:/app/data/nook.db";
    let pool = SqlitePool::connect(db_path).await?;

    // Créer les tables si elles n'existent pas
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            name TEXT,
            role TEXT DEFAULT 'user',
            approved BOOLEAN DEFAULT 0,
            needs_password_change BOOLEAN DEFAULT 0,
            token TEXT,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            name TEXT,
    is_group BOOLEAN DEFAULT 0,
            created_at INTEGER NOT NULL,
            created_by TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS conversation_participants (
            conversation_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            joined_at INTEGER NOT NULL,
            PRIMARY KEY (conversation_id, user_id),
            FOREIGN KEY (conversation_id) REFERENCES conversations(id),
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            content TEXT,
            message_type TEXT DEFAULT 'text',
            file_id TEXT,
            encrypted BOOLEAN DEFAULT 0,
            timestamp INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            edited_at INTEGER,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id),
            FOREIGN KEY (sender_id) REFERENCES users(id),
            FOREIGN KEY (file_id) REFERENCES uploads(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS uploads (
            id TEXT PRIMARY KEY,
            conversation_id TEXT,
            from_user_id TEXT,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            content_type TEXT,
            uploaded_at INTEGER NOT NULL,
            encrypted BOOLEAN DEFAULT 0,
            nonce TEXT,
            key_text TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    eprintln!("[DB] Base de données initialisée");
    Ok(pool)
}

// ---------------------------------------------------------------------------
//  VÉRIFICATION / CRÉATION DE L'ADMINISTRATEUR INITIAL
// ---------------------------------------------------------------------------
/// Vérifie s'il existe déjà un utilisateur dans la table `users`.
/// Si la table est vide, crée l'administrateur initial.
///
/// # Arguments
/// * `pool` – Référence vers le `SqlitePool` déjà ouvert.
///
/// # Retour
/// `Ok(())` si tout s'est bien passé, sinon l'erreur `sqlx::Error`.
async fn check_initial_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 1️⃣ Compter le nombre d'utilisateurs
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    // 2️⃣ Si aucun utilisateur n'existe → créer l'admin
    if user_count.0 == 0 {
        let admin_id = "admin-initial-id-0000-0000-000000000001".to_string();
        let default_password = "changeme2026"; // À changer dès le premier login
        // Utilise ta fonction de hachage déjà définie dans le module `auth`
        let password_hash = crate::auth::hash_password(default_password);

        let _now = chrono::Utc::now();
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
        .bind("now")
        .bind("admin@nook.local")
        .bind(&password_hash)
        .bind("Administrateur Initial")
        .execute(pool)
        .await?;

        eprintln!(
            "[Init] Admin initial créé (ID: {}). Change username/password au premier login !",
            admin_id
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
//  POINT D'ENTRÉE PRINCIPAL
// ---------------------------------------------------------------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise le logger
    tracing_subscriber::fmt::init();

    
    // 1. S'assurer que le répertoire data existe
    tokio::fs::create_dir_all("/app/data").await?;
    
    // 2. S'assurer que le répertoire uploads existe
    tokio::fs::create_dir_all("/app/data/uploads").await?;
    
    // 3. Créer le fichier DB vide s'il n'existe pas
    let db_file_path = std::path::Path::new("/app/data/nook.db");
    if !db_file_path.exists() {
        eprintln!("[Info] Création du fichier de base de données...");
        tokio::fs::File::create(db_file_path).await?;
    }

    // -------------------------------------------------
    // 1️⃣ Initialisation de la base de données
    // -------------------------------------------------
    let pool = match init_db().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[Erreur] Échec de l'initialisation de la DB: {}", e);
            std::process::exit(1);
        }
    };

    // -------------------------------------------------
    // 2️⃣ Vérifier / créer l'administrateur initial
    // -------------------------------------------------
    if let Err(e) = check_initial_admin(&pool).await {
        eprintln!("[Erreur] Création de l'admin initial échouée: {}", e);
        std::process::exit(1);
    }

    // -------------------------------------------------
    // 3️⃣ Préparer le répertoire d'uploads
    // -------------------------------------------------
    let uploads_dir = PathBuf::from("/app/data/uploads");
    if !uploads_dir.exists() {
        if let Err(e) = fs::create_dir_all(&uploads_dir) {
            eprintln!("[Erreur] Échec de la création du dossier uploads: {}", e);
            std::process::exit(1);
        }
    }

    // -------------------------------------------------
    // 4️⃣ Initialiser le FileManager et le WebRTC state
    // -------------------------------------------------
    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));
    let webrtc_state = WebRtcState::new();

    // Lancer la tâche de nettoyage des fichiers expirés
    let file_manager_clone = (*file_manager).clone();
    tokio::spawn(async move {
        file_manager_clone.start_cleanup_task().await;
    });

    // ===== Nettoyage automatique au bout de 7 jours =====
    let pool_clone = pool.clone();

tokio::spawn(async move {
    // Attente au démarrage pour laisser l'app se lancer
    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

    // Prune immédiat au démarrage (utile si le container a été arrêté plusieurs jours)
    if let Err(e) = prune_old_data(&pool_clone).await {
        eprintln!("[Prune] Échec du pruning initial : {}", e);
    }

    loop {
        if let Err(e) = prune_old_data(&pool_clone).await {
            eprintln!("[Prune] Échec du pruning périodique : {}", e);
        }

        // Toutes les 24h → ajuste si tu veux plus souvent (ex: from_hours(6))
        tokio::time::sleep(tokio::time::Duration::from_hours(24)).await;
    }
});

    // -------------------------------------------------
    // 5️⃣ Construire le SharedState
    // -------------------------------------------------
    let shared_state = Arc::new(SharedState {
        db: pool.clone(),
        webrtc_state: webrtc_state.clone(),
        file_manager: file_manager.clone(),
    });

    // -------------------------------------------------
    // 6️⃣ Configurer le routeur Axum
    // -------------------------------------------------
// === ROUTEUR API (toutes tes routes existantes, déjà préfixées /api/) ===
    let api_router = Router::new()
        // Auth routes
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        // Conversation routes
        .route("/conversations", get(db::get_user_conversations))
        .route("/conversations", post(db::create_conversation))
        .route("/conversations/:id", get(db::get_conversation))
        .route("/conversations/:id/join", post(db::join_conversation))
        // Message routes
        .route("/conversations/:id/messages", get(db::get_conversation_messages))
        .route("/conversations/:id/messages", post(db::send_message))
        // WebRTC routes
        .merge(webrtc::webrtc_routes())
        // Upload routes
        .route("/upload", post(upload::upload_handler))
        .route("/upload/chat", post(upload::upload_chat_file))
        // Health‑check
        .route("/health", get(|| async { "OK" }))
        .with_state(shared_state.clone());

    // === SERVICE STATIQUE POUR LE FRONTEND (SvelteKit SPA) ===
    // Chemin où les fichiers du frontend sont copiés dans le container Docker
    // (via ton workflow : frontend-artifact → /app/static dans le Dockerfile)
    let static_path = "/app/static";

    // Gestion d'erreur personnalisée pour ServeDir (optionnel mais recommandé)
    let static_service = get_service(
        ServeDir::new(static_path)
            .append_index_html_on_directories(true)  // Gère / → /index.html
            .precompressed_gzip()                    // Support des .gz si npm run build les génère
            .precompressed_br()                      // Support brotli
            .fallback(ServeFile::new(format!("{}/index.html", static_path))), // Fallback SPA crucial pour les routes client-side
    )
    .handle_error(|error: std::io::Error| async move {
        eprintln!("[Static] Erreur service fichier : {}", error);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur fichier statique : {}", error),
        )
    });

    // === ROUTEUR FINAL ===
    let app = Router::new()
        // Toutes les API sous /api
        .nest("/api", api_router)
        // CORS sur tout (tu peux le restreindre si tu veux)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        // Fallback : tout le reste → frontend statique (SPA)
        .fallback(static_service.into_make_service());

// -------------------------------------------------
    // 7️⃣ Démarrer le serveur HTTP
    // -------------------------------------------------
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    eprintln!("[Serveur] Démarrage sur http://{}", addr);
    eprintln!("[Static] Servir les fichiers frontend depuis : {}", static_path);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
