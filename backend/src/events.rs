// events.rs – Gestion du calendrier/événements
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router, routing::{get, post},
};
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::SharedState;

// Structure d'un événement (API publique avec champs calculés)
// Note: Utilise i64 (Unix timestamps) pour compatibilité sqlx, comme le reste de Nook
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub date: String,      // YYYY-MM-DD (dérivé de start_time)
    pub time: String,      // HH:MM (dérivé de start_time)
}

/// Ligne brute de la DB — correspond exactement au schéma SQL events
#[derive(sqlx::FromRow)]
struct EventRow {
    pub id: String,
    pub creator_id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<EventRow> for Event {
    fn from(row: EventRow) -> Self {
        let (date_str, time_str) = timestamp_to_date_time(row.start_time);
        Self {
            id: row.id,
            creator_id: row.creator_id,
            title: row.title,
            description: row.description,
            start_time: row.start_time,
            end_time: row.end_time,
            created_at: row.created_at,
            updated_at: row.updated_at,
            date: date_str,
            time: time_str,
        }
    }
}

fn timestamp_to_date_time(ts: i64) -> (String, String) {
    let dt = Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now);
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
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Serialize)]
pub struct ListEventsResponse {
    pub events: Vec<Event>,
}

// Créer un événement
pub async fn create_event(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(payload): Json<CreateEventPayload>,
) -> impl IntoResponse {
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

    let now = Utc::now().timestamp();
    let event_id = Uuid::new_v4().to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO events (id, creator_id, title, description, start_time, end_time, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&event_id)
    .bind(&user.id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(start_time)
    .bind(end_time)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let (date_str, time_str) = timestamp_to_date_time(start_time);
            let event = Event {
                id: event_id.clone(),
                creator_id: user.id.clone(),
                title: payload.title,
                description: payload.description,
                start_time,
                end_time,
                created_at: now,
                updated_at: now,
                date: date_str,
                time: time_str,
            };
            (StatusCode::CREATED, Json(event)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to create event: {}", e) }),
        ).into_response(),
    }
}

// Lister les événements
pub async fn list_events(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Query(query): Query<ListEventsQuery>,
) -> impl IntoResponse {
    let events = match (query.start, query.end) {
        (Some(start), Some(end)) => {
            sqlx::query_as::<_, EventRow>(
                "SELECT * FROM events WHERE creator_id = ? AND start_time >= ? AND start_time <= ? ORDER BY start_time ASC"
            )
            .bind(&user.id)
            .bind(start)
            .bind(end)
            .fetch_all(&state.db)
            .await
        }
        (Some(start), None) => {
            sqlx::query_as::<_, EventRow>(
                "SELECT * FROM events WHERE creator_id = ? AND start_time >= ? ORDER BY start_time ASC"
            )
            .bind(&user.id)
            .bind(start)
            .fetch_all(&state.db)
            .await
        }
        (None, Some(end)) => {
            sqlx::query_as::<_, EventRow>(
                "SELECT * FROM events WHERE creator_id = ? AND start_time <= ? ORDER BY start_time ASC"
            )
            .bind(&user.id)
            .bind(end)
            .fetch_all(&state.db)
            .await
        }
        (None, None) => {
            sqlx::query_as::<_, EventRow>(
                "SELECT * FROM events WHERE creator_id = ? ORDER BY start_time ASC"
            )
            .bind(&user.id)
            .fetch_all(&state.db)
            .await
        }
    };

    let events: Vec<Event> = events.unwrap_or_default().into_iter().map(Into::into).collect();

    (StatusCode::OK, Json(ListEventsResponse { events })).into_response()
}

// Récupérer un événement par ID
pub async fn get_event(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let event = sqlx::query_as::<_, EventRow>(
        "SELECT * FROM events WHERE id = ? AND creator_id = ?"
    )
    .bind(&id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await;

    match event {
        Ok(Some(row)) => {
            let event: Event = row.into();
            (StatusCode::OK, Json(event)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "Event not found".to_string() }),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to get event: {}", e) }),
        ).into_response(),
    }
}

// Résultat interne pour update_event
enum UpdateResult {
    Updated(Event),
    Error(String),
}

// Mettre à jour un événement
pub async fn update_event(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateEventPayload>,
) -> impl IntoResponse {
    // Vérifier que l'événement existe et appartient à l'utilisateur
    let event = sqlx::query_as::<_, EventRow>(
        "SELECT * FROM events WHERE id = ? AND creator_id = ?"
    )
    .bind(&id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await;

    let event = match event {
        Ok(Some(row)) => row.into(),
        Ok(None) => return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "Event not found".to_string() }),
        ).into_response(),
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to get event: {}", e) }),
        ).into_response(),
    };

    // Compute new start_time if date/time provided
    let new_start_time = if payload.date.is_some() || payload.time.as_ref().is_some_and(|o| o.is_some()) {
        let default_date_time = timestamp_to_date_time(event.start_time);
        let date_str = payload.date.as_deref().unwrap_or(&default_date_time.0);
        let time_str = payload.time.as_ref().and_then(|o| o.as_deref()).unwrap_or(&default_date_time.1);

        let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: "Invalid date format (expected YYYY-MM-DD)".to_string() }),
            ).into_response(),
        };

        let time = match NaiveTime::parse_from_str(time_str, "%H:%M") {
            Ok(t) => t,
            Err(_) => return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: "Invalid time format (expected HH:MM)".to_string() }),
            ).into_response(),
        };

        let start_dt = Utc.from_utc_datetime(&date.and_time(time));
        start_dt.timestamp()
    } else {
        event.start_time
    };

    let new_end_time = new_start_time + (event.end_time - event.start_time);

    // Extract values from payload BEFORE binding (to avoid partial moves)
    let title_val = payload.title.as_deref();
    let desc_val = payload.description.as_ref().and_then(|opt| opt.as_deref());
    let has_date_time = payload.date.is_some() || payload.time.as_ref().is_some_and(|o| o.is_some());
    let now = Utc::now().timestamp();

    // Build update query based on what fields are provided
    let update_result = if let (Some(title), Some(desc)) = (title_val, desc_val) {
        if has_date_time {
            // All fields
            let result = sqlx::query(
                "UPDATE events SET title = ?, description = ?, start_time = ?, end_time = ?, updated_at = ? WHERE id = ? AND creator_id = ?"
            )
            .bind(title)
            .bind(desc)
            .bind(new_start_time)
            .bind(new_end_time)
            .bind(now)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await;
            handle_update_result(result, state.clone(), id).await
        } else {
            // title + description
            let result = sqlx::query(
                "UPDATE events SET title = ?, description = ?, updated_at = ? WHERE id = ? AND creator_id = ?"
            )
            .bind(title)
            .bind(desc)
            .bind(now)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await;
            handle_update_result(result, state.clone(), id).await
        }
    } else if let Some(title) = title_val {
        if has_date_time {
            // title + date/time
            let result = sqlx::query(
                "UPDATE events SET title = ?, start_time = ?, end_time = ?, updated_at = ? WHERE id = ? AND creator_id = ?"
            )
            .bind(title)
            .bind(new_start_time)
            .bind(new_end_time)
            .bind(now)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await;
            handle_update_result(result, state.clone(), id).await
        } else {
            // title only
            let result = sqlx::query(
                "UPDATE events SET title = ?, updated_at = ? WHERE id = ? AND creator_id = ?"
            )
            .bind(title)
            .bind(now)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await;
            handle_update_result(result, state.clone(), id).await
        }
    } else if let Some(desc) = desc_val {
        if has_date_time {
            // description + date/time
            let result = sqlx::query(
                "UPDATE events SET description = ?, start_time = ?, end_time = ?, updated_at = ? WHERE id = ? AND creator_id = ?"
            )
            .bind(desc)
            .bind(new_start_time)
            .bind(new_end_time)
            .bind(now)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await;
            handle_update_result(result, state.clone(), id).await
        } else {
            // description only
            let result = sqlx::query(
                "UPDATE events SET description = ?, updated_at = ? WHERE id = ? AND creator_id = ?"
            )
            .bind(desc)
            .bind(now)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await;
            handle_update_result(result, state.clone(), id).await
        }
    } else if has_date_time {
        // date/time only
        let result = sqlx::query(
            "UPDATE events SET start_time = ?, end_time = ?, updated_at = ? WHERE id = ? AND creator_id = ?"
        )
        .bind(new_start_time)
        .bind(new_end_time)
        .bind(now)
        .bind(&id)
        .bind(&user.id)
        .execute(&state.db)
        .await;
        handle_update_result(result, state.clone(), id).await
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "No fields to update".to_string() }),
        ).into_response()
    };

    match update_result {
        UpdateResult::Updated(event) => (StatusCode::OK, Json(event)).into_response(),
        UpdateResult::Error(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e })).into_response(),
    }
}

async fn handle_update_result(
    result: Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>,
    state: Arc<SharedState>,
    id: String,
) -> UpdateResult {
    match result {
        Ok(_) => {
            // Fetch updated event
            let updated = sqlx::query_as::<_, EventRow>(
                "SELECT * FROM events WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.db)
            .await;

            match updated {
                Ok(row) => UpdateResult::Updated(row.into()),
                Err(e) => UpdateResult::Error(format!("Failed to fetch updated event: {}", e)),
            }
        }
        Err(e) => UpdateResult::Error(format!("Failed to update event: {}", e)),
    }
}

// Supprimer un événement
pub async fn delete_event(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM events WHERE id = ? AND creator_id = ?")
        .bind(&id)
        .bind(&user.id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse { error: "Event not found".to_string() }),
                ).into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to delete event: {}", e) }),
        ).into_response(),
    }
}

// Router pour les événements
pub fn events_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/events", post(create_event).get(list_events))
        .route("/events/{id}", get(get_event).patch(update_event).delete(delete_event))
}