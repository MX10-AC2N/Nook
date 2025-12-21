// backend/src/webrtc.rs
// Signaling WebRTC pour appels 1:1 et groupes
// Architecture : un WebSocket par conversation
// Chaque signal inclut conversation_id, from, (optionnel) to, type, sdp, candidate
// Aucune donnée persistée — signaling éphémère via broadcast local

use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{stream::StreamExt, sink::SinkExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::SharedState;

// Structure du message de signaling
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallSignal {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "from")]
    pub from_user_id: String,
    #[serde(rename = "to")]
    pub to_user_id: Option<String>, // None = broadcast dans la conversation
    #[serde(rename = "type")]
    pub signal_type: String, // "offer" | "answer" | "ice" | "join" | "leave"
    pub sdp: Option<String>,
    pub candidate: Option<serde_json::Value>,
}

// Paramètres de la requête WebSocket
#[derive(Deserialize)]
pub struct WsQuery {
    conv: String, // conversation_id
}

// État partagé pour les connexions par conversation
type ConversationSubscribers = Arc<RwLock<HashMap<String, broadcast::Sender<CallSignal>>>>;

pub async fn call_ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Validation de la session utilisateur
    let token = match crate::auth::get_cookie(&headers, "nook_session")
        .or_else(|| crate::auth::get_cookie(&headers, "nook_admin"))
    {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // Vérifier que l'utilisateur est approuvé et existe
    let user_row = match sqlx::query(
        "SELECT u.id FROM sessions s 
         JOIN users u ON s.user_id = u.id 
         WHERE s.token = ? AND u.approved = 1 AND s.expires_at > strftime('%s', 'now')"
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => row,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_id: String = user_row.try_get("id").unwrap_or_default();
    if user_id.is_empty() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Upgrade WebSocket
    ws.on_upgrade(move |socket| {
        handle_call_socket(socket, state, query.conv, user_id)
    })
}

async fn handle_call_socket(
    mut socket: WebSocket,
    state: SharedState,
    conversation_id: String,
    user_id: String,
) {
    // Obtenir ou créer un broadcast channel pour cette conversation
    let tx = {
        let mut subs = state.webrtc_broadcasts.write().await;
        subs.entry(conversation_id.clone())
            .or_insert_with(|| broadcast::channel(64).0)
            .clone()
    };

    let mut rx = tx.subscribe();

    // Annoncer l'arrivée dans la conversation
    let join_signal = CallSignal {
        conversation_id: conversation_id.clone(),
        from_user_id: user_id.clone(),
        to_user_id: None,
        signal_type: "join".to_string(),
        sdp: None,
        candidate: None,
    };
    let _ = tx.send(join_signal.clone());

    // Envoyer le signal de join au nouvel arrivant
    if let Ok(json) = serde_json::to_string(&join_signal) {
        let _ = socket.send(WsMessage::Text(json)).await;
    }

    // Tâche d'envoi : reçoit du broadcast et envoie au client
    let send_task = tokio::spawn(async move {
        while let Ok(signal) = rx.recv().await {
            // Ne pas renvoyer ses propres signaux
            if signal.from_user_id == user_id {
                continue;
            }
            // Ne pas envoyer les signaux d'autres conversations
            if signal.conversation_id != conversation_id {
                continue;
            }
            if let Ok(json) = serde_json::to_string(&signal) {
                let _ = socket.send(WsMessage::Text(json)).await;
            }
        }
    });

    // Tâche de réception : reçoit du client et broadcast
    let recv_task = tokio::spawn({
        let tx = tx.clone();
        let conversation_id = conversation_id.clone();
        async move {
            while let Some(Ok(msg)) = socket.next().await {
                match msg {
                    WsMessage::Text(text) => {
                        if let Ok(mut signal) = serde_json::from_str::<CallSignal>(&text) {
                            // Forcer les champs critiques (sécurité)
                            signal.from_user_id = user_id.clone();
                            signal.conversation_id = conversation_id.clone();

                            let _ = tx.send(signal);
                        }
                    }
                    WsMessage::Close(_) => {
                        // Annoncer le départ
                        let leave_signal = CallSignal {
                            conversation_id,
                            from_user_id: user_id.clone(),
                            to_user_id: None,
                            signal_type: "leave".to_string(),
                            sdp: None,
                            candidate: None,
                        };
                        let _ = tx.send(leave_signal);
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    // Nettoyage à la fin
    tokio::select! {
        _ = send_task => {
            recv_task.abort();
        }
        _ = recv_task => {}
    };

    // Optionnel : nettoyer le channel s'il n'y a plus d'abonnés
    // (non critique car mémoire faible, et recréé si besoin)
}
