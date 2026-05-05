# Template : Nouvelle Route Backend (Axum)

```rust
// backend/src/routes/new_module.rs

use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NewModel {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNewModel {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewModel {
    pub name: Option<String>,
}

// State partagé (DB pool, etc.)
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
}

// Handlers
pub async fn list_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<NewModel>>, StatusCode> {
    let items = sqlx::query_as::<_, NewModel>("SELECT * FROM table_name")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(items))
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateNewModel>,
) -> Result<Json<NewModel>, StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    
    sqlx::query("INSERT INTO table_name (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&payload.name)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let item = sqlx::query_as::<_, NewModel>("SELECT * FROM table_name WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(item))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NewModel>, StatusCode> {
    let item = sqlx::query_as::<_, NewModel>("SELECT * FROM table_name WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNewModel>,
) -> Result<Json<NewModel>, StatusCode> {
    let now = chrono::Utc::now();
    
    // Build dynamic update query
    let mut query = "UPDATE table_name SET updated_at = ?".to_string();
    let mut params: Vec<String> = vec![];
    
    if payload.name.is_some() {
        query.push_str(", name = ?");
    }
    
    query.push_str(" WHERE id = ?");
    
    // Execute update...
    // Return updated item
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM table_name WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

// Router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_items).post(create_item))
        .route("/:id", get(get_item).put(update_item).delete(delete_item))
}
```

## Migration SQL associée

```sql
-- backend/migrations/YYYYMMDDHHMMSS_add_table_name.sql

CREATE TABLE IF NOT EXISTS table_name (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_table_name_created_at ON table_name(created_at);
```
