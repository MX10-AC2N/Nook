use sqlx::{Pool, Row, Sqlite};
use std::path::Path;

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub name: Option<String>,
    pub role: Option<String>,
    pub approved: bool,
    pub needs_password_change: bool,
    pub created_at: Option<String>,
    pub token: Option<String>,
    pub public_key: Option<String>,
    pub joined_at: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

#[derive(Clone, Debug)]
pub struct Upload {
    pub id: String,
    pub file_name: String,
    pub content_type: String,
    pub size: i64,
    pub path: String,
    pub sender_id: String,
    pub timestamp: i64,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Upload {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Upload {
            id: row.try_get("id")?,
            file_name: row.try_get("file_name")?,
            content_type: row.try_get("content_type")?,
            size: row.try_get("size")?,
            path: row.try_get("path")?,
            sender_id: row.try_get("sender_id")?,
            timestamp: row.try_get("timestamp")?,
        })
    }
}

#[derive(Clone, sqlx::FromRow)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: i64,
    pub file: Option<String>,
}

#[derive(Clone)]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    File,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text => write!(f, "text"),
            MessageType::Image => write!(f, "image"),
            MessageType::Video => write!(f, "video"),
            MessageType::Audio => write!(f, "audio"),
            MessageType::File => write!(f, "file"),
        }
    }
}

pub async fn init_db() -> AppState {
    let data_dir = Path::new("/app/data");
    tokio::fs::create_dir_all(data_dir).await.ok();

    let db_url = "sqlite:/app/data/nook.db?mode=rwc";
    let pool = sqlx::SqlitePool::connect(db_url).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            name TEXT,
            role TEXT DEFAULT 'user',
            approved BOOLEAN DEFAULT 0,
            needs_password_change BOOLEAN DEFAULT 1,
            created_at TEXT,
            token TEXT,
            public_key TEXT,
            joined_at TEXT
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            name TEXT,
            created_at TEXT
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS chat_messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            sender_name TEXT NOT NULL,
            content TEXT DEFAULT '',
            message_type TEXT DEFAULT 'text',
            timestamp INTEGER DEFAULT 0,
            file TEXT,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS uploads (
            id TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            content_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            path TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            timestamp INTEGER DEFAULT 0
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS public_keys (
            user_id TEXT PRIMARY KEY,
            public_key TEXT NOT NULL,
            updated_at TEXT
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    AppState { db: pool }
}
