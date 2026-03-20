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

// VAPID helpers (ring + base64ct — pas de dépendance web-push D10)
use base64ct::{Base64UrlUnpadded, Encoding as _};

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

/// Routes push protégées — montées sous /push dans protected_routes (require_auth)
pub fn router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/subscribe",   post(subscribe))
        .route("/unsubscribe", delete(unsubscribe))
        .route("/preferences", get(get_preferences).post(update_preferences))
}

/// Route push publique — vapid-public-key accessible sans auth
/// (le browser en a besoin pour s'abonner aux notifications avant même le login)
pub fn public_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/vapid-public-key", get(get_vapid_public_key))
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
// ─────────────────────────────────────────────────────────────────────────────
// VAPID — Envoi réel via ring + reqwest (session 39)
// RFC 8292 : JWT ES256, pas de dépendance web-push (D10 interdit async-trait)
// ─────────────────────────────────────────────────────────────────────────────

/// Encode en base64url sans padding (RFC 4648 §5)
fn b64url(data: &[u8]) -> String {
    Base64UrlUnpadded::encode_string(data)
}

/// Construit et signe le JWT VAPID (ES256)
fn build_vapid_jwt(endpoint: &str, subject: &str, private_key_b64url: &str) -> Result<String, String> {
    use ring::rand::SystemRandom;
    use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};

    let origin = endpoint.split('/').take(3).collect::<Vec<_>>().join("/");
    let now = chrono::Utc::now().timestamp();
    let exp = now + 43_200; // 12h max (RFC 8292 §2)

    let header = b64url(br#"{"typ":"JWT","alg":"ES256"}"#);
    let claims = b64url(format!(
        r#"{{"aud":"{}","exp":{},"sub":"{}"}}"#,
        origin, exp, subject
    ).as_bytes());
    let signing_input = format!("{}.{}", header, claims);

    let pkcs8_der = Base64UrlUnpadded::decode_vec(private_key_b64url)
        .map_err(|_| "Clé VAPID_PRIVATE_KEY invalide (base64url)")?;

    let rng = SystemRandom::new();
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8_der, &rng)
        .map_err(|e| format!("Clé VAPID PKCS8 invalide : {:?}", e))?;

    let sig = key_pair.sign(&rng, signing_input.as_bytes())
        .map_err(|e| format!("Signature VAPID échouée : {:?}", e))?;

    Ok(format!("{}.{}", signing_input, b64url(sig.as_ref())))
}

/// Envoie une notification push Web Push VAPID vers un endpoint unique.
/// Payload en JSON texte — pas de chiffrement ECE (homeserver familial LAN/WAN).
async fn send_web_push(
    client: &reqwest::Client,
    endpoint: &str,
    payload_json: &str,
    jwt: &str,
    public_key_b64url: &str,
) -> Result<(), String> {
    let auth_header = format!("vapid t={},k={}", jwt, public_key_b64url);
    let res = client
        .post(endpoint)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .header("TTL", "86400")
        .body(payload_json.to_string())
        .send()
        .await
        .map_err(|e| format!("Erreur réseau push : {}", e))?;

    match res.status().as_u16() {
        200 | 201 => Ok(()),
        410 => Err(format!("ENDPOINT_GONE:{}", endpoint)), // endpoint expiré → supprimer
        s   => Err(format!("Push HTTP {}", s)),
    }
}

/// Envoie une notification push à tous les devices d'un utilisateur.
/// Sans clés VAPID configurées → no-op silencieux.
pub async fn send_push_notification(
    pool: &sqlx::SqlitePool,
    recipient_user_id: &str,
    payload: &PushPayload,
) -> Result<(), String> {
    // Vérifier que VAPID est configuré — sinon no-op silencieux
    let (priv_key, pub_key) = match (
        std::env::var("VAPID_PRIVATE_KEY").ok().filter(|s| !s.is_empty()),
        std::env::var("VAPID_PUBLIC_KEY").ok().filter(|s| !s.is_empty()),
    ) {
        (Some(private_key), Some(public_key)) => (private_key, public_key),
        _ => {
            tracing::debug!(user_id = %recipient_user_id, "Push ignoré — VAPID_PRIVATE_KEY/PUBLIC_KEY non configurés");
            return Ok(());
        }
    };
    let subject = std::env::var("VAPID_SUBJECT")
        .unwrap_or_else(|_| "mailto:admin@nook.local".to_string());

    if !should_notify(pool, recipient_user_id).await {
        tracing::debug!(user_id = %recipient_user_id, "Push skippé — période silencieuse ou désactivé");
        return Ok(());
    }

    // Récupérer les abonnements de l'utilisateur
    #[derive(sqlx::FromRow)]
    struct Sub { id: String, endpoint: String }
    let subs = sqlx::query_as::<_, Sub>(
        "SELECT id, endpoint FROM push_subscriptions WHERE user_id = ?",
    )
    .bind(recipient_user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if subs.is_empty() {
        return Ok(());
    }

    let payload_json = serde_json::to_string(payload).unwrap_or_default();
    let client = reqwest::Client::new();
    let mut expired: Vec<String> = Vec::new();

    for sub in &subs {
        match build_vapid_jwt(&sub.endpoint, &subject, &priv_key) {
            Err(e) => {
                tracing::warn!(sub_id = %sub.id, error = %e, "JWT VAPID invalide");
                continue;
            }
            Ok(jwt) => {
                match send_web_push(&client, &sub.endpoint, &payload_json, &jwt, &pub_key).await {
                    Ok(_) => {
                        tracing::info!(
                            user_id = %recipient_user_id,
                            endpoint = %&sub.endpoint[..sub.endpoint.len().min(50)],
                            title = %payload.title,
                            "Push envoyé ✓"
                        );
                    }
                    Err(e) if e.starts_with("ENDPOINT_GONE:") => {
                        expired.push(sub.id.clone());
                        tracing::info!(sub_id = %sub.id, "Endpoint push expiré — suppression");
                    }
                    Err(e) => {
                        tracing::warn!(sub_id = %sub.id, error = %e, "Push échoué");
                    }
                }
            }
        }
    }

    // Nettoyer les endpoints expirés
    for sub_id in expired {
        let _ = sqlx::query("DELETE FROM push_subscriptions WHERE id = ?")
            .bind(&sub_id)
            .execute(pool)
            .await;
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
