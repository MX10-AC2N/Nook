use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;

pub struct AppState {
    pub db: SqlitePool,
}

pub async fn init_db() -> Arc<AppState> {
    std::fs::create_dir_all("data").ok();
    let db_url = "sqlite:data/nook.db";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Could not connect to SQLite");
    sqlx::migrate!().run(&pool).await.unwrap();
    Arc::new(AppState { db: pool })
}