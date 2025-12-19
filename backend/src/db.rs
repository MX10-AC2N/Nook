use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

pub async fn init_db() -> AppState {
    std::fs::create_dir_all("/app/data").ok();
    let db_url = "sqlite:/app/data/members.db?mode=rwc";
    let pool = sqlx::SqlitePool::connect(db_url).await.unwrap();

    // Table membres MODIFIÉE
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS members (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE,
            name TEXT NOT NULL,
            password_hash TEXT,
            public_key TEXT NOT NULL,
            approved BOOLEAN DEFAULT 0,
            has_password BOOLEAN DEFAULT 0,
            joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Table invitations
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS invites (
            token TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Table sessions utilisateurs
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            member_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(member_id) REFERENCES members(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Table administrateurs
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS admins (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Table sessions administrateurs
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS admin_sessions (
            token TEXT PRIMARY KEY,
            admin_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(admin_id) REFERENCES admins(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Créer un admin par défaut
    let admin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    if admin_count.0 == 0 {
        let admin_id = Uuid::new_v4().to_string();
        let default_password = "admin123";
        let password_hash = hash(default_password, DEFAULT_COST).unwrap();
        
        sqlx::query("INSERT INTO admins (id, username, password_hash) VALUES (?, ?, ?)")
            .bind(&admin_id)
            .bind("admin")
            .bind(&password_hash)
            .execute(&pool)
            .await
            .unwrap();
        
        println!("==================================================");
        println!("ADMIN PAR DÉFAUT CRÉÉ !");
        println!("Username: admin | Password: admin123");
        println!("==================================================");
    }

    AppState { db: pool }
}
