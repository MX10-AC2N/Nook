// main.rs - Point d'entrée du serveur Nook Backend
// Backend Axum pour gestion d'upload, WebRTC, et chiffrement

use axum::{
    routing::{get, post},
    Router,
};
use std::{fs, path::PathBuf, sync::Arc};
use sqlx::{SqlitePool, migrate};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;

// Modules
mod db;
mod auth;
mod webrtc;
mod upload;

// Import des structures partagées
use webrtc::{WebRtcState, FileManager};

// === SHARED STATE ===

#[derive(Clone)]
pub struct SharedState {
    pub db: SqlitePool,
    pub webrtc_state: WebRtcState,
    pub file_manager: Arc<FileManager>,
}

// === FONCTIONS DE DÉMARRAGE ===

async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = "nook.db";
    let pool = SqlitePool::connect(db_path).await?;
    
    // Créer les tables si elles n'existent pas
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
    "#).execute(&pool).await?;
    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
    "#).execute(&pool).await?;
    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS conversation_participants (
            conversation_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            joined_at INTEGER NOT NULL,
            PRIMARY KEY (conversation_id, user_id),
            FOREIGN KEY (conversation_id) REFERENCES conversations(id),
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
    "#).execute(&pool).await?;
    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            content TEXT,
            message_type TEXT DEFAULT 'text',
            file_id TEXT,
            created_at INTEGER NOT NULL,
            edited_at INTEGER,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id),
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (file_id) REFERENCES uploads(id)
        )
    "#).execute(&pool).await?;
    
    sqlx::query(r#"
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
    "#).execute(&pool).await?;
    
    eprintln!("[DB] Base de données initialisée");
    Ok(pool)
}

#[tokio::main]
async fn main() {
    // Initialiser le logger
    env_logger::init();
    
    // Initialiser la base de données
    let pool = match init_db().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("[Erreur] Échec de l'initialisation de la DB: {}", e);
            std::process::exit(1);
        }
    };
    
    // Créer le dossier d'upload
    let uploads_dir = PathBuf::from("uploads");
    if !uploads_dir.exists() {
        if let Err(e) = fs::create_dir_all(&uploads_dir) {
            eprintln!("[Erreur] Échec de la création du dossier uploads: {}", e);
            std::process::exit(1);
        }
    }
    
    // Initialiser le FileManager et WebRtcState
    let file_manager = Arc::new(FileManager::new(uploads_dir.clone()));
    let webrtc_state = WebRtcState::new();
    
    // Lancer la tâche de nettoyage des fichiers expirés
    let file_manager_clone = (*file_manager).clone();
    tokio::spawn(async move {
        file_manager_clone.start_cleanup_task().await;
    });
    
    // Créer le SharedState
    let shared_state = SharedState {
        db: pool.clone(),
        webrtc_state: webrtc_state.clone(),
        file_manager: file_manager.clone(),
    };
    
    // Configurer le routeur
    let app = Router::new()
        // Routes d'authentification
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/logout", post(auth::logout))
        
        // Routes de conversations
        .route("/api/conversations", get(auth::list_conversations))
        .route("/api/conversations", post(auth::create_conversation))
        .route("/api/conversations/:id", get(auth::get_conversation))
        .route("/api/conversations/:id/join", post(auth::join_conversation))
        
        // Routes de messages
        .route("/api/conversations/:id/messages", get(auth::list_messages))
        .route("/api/conversations/:id/messages", post(auth::send_message))
        
        // Routes WebRTC
        .merge(webrtc::webrtc_routes())
        
        // Routes d'upload
        .route("/api/upload", post(upload::upload_handler))
        .route("/api/upload/chat", post(upload::upload_chat_file))
        
        // Route de health check
        .route("/api/health", get(|| async { "OK" }))
        
        // Configuration CORS
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        
        // Ajouter le state partagé
        .with_state(Arc::new(shared_state));
    
    // Démarrer le serveur
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    eprintln!("[Serveur] Démarrage sur {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("[Erreur] Échec du démarrage du serveur");
}
