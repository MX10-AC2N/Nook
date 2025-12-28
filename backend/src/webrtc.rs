use crate::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

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

#[derive(Clone)]
pub struct SignalingData {
    pub call_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub sender: broadcast::Sender<String>,
}

pub struct SharedCallState {
    pub active_calls: Arc<RwLock<HashMap<String, SignalingData>>>,
}

impl SharedCallState {
    pub fn new() -> Self {
        SharedCallState {
            active_calls: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub async fn broadcast_message(
    state: Arc<State>,
    conversation_id: String,
    message_type: String,
    content: String,
) {
    let broadcasts: tokio::sync::RwLockReadGuard<'_, HashMap<String, Arc<RwLock<broadcast::Sender<String>>>>> = state.webrtc_broadcasts.read().await;
    if let Some(tx) = broadcasts.get(&conversation_id) {
        let sender: tokio::sync::RwLockWriteGuard<'_, broadcast::Sender<String>> = tx.read().await;
        let json_content = serde_json::to_string(&content).unwrap_or_default();
        let _ = sender.send(format!(
            "{{\"type\":\"{}\",\"content\":{},\"conversationId\":\"{}\"}}",
            message_type,
            json_content,
            conversation_id
        ));
    }
}

pub async fn ws_handler(
    State(state): AxumState<Arc<State>>,
    axum::extract::ws::WebSocketUpgrade {
        mut handler,
        ..
    }: WebSocketUpgrade,
) -> impl IntoResponse {
    handler.on_upgrade(|socket: WebSocket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<State>) {
    let ws_sender: SplitSink<WebSocket, Message> = socket.split().0;
    let mut ws_receiver: SplitStream<WebSocket> = socket.split().1;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    let send_task: JoinHandle<()> = tokio::spawn(async move {
        let mut sender = ws_sender;
        while let Some(message) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(message)).await {
                eprintln!("WebSocket send error: {}", e);
                break;
            }
        }
    });

    let recv_task: JoinHandle<()> = tokio::spawn(async move {
        let mut receiver = ws_receiver;
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(call_message) = serde_json::from_str::<CallMessage>(&text) {
                        let state = state.clone();
                        let tx = tx.clone();

                        tokio::spawn(async move {
                            handle_call_message(state, call_message, tx).await;
                        });
                    }
                }
                Ok(Message::Binary(data)) => {
                    eprintln!("Received binary data: {:?}", data);
                }
                Ok(Message::Ping(_)) => {
                    eprintln!("Received ping");
                }
                Ok(Message::Pong(_)) => {
                    eprintln!("Received pong");
                }
                Ok(Message::Close(_)) => {
                    eprintln!("WebSocket closed");
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    let mut send_task_mut = send_task;
    let mut recv_task_mut = recv_task;

    tokio::select! {
        _ = &mut send_task_mut => {
            eprintln!("Send task ended");
        }
        _ = &mut recv_task_mut => {
            eprintln!("Receive task ended");
        }
    }

    let _ = tokio::join!(send_task, recv_task);
}

async fn handle_call_message(state: Arc<State>, message: CallMessage, _tx: tokio::sync::mpsc::Sender<String>) {
    match message.r#type.as_str() {
        "call_request" => {
            let conversation_id = format!("{}-{}", message.from, message.to);

            let (sender, _) = broadcast::channel::<String>(100);

            let signaling_data = SignalingData {
                call_id: message.call_id.clone(),
                from_user_id: message.from.clone(),
                to_user_id: message.to.clone(),
                sender: sender.clone(),
            };

            let mut subs: tokio::sync::RwLockWriteGuard<'_, HashMap<String, Arc<RwLock<broadcast::Sender<String>>>>> = state.webrtc_broadcasts.write().await;
            subs.entry(conversation_id.clone())
                .or_insert_with(|| Arc::new(RwLock::new(sender)));

            let message_json = serde_json::to_string(&CallMessage {
                r#type: "incoming_call".to_string(),
                from: message.from.clone(),
                to: message.to.clone(),
                sdp: None,
                candidate: None,
                call_id: message.call_id.clone(),
            }).unwrap();
            let _ = sender.send(message_json);

            let mut active_calls: tokio::sync::RwLockWriteGuard<'_, HashMap<String, SignalingData>> = state.active_calls.write().await;
            active_calls.insert(message.call_id.clone(), signaling_data);

            println!("Call request from {} to {}", message.from, message.to);
        }
        "call_response" | "ice_candidate" | "end_call" | "offer" | "answer" => {
            let conversation_id = format!("{}-{}", message.from, message.to);

            let broadcasts: tokio::sync::RwLockReadGuard<'_, HashMap<String, Arc<RwLock<broadcast::Sender<String>>>>> = state.webrtc_broadcasts.read().await;
            let tx_lock = broadcasts.get(&conversation_id).cloned();

            if let Some(tx_arc) = tx_lock {
                let sender_inner: tokio::sync::RwLockWriteGuard<'_, broadcast::Sender<String>> = tx_arc.write().await;
                let message_json = serde_json::to_string(&message).unwrap();
                let _ = sender_inner.send(message_json);
            }

            match message.r#type.as_str() {
                "call_response" => println!("Call response from {} to {}", message.from, message.to),
                "ice_candidate" => println!("ICE candidate from {} to {}", message.from, message.to),
                "end_call" => {
                    println!("Call ended from {} to {}", message.from, message.to);
                    
                    let mut active_calls: tokio::sync::RwLockWriteGuard<'_, HashMap<String, SignalingData>> = state.active_calls.write().await;
                    active_calls.remove(&message.call_id);

                    let mut subs: tokio::sync::RwLockWriteGuard<'_, HashMap<String, Arc<RwLock<broadcast::Sender<String>>>>> = state.webrtc_broadcasts.write().await;
                    subs.remove(&conversation_id);
                }
                _ => {}
            }
        }
        _ => {
            println!("Unknown message type: {}", message.r#type);
        }
    }
}
