// backend/src/push.rs — Notifications push Web Push API (VAPID)
// Session 37
//
// Architecture :
//   - Chaque client browser s'abonne via POST /api/push/subscribe
//   - Les abonnements sont stockés dans push_subscriptions (migration 006)
//   - Le backend envoie les notifs via POST /api/push/notify (interne)
//     ou depuis db.rs après chaque nouveau message
//
// Dépendances Cargo.toml à ajouter :
//   web-push = { version = "0.10", default-features = false, features = ["hyper-rustls"] }
//
// Variables d'env requises :
//   VAPID_PRIVATE_KEY  — clé privée VAPID (base64url, 32 bytes P-256)
//   VAPID_PUBLIC_KEY   — clé publique VAPID (base64url, 65 bytes P-256 uncompressed)
//   VAPID_SUBJECT      — mailto:admin@nook.local ou URL du homeserver
//
// Génération des clés VAPID (une seule fois) :
//   openssl ecparam -name prime256v1 -genkey -noout -out vapid_private.pem
//   openssl ec -in vapid_private.pem -pubout -out vapid_public.pem
//   # Encoder en base64url sans padding pour les variables d'env

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

/// Payload d'abonnement envoyé par le browser (PushSubscription.toJSON())
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

/// Préférences de notification
#[derive(Debug, Deserialize)]
pub struct UpdatePrefsRequest {
    pub enabled: Option<bool>,
    pub quiet_start: Option<String>,  // "HH:MM"
    pub quiet_end: Option<String>,    // "HH:MM"
    pub on_message: Option<bool>,
    pub on_mention: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PushPrefs {
    pub enabled: bool,
    pub quiet_start: String,
    pub quiet_end: String,
    pub on_message: bool,
    pub on_mention: bool,
}

/// Payload d'une notification à envoyer
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,    // URL icône
    pub url: Option<String>,     // URL de destination au clic
    pub tag: Option<String>,     // Déduplique les notifs du même tag
}

// ─────────────────────────────────────────────────────────────────────────────
// Routes
// ─────────────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/subscribe",        post(subscribe))
        .route("/unsubscribe",      delete(unsubscribe))
        .route("/preferences",      get(get_preferences).post(update_preferences))
        .route("/vapid-public-key", get(get_vapid_public_key))
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/push/vapid-public-key — retourne la clé publique VAPID pour le SW
async fn get_vapid_public_key() -> impl IntoResponse {
    let key = std::env::var("VAPID_PUBLIC_KEY").unwrap_or_default();
    Json(json!({ "public_key": key }))
}

/// POST /api/push/subscribe — enregistre l'abonnement push du device courant
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

    // INSERT OR REPLACE pour éviter les doublons sur même endpoint
    let result = sqlx::query(
        r#"INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh, auth, user_agent, created_at, last_used)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT(endpoint) DO UPDATE SET
             user_id = excluded.user_id,
             p256dh = excluded.p256dh,
             auth = excluded.auth,
             user_agent = excluded.user_agent,
             last_used = excluded.last_used"#,
    )
    .bind(&id)
    .bind(&current_user.id)
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
            // Initialiser les prefs si premier abonnement
            let _ = sqlx::query(
                r#"INSERT OR IGNORE INTO push_preferences
                   (user_id, enabled, quiet_start, quiet_end, on_message, on_mention, updated_at)
                   VALUES (?, 1, '22:00', '07:00', 1, 1, ?)"#,
            )
            .bind(&current_user.id)
            .bind(now)
            .execute(&state.db)
            .await;

            tracing::info!(user_id = %current_user.id, "Push subscription enregistrée");
            Json(json!({"success": true})).into_response()
        }
        Err(e) => {
            tracing::error!(err = %e, "Erreur enregistrement push subscription");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur serveur"}))).into_response()
        }
    }
}

/// DELETE /api/push/unsubscribe — supprime l'abonnement (logout ou désactivation)
async fn unsubscribe(
    State(state): State<Arc<SharedState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let endpoint = body.get("endpoint").and_then(|v| v.as_str()).unwrap_or("");
    if endpoint.is_empty() {
        // Supprimer tous les abonnements de l'utilisateur
        let _ = sqlx::query("DELETE FROM push_subscriptions WHERE user_id = ?")
            .bind(&current_user.id)
            .execute(&state.db)
            .await;
    } else {
        let _ = sqlx::query("DELETE FROM push_subscriptions WHERE user_id = ? AND endpoint = ?")
            .bind(&current_user.id)
            .bind(endpoint)
            .execute(&state.db)
            .await;
    }
    Json(json!({"success": true}))
}

/// GET /api/push/preferences
async fn get_preferences(
    State(state): State<Arc<SharedState>>,
    Extension(current_user): Extension<CurrentUser>,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, (i64, String, String, i64, i64)>(
        "SELECT enabled, quiet_start, quiet_end, on_message, on_mention
         FROM push_preferences WHERE user_id = ?",
    )
    .bind(&current_user.id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((enabled, quiet_start, quiet_end, on_message, on_mention))) => {
            Json(json!({
                "enabled": enabled == 1,
                "quiet_start": quiet_start,
                "quiet_end": quiet_end,
                "on_message": on_message == 1,
                "on_mention": on_mention == 1,
            })).into_response()
        }
        Ok(None) => {
            // Prefs par défaut
            Json(json!({
                "enabled": true,
                "quiet_start": "22:00",
                "quiet_end": "07:00",
                "on_message": true,
                "on_mention": true,
            })).into_response()
        }
        Err(e) => {
            tracing::error!(err = %e, "Erreur lecture push preferences");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// POST /api/push/preferences
async fn update_preferences(
    State(state): State<Arc<SharedState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdatePrefsRequest>,
) -> impl IntoResponse {
    let now = Utc::now().timestamp();
    let result = sqlx::query(
        r#"INSERT INTO push_preferences (user_id, enabled, quiet_start, quiet_end, on_message, on_mention, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT(user_id) DO UPDATE SET
             enabled     = COALESCE(excluded.enabled,     enabled),
             quiet_start = COALESCE(excluded.quiet_start, quiet_start),
             quiet_end   = COALESCE(excluded.quiet_end,   quiet_end),
             on_message  = COALESCE(excluded.on_message,  on_message),
             on_mention  = COALESCE(excluded.on_mention,  on_mention),
             updated_at  = excluded.updated_at"#,
    )
    .bind(&current_user.id)
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
// Envoi d'une notification — appelé depuis db.rs après send_message
// ─────────────────────────────────────────────────────────────────────────────

/// Envoie une notification push à tous les devices d'un utilisateur.
/// Utilise reqwest pour appeler l'endpoint Web Push directement.
/// VAPID signature construite avec les clés d'env.
///
/// ⚠️  Cette fonction est un STUB fonctionnel :
///    - Elle récupère les abonnements et vérifie les préférences (période silencieuse)
///    - L'envoi réel via VAPID nécessite la crate `web-push` (à ajouter dans Cargo.toml)
///    - En attendant, les logs indiquent quelles notifs seraient envoyées
pub async fn send_push_notification(
    pool: &sqlx::SqlitePool,
    recipient_user_id: &str,
    payload: &PushPayload,
) -> Result<(), String> {
    // 1. Récupérer les abonnements actifs de cet utilisateur
    let subs = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, endpoint, p256dh, auth FROM push_subscriptions WHERE user_id = ?",
    )
    .bind(recipient_user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if subs.is_empty() {
        return Ok(()); // Pas d'abonnement — silencieux
    }

    // 2. Vérifier les préférences (période silencieuse)
    if !should_notify(pool, recipient_user_id).await {
        tracing::debug!(user_id = %recipient_user_id, "Push skippé — période silencieuse");
        return Ok(());
    }

    // 3. Logger les notifications à envoyer (stub — web-push non encore ajouté)
    let payload_json = serde_json::to_string(payload).unwrap_or_default();
    for (sub_id, endpoint, _p256dh, _auth) in &subs {
        tracing::info!(
            sub_id = %sub_id,
            endpoint = %&endpoint[..endpoint.len().min(60)],
            payload = %payload_json,
            "PUSH → notification à envoyer (stub — ajouter web-push dans Cargo.toml)"
        );
        // TODO session 38 : implémenter l'envoi VAPID avec la crate web-push
        // web_push::WebPushClient::new()
        //   .send(WebPushMessageBuilder::new(sub).set_payload(...).build())
    }

    Ok(())
}

/// Vérifie si on doit notifier l'utilisateur maintenant (hors période silencieuse)
async fn should_notify(pool: &sqlx::SqlitePool, user_id: &str) -> bool {
    let row = sqlx::query_as::<_, (i64, String, String, i64)>(
        "SELECT enabled, quiet_start, quiet_end, on_message FROM push_preferences WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let (enabled, quiet_start, quiet_end, on_message) = match row {
        Some(r) => r,
        None => return true, // Pas de prefs → notifier par défaut
    };

    if enabled == 0 || on_message == 0 {
        return false;
    }

    // Vérifier la période silencieuse
    let now = chrono::Local::now();
    let current_time = now.format("%H:%M").to_string();

    // Comparaison simple HH:MM (fonctionne si quiet_start > quiet_end → nuit)
    let in_quiet = if quiet_start > quiet_end {
        // Période nocturne : ex 22:00 → 07:00
        current_time >= quiet_start || current_time < quiet_end
    } else {
        // Période diurne : ex 12:00 → 14:00
        current_time >= quiet_start && current_time < quiet_end
    };

    !in_quiet
}
