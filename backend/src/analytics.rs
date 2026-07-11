// backend/src/analytics.rs — Analytics endpoints for admin dashboard
// DT-06: Analytics endpoint — complete dashboard backend

use axum::{
    extract::{State, Extension, ws::{Message, WebSocket, WebSocketUpgrade}},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tokio::fs;
use tokio::time::{interval, Duration};
use crate::{SharedState, auth::CurrentUser};

#[derive(Serialize)]
pub struct DayCount {
    pub day: String,
    pub count: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/analytics/overview
// Global counters: messages, users, calls, storage
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct OverviewResponse {
    // Global counters
    pub user_count: i64,
    pub message_count: i64,
    pub conversation_count: i64,
    pub poll_count: i64,
    pub upload_count: i64,
    pub missed_call_count: i64,
    // Last 7 days
    pub active_users_7d: i64,
    pub messages_7d: i64,
    pub calls_7d: i64,
    pub messages_per_day: Vec<DayCount>,
    pub calls_per_day: Vec<DayCount>,
}

pub async fn get_overview(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<OverviewResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let now = chrono::Utc::now().timestamp();
    let cutoff_7d = now - 7 * 86400;

    // Global counters
    let (user_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE approved = 1")
            .fetch_one(&state.db).await
            .unwrap_or((0,));

    let (message_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM messages")
            .fetch_one(&state.db).await
            .unwrap_or((0,));

    let (conversation_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM conversations")
            .fetch_one(&state.db).await
            .unwrap_or((0,));

    let (poll_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM polls")
            .fetch_one(&state.db).await
            .unwrap_or((0,));

    let (upload_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM uploads")
            .fetch_one(&state.db).await
            .unwrap_or((0,));

    let (missed_call_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM missed_calls")
            .fetch_one(&state.db).await
            .unwrap_or((0,));

    // Active users 7d
    let (active_users_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT sender_id) FROM messages WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await
    .unwrap_or((0,));

    // Messages 7d
    let (messages_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await
    .unwrap_or((0,));

    // Missed calls 7d
    let (calls_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM missed_calls WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await
    .unwrap_or((0,));

    // Messages per day (7d)
    #[derive(sqlx::FromRow)]
    struct DayRow { day: String, count: i64 }

    let rows: Vec<DayRow> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM messages
         WHERE created_at > ?
         GROUP BY day
         ORDER BY day ASC",
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await
    .unwrap_or_default();

    let messages_per_day = rows.into_iter()
        .map(|r| DayCount { day: r.day, count: r.count })
        .collect();

    // Calls per day (7d)
    let call_rows: Vec<DayRow> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM missed_calls
         WHERE created_at > ?
         GROUP BY day
         ORDER BY day ASC",
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await
    .unwrap_or_default();

    let calls_per_day = call_rows.into_iter()
        .map(|r| DayCount { day: r.day, count: r.count })
        .collect();

    Ok(Json(OverviewResponse {
        user_count,
        message_count,
        conversation_count,
        poll_count,
        upload_count,
        missed_call_count,
        active_users_7d,
        messages_7d,
        calls_7d,
        messages_per_day,
        calls_per_day,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/analytics/activity
// Time series: 7d and 30d
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ActivityResponse {
    pub messages_7d: Vec<DayCount>,
    pub messages_30d: Vec<DayCount>,
    pub calls_7d: Vec<DayCount>,
    pub calls_30d: Vec<DayCount>,
    pub active_users_7d: Vec<DayCount>,
    pub active_users_30d: Vec<DayCount>,
    pub new_users_7d: Vec<DayCount>,
    pub new_users_30d: Vec<DayCount>,
    pub uploads_7d: Vec<DayCount>,
    pub uploads_30d: Vec<DayCount>,
}

pub async fn get_activity(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<ActivityResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let now = chrono::Utc::now().timestamp();
    let cutoff_7d = now - 7 * 86400;
    let cutoff_30d = now - 30 * 86400;

    #[derive(sqlx::FromRow)]
    struct DayRow { day: String, count: i64 }

    // Messages 7d
    let messages_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Messages 30d
    let messages_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Missed calls 7d
    let calls_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
    "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM missed_calls WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Missed calls 30d
    let calls_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM missed_calls WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Active users 7d (sent message)
    let active_users_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(DISTINCT sender_id) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Active users 30d
    let active_users_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(DISTINCT sender_id) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // New users 7d
    let new_users_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM users WHERE approved = 1 AND created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // New users 30d
    let new_users_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM users WHERE approved = 1 AND created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Uploads 7d
    let uploads_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(uploaded_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM uploads WHERE uploaded_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Uploads 30d
    let uploads_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(uploaded_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM uploads WHERE uploaded_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await
    .unwrap_or_default()
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    Ok(Json(ActivityResponse {
        messages_7d,
        messages_30d,
        calls_7d,
        calls_30d,
        active_users_7d,
        active_users_30d,
        new_users_7d,
        new_users_30d,
        uploads_7d,
        uploads_30d,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/analytics/storage
// Storage: uploads, GIFs, DB size
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ContentTypeStat {
    pub content_type: String,
    pub count: i64,
    pub total_size_bytes: i64,
}

#[derive(Serialize)]
pub struct StorageResponse {
    pub db_size_bytes: i64,
    pub uploads_size_bytes: i64,
    pub uploads_count: i64,
    pub gifs_size_bytes: i64,
    pub gifs_count: i64,
    pub total_size_bytes: i64,
    pub uploads_by_type: Vec<ContentTypeStat>,
}

pub async fn get_storage(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<StorageResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    // DB size
    let db_size_bytes = get_db_size(&state.db).await;

    // Uploads directory size
    let (uploads_size_bytes, uploads_count) = calculate_dir_size(&state.config.uploads_dir).await;

    // GIFs directory size
    let (gifs_size_bytes, gifs_count) = calculate_dir_size(&state.config.gifs_dir).await;

    // Uploads by content type from DB
    #[derive(sqlx::FromRow)]
    struct TypeStat { content_type: String, count: i64, total_size: i64 }

    let type_stats: Vec<TypeStat> = sqlx::query_as::<_, TypeStat>(
        "SELECT COALESCE(content_type, 'unknown') AS content_type, COUNT(*) AS count, SUM(file_size) AS total_size
         FROM uploads
         GROUP BY content_type
         ORDER BY total_size DESC"
    )
    .fetch_all(&state.db).await
    .unwrap_or_default();

    let uploads_by_type = type_stats.into_iter()
        .map(|t| ContentTypeStat {
            content_type: t.content_type,
            count: t.count,
            total_size_bytes: t.total_size,
        })
        .collect();

    let total_size_bytes = db_size_bytes + uploads_size_bytes + gifs_size_bytes;

    Ok(Json(StorageResponse {
        db_size_bytes,
        uploads_size_bytes,
        uploads_count,
        gifs_size_bytes,
        gifs_count,
        total_size_bytes,
        uploads_by_type,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/analytics/ws
// WebSocket for real-time analytics updates
// ─────────────────────────────────────────────────────────────────────────────

async fn analytics_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    if user.role != "admin" {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }

    ws.on_upgrade(move |socket| analytics_ws(socket, state))
}

async fn analytics_ws(mut socket: WebSocket, state: Arc<SharedState>) {
    let mut interval = interval(Duration::from_secs(30)); // Send updates every 30 seconds

    loop {
        tokio::select! {
            // Send periodic analytics updates
            _ = interval.tick() => {
                if let Ok(overview) = fetch_overview_data(&state).await {
                    let msg = serde_json::to_string(&serde_json::json!({
                        "type": "overview_update",
                        "data": overview
                    })).unwrap_or_default();
                    if socket.send(Message::Text(msg.into())).await.is_err() {
                        break; // Client disconnected
                    }
                }

                if let Ok(activity) = fetch_activity_data(&state).await {
                    let msg = serde_json::to_string(&serde_json::json!({
                        "type": "activity_update",
                        "data": activity
                    })).unwrap_or_default();
                    if socket.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }

                if let Ok(storage) = fetch_storage_data(&state).await {
                    let msg = serde_json::to_string(&serde_json::json!({
                        "type": "storage_update",
                        "data": storage
                    })).unwrap_or_default();
                    if socket.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }
            // Handle incoming messages (ping/pong, close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Client responded to ping
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Handle client messages if needed (e.g., subscription filters)
                        if text.trim() == "ping" {
                            if socket.send(Message::Text("pong".into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        tracing::debug!(error = %e, "WebSocket error");
                        break;
                    }
                    None => break, // Stream ended
                    _ => {}
                }
            }
        }
    }

    tracing::debug!("Analytics WebSocket disconnected");
}

async fn fetch_overview_data(state: &Arc<SharedState>) -> Result<OverviewResponse, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let cutoff_7d = now - 7 * 86400;

    // Global counters
    let (user_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE approved = 1")
        .fetch_one(&state.db).await?;

    let (message_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
        .fetch_one(&state.db).await?;

    let (conversation_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM conversations")
        .fetch_one(&state.db).await?;

    let (poll_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM polls")
        .fetch_one(&state.db).await?;

    let (upload_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM uploads")
        .fetch_one(&state.db).await?;

    let (missed_call_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM missed_calls")
        .fetch_one(&state.db).await?;

    // Active users 7d
    let (active_users_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT sender_id) FROM messages WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await?;

    // Messages 7d
    let (messages_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await?;

    // Missed calls 7d
    let (calls_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM missed_calls WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await?;

    // Messages per day (7d)
    #[derive(sqlx::FromRow)]
    struct DayRow { day: String, count: i64 }

    let rows: Vec<DayRow> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM messages
         WHERE created_at > ?
         GROUP BY day
         ORDER BY day ASC",
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await?;

    let messages_per_day = rows.into_iter()
        .map(|r| DayCount { day: r.day, count: r.count })
        .collect();

    // Calls per day (7d)
    let call_rows: Vec<DayRow> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM missed_calls
         WHERE created_at > ?
         GROUP BY day
         ORDER BY day ASC",
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await?;

    let calls_per_day = call_rows.into_iter()
        .map(|r| DayCount { day: r.day, count: r.count })
        .collect();

    Ok(OverviewResponse {
        user_count,
        message_count,
        conversation_count,
        poll_count,
        upload_count,
        missed_call_count,
        active_users_7d,
        messages_7d,
        calls_7d,
        messages_per_day,
        calls_per_day,
    })
}

async fn fetch_activity_data(state: &Arc<SharedState>) -> Result<ActivityResponse, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let cutoff_7d = now - 7 * 86400;
    let cutoff_30d = now - 30 * 86400;

    #[derive(sqlx::FromRow)]
    struct DayRow { day: String, count: i64 }

    // Messages 7d
    let messages_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Messages 30d
    let messages_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Missed calls 7d
    let calls_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM missed_calls WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Missed calls 30d
    let calls_30d: Vec<DayCount>
= sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM missed_calls WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Active users 7d (sent message)
    let active_users_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(DISTINCT sender_id) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Active users 30d
    let active_users_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(DISTINCT sender_id) AS count
         FROM messages WHERE created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // New users 7d
    let new_users_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM users WHERE approved = 1 AND created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // New users 30d
    let new_users_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(created_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM users WHERE approved = 1 AND created_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Uploads 7d
    let uploads_7d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(uploaded_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM uploads WHERE uploaded_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_7d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    // Uploads 30d
    let uploads_30d: Vec<DayCount> = sqlx::query_as::<_, DayRow>(
        "SELECT date(uploaded_at, 'unixepoch') AS day, COUNT(*) AS count
         FROM uploads WHERE uploaded_at > ? GROUP BY day ORDER BY day ASC"
    )
    .bind(cutoff_30d)
    .fetch_all(&state.db).await?
    .into_iter().map(|r| DayCount { day: r.day, count: r.count }).collect();

    Ok(ActivityResponse {
        messages_7d,
        messages_30d,
        calls_7d,
        calls_30d,
        active_users_7d,
        active_users_30d,
        new_users_7d,
        new_users_30d,
        uploads_7d,
        uploads_30d,
    })
}

async fn fetch_storage_data(state: &Arc<SharedState>) -> Result<StorageResponse, sqlx::Error> {
    // DB size
    let db_size_bytes = get_db_size(&state.db).await;

    // Uploads directory size
    let (uploads_size_bytes, uploads_count) = calculate_dir_size(&state.config.uploads_dir).await;

    // GIFs directory size
    let (gifs_size_bytes, gifs_count) = calculate_dir_size(&state.config.gifs_dir).await;

    // Uploads by content type from DB
    #[derive(sqlx::FromRow)]
    struct TypeStat { content_type: String, count: i64, total_size: i64 }

    let type_stats: Vec<TypeStat> = sqlx::query_as::<_, TypeStat>(
        "SELECT COALESCE(content_type, 'unknown') AS content_type, COUNT(*) AS count, SUM(file_size) AS total_size
         FROM uploads
         GROUP BY content_type
         ORDER BY total_size DESC"
    )
    .fetch_all(&state.db).await
    .unwrap_or_default();

    let uploads_by_type = type_stats.into_iter()
        .map(|t| ContentTypeStat {
            content_type: t.content_type,
            count: t.count,
            total_size_bytes: t.total_size,
        })
        .collect();

    let total_size_bytes = db_size_bytes + uploads_size_bytes + gifs_size_bytes;

    Ok(StorageResponse {
        db_size_bytes,
        uploads_size_bytes,
        uploads_count,
        gifs_size_bytes,
        gifs_count,
        total_size_bytes,
        uploads_by_type,
    })
}

async fn get_db_size(db: &sqlx::SqlitePool) -> i64 {
    // SQLite PRAGMA page_count * page_size gives the DB size
    let row: Option<(i64,)> = sqlx::query_as("PRAGMA page_count")
        .fetch_optional(db).await.ok().flatten();
    let page_count = row.map(|(c,)| c).unwrap_or(0);

    let row: Option<(i64,)> = sqlx::query_as("PRAGMA page_size")
        .fetch_optional(db).await.ok().flatten();
    let page_size = row.map(|(s,)| s).unwrap_or(4096);

    page_count * page_size
}

async fn calculate_dir_size(path: &str) -> (i64, i64) {
    use tokio::fs;
    let mut total_size = 0i64;
    let mut count = 0i64;

    if let Ok(mut entries) = fs::read_dir(path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    total_size += metadata.len() as i64;
                    count += 1;
                } else if metadata.is_dir() {
                    let (sub_size, sub_count) = Box::pin(calculate_dir_size(&entry.path().to_string_lossy())).await;
                    total_size += sub_size;
                    count += sub_count;
                }
            }
        }
    }

    (total_size, count)
}

// ─────────────────────────────────────────────────────────────────────────────
// Routes
// ─────────────────────────────────────────────────────────────────────────────

pub fn analytics_routes() -> axum::Router<Arc<SharedState>> {
    use axum::routing::get;

    axum::Router::new()
        .route("/overview", get(get_overview))
        .route("/activity", get(get_activity))
        .route("/storage", get(get_storage))
        .route("/ws", get(analytics_ws_handler))
        .layer(axum::middleware::from_fn(crate::auth::require_admin))
}
