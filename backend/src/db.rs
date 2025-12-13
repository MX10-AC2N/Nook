use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

pub async fn init_db() -> AppState {
    std::fs::create_dir_all("data").ok();
    let db_url = "sqlite:data/members.db?mode=rwc";
    let pool = sqlx::SqlitePool::connect(db_url).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS members (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            public_key TEXT NOT NULL,
            approved BOOLEAN DEFAULT 0,
            joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS invites (
            token TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&pool).await.unwrap();

    AppState { db: pool }
}
