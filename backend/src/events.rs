// events.rs – Gestion du calendrier/événements
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router, routing::{get, post, patch, delete},
};
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::CurrentUser;
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

    let now = Utc::now().timestamp();  // i64 Unix timestamp
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
    let mut sql = "SELECT * FROM events WHERE creator_id = ?".to_string();
    let mut args: Vec<&dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + Sync> = vec![&user.id];

    if let Some(start) = query.start {
        sql.push_str(" AND start_time >= ?");
        args.push(&start);
    }
    if let Some(end) = query.end {
        sql.push_str(" AND start_time <= ?");
        args.push(&end);
    }

    sql.push_str(" ORDER BY start_time ASC");

    let mut query_builder = sqlx::query_as::<_, Event>(&sql);
    for arg in args {
        query_builder = query_builder.bind(arg);
    }

    let mut events = query_builder.fetch_all(&state.db).await.unwrap_or_default();

    // Add computed date/time fields
    for event in &mut events {
        let (date_str, time_str) = timestamp_to_date_time(event.start_time);
        event.date = date_str;
        event.time = time_str;
    }

    (StatusCode::OK, Json(ListEventsResponse { events })).into_response()
}

// Récupérer un événement par ID
pub async fn get_event(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let event = sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE id = ? AND creator_id = ?"
    )
    .bind(&id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await;

    match event {
        Ok(Some(mut e)) => {
            let (date_str, time_str) = timestamp_to_date_time(e.start_time);
            e.date = date_str;
            e.time = time_str;
            (StatusCode::OK, Json(e)).into_response()
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

// Mettre à jour un événement
pub async fn update_event(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateEventPayload>,
) -> impl IntoResponse {
    // Vérifier que l'événement existe et appartient à l'utilisateur
    let event = sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE id = ? AND creator_id = ?"
    )
    .bind(&id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await;

    let mut event = match event {
        Ok(Some(e)) => e,
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
        let date_str = payload.date.as_deref().unwrap_or(&{
            let (d, _) = timestamp_to_date_time(event.start_time);
            d
        });
        let time_str = payload.time.as_ref().and_then(|o| o.as_deref()).unwrap_or(&{
            let (_, t) = timestamp_to_date_time(event.start_time);
            t
        });

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

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut bindings: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + Sync>> = Vec::new();

    if let Some(title) = payload.title {
        updates.push("title = ?");
        bindings.push(Box::new(title));
    }
    if let Some(desc) = payload.description {
        updates.push("description = ?");
        bindings.push(Box::new(desc));
    }
    if payload.date.is_some() || payload.time.as_ref().is_some_and(|o| o.is_some()) {
        updates.push("start_time = ?");
        updates.push("end_time = ?");
        bindings.push(Box::new(new_start_time));
        bindings.push(Box::new(new_end_time));
    }
    if updates.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "No fields to update".to_string() }),
        ).into_response();
    }

    updates.push("updated_at = ?");
    let now = Utc::now().timestamp();
    bindings.push(Box::new(now));

    let sql = format!(
        "UPDATE events SET {} WHERE id = ? AND creator_id = ?",
        updates.join(", ")
    );

    let mut query_builder = sqlx::query(&sql);
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }
    query_builder = query_builder.bind(&id).bind(&user.id);

    let result = query_builder.execute(&state.db).await;

    match result {
        Ok(_) => {
            // Fetch updated event
            let updated = sqlx::query_as::<_, Event>(
                "SELECT * FROM events WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.db)
            .await;

            match updated {
                Ok(mut e) => {
                    let (date_str, time_str) = timestamp_to_date_time(e.start_time);
                    e.date = date_str;
                    e.time = time_str;
                    (StatusCode::OK, Json(e)).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error: format!("Failed to fetch updated event: {}", e) }),
                ).into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to update event: {}", e) }),
        ).into_response(),
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
        .route("/", post(create_event).get(list_events))
        .route("/{id}", get(get_event).patch(update_event).delete(delete_event))
}