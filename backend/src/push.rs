// backend/src/push.rs — Notifications push Web Push API (VAPID)
// Session 37
//
// Architecture :
//   - Chaque client browser s'abonne via POST /api/push/subscribe
//   - Les abonnements sont stockés dans push_subscriptions (migration 006)
//   - L'envoi VAPID réel sera implémenté via reqwest directement (pas de web-push)
//     → web-push 0.10 tire async-trait → crash proc-macro dans distroless (D10)
//
// Variables d'env (à préparer pour session 38) :
//   VAPID_PRIVATE_KEY  — clé privée VAPID base64url P-256
//   VAPID_PUBLIC_KEY   — clé publique VAPID base64url P-256 uncompressed
//   VAPID_SUBJECT      — mailto:admin@nook.local
//
// Génération des clés VAPID (une seule fois) :
//   npx web-push generate-vapid-keys

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Extension, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::SharedState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: PushKeys,
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePrefsRequest {
    pub enabled: Option<bool>,
    pub quiet_start: Option<String>,
    pub quiet_end: Option<String>,
    pub on_message: Option<bool>,
    pub on_mention: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)] // S38 : utilisé par send_push_notification
pub struct PushPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub url: Option<String>,
    pub tag: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Routes
// ─────────────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/subscribe",         post(subscribe))
        .route("/unsubscribe",       delete(unsubscribe))
        .route("/preferences",       get(get_preferences).post(update_preferences))
        .route("/vapid-public-key",  get(get_vapid_public_key))
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

async fn get_vapid_public_key() -> impl IntoResponse {
    let key = std::env::var("VAPID_PUBLIC_KEY").unwrap_or_default();
    Json(json!({ "public_key": key }))
}

async fn subscribe(
    State(state): State<Arc<SharedState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SubscribeRequest>,
) -> impl IntoResponse {
    if req.endpoint.is_empty() || req.keys.p256dh.is_empty() || req.keys.auth.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Abonnement invalide"}))).into_response();
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let result = sqlx::query(
        r#"INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh, auth, user_agent, created_at, last_used)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT(endpoint) DO UPDATE SET
             user_id    = excluded.user_id,
             p256dh     = excluded.p256dh,
             auth       = excluded.auth,
             user_agent = excluded.user_agent,
             last_used  = excluded.last_used"#,
    )
    .bind(&id)
    .bind(&current_user.0.id)
    .bind(&req.endpoint)
    .bind(&req.keys.p256dh)
    .bind(&req.keys.auth)
    .bind(&req.user_agent)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let _ = sqlx::query(
                r#"INSERT OR IGNORE INTO push_preferences
                   (user_id, enabled, quiet_start, quiet_end, on_message, on_mention, updated_at)
                   VALUES (?, 1, '22:00', '07:00', 1, 1, ?)"#,
            )
            .bind(&current_user.0.id)
            .bind(now)
            .execute(&state.db)
            .await;

            tracing::info!(user_id = %current_user.0.id, "Push subscription enregistrée");
            Json(json!({"success": true})).into_response()
        }
        Err(e) => {
            tracing::error!(err = %e, "Erreur enregistrement push subscription");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur serveur"}))).into_response()
        }
    }
}

async fn unsubscribe(
    State(state): State<Arc<SharedState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let endpoint = body.get("endpoint").and_then(|v| v.as_str()).unwrap_or("");
    if endpoint.is_empty() {
        let _ = sqlx::query("DELETE FROM push_subscriptions WHERE user_id = ?")
            .bind(&current_user.0.id)
            .execute(&state.db)
            .await;
    } else {
        let _ = sqlx::query(
            "DELETE FROM push_subscriptions WHERE user_id = ? AND endpoint = ?",
        )
        .bind(&current_user.0.id)
        .bind(endpoint)
        .execute(&state.db)
        .await;
    }
    Json(json!({"success": true}))
}

async fn get_preferences(
    State(state): State<Arc<SharedState>>,
    Extension(current_user): Extension<CurrentUser>,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, (i64, String, String, i64, i64)>(
        "SELECT enabled, quiet_start, quiet_end, on_message, on_mention
         FROM push_preferences WHERE user_id = ?",
    )
    .bind(&current_user.0.id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((enabled, quiet_start, quiet_end, on_message, on_mention))) => Json(json!({
            "enabled":     enabled == 1,
            "quiet_start": quiet_start,
            "quiet_end":   quiet_end,
            "on_message":  on_message == 1,
            "on_mention":  on_mention == 1,
        })).into_response(),
        Ok(None) => Json(json!({
            "enabled": true, "quiet_start": "22:00", "quiet_end": "07:00",
            "on_message": true, "on_mention": true,
        })).into_response(),
        Err(e) => {
            tracing::error!(err = %e, "Erreur lecture push preferences");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_preferences(
    State(state): State<Arc<SharedState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdatePrefsRequest>,
) -> impl IntoResponse {
    let now = Utc::now().timestamp();
    let result = sqlx::query(
        r#"INSERT INTO push_preferences
           (user_id, enabled, quiet_start, quiet_end, on_message, on_mention, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT(user_id) DO UPDATE SET
             enabled     = COALESCE(excluded.enabled,     enabled),
             quiet_start = COALESCE(excluded.quiet_start, quiet_start),
             quiet_end   = COALESCE(excluded.quiet_end,   quiet_end),
             on_message  = COALESCE(excluded.on_message,  on_message),
             on_mention  = COALESCE(excluded.on_mention,  on_mention),
             updated_at  = excluded.updated_at"#,
    )
    .bind(&current_user.0.id)
    .bind(req.enabled.map(|b| b as i64))
    .bind(&req.quiet_start)
    .bind(&req.quiet_end)
    .bind(req.on_message.map(|b| b as i64))
    .bind(req.on_mention.map(|b| b as i64))
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(json!({"success": true})).into_response(),
        Err(e) => {
            tracing::error!(err = %e, "Erreur update push preferences");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Envoi — stub reqwest (envoi VAPID réel : session 38)
// ─────────────────────────────────────────────────────────────────────────────

/// Envoie une notification push à tous les devices d'un utilisateur.
///
/// ⚠️ Stub S37 — stocke + vérifie les prefs, log les notifs à envoyer.
///    L'envoi VAPID réel (reqwest POST vers endpoint) sera implémenté en S38
///    sans dépendance externe (web-push tire async-trait → interdit, voir D10).
#[allow(dead_code)] // S38 : appelé depuis db.rs après send_message
pub async fn send_push_notification(
    pool: &sqlx::SqlitePool,
    recipient_user_id: &str,
    payload: &PushPayload,
) -> Result<(), String> {
    let subs = sqlx::query_as::<_, (String, String)>(
        "SELECT id, endpoint FROM push_subscriptions WHERE user_id = ?",
    )
    .bind(recipient_user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if subs.is_empty() {
        return Ok(());
    }

    if !should_notify(pool, recipient_user_id).await {
        tracing::debug!(user_id = %recipient_user_id, "Push skippé — période silencieuse");
        return Ok(());
    }

    for (sub_id, endpoint) in &subs {
        tracing::info!(
            sub_id = %sub_id,
            endpoint = %&endpoint[..endpoint.len().min(50)],
            title = %payload.title,
            "PUSH stub → S38 implémentera l'envoi VAPID via reqwest"
        );
    }

    Ok(())
}

#[allow(dead_code)]
async fn should_notify(pool: &sqlx::SqlitePool, user_id: &str) -> bool {
    let row = sqlx::query_as::<_, (i64, String, String, i64)>(
        "SELECT enabled, quiet_start, quiet_end, on_message
         FROM push_preferences WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let (enabled, quiet_start, quiet_end, on_message) = match row {
        Some(r) => r,
        None => return true,
    };

    if enabled == 0 || on_message == 0 {
        return false;
    }

    let now = chrono::Local::now();
    let current_time = now.format("%H:%M").to_string();

    let in_quiet = if quiet_start > quiet_end {
        current_time >= quiet_start || current_time < quiet_end
    } else {
        current_time >= quiet_start && current_time < quiet_end
    };

    !in_quiet
}
