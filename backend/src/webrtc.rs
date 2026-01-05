// webrtc.rs - Signalisation P2P + Chiffrement fichiers (libsodium-compatible)
// + Nettoyage automatique des fichiers après 48h

use axum::{
    extract::{State as AxumState, Json as AxumJson, ws::WebSocket},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::time::{interval, sleep};
use chacha20poly1305::KeyInit;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XChaCha20Poly1305; // Support nonces de 24 bytes
use rand::RngCore;
use base64ct::{Encoding, Base64Unpadded};
use uuid::Uuid;

// === CRYPTO - Compatible libsodium (crypto_secretbox / XChaCha20-Poly1305) ===
// Note: libsodium utilise XChaCha20-Poly1305 avec des nonces de 24 bytes

const CRYPTO_SECRETBOX_NONCEBYTES: usize = 24;
const CRYPTO_SECRETBOX_KEYBYTES: usize = 32;
const CRYPTO_SECRETBOX_MACBYTES: usize = 16;

// Configuration
const FILE_EXPIRATION_HOURS: u64 = 48;
const CLEANUP_INTERVAL_HOURS: u64 = 1;

/// Génère une clé aléatoire (compatible sodium.randombytes_buf)
fn crypto_secretbox_keygen() -> Vec<u8> {
    let mut key = vec![0u8; CRYPTO_SECRETBOX_KEYBYTES];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Génère un nonce aléatoire (compatible sodium.randombytes_buf)
fn crypto_secretbox_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; CRYPTO_SECRETBOX_NONCEBYTES];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// Chiffrement (compatible sodium.crypto_secretbox_easy)
/// Retourne: nonce || ciphertext
/// Utilise XChaCha20-Poly1305 pour compatibilité avec libsodium (nonces 24 bytes)
fn crypto_secretbox_easy(message: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(nonce.len() + message.len() + CRYPTO_SECRETBOX_MACBYTES);
    result.extend_from_slice(nonce);

    // Utiliser XChaCha20Poly1305 qui supporte des nonces de 24 bytes (comme libsodium)
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .expect("Clé invalide");

    let nonce_array = GenericArray::from_slice(nonce);

    let encrypted = cipher.encrypt(nonce_array, message)
        .expect("Échec du chiffrement");

    result.extend_from_slice(&encrypted);
    result
}

/// Déchiffrement (compatible sodium.crypto_secretbox_open_easy)
#[allow(dead_code)]
fn crypto_secretbox_open_easy(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
    if ciphertext.len() < CRYPTO_SECRETBOX_NONCEBYTES + CRYPTO_SECRETBOX_MACBYTES {
        return Err("Ciphertext trop court");
    }

    let nonce = &ciphertext[0..CRYPTO_SECRETBOX_NONCEBYTES];
    let encrypted = &ciphertext[CRYPTO_SECRETBOX_NONCEBYTES..];

    // Utiliser XChaCha20Poly1305 qui supporte des nonces de 24 bytes (comme libsodium)
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| "Clé invalide")?;

    let nonce_array = GenericArray::from_slice(nonce);

    cipher.decrypt(nonce_array, encrypted)
        .map_err(|_| "Échec du déchiffrement")
}

/// Encodage base64 (compatible sodium.to_base64)
#[allow(dead_code)]
pub fn to_base64(data: &[u8]) -> String {
    Base64Unpadded::encode_string(data)
}

/// Décodage base64 (compatible sodium.from_base64)
pub fn from_base64(encoded: &str) -> Result<Vec<u8>, &'static str> {
    Base64Unpadded::decode_vec(encoded).map_err(|_| "Base64 invalide")
}

// === STRUCTURES ===

pub type BroadcastSender = broadcast::Sender<String>;
pub type SharedCallState = Arc<Mutex<HashMap<Uuid, BroadcastSender>>>;

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

// Structure pour suivre les fichiers uploadés (avec date d'expiration)
#[derive(Clone)]
#[allow(dead_code)]
struct TrackedFile {
    file_id: String,
    path: PathBuf,
    uploaded_at: SystemTime,
    expires_at: SystemTime,
}

#[derive(Clone)]
pub struct FileManager {
    tracked_files: Arc<Mutex<Vec<TrackedFile>>>,
    uploads_dir: PathBuf,
}

impl FileManager {
    pub fn new(uploads_dir: PathBuf) -> Self {
        Self {
            tracked_files: Arc::new(Mutex::new(Vec::new())),
            uploads_dir,
        }
    }

    /// Méthode publique pour accéder à uploads_dir
    pub fn get_uploads_dir(&self) -> &PathBuf {
        &self.uploads_dir
    }

    /// Enregistre un nouveau fichier pour suivi
    pub async fn register_file(&self, file_id: &str, path: PathBuf) {
        let now = SystemTime::now();
        let uploaded_at = now;
        let expires_at = uploaded_at + Duration::from_secs(FILE_EXPIRATION_HOURS * 3600);

        let mut files = self.tracked_files.lock().await;
        files.push(TrackedFile {
            file_id: file_id.to_string(),
            path,
            uploaded_at,
            expires_at,
        });

        eprintln!("[FileManager] Fichier {} enregistré, expire dans {}h", file_id, FILE_EXPIRATION_HOURS);
    }

    /// Nettoie les fichiers expirés (à appeler périodiquement)
    pub async fn cleanup_expired_files(&self) -> usize {
        let now = SystemTime::now();
        let mut files = self.tracked_files.lock().await;
        let mut deleted_count = 0;

        let mut i = 0;
        while i < files.len() {
            if files[i].expires_at < now {
                let file = files[i].clone();

                // Supprimer le fichier physique
                if let Err(e) = tokio::fs::remove_file(&file.path).await {
                    eprintln!("[FileManager] Erreur suppression {}: {}", file.file_id, e);
                } else {
                    eprintln!("[FileManager] Fichier expiré supprimé: {} ({} bytes)", 
                        file.file_id, 
                        file.path.metadata().map(|m: std::fs::Metadata| m.len()).unwrap_or(0)
                    );
                    deleted_count += 1;
                }

                files.remove(i);
            } else {
                i += 1;
            }
        }

        deleted_count
    }

    /// Démarre la tâche de nettoyage périodique
    pub async fn start_cleanup_task(self) {
        let mut interval = interval(Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600));

        // Attendre un peu avant le premier nettoyage
        sleep(Duration::from_secs(60)).await;

        loop {
            interval.tick().await;
            let deleted = self.cleanup_expired_files().await;
            if deleted > 0 {
                eprintln!("[FileManager] Nettoyage: {} fichiers expirés supprimés", deleted);
            }
        }
    }
}

// === FONCTIONS PUBLIQUES POUR UPLOAD.RS ===

/// Chiffre un fichier pour stockage sécurisé sur le serveur
/// Retourne: (ciphertext_with_nonce, nonce_base64, key_base64)
pub fn encrypt_file_for_storage(data: &[u8]) -> (Vec<u8>, String, String) {
    let key = crypto_secretbox_keygen();
    let nonce = crypto_secretbox_nonce();
    let ciphertext = crypto_secretbox_easy(data, &key, &nonce);

    (ciphertext, to_base64(&nonce), to_base64(&key))
}

/// Déchiffre un fichier stocké sur le serveur
#[allow(dead_code)]
pub fn decrypt_file_from_storage(ciphertext: &[u8], nonce_base64: &str, key_base64: &str) -> Result<Vec<u8>, &'static str> {
    let nonce = from_base64(nonce_base64)?;
    let key = from_base64(key_base64)?;

    let mut data = Vec::with_capacity(nonce.len() + ciphertext.len());
    data.extend_from_slice(&nonce);
    data.extend_from_slice(ciphertext);

    crypto_secretbox_open_easy(&data, &key)
}

/// Fonction broadcast_message compatible avec upload.rs
#[allow(dead_code)]
pub async fn broadcast_message(
    state: SharedCallState,
    _conversation_id: String,
    _event: String,
    message: String,
) {
    let guard = state.lock().await; 
        for (_, tx) in guard.iter() {
            let _ = tx.send(message.clone());
 }
    }

// === HANDLERS HTTP ===

pub async fn handle_offer(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    AxumJson(payload): AxumJson<Value>,
) -> impl IntoResponse {
    let offer = payload.get("offer").and_then(|o| o.as_str());
    let from_user_id = payload.get("from_user_id").and_then(|u| u.as_str()).unwrap_or("unknown");
    let conversation_id = payload.get("conversation_id").and_then(|c| c.as_str()).unwrap_or("general");

    if let Some(offer_sdp) = offer {
        let response = json!({
            "type": "offer",
            "offer": offer_sdp,
            "from_user_id": from_user_id,
            "conversation_id": conversation_id,
            "timestamp": chrono::Utc::now().timestamp()
        });

    let guard = state.webrtc_state.broadcasts.lock().await;
    for (_, tx) in guard.iter() {
        let _ = tx.send(response.to_string());
    }

        eprintln!("[Signalisation] Offre P2P diffusée pour {}", from_user_id);
        (axum::http::StatusCode::OK, AxumJson(json!({"status": "offer_sent"})))
    } else {
        (axum::http::StatusCode::BAD_REQUEST, AxumJson(json!({"error": "Missing offer"})))
    }
}

pub async fn handle_answer(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    AxumJson(payload): AxumJson<Value>,
) -> impl IntoResponse {
    let answer = payload.get("answer").and_then(|a| a.as_str());
    let from_user_id = payload.get("from_user_id").and_then(|u| u.as_str()).unwrap_or("unknown");
    let conversation_id = payload.get("conversation_id").and_then(|c| c.as_str()).unwrap_or("general");

    if let Some(answer_sdp) = answer {
        let response = json!({
            "type": "answer",
            "answer": answer_sdp,
            "from_user_id": from_user_id,
            "conversation_id": conversation_id,
            "timestamp": chrono::Utc::now().timestamp()
        });

        let guard = state.webrtc_state.broadcasts.lock().await;
        for (_, tx) in guard.iter() {
            let _ = tx.send(response.to_string());
        }

        eprintln!("[Signalisation] Réponse P2P diffusée pour {}", from_user_id);
        (axum::http::StatusCode::OK, AxumJson(json!({"status": "answer_sent"})))
    } else {
        (axum::http::StatusCode::BAD_REQUEST, AxumJson(json!({"error": "Missing answer"})))
    }
}

// === WEBSOCKET ===

pub async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    AxumState(state): AxumState<Arc<crate::SharedState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: Arc<crate::SharedState>) {
    let (mut sender, mut receiver) = socket.split();
    let id = Uuid::new_v4();
    let (broadcast_tx, _) = broadcast::channel::<String>(100);

    // Créer deux clones distincts pour éviter le problème de move
    let broadcast_tx_for_send = broadcast_tx.clone();
    let broadcast_tx_for_receive = broadcast_tx.clone();

    let mut guard = state.webrtc_state.broadcasts.lock().await;
    guard.insert(id, broadcast_tx);
    drop(guard);

    eprintln!("[WebSocket] Client connecté pour signalisation P2P: {}", id);

    let send_task = tokio::spawn(async move {
        let mut rx = broadcast_tx_for_send.subscribe();
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender.send(axum::extract::ws::Message::Text(msg)).await {
                eprintln!("[WebSocket] Erreur d'envoi: {}", e);
                break;
            }
        }
    });

    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    let parse_result = serde_json::from_str::<Value>(&text);

                    match parse_result {
                        Ok(json) => {
                            let msg_type = json.get("type").or(json.get("event"))
                                .and_then(|v| v.as_str()).unwrap_or("unknown");
                            let from_user = json.get("from_user_id").or(json.get("sender_id"))
                                .and_then(|v| v.as_str()).unwrap_or("unknown");
                            eprintln!("[Signalisation] Message {} de {}", msg_type, from_user);
                        },
                        Err(_) => {
                            eprintln!("[WebSocket] Message texte reçu: {} bytes", text.len());
                        }
                    }

                    let _ = broadcast_tx_for_receive.send(text.clone());
                }
                Ok(axum::extract::ws::Message::Binary(data)) => {
                    eprintln!("[WebSocket] Message binaire ignoré (transfert P2P direct): {} bytes", data.len());
                }
                Err(e) => {
                    eprintln!("[WebSocket] Erreur de réception: {}", e);
                    break;
                }
                _ => break,
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    let mut guard = state.webrtc_state.broadcasts.lock().await;
    guard.remove(&id);
    eprintln!("[WebSocket] Client déconnecté: {}", id);
}

// === ROUTES ===

pub fn webrtc_routes() -> Router<Arc<crate::SharedState>> {
    Router::new()
        .route("/api/webrtc/offer", post(handle_offer))
        .route("/api/webrtc/answer", post(handle_answer))
        .route("/ws", get(ws_handler))
}

// === PROTOCOLE E2E P2P (pour documentation du frontend) ===
/*
PROTOCOLE DE TRANSFERT P2P E2E (chiffré) - COMPATIBLE LIBSODIUM:

IMPORTANT: Ce backend utilise XChaCha20-Poly1305 avec des nonces de 24 bytes
pour être 100% compatible avec libsodium côté frontend (crypto_secretbox).

1. EXPÉDITEUR (Client A - libsodium):
   - Génère une clé de session aléatoire (32 bytes): sodium.crypto_secretbox_keygen()
   - Chiffre le fichier avec crypto_secretbox_easy(message, nonce, key)
   - Nonce de 24 bytes généré par sodium.randombytes_buf(24)
   - Pour chaque chunk:
     * Chiffre avec crypto_secretbox_easy(chunk, nonce, key)
     * Envoie via WebRTC DataChannel

2. SIGNALISATION (Serveur - Rust XChaCha20Poly1305):
   - Échange des offres/réponses WebRTC
   - Échange des candidats ICE
   - Ne voit jamais le contenu des fichiers
   - Utilise XChaCha20Poly1305 (nonces 24 bytes) pour compatibilité totale

3. DESTINATAIRE (Client B - libsodium):
   - Reçoit les chunks chiffrés via DataChannel
   - Déchiffre avec crypto_secretbox_open_easy(ciphertext, nonce, key)
   - Reconstruit le fichier original

FORMAT DES MESSAGES P2P:
{
  "event": "file_transfer",
  "file_id": "uuid",
  "file_name": "video.mp4",
  "total_size": 52428800,
  "encrypted": true,
  "chunks": [
    {
      "index": 0,
      "data": "base64_du_chunk_chiffré",
      "nonce": "base64_du_nonce_24_bytes"
    }
  ]
}

COMPATIBILITÉ:
- Frontend (libsodium): crypto_secretbox = XChaCha20-Poly1305 + nonces 24 bytes
- Backend (Rust): XChaCha20Poly1305 + nonces 24 bytes
- 100% interopérable entre JavaScript/libsodium et Rust
*/
