use crate::SharedState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

#[allow(dead_code)]
pub type CallSignal = Arc<RwLock<broadcast::Sender<String>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallMessage {
    pub r#type: String,
    pub from: String,
    pub to: String,
    pub sdp: Option<String>,
    pub candidate: Option<String>,
    pub call_id: String,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct SignalingData {
    pub call_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub sender: broadcast::Sender<String>,
}

#[allow(dead_code)]
pub struct SharedCallState {
    pub active_calls: Arc<RwLock<HashMap<String, SignalingData>>>,
}

#[allow(dead_code)]
impl SharedCallState {
    pub fn new() -> Self {
        SharedCallState {
            active_calls: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub async fn broadcast_message(
    state: Arc<SharedState>,
    conversation_id: String,
    message_type: String,
    content: String,
) {
    // Type annotation for broadcasts HashMap
    let broadcasts = state.webrtc_broadcasts.read().await;

    if let Some(tx_arc) = broadcasts.get(&conversation_id) {
        // Type annotation for sender
        let sender = tx_arc.read().await;
        let json_content = serde_json::to_string(&content).unwrap_or_default();
        let _ = sender.send(format!(
            "{{\"type\":\"{}\",\"content\":{},\"conversationId\":\"{}\"}}",
            message_type, json_content, conversation_id
        ));
    }
}

#[allow(dead_code)]
pub async fn ws_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<SharedState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    let send_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_sender.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
    });

    let recv_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(call_message) = serde_json::from_str::<CallMessage>(&text) {
                        let state_clone = state.clone();
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            handle_call_message(state_clone, call_message, tx_clone).await;
                        });
                    }
                }
                Ok(Message::Binary(data)) => {
                    eprintln!("Received binary data: {:?}", data);
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => eprintln!("Send task ended"),
        _ = recv_task => eprintln!("Receive task ended"),
    }
}

#[allow(dead_code)]
async fn handle_call_message(
    state: Arc<SharedState>,
    message: CallMessage,
    _tx: tokio::sync::mpsc::Sender<String>,
) {
    match message.r#type.as_str() {
        "call_request" => {
            let conversation_id = format!("{}-{}", message.from, message.to);
            let (sender, _) = broadcast::channel::<String>(100);

            let _signaling_data = SignalingData {
                call_id: message.call_id.clone(),
                from_user_id: message.from.clone(),
                to_user_id: message.to.clone(),
                sender: sender.clone(),
            };

            {
                let mut subs = state.webrtc_broadcasts.write().await;
                subs.entry(conversation_id.clone())
                    .or_insert(Arc::new(RwLock::new(sender.clone())));
            }

            let incoming_call = CallMessage {
                r#type: "incoming_call".to_string(),
                from: message.from.clone(),
                to: message.to.clone(),
                sdp: None,
                candidate: None,
                call_id: message.call_id.clone(),
            };
            let message_json = serde_json::to_string(&incoming_call).unwrap();
            let _ = sender.send(message_json);

            println!("Call request from {} to {}", message.from, message.to);
        }
        "call_response" | "ice_candidate" | "offer" | "answer" => {
            let conversation_id = format!("{}-{}", message.from, message.to);
            let broadcasts = state.webrtc_broadcasts.read().await;
            if let Some(tx_arc) = broadcasts.get(&conversation_id) {
                let sender = tx_arc.read().await;
                let message_json = serde_json::to_string(&message).unwrap();
                let _ = sender.send(message_json);
            }
        }
        "end_call" => {
            println!("Call ended from {} to {}", message.from, message.to);
            let conversation_id = format!("{}-{}", message.from, message.to);
            {
                let mut subs = state.webrtc_broadcasts.write().await;
                subs.remove(&conversation_id);
            }
        }
        _ => {
            println!("Unknown message type: {}", message.r#type);
        }
    }
}

// WebRTC offer handler
pub async fn handle_offer() -> impl IntoResponse {
    use serde_json::json;
    axum::Json(json!({
        "status": "ready",
        "message": "WebRTC signaling endpoint ready"
    }))
}

// WebRTC answer handler
pub async fn handle_answer() -> impl IntoResponse {
    use serde_json::json;
    axum::Json(json!({
        "status": "ready",
        "message": "WebRTC signaling endpoint ready"
    }))
}
