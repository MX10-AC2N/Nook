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
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, mpsc};
use crate::SharedState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallSignal {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "from")]
    pub from_user_id: String,
    #[serde(rename = "to")]
    pub to_user_id: Option<String>,
    #[serde(rename = "type")]
    pub signal_type: String,
    pub sdp: Option<String>,
    pub candidate: Option<String>,
}

enum WsSendMessage {
    Text(String),
    Close,
}

#[derive(Deserialize)]
pub struct WsQuery {
    conv: String,
}

type ConversationSubscribers = Arc<RwLock<HashMap<String, broadcast::Sender<CallSignal>>>>;

pub async fn call_ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let token = match crate::auth::get_cookie(&headers, "nook_session")
        .or_else(|| crate::auth::get_cookie(&headers, "nook_admin"))
    {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

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

    ws.on_upgrade(move |socket| {
        handle_call_socket(socket, state, query.conv, user_id)
    })
}

async fn handle_call_socket(
    socket: WebSocket,
    state: SharedState,
    conversation_id: String,
    user_id: String,
) {
    // Obtenir ou créer un broadcast channel pour cette conversation
    // et cloner le Sender pour l'utiliser dans les tâches
    let tx = {
        let mut subs = state.webrtc_broadcasts.write().await;
        subs.entry(conversation_id.clone())
            .or_insert_with(|| broadcast::channel(64).0)
            .clone()
    };
    
    // rx est le receiver pour recevoir les messages broadcast
    let rx = tx.subscribe();
    
    // Créer un canal pour envoyer des messages à la socket
    let (send_tx, mut send_rx) = mpsc::channel::<WsSendMessage>(64);
    let (mut ws_sink, mut ws_stream) = socket.split();

    // Annoncer l'arrivée dans la conversation
    let join_signal = CallSignal {
        conversation_id: conversation_id.clone(),
        from_user_id: user_id.clone(),
        to_user_id: None,
        signal_type: "join".to_string(),
        sdp: None,
        candidate: None,
    };
    
    // Envoyer le signal de join à tous les abonnés (y compris nous-mêmes)
    let _ = tx.send(join_signal.clone());

    // Envoyer le signal de join au nouvel arrivant via WebSocket
    if let Ok(json) = serde_json::to_string(&join_signal) {
        let _ = ws_sink.send(WsMessage::Text(json)).await;
    }

    // Tâche d'envoi : reçoit du canal mpsc et envoie au client WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = send_rx.recv().await {
            match msg {
                WsSendMessage::Text(text) => {
                    if ws_sink.send(WsMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
                WsSendMessage::Close => {
                    let _ = ws_sink.send(WsMessage::Close(None)).await;
                    break;
                }
            }
        }
    });

    // Variables clonées pour la tâche de réception
    let tx_for_broadcast = tx.clone(); // Pour broadcaster aux autres
    let user_id_recv = user_id.clone();
    let conversation_id_recv = conversation_id.clone();
    let send_tx_clone = send_tx.clone();

    // Tâche de réception : reçoit du client et broadcast
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_stream.next().await {
            match msg {
                WsMessage::Text(text) => {
                    if let Ok(mut signal) = serde_json::from_str::<CallSignal>(&text) {
                        // Forcer les champs critiques (sécurité)
                        signal.from_user_id = user_id_recv.clone();
                        signal.conversation_id = conversation_id_recv.clone();
                        // broadcast aux autres participants
                        let _ = tx_for_broadcast.send(signal);
                    }
                }
                WsMessage::Close(_) => {
                    // Annoncer le départ à tous les autres
                    let leave_signal = CallSignal {
                        conversation_id: conversation_id_recv,
                        from_user_id: user_id_recv,
                        to_user_id: None,
                        signal_type: "leave".to_string(),
                        sdp: None,
                        candidate: None,
                    };
                    let _ = tx_for_broadcast.send(leave_signal);
                    let _ = send_tx_clone.send(WsSendMessage::Close).await;
                    break;
                }
                _ => {}
            }
        }
    });

    // Variables pour la tâche broadcast (recevoir du channel et envoyer au client)
    let user_id_for_send = user_id;
    let conversation_id_for_send = conversation_id;
    let send_tx_for_send = send_tx;

    // Tâche de broadcast : reçoit du broadcast channel et envoie au client
    let broadcast_task = tokio::spawn(async move {
        let mut rx = rx;
        while let Ok(signal) = rx.recv().await {
            // Ne pas renvoyer ses propres signaux
            if signal.from_user_id == user_id_for_send {
                continue;
            }
            // Ne pas envoyer les signaux d'autres conversations
            if signal.conversation_id != conversation_id_for_send {
                continue;
            }
            if let Ok(json) = serde_json::to_string(&signal) {
                if send_tx_for_send.send(WsSendMessage::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut send_task_mut = send_task;
    let mut recv_task_mut = recv_task;
    let mut broadcast_task_mut = broadcast_task;

    tokio::select! {
        _ = &mut send_task_mut => {
            recv_task_mut.abort();
            broadcast_task_mut.abort();
        }
        _ = &mut recv_task_mut => {
            send_task_mut.abort();
            broadcast_task_mut.abort();
        }
        _ = &mut broadcast_task_mut => {
            send_task_mut.abort();
            recv_task_mut.abort();
        }
    }
}
