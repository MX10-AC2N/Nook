use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
    Router,
};
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use uuid::Uuid;

// Utiliser le même type SharedState que dans main.rs
pub type SharedState = Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>;

#[derive(Clone)]
pub struct WebRtcState {
    pub broadcasts: SharedState,
}

// Structure pour le transfert de fichiers volumineux
struct FileTransfer {
    file_id: String,
    file_name: String,
    total_size: u64,
    received_size: u64,
    data: Vec<u8>,
}

pub fn broadcast_message(
    state: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
    _conversation_id: String,
    _event: String,
    message: String,
) {
    if let Ok(guard) = state.lock() {
        for (_, tx) in guard.iter() {
            let _ = tx.send(message.clone());
        }
    }
}

pub async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver): (SinkExt, SplitStream<WebSocket>) = socket.split();
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

    // Tâche pour recevoir les messages du client (y compris fichiers volumineux)
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // Traitement des messages textuels
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        let event = json.get("event").map(|e| e.as_str()).unwrap_or("");
                        
                        match event {
                            "file_start" => {
                                // Début d'un transfert de fichier volumineux
                                let file_id = json.get("file_id").map(|s| s.as_str().unwrap_or("")).unwrap_or("").to_string();
                                let file_name = json.get("file_name").map(|s| s.as_str().unwrap_or("unknown")).unwrap_or("unknown").to_string();
                                let total_size = json.get("total_size").map(|s| s.as_u64().unwrap_or(0)).unwrap_or(0);
                                
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
                                if let Some(chunk_data) = json.get("data").and_then(|d| d.as_str()) {
                                    if let Ok(bytes) = base64::decode(chunk_data) {
                                        let file_id = json.get("file_id").map(|s| s.as_str().unwrap_or("")).unwrap_or("").to_string();
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
                                                if let Ok(_) = std::fs::create_dir_all("uploads") {
                                                    if let Ok(_) = std::fs::write(&path, &transfer.data) {
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
                                // Message normal, diffuser à tous
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
                    eprintln!("Message reçu: {}", text);
                }
                Ok(Message::Binary(data)) => {
                    // Réception directe de données binaires (alternative au base64)
                    let file_id = format!("bin_{}", id); // ID temporaire pour les binaires
                    let path = format!("uploads/{}", file_id);
                    let _ = std::fs::create_dir_all("uploads");
                    
                    if let Ok(_) = std::fs::write(&path, &data) {
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
    println!("Client déconnecté: {}", id);
}

pub fn webrtc_routes(state: WebRtcState) -> Router {
    Router::new()
        .route("/ws", get(move |ws: WebSocketUpgrade| {
            ws.on_upgrade(|socket| handle_socket(socket, state.broadcasts.clone()))
        }))
        .with_state(state)
}
