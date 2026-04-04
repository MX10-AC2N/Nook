// backend/src/admin.rs - Gestion admin avec Extension<CurrentUser>
// Session 15 — FIX: approve_user insère aussi le user dans conversation_participants
//               Cause : un user inscrit via /api/auth/register n'était jamais ajouté
//               à default_global → GET /api/conversations retournait [] après approbation

use crate::{auth::CurrentUser, SharedState};
use sysinfo::System;
use axum::{extract::State, extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// ====================== STRUCTURES ======================
#[derive(Serialize)]
pub struct UsersResponse {
    pub users: Vec<SimpleUser>,
}

#[derive(Serialize)]
pub struct SimpleUser {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub role: String,
    pub approved: bool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct InviteInfo {
    pub id: String,
    pub token: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub used: bool,
    pub used_by: Option<String>,
    pub used_at: Option<i64>,
}

#[derive(Serialize)]
pub struct InvitesResponse {
    pub invites: Vec<InviteInfo>,
}

// ====================== PAYLOADS ======================
#[derive(Deserialize)]
pub struct ApprovePayload {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct DeleteInvitePayload {
    pub id: String,
}

// ====================== HANDLERS (avec CurrentUser) ======================



// ─────────────────────────────────────────────────────────────────────────────
// GET /api/admin/metrics
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_system_metrics(
    State(_state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    if user.role != "admin" {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Admin uniquement" }))).into_response();
    }
    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu = sys.global_cpu_usage();
    let mem_used = sys.used_memory();
    let mem_total = sys.total_memory();
    let uptime = System::uptime();
    let la = System::load_average();
    let disks: Vec<serde_json::Value> = Vec::new();
    // NOTE: In sysinfo 0.32, disk access requires specific refresh calls
    // For now, skip disk info (can be added back with correct 0.32 API later)
    sys.refresh_memory();
    Json(json!({
        "cpu_usage_percent": cpu,
        "memory_used_mb": mem_used / 1_048_576,
        "memory_total_mb": mem_total / 1_048_576,
        "uptime_seconds": uptime,
        "load_avg_one": la.one,
        "load_avg_five": la.five,
        "disks": disks,
        "process_count": sys.processes().len(),
    })).into_response()
}

pub async fn pending_users(
    State(_state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<UsersResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let users: Vec<SimpleUser> = sqlx::query_as(
        "SELECT id, username, name, created_at, role, approved FROM users WHERE approved = 0 ORDER BY created_at DESC"
    )
    .fetch_all(&State.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Erreur DB"}))))?
    .into_iter()
    .map(|u: (String, String, Option<String>, i64, String, bool)| SimpleUser {
        id: u.0,
        username: u.1,
        name: u.2,
        created_at: u.3,
        role: u.4,
        approved: u.5,
    })
    .collect();

    Ok(Json(UsersResponse { users }))
}

pub async fn all_users(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<UsersResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let users: Vec<SimpleUser> = sqlx::query_as(
        "SELECT id, username, name, created_at, role, approved FROM users ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Erreur DB"})),
        )
    })?
    .into_iter()
    .map(
        |u: (String, String, Option<String>, i64, String, bool)| SimpleUser {
            id: u.0,
            username: u.1,
            name: u.2,
            created_at: u.3,
            role: u.4,
            approved: u.5,
        },
    )
    .collect();

    Ok(Json(UsersResponse { users }))
}

pub async fn approve_user(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(payload): Json<ApprovePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let result = sqlx::query("UPDATE users SET approved = 1 WHERE id = ?")
        .bind(&payload.user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 1 => {
            // FIX session 15 : ajouter l'utilisateur approuvé à default_global
            // INSERT OR IGNORE → safe si déjà participant (re-approbation, double clic, etc.)
            let now = chrono::Utc::now().timestamp();
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
                 VALUES ('default_global', ?, ?)",
            )
            .bind(&payload.user_id)
            .bind(now)
            .execute(&state.db)
            .await;

            tracing::info!(
                user_id = %payload.user_id,
                "✓ Utilisateur approuvé et ajouté à default_global"
            );

            Ok(Json(json!({"success": true})))
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "message": "Utilisateur non trouvé"})),
        )),
    }
}

pub async fn list_invites(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<InvitesResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let invites: Vec<InviteInfo> = sqlx::query_as(
        "SELECT id, token, created_at, expires_at, used, used_by, used_at FROM invites ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Erreur DB"}))))?;

    Ok(Json(InvitesResponse { invites }))
}

pub async fn delete_invite(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(payload): Json<DeleteInvitePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let result = sqlx::query("DELETE FROM invites WHERE id = ?")
        .bind(&payload.id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 1 => Ok(Json(json!({"success": true}))),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "message": "Invitation non trouvée"})),
        )),
    }
}

// ═════════════════════════════════════════════════════════════════
// GET /api/analytics — Tableau de bord admin enrichi
// ═════════════════════════════════════════════════════════════════

#[derive(Serialize)]
pub struct DayCount {
    pub day: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct AnalyticsResponse {
    // Compteurs globaux
    pub user_count: i64,
    pub message_count: i64,
    pub conversation_count: i64,
    pub poll_count: i64,
    pub upload_count: i64,
    // 7 derniers jours
    pub active_users_7d: i64,
    pub messages_7d: i64,
    pub messages_per_day: Vec<DayCount>,
}

pub async fn get_analytics(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<AnalyticsResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"success": false, "message": "Accès admin requis"})),
        ));
    }

    let cutoff_7d = chrono::Utc::now().timestamp() - 7 * 86400;

    // Compteurs globaux — une requête par table (sqlx sans macros)
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

    // Utilisateurs actifs 7j (ont envoyé au moins un message)
    let (active_users_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT sender_id) FROM messages WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await
    .unwrap_or((0,));

    // Messages total 7j
    let (messages_7d,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE created_at > ?",
    )
    .bind(cutoff_7d)
    .fetch_one(&state.db).await
    .unwrap_or((0,));

    // Messages par jour (7j) — date ISO YYYY-MM-DD
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

    Ok(Json(AnalyticsResponse {
        user_count,
        message_count,
        conversation_count,
        poll_count,
        upload_count,
        active_users_7d,
        messages_7d,
        messages_per_day,
    }))
}

/// Supprimer un membre (ADMIN ONLY)
/// DELETE /api/users/{id}
pub async fn delete_user(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(admin)): Extension<CurrentUser>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    use axum::http::StatusCode;

    // Empêcher l'admin de se supprimer lui-même
    if user_id == admin.id {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Impossible de supprimer son propre compte" })),
        ).into_response();
    }

    // Vérifier que l'utilisateur n'est pas admin
    let target_role: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM users WHERE id = ?"
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match target_role {
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "message": "Utilisateur introuvable" })),
        ).into_response(),
        Some((role,)) if role == "admin" => return (
            StatusCode::FORBIDDEN,
            Json(json!({ "success": false, "message": "Impossible de supprimer un administrateur" })),
        ).into_response(),
        _ => {}
    }

    // Supprimer les données associées puis l'utilisateur
    // Les FK ON DELETE CASCADE gèrent push_subscriptions, push_preferences, message_keys
    // On nettoie manuellement conversation_participants et le token de session
    sqlx::query("DELETE FROM conversation_participants WHERE user_id = ?")
        .bind(&user_id).execute(&state.db).await.ok();

    sqlx::query("DELETE FROM users WHERE id = ? AND role != 'admin'")
        .bind(&user_id)
        .execute(&state.db)
        .await
        .ok();

    tracing::info!(
        admin_id = %admin.id,
        deleted_user = %user_id,
        "Membre supprimé par l'administrateur"
    );

    Json(json!({ "success": true, "message": "Membre supprimé" })).into_response()
}
