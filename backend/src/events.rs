// events.rs – Gestion du calendrier/événements
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json, Router, routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;
use rand::Rng;

use crate::SharedState;

// Structure d'un événement
// Note: Utilise i64 (Unix timestamps) pour compatibilité sqlx, comme le reste de Nook
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: String,
    pub creator_id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: i64,  // Unix timestamp (seconds since epoch)
    pub end_time: i64,
    pub created_at: i64,
    pub updated_at: i64,
    // Champs calculés pour le frontend (non stockés en DB)
    #[serde(skip_deserializing)]
    pub date: String,      // YYYY-MM-DD (dérivé de start_time)
    #[serde(skip_deserializing)]
    pub time: String,      // HH:MM (dérivé de start_time)
}

fn timestamp_to_date_time(ts: i64) -> (String, String) {
    use chrono::{TimeZone, Utc};
    let dt = Utc.timestamp_opt(ts, 0).single().unwrap_or_else(|| Utc::now());
    let date = dt.format("%Y-%m-%d").to_string();
    let time = dt.format("%H:%M").to_string();
    (date, time)
}

// Payload de création
#[derive(Debug, Deserialize)]
pub struct CreateEventPayload {
    pub title: String,
    pub description: Option<String>,
    pub date: String,      // YYYY-MM-DD
    pub time: Option<String>, // HH:MM (optional)
}

// Payload de mise à jour
#[derive(Debug, Deserialize)]
pub struct UpdateEventPayload {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub date: Option<String>,
    pub time: Option<Option<String>>,
}

// Paramètres de requête pour lister les événements
#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

// Réponse d'erreur standard
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// Créer un événement
pub async fn create_event(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<CreateEventPayload>,
) -> impl IntoResponse {
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};

    let event_id: String = std::iter::repeat_with(|| rand::rng().sample(rand::distr::Alphanumeric) as char)
        .take(12)
        .collect();

    // Parse date (YYYY-MM-DD)
    let date = match NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid date format (expected YYYY-MM-DD)".to_string() })
        ).into_response(),
    };
    
    // Parse optional time (HH:MM)
    let time = payload.time
        .as_deref()
        .and_then(|t| NaiveTime::parse_from_str(t, "%H:%M").ok())
        .unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    
    let start_dt = Utc.from_utc_datetime(&date.and_time(time));
    let start_time = start_dt.timestamp();
    let end_time = start_time + 3600; // Default 1 hour duration

    let now = Utc::now().timestamp();  // i64 Unix timestamp

    let result = sqlx::query(
        r#"
        INSERT INTO events (id, creator_id, title, description, start_time, end_time, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&event_id)
    .bind(&user_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(start_time)
    .bind(end_time)
    .bind(now)
    .bind(now)
    .execute(&*pool)
    .await;

    match result {
        Ok(_) => {
            let mut event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ?")
                .bind(&event_id)
                .fetch_one(&*pool)
                .await
                .unwrap();
            // Add computed date/time fields for frontend
            let (date_str, time_str) = timestamp_to_date_time(event.start_time);
            event.date = date_str;
            event.time = time_str;
            (StatusCode::CREATED, Json(event)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create event: {}", e),
            }),
        ).into_response(),
    }
}

// Lister les événements (filtrés par plage de dates)
pub async fn list_events(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Query(query): Query<ListEventsQuery>,
) -> impl IntoResponse {
    let mut sql = "SELECT * FROM events WHERE 1=1".to_string();
    let mut bindings = vec![];

    if let Some(start) = query.start {
        sql.push_str(" AND end_time >= ?");
        bindings.push(start);
    }
    if let Some(end) = query.end {
        sql.push_str(" AND start_time <= ?");
        bindings.push(end);
    }
    sql.push_str(" ORDER BY start_time ASC");

    // Bind parameters dynamiquement
    let mut query_builder = sqlx::query_as::<_, Event>(&sql);
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }

    let mut events = query_builder
        .fetch_all(&*pool)
        .await;

    match events {
        Ok(ref mut events) => {
            // Add computed date/time fields for frontend
            for event in events.iter_mut() {
                let (date_str, time_str) = timestamp_to_date_time(event.start_time);
                event.date = date_str;
                event.time = time_str;
            }
            (StatusCode::OK, Json(events.clone())).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to list events: {}", e),
            }),
        ).into_response(),
    }
}

// Obtenir un événement par ID
pub async fn get_event(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Path(event_id): Path<String>,
) -> impl IntoResponse {
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ?")
        .bind(&event_id)
        .fetch_optional(&*pool)
        .await;

    match event {
        Ok(Some(mut event)) => {
            let (date_str, time_str) = timestamp_to_date_time(event.start_time);
            event.date = date_str;
            event.time = time_str;
            (StatusCode::OK, Json(event)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Event not found".to_string(),
            }),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get event: {}", e),
            }),
        ).into_response(),
    }
}

// Mettre à jour un événement
pub async fn update_event(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Extension(user_id): Extension<String>,
    Path(event_id): Path<String>,
    Json(payload): Json<UpdateEventPayload>,
) -> impl IntoResponse {
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};

    // Vérifier que l'utilisateur est le créateur
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ?")
        .bind(&event_id)
        .fetch_optional(&*pool)
        .await;

    let event = match event {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Event not found".to_string(),
                }),
            ).into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get event: {}", e),
                }),
            ).into_response()
        }
    };

    if event.creator_id != user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You can only update your own events".to_string(),
            }),
        ).into_response();
    }

    let now = Utc::now().timestamp();  // i64 Unix timestamp

    // Parse date/time if provided
    let mut new_start_time = event.start_time;
    let mut new_end_time = event.end_time;

    if let Some(date_str) = payload.date {
        if let Some(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok() {
            let time = payload.time
                .as_deref()
                .and_then(|t| NaiveTime::parse_from_str(t, "%H:%M").ok())
                .unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            let start_dt = Utc.from_utc_datetime(&date.and_time(time));
            new_start_time = start_dt.timestamp();
            // Keep same duration
            let duration = event.end_time - event.start_time;
            new_end_time = new_start_time + duration;
        }
    }

    let result = sqlx::query(
        r#"
        UPDATE events
        SET title = COALESCE(?, title),
            description = COALESCE(?, description),
            start_time = ?,
            end_time = ?,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(new_start_time)
    .bind(new_end_time)
    .bind(now)
    .bind(&event_id)
    .execute(&*pool)
    .await;

    match result {
        Ok(_) => {
            let mut updated = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ?")
                .bind(&event_id)
                .fetch_one(&*pool)
                .await
                .unwrap();
            let (date_str, time_str) = timestamp_to_date_time(updated.start_time);
            updated.date = date_str;
            updated.time = time_str;
            (StatusCode::OK, Json(updated)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to update event: {}", e),
            }),
        ).into_response(),
    }
}

// Supprimer un événement
pub async fn delete_event(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Extension(user_id): Extension<String>,
    Path(event_id): Path<String>,
) -> impl IntoResponse {
    // Vérifier que l'utilisateur est le créateur
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ?")
        .bind(&event_id)
        .fetch_optional(&*pool)
        .await;

    let event = match event {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Event not found".to_string(),
                }),
            ).into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get event: {}", e),
                }),
            ).into_response()
        }
    };

    if event.creator_id != user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You can only delete your own events".to_string(),
            }),
        ).into_response();
    }

    let result = sqlx::query("DELETE FROM events WHERE id = ?")
        .bind(&event_id)
        .execute(&*pool)
        .await;

    match result {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete event: {}", e),
            }),
        ).into_response(),
    }
}

// Router pour les événements
pub fn events_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/", post(create_event).get(list_events))
        .route("/{id}", get(get_event).patch(update_event).delete(delete_event))
}
