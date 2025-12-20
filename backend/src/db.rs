use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

pub async fn init_db() -> AppState {
    std::fs::create_dir_all("/app/data").ok();
    let db_url = "sqlite:/app/data/nook.db?mode=rwc";
    let pool = sqlx::SqlitePool::connect(db_url).await.unwrap();

    // Table utilisateurs unique (admin + membres)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            public_key TEXT,  -- Clé publique pour E2EE
            private_key_encrypted TEXT,  -- Clé privée chiffrée avec mot de passe
            role TEXT NOT NULL DEFAULT 'member',  -- 'admin' ou 'member'
            approved BOOLEAN DEFAULT 0,  -- Pour les membres, besoin d'approbation
            needs_password_change BOOLEAN DEFAULT 1,  -- Premier login = changer mdp
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            last_login TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Table sessions (pour éviter localStorage)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            public_key TEXT,  -- Clé publique temporaire de session
            encrypted_private_key TEXT,  -- Clé privée chiffrée temporaire
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Créer l'admin par défaut (doit changer son mot de passe)
    let admin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'admin'")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    if admin_count.0 == 0 {
        let admin_id = Uuid::new_v4().to_string();
        let default_password = "admin123";
        let password_hash = hash(default_password, DEFAULT_COST).unwrap();
        
        sqlx::query(
            "INSERT INTO users (id, username, name, password_hash, role, approved, needs_password_change) 
             VALUES (?, ?, ?, ?, 'admin', 1, 1)"
        )
        .bind(&admin_id)
        .bind("admin")
        .bind("Administrateur")
        .bind(&password_hash)
        .execute(&pool)
        .await
        .unwrap();
        
        println!("==================================================");
        println!("ADMIN PAR DÉFAUT CRÉÉ !");
        println!("Username: admin | Password: admin123");
        println!("Changez ce mot de passe dès la première connexion !");
        println!("==================================================");
    }

    AppState { db: pool }
}
