use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::{FromRow, Pool, Sqlite};
use std::path::Path;
use std::fs;
use tokio::sync::OnceCell;

pub static DB_INIT: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

pub struct AppState {
    pub db: Pool<Sqlite>,
}

pub async fn init_db() -> AppState {
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();

    let db_path = "/app/data/nook.db";
    let db_url = format!("sqlite:{}", db_path);

    let options = SqliteConnectOptions::from_str(&db_url)
        .unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true);

    let pool = Pool::connect_with(options).await.unwrap();
    apply_migrations(&pool).await.unwrap();

    AppState { db: pool }
}

async fn apply_migrations(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    println!("Application des migrations...");
    let migrations_dir = Path::new("backend/migrations");
    if !migrations_dir.exists() {
        fs::create_dir_all(migrations_dir).unwrap();
    }

    let mut migration_files: Vec<_> = fs::read_dir(migrations_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "sql") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    migration_files.sort();

    for migration_path in migration_files {
        let migration_name = migration_path.file_name().unwrap().to_str().unwrap();
        println!("Application de la migration: {}", migration_name);
        let sql = fs::read_to_string(&migration_path).unwrap();
        for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement.trim())
                .execute(pool)
                .await?;
        }
    }
    println!("✅ Toutes les migrations ont été appliquées avec succès");
    Ok(())
}

pub fn get_pool() -> &'static Pool<Sqlite> {
    DB_INIT.get().expect("Database not initialized")
}

// Struct User - compatible avec table `users`
#[derive(FromRow, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub role: String,
    pub approved: bool,
    pub needs_password_change: bool,
    pub created_at: i64,
    pub token: Option<String>,
    pub public_key: Option<String>,
    pub joined_at: Option<i64>,
}

// Struct Upload - compatible avec table `uploads`
#[derive(FromRow, Clone)]
pub struct Upload {
    pub id: String,
    pub file_name: String,
    pub content_type: String,
    pub size: i64,
    pub path: String,
    pub timestamp: i64,
}

// Struct ChatMessage - compatible avec table `chat_messages`
#[derive(Clone)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: i64,
    pub file: Option<serde_json::Value>,
}

#[derive(Clone)]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    File,
}

impl MessageType {
    pub fn to_string(&self) -> String {
        match self {
            MessageType::Text => "text".to_string(),
            MessageType::Image => "image".to_string(),
            MessageType::Video => "video".to_string(),
            MessageType::Audio => "audio".to_string(),
            MessageType::File => "file".to_string(),
        }
    }
}

impl From<String> for MessageType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "image" => MessageType::Image,
            "video" => MessageType::Video,
            "audio" => MessageType::Audio,
            "file" => MessageType::File,
            _ => MessageType::Text,
        }
    }
}

// Fonction utilitaire pour obtenir toutes les conversations d'un utilisateur
pub async fn get_user_conversations(
    pool: &Pool<Sqlite>,
    user_id: &str,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT
            c.id,
            c.name,
            c.is_group,
            c.created_at,
            c.last_message_at,
            c.last_message_preview,
            c.unread_count,
            (SELECT COUNT(*) FROM conversation_members WHERE conversation_id = c.id) as participant_count
        FROM conversations c
        JOIN conversation_members cm ON c.id = cm.conversation_id
        WHERE cm.user_id = ?
        ORDER BY c.last_message_at DESC NULLS LAST, c.created_at DESC
    "#;

    let rows = sqlx::query_as::<_, (String, Option<String>, bool, i64, Option<i64>, Option<String>, i64, i64)>(query)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    let mut conversations: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        let (id, name, is_group, created_at, last_message_at, last_message_preview, unread_count, participant_count) = row;
        let mut conv = serde_json::Map::new();
        conv.insert("id".to_string(), serde_json::Value::String(id));
        conv.insert("name".to_string(), serde_json::Value::String(name.unwrap_or("Conversation".to_string())));
        conv.insert("is_group".to_string(), serde_json::Value::Bool(is_group));
        conv.insert("created_at".to_string(), serde_json::Value::Number(created_at.into()));
        conv.insert("last_message_at".to_string(), serde_json::Value::Number(last_message_at.unwrap_or(0).into()));
        conv.insert("last_message_preview".to_string(), serde_json::Value::String(last_message_preview.unwrap_or_default()));
        conv.insert("unread_count".to_string(), serde_json::Value::Number(unread_count.into()));
        conv.insert("participant_count".to_string(), serde_json::Value::Number(participant_count.into()));
        conversations.push(serde_json::Value::Object(conv));
    }

    Ok(conversations)
}

// Fonction utilitaire pour obtenir les participants d'une conversation
pub async fn get_conversation_participants(
    pool: &Pool<Sqlite>,
    conversation_id: &str,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT
            u.id,
            u.name,
            u.username,
            u.role,
            u.approved,
            u.public_key
        FROM users u
        JOIN conversation_members cm ON u.id = cm.user_id
        WHERE cm.conversation_id = ?
    "#;

    let rows = sqlx::query_as::<_, (String, String, String, String, bool, Option<String>)>(query)
        .bind(conversation_id)
        .fetch_all(pool)
        .await?;

    let mut participants: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        let (id, name, username, role, approved, public_key) = row;
        let mut participant = serde_json::Map::new();
        participant.insert("id".to_string(), serde_json::Value::String(id));
        participant.insert("name".to_string(), serde_json::Value::String(name));
        participant.insert("username".to_string(), serde_json::Value::String(username));
        participant.insert("role".to_string(), serde_json::Value::String(role));
        participant.insert("approved".to_string(), serde_json::Value::Bool(approved));
        participant.insert("public_key".to_string(), serde_json::Value::String(public_key.unwrap_or_default()));
        participants.push(serde_json::Value::Object(participant));
    }

    Ok(participants)
}

// Fonction utilitaire pour obtenir les messages d'une conversation
pub async fn get_conversation_messages(
    pool: &Pool<Sqlite>,
    conversation_id: &str,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT
            m.id,
            m.conversation_id,
            m.sender_id,
            u.name as sender_name,
            m.content,
            m.encrypted_keys,
            m.nonce,
            m.media_type,
            m.media_url,
            m.duration,
            m.timestamp,
            COALESCE((
                SELECT json_group_object(emoji, count)
                FROM (
                    SELECT emoji, COUNT(*) as count
                    FROM message_reactions
                    WHERE message_id = m.id
                    GROUP BY emoji
                )
            ), '{}') as reactions
        FROM chat_messages m
        JOIN users u ON m.sender_id = u.id
        WHERE m.conversation_id = ?
        ORDER BY m.timestamp ASC
    "#;

    let rows = sqlx::query_as::<_, (
        String, String, String, String, Option<String>, Option<String>, Option<String>,
        Option<String>, Option<String>, Option<i64>, i64, String
    )>(query)
        .bind(conversation_id)
        .fetch_all(pool)
        .await?;

    let mut messages: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        let (
            id, conv_id, sender_id, sender_name, content, encrypted_keys, nonce,
            media_type, media_url, duration, timestamp, reactions_json
        ) = row;

        let mut message = serde_json::Map::new();
        message.insert("id".to_string(), serde_json::Value::String(id));
        message.insert("conversation_id".to_string(), serde_json::Value::String(conv_id));
        message.insert("sender_id".to_string(), serde_json::Value::String(sender_id));
        message.insert("sender_name".to_string(), serde_json::Value::String(sender_name));
        message.insert("content".to_string(), serde_json::Value::String(content.unwrap_or_default()));
        message.insert("encrypted_keys".to_string(), serde_json::Value::String(encrypted_keys.unwrap_or_default()));
        message.insert("nonce".to_string(), serde_json::Value::String(nonce.unwrap_or_default()));
        message.insert("media_type".to_string(), serde_json::Value::String(media_type.unwrap_or_default()));
        message.insert("media_url".to_string(), serde_json::Value::String(media_url.unwrap_or_default()));
        message.insert("duration".to_string(), serde_json::Value::Number(duration.unwrap_or(0).into()));
        message.insert("timestamp".to_string(), serde_json::Value::Number(timestamp.into()));

        match serde_json::from_str::<serde_json::Value>(&reactions_json) {
            Ok(reactions) => {
                message.insert("reactions".to_string(), reactions);
            }
            Err(_) => {
                message.insert("reactions".to_string(), serde_json::Value::Object(serde_json::Map::new()));
            }
        }

        messages.push(serde_json::Value::Object(message));
    }

    Ok(messages)
}
