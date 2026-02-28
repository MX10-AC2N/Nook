// backend/src/webrtc.rs
// Signalisation P2P + Chiffrement fichiers (XChaCha20-Poly1305)
// Session 9 — fix sécurité : authentification du WebSocket
//   → le cookie auth_token est vérifié dès la connexion WS
//   → connexion refusée si token invalide ou manquant

use axum::{
    extract::{ws::WebSocket, Json as AxumJson, State as AxumState},
    http::{header::COOKIE, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use base64ct::{Base64Unpadded, Encoding};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::KeyInit;
use chacha20poly1305::XChaCha20Poly1305;
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════
// CRYPTO — Compatible libsodium (XChaCha20-Poly1305, nonces 24 bytes)
// ════════════════════════════════════════════════════════════════

const CRYPTO_SECRETBOX_NONCEBYTES: usize = 24;
const CRYPTO_SECRETBOX_KEYBYTES: usize = 32;
const CRYPTO_SECRETBOX_MACBYTES: usize = 16;

const FILE_EXPIRATION_HOURS: u64 = 48;
const CLEANUP_INTERVAL_HOURS: u64 = 1;

fn crypto_secretbox_keygen() -> Vec<u8> {
    let mut key = vec![0u8; CRYPTO_SECRETBOX_KEYBYTES];
    rand::rng().fill_bytes(&mut key);
    key
}

fn crypto_secretbox_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; CRYPTO_SECRETBOX_NONCEBYTES];
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

fn crypto_secretbox_easy(message: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(nonce.len() + message.len() + CRYPTO_SECRETBOX_MACBYTES);
    result.extend_from_slice(nonce);

    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("Clé invalide");
    let nonce_array = GenericArray::from_slice(nonce);
    let encrypted = cipher
        .encrypt(nonce_array, message)
        .expect("Échec chiffrement");
    result.extend_from_slice(&encrypted);
    result
}

#[allow(dead_code)]
fn crypto_secretbox_open_easy(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
    if ciphertext.len() < CRYPTO_SECRETBOX_NONCEBYTES + CRYPTO_SECRETBOX_MACBYTES {
        return Err("Ciphertext trop court");
    }
    let nonce = &ciphertext[0..CRYPTO_SECRETBOX_NONCEBYTES];
    let encrypted = &ciphertext[CRYPTO_SECRETBOX_NONCEBYTES..];
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| "Clé invalide")?;
    let nonce_array = GenericArray::from_slice(nonce);
    cipher
        .decrypt(nonce_array, encrypted)
        .map_err(|_| "Échec déchiffrement")
}

#[allow(dead_code)]
pub fn to_base64(data: &[u8]) -> String {
    Base64Unpadded::encode_string(data)
}

pub fn from_base64(encoded: &str) -> Result<Vec<u8>, &'static str> {
    Base64Unpadded::decode_vec(encoded).map_err(|_| "Base64 invalide")
}

// ════════════════════════════════════════════════════════════════
// STRUCTURES
// ════════════════════════════════════════════════════════════════

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

    pub fn get_uploads_dir(&self) -> &PathBuf {
        &self.uploads_dir
    }

    pub async fn register_file(&self, file_id: &str, path: PathBuf) {
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(FILE_EXPIRATION_HOURS * 3600);
        let mut files = self.tracked_files.lock().await;
        files.push(TrackedFile {
            file_id: file_id.to_string(),
            path,
            uploaded_at: now,
            expires_at,
        });
        tracing::debug!(file_id = %file_id, expires_in_hours = FILE_EXPIRATION_HOURS, "Fichier enregistré");
    }

    pub async fn cleanup_expired_files(&self) -> usize {
        let now = SystemTime::now();
        let mut files = self.tracked_files.lock().await;
        let mut deleted_count = 0;
        let mut i = 0;
        while i < files.len() {
            if files[i].expires_at < now {
                let file = files[i].clone();
                if let Err(e) = tokio::fs::remove_file(&file.path).await {
                    tracing::warn!(file_id = %file.file_id, error = %e, "Échec suppression fichier expiré");
                } else {
                    deleted_count += 1;
                }
                files.remove(i);
            } else {
                i += 1;
            }
        }
        deleted_count
    }

    pub async fn start_cleanup_task(self) {
        let mut tick = interval(Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600));
        sleep(Duration::from_secs(60)).await;
        loop {
            tick.tick().await;
            let deleted = self.cleanup_expired_files().await;
            if deleted > 0 {
                tracing::info!(count = deleted, "FileManager : fichiers expirés supprimés");
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════
// FONCTIONS PUBLIQUES POUR UPLOAD.RS
// ════════════════════════════════════════════════════════════════

pub fn encrypt_file_for_storage(data: &[u8]) -> (Vec<u8>, String, String) {
    let key = crypto_secretbox_keygen();
    let nonce = crypto_secretbox_nonce();
    let ciphertext = crypto_secretbox_easy(data, &key, &nonce);
    (ciphertext, to_base64(&nonce), to_base64(&key))
}

#[allow(dead_code)]
pub fn decrypt_file_from_storage(
    ciphertext: &[u8],
    nonce_base64: &str,
    key_base64: &str,
) -> Result<Vec<u8>, &'static str> {
    let nonce = from_base64(nonce_base64)?;
    let key = from_base64(key_base64)?;
    let mut data = Vec::with_capacity(nonce.len() + ciphertext.len());
    data.extend_from_slice(&nonce);
    data.extend_from_slice(ciphertext);
    crypto_secretbox_open_easy(&data, &key)
}

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

// ════════════════════════════════════════════════════════════════
// AUTHENTIFICATION WEBSOCKET
// ════════════════════════════════════════════════════════════════

/// Extrait et vérifie le cookie auth_token depuis les headers WS.
/// Retourne Some(user_id) si valide, None sinon.
async fn verify_ws_auth(
    headers: &axum::http::HeaderMap,
    state: &Arc<crate::SharedState>,
) -> Option<String> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;

    let token_value = cookie_header
        .split(';')
        .find(|c| c.trim().starts_with("auth_token="))
        .and_then(|c| c.trim().strip_prefix("auth_token="))?;

    let (user_id, token) = token_value.split_once(':')?;

    if user_id.is_empty() || token.is_empty() {
        return None;
    }

    // Vérification en DB
    let result: Option<(String,)> =
        sqlx::query_as("SELECT id FROM users WHERE id = ? AND token = ? AND approved = 1 LIMIT 1")
            .bind(user_id)
            .bind(token)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    result.map(|(id,)| id)
}

// ════════════════════════════════════════════════════════════════
// HANDLERS HTTP WEBRTC
// ════════════════════════════════════════════════════════════════

pub async fn handle_offer(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    AxumJson(payload): AxumJson<Value>,
) -> impl IntoResponse {
    let offer = payload.get("offer").and_then(|o| o.as_str());
    let from_user_id = payload
        .get("from_user_id")
        .and_then(|u| u.as_str())
        .unwrap_or("unknown");
    let conversation_id = payload
        .get("conversation_id")
        .and_then(|c| c.as_str())
        .unwrap_or("general");

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

        tracing::info!(from = %from_user_id, "Offre WebRTC diffusée");
        (StatusCode::OK, AxumJson(json!({"status": "offer_sent"})))
    } else {
        (
            StatusCode::BAD_REQUEST,
            AxumJson(json!({"error": "Missing offer"})),
        )
    }
}

pub async fn handle_answer(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    AxumJson(payload): AxumJson<Value>,
) -> impl IntoResponse {
    let answer = payload.get("answer").and_then(|a| a.as_str());
    let from_user_id = payload
        .get("from_user_id")
        .and_then(|u| u.as_str())
        .unwrap_or("unknown");
    let conversation_id = payload
        .get("conversation_id")
        .and_then(|c| c.as_str())
        .unwrap_or("general");

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

        tracing::info!(from = %from_user_id, "Réponse WebRTC diffusée");
        (StatusCode::OK, AxumJson(json!({"status": "answer_sent"})))
    } else {
        (
            StatusCode::BAD_REQUEST,
            AxumJson(json!({"error": "Missing answer"})),
        )
    }
}

// ════════════════════════════════════════════════════════════════
// WEBSOCKET — avec vérification d'authentification
// ════════════════════════════════════════════════════════════════

pub async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    // Les headers de la requête HTTP d'upgrade contiennent le cookie
    headers: axum::http::HeaderMap,
    AxumState(state): AxumState<Arc<crate::SharedState>>,
) -> impl IntoResponse {
    // Vérification du cookie avant d'upgrader
    let user_id = verify_ws_auth(&headers, &state).await;

    match user_id {
        Some(uid) => {
            tracing::info!(user_id = %uid, "WebSocket : connexion authentifiée");
            ws.on_upgrade(move |socket| handle_websocket(socket, state, uid))
        }
        None => {
            tracing::warn!("WebSocket : tentative de connexion non authentifiée refusée");
            // on_upgrade ne peut pas retourner une erreur HTTP directement —
            // on refuse l'upgrade en renvoyant 401 sans appeler ws.on_upgrade
            axum::http::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from(
                    "WebSocket : authentification requise",
                ))
                .unwrap()
                .into_response()
        }
    }
}

async fn handle_websocket(socket: WebSocket, state: Arc<crate::SharedState>, user_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let id = Uuid::new_v4();

    let (broadcast_tx, _) = broadcast::channel::<String>(100);
    let broadcast_tx_for_send = broadcast_tx.clone();
    let broadcast_tx_for_receive = broadcast_tx.clone();

    {
        let mut guard = state.webrtc_state.broadcasts.lock().await;
        guard.insert(id, broadcast_tx);
    }

    tracing::info!(ws_id = %id, user_id = %user_id, "WebSocket connecté");

    let send_task = tokio::spawn(async move {
        let mut rx = broadcast_tx_for_send.subscribe();
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender
                .send(axum::extract::ws::Message::Text(msg.into()))
                .await
            {
                tracing::debug!(error = %e, "WebSocket : erreur d'envoi");
                break;
            }
        }
    });

    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    let text_str = text.to_string();

                    // Parse pour logging
                    if let Ok(json) = serde_json::from_str::<Value>(&text_str) {
                        let msg_type = json
                            .get("type")
                            .or(json.get("event"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        tracing::debug!(
                            ws_id = %id,
                            msg_type = %msg_type,
                            "WebSocket : message reçu"
                        );
                    }

                    let _ = broadcast_tx_for_receive.send(text_str);
                }
                Ok(axum::extract::ws::Message::Binary(data)) => {
                    tracing::debug!(
                        bytes = data.len(),
                        "WebSocket : binaire ignoré (P2P direct)"
                    );
                }
                Ok(axum::extract::ws::Message::Close(_)) => {
                    tracing::debug!(ws_id = %id, "WebSocket : fermeture propre");
                    break;
                }
                Err(e) => {
                    tracing::debug!(ws_id = %id, error = %e, "WebSocket : erreur réception");
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    let mut guard = state.webrtc_state.broadcasts.lock().await;
    guard.remove(&id);
    tracing::info!(ws_id = %id, user_id = %user_id, "WebSocket déconnecté");
}

// ════════════════════════════════════════════════════════════════
// ROUTES
// ════════════════════════════════════════════════════════════════

pub fn webrtc_routes() -> Router<Arc<crate::SharedState>> {
    Router::new()
        .route("/api/webrtc/offer", post(handle_offer))
        .route("/api/webrtc/answer", post(handle_answer))
        .route("/ws", get(ws_handler))
    // Note : /ws est authentifié via cookie dans ws_handler lui-même.
    // Les routes /api/webrtc/* sont dans un contexte public (le client authentifié
    // envoie le cookie automatiquement) — à migrer dans protected_routes si besoin.
}
