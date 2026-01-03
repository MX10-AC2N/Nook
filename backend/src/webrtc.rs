use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use futures_util::{stream::SplitSink, StreamExt};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

// Utiliser le même type SharedState que dans main.rs
pub type SharedState = Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>;

pub struct WebRtcState {
    pub broadcasts: SharedState,
    pub tx: mpsc::Sender<String>,
}

pub async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let id = Uuid::new_v4();
    let (broadcast_tx, _) = broadcast::channel::<String>(100);
    
    // Stocker l'émetteur dans l'état partagé
    let mut guard = state.lock().unwrap();
    guard.insert(id, broadcast_tx.clone());
    drop(guard);

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

    // Tâche pour recevoir les messages du canal interne
    let internal_rx_task = tokio::spawn(async move {
        // Cette partie serait reliée au canal mpsc si nécessaire
    });

    // Tâche pour recevoir les messages du client
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // Diffuser à tous les clients connectés
                    if let Err(e) = broadcast_tx.send(text.clone()) {
                        eprintln!("Erreur de diffusion: {}", e);
                    }
                    eprintln!("Message reçu: {}", text);
                }
                Ok(Message::Binary(_)) => {
                    eprintln!("Message binaire reçu, ignoré");
                }
                Err(e) => {
                    eprintln!("Erreur de réception WebSocket: {}", e);
                    break;
                }
                _ => break,
            }
        }
    });

    // Attendre que les tâches se terminent ou gérer la déconnexion
    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
        _ = internal_rx_task => {},
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
