use crate::SharedState;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::{get, post},
    Router,
};
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use uuid::Uuid;

// Type pour les sender broadcast
pub type BroadcastSender = broadcast::Sender<String>;
pub type SharedCallState = Arc<Mutex<HashMap<Uuid, BroadcastSender>>>;

// Structure pour l'état WebRTC
#[derive(Clone)]
pub struct WebRtcState {
    pub broadcasts: SharedCallState,
}

impl WebRtcState {
    pub fn new() -> Self {
        Self {
            broadcasts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// Structure pour le transfert de fichiers volumineux
struct FileTransfer {
    file_id: String,
    file_name: String,
    total_size: u64,
    received_size: u64,
    data: Vec<u8>,
}

// Fonction broadcast_message compatible avec upload.rs
pub fn broadcast_message(
    state: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
    conversation_id: String,
    event: String,
    message: String,
) {
    if let Ok(guard) = state.lock() {
        for (_, tx) in guard.iter() {
            let _ = tx.send(message.clone());
        }
    }
}

// Handler pour les offres WebRTC
pub async fn handle_offer(
    State(state): State<SharedState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let offer = payload.get("offer").and_then(|o| o.as_str());
    let from_user_id = payload.get("from_user_id").and_then(|u| u.as_str()).unwrap_or("unknown");
    let to_user_id = payload.get("to_user_id").and_then(|u| u.as_str()).unwrap_or("unknown");
    
    if let Some(offer_sdp) = offer {
        let response = json!({
            "type": "offer",
            "offer": offer_sdp,
            "from_user_id": from_user_id,
            "to_user_id": to_user_id,
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        // Diffuser l'offre à tous les clients connectés
        let guard = state.lock().unwrap();
        for (_, tx) in guard.iter() {
            let _ = tx.send(response.to_string());
        }
        
        (axum::http::StatusCode::OK, Json(json!({"status": "offer_sent"})))
    } else {
        (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "Missing offer"})))
    }
}

// Handler pour les réponses WebRTC
pub async fn handle_answer(
    State(state): State<SharedState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let answer = payload.get("answer").and_then(|a| a.as_str());
    
    if let Some(answer_sdp) = answer {
        let response = json!({
            "type": "answer",
            "answer": answer_sdp,
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        // Diffuser la réponse à tous les clients connectés
        let guard = state.lock().unwrap();
        for (_, tx) in guard.iter() {
            let _ = tx.send(response.to_string());
        }
        
        (axum::http::StatusCode::OK, Json(json!({"status": "answer_sent"})))
    } else {
        (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "Missing answer"})))
    }
}

// WebSocket handler principal
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// Fonction interne pour gérer la connexion WebSocket
async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver): (SinkExt<_, _>, SplitStream<WebSocket>) = socket.split();
    let id = Uuid::new_v4();
    let (broadcast_tx, _) = broadcast::channel::<String>(100);
    
    // Stocker l'émetteur dans l'état partagé
    let mut guard = state.lock().unwrap();
    guard.insert(id, broadcast_tx.clone());
    drop(guard);

    // Map pour les transferts de fichiers en cours
    let file_transfers: Arc<Mutex<HashMap<String, FileTransfer>>> = Arc::new(Mutex::new(HashMap::new()));
    let transfers_clone = file_transfers.clone();

    // Tâche pour recevoir les messages et les diffuser
    let send_task = tokio::spawn(async move {
        let mut rx = broadcast_tx.subscribe();
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(msg)).await {
                eprintln!("Erreur d'envoi WebSocket: {}", e);
                break;
            }
        }
    });

    // Tâche pour recevoir les messages du client
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // Parser le message JSON
                    let event_opt = serde_json::from_str::<Value>(&text).ok()
                        .and_then(|json| json.get("event").and_then(|e| e.as_str()));
                    
                    if let Some(event) = event_opt {
                        match event {
                            "file_start" => {
                                // Début d'un transfert de fichier volumineux
                                let json = serde_json::from_str::<Value>(&text).ok().unwrap_or_else(|| json!({}));
                                let file_id = json.get("file_id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                                let file_name = json.get("file_name").and_then(|s| s.as_str()).unwrap_or("unknown").to_string();
                                let total_size = json.get("total_size").and_then(|s| s.as_u64()).unwrap_or(0);
                                
                                if total_size > 50 * 1024 * 1024 { // > 50MB
                                    let mut transfers = transfers_clone.lock().unwrap();
                                    transfers.insert(file_id.clone(), FileTransfer {
                                        file_id: file_id.clone(),
                                        file_name: file_name.clone(),
                                        total_size,
                                        received_size: 0,
                                        data: Vec::with_capacity(total_size as usize),
                                    });
                                    
                                    eprintln!("Début transfert fichier volumineux: {} ({} MB)", file_name, total_size / 1024 / 1024);
                                }
                            }
                            "file_chunk" => {
                                // Réception d'un chunk de fichier
                                let json = serde_json::from_str::<Value>(&text).ok().unwrap_or_else(|| json!({}));
                                if let Some(chunk_data) = json.get("data").and_then(|d| d.as_str()) {
                                    if let Ok(bytes) = base64::decode(chunk_data) {
                                        let file_id = json.get("file_id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                                        let chunk_size = bytes.len() as u64;
                                        
                                        let mut transfers = transfers_clone.lock().unwrap();
                                        if let Some(transfer) = transfers.get_mut(&file_id) {
                                            transfer.data.extend_from_slice(&bytes);
                                            transfer.received_size += chunk_size;
                                            
                                            // Afficher la progression
                                            let progress = (transfer.received_size * 100) / transfer.total_size;
                                            eprintln!("Progression: {}% ({}/{} bytes)", progress, transfer.received_size, transfer.total_size);
                                            
                                            // Vérifier si le transfert est complet
                                            if transfer.received_size >= transfer.total_size {
                                                // Sauvegarder le fichier
                                                let path = format!("uploads/{}", transfer.file_id);
                                                if let Ok(()) = std::fs::create_dir_all("uploads") {
                                                    if let Ok(()) = std::fs::write(&path, &transfer.data) {
                                                        eprintln!("Fichier sauvegardé: {} ({} bytes)", path, transfer.data.len());
                                                    }
                                                }
                                                // Nettoyer
                                                transfers.remove(&file_id);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Message normal, diffuser à tous les clients
                                if let Err(e) = broadcast_tx.send(text.clone()) {
                                    eprintln!("Erreur de diffusion: {}", e);
                                }
                            }
                        }
                    } else {
                        // Message texte simple, diffuser à tous
                        if let Err(e) = broadcast_tx.send(text.clone()) {
                            eprintln!("Erreur de diffusion: {}", e);
                        }
                    }
                    eprintln!("Message WebSocket reçu: {}", text);
                }
                Ok(Message::Binary(data)) => {
                    // Réception directe de données binaires
                    let file_id = format!("bin_{}", id);
                    let path = format!("uploads/{}", file_id);
                    let _ = std::fs::create_dir_all("uploads");
                    
                    if let Ok(()) = std::fs::write(&path, &data) {
                        eprintln!("Fichier binaire sauvegardé: {} ({} bytes)", path, data.len());
                    }
                }
                Err(e) => {
                    eprintln!("Erreur de réception WebSocket: {}", e);
                    break;
                }
                _ => break,
            }
        }
    });

    // Attendre que les tâches se terminent
    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    // Nettoyer lors de la déconnexion
    let mut guard = state.lock().unwrap();
    guard.remove(&id);
    println!("Client WebSocket déconnecté: {}", id);
}

// Routeur WebRTC
pub fn webrtc_routes(state: WebRtcState) -> Router {
    Router::new()
        .route("/api/webrtc/offer", post(handle_offer))
        .route("/api/webrtc/answer", get(handle_answer))
        .route("/ws", get(ws_handler))
        .with_state(state)
}
