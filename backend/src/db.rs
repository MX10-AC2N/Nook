use sqlx::{Pool, SqlitePool};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

pub async fn init_db() -> Arc<AppState> {
    std::fs::create_dir_all("data").ok();
    let db_url = "sqlite:data/nook.db";
    let pool = SqlitePool::connect(db_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    Arc::new(AppState { db: pool })
}