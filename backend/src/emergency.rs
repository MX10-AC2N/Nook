// backend/src/emergency.rs — Mode urgence Nook
// Route : POST /api/emergency (require_auth)
//
// Quand un membre envoie une alerte d'urgence :
//   1. Log immédiat dans les logs Docker (toujours visible)
//   2. Diffusion WS à tous les clients connectés (type: "emergency")
//   3. Push notification à tous les membres de la famille
//
// Le message WS est géré automatiquement par le broadcast webrtc.rs.
// La route POST ici déclenche le push push + le log.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::auth::CurrentUser;
use crate::push::PushPayload;
use crate::SharedState;

#[derive(Deserialize)]
pub struct EmergencyRequest {
    pub message: String,
}

pub async fn handle_emergency(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(payload): Json<EmergencyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let msg = payload.message.trim().to_string();
    if msg.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 1. Log immédiat — visible dans `docker compose logs nook`
    tracing::warn!(
        user_id  = %user.id,
        username = %user.username,
        message  = %msg,
        "🚨 ALERTE D'URGENCE"
    );

    // 2. Push notification à tous les membres de la famille
    //    Fire-and-forget — ne bloque pas la réponse HTTP
    {
        let pool    = state.db.clone();
        let sender  = user.username.clone();
        let message = msg.clone();

        tokio::task::spawn(async move {
            // Récupérer tous les user_ids actifs (approved = 1)
            let members: Vec<(String,)> = sqlx::query_as(
                "SELECT id FROM users WHERE approved = 1",
            )
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            let push_payload = PushPayload {
                title: format!("🚨 Urgence — {}", sender),
                body:  message,
                icon:  Some("/icon-192.png".to_string()),
                url:   Some("/chat".to_string()),
                tag:   Some("nook-emergency".to_string()),
            };

            for (member_id,) in members {
                if let Err(e) = crate::push::send_push_notification(
                    &pool, &member_id, &push_payload,
                ).await {
                    tracing::debug!(
                        error    = %e,
                        member   = %member_id,
                        "Push urgence non envoyé"
                    );
                }
            }
        });
    }

    Ok(Json(json!({
        "success": true,
        "message": "Alerte d'urgence reçue — push envoyé à tous les membres"
    })))
}
