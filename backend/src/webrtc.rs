// backend/src/webrtc.rs
// Signalisation P2P + Chiffrement fichiers (XChaCha20-Poly1305)
// Session 9  — fix sécurité : authentification du WebSocket
//   → le cookie auth_token est vérifié dès la connexion WS
//   → connexion refusée si token invalide ou manquant
// Session 36 — SEC-05 : limite 64 KB sur les messages WS de signaling

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
/// Mapping user_id → sender pour router les signaux WebRTC vers le bon destinataire.
pub type UserSenderMap = Arc<Mutex<HashMap<String, BroadcastSender>>>;

#[derive(Clone)]
pub struct WebRtcState {
    pub broadcasts: SharedCallState,
    /// Index user_id → canal de broadcast pour le routage des signaux d'appel.
    pub user_senders: UserSenderMap,
}

impl WebRtcState {
    pub fn new() -> Self {
        Self {
            broadcasts: Arc::new(Mutex::new(HashMap::new())),
            user_senders: Arc::new(Mutex::new(HashMap::new())),
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
    _nonce_base64: &str,  // non utilisé : le nonce est déjà intégré dans les premiers bytes du ciphertext
    key_base64: &str,
) -> Result<Vec<u8>, &'static str> {
    // encrypt_file_for_storage stocke nonce||encrypted dans le fichier
    // crypto_secretbox_open_easy sépare lui-même nonce[0..24] du reste
    let key = from_base64(key_base64)?;
    crypto_secretbox_open_easy(ciphertext, &key)
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
    headers: axum::http::HeaderMap,
    AxumJson(payload): AxumJson<Value>,
) -> impl IntoResponse {
    // FIX M3: verifier l'authentification avant de traiter l'offre
    let Some(user_id) = verify_ws_auth(&headers, &state).await else {
        return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifie"}))).into_response();
    };

    // Use authenticated user_id instead of trusting payload
    let from_user_id = &user_id;
    let offer = payload.get("offer").and_then(|o| o.as_str());
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

        tracing::info!(from = %from_user_id, "Offre WebRTC diffusee");
        (StatusCode::OK, AxumJson(json!({"status": "offer_sent"})))
    } else {
        (
            StatusCode::BAD_REQUEST,
            AxumJson(json!({"error": "Missing offer"})),
        )
    }
    .into_response()
}

pub async fn handle_answer(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    headers: axum::http::HeaderMap,
    AxumJson(payload): AxumJson<Value>,
) -> impl IntoResponse {
    // FIX M3: verifier l'authentification avant de traiter la reponse
    let Some(user_id) = verify_ws_auth(&headers, &state).await else {
        return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifie"}))).into_response();
    };

    // Use authenticated user_id instead of trusting payload
    let from_user_id = &user_id;
    let answer = payload.get("answer").and_then(|a| a.as_str());
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

        tracing::info!(from = %from_user_id, "Reponse WebRTC diffusee");
        (StatusCode::OK, AxumJson(json!({"status": "answer_sent"})))
    } else {
        (
            StatusCode::BAD_REQUEST,
            AxumJson(json!({"error": "Missing answer"})),
        )
    }
    .into_response()
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

    // Enregistrer dans les deux maps : uuid→sender (broadcast chat) et user_id→sender (signaling)
    {
        let mut guard = state.webrtc_state.broadcasts.lock().await;
        guard.insert(id, broadcast_tx.clone());
    }
    {
        let mut guard = state.webrtc_state.user_senders.lock().await;
        guard.insert(user_id.clone(), broadcast_tx);
    }

    // Marquer l'utilisateur comme en ligne
    state.presence_state.set_online(&user_id).await;

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

    let state_recv = state.clone();
    let user_id_recv = user_id.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    let text_str = text.to_string();

                    // SEC-05 : limite 64 KB
                    if text_str.len() > 65_536 {
                        tracing::warn!(ws_id = %id, bytes = text_str.len(), "WebSocket : message trop volumineux — ignoré");
                        continue;
                    }

                    // Parser le message pour extraire type et to_user_id
                    let json_val = match serde_json::from_str::<Value>(&text_str) {
                        Ok(v) => v,
                        Err(_) => {
                            // Non-JSON : broadcast global (rétrocompatibilité)
                            let _ = broadcast_tx_for_receive.send(text_str);
                            continue;
                        }
                    };

                    let msg_type = json_val
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    tracing::debug!(ws_id = %id, user_id = %user_id_recv, msg_type = %msg_type, "WebSocket : message reçu");

                    // ── Typing indicator ───────────────────────────────────────
                    if msg_type == "typing" || msg_type == "stop_typing" {
                        let conv_id = json_val
                            .get("conversation_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        // Broadcast to all other users
                        let typing_msg = serde_json::json!({
                            "type": msg_type,
                            "user_id": user_id_recv,
                            "conversation_id": conv_id,
                        }).to_string();

                        let guard = state_recv.webrtc_state.broadcasts.lock().await;
                        for (ws_id, tx) in guard.iter() {
                            if *ws_id != id { // Don't send to self
                                let _ = tx.send(typing_msg.clone());
                            }
                        }
                        continue;
                    }

                    // ── Routage des signaux WebRTC par to_user_id ─────────────────
                    // Types d'appel : offer, answer, ice, join, leave, decline,
                    //                 call_request, call_accepted, call_rejected
                    let webrtc_types = ["offer", "answer", "ice", "ice_candidate",
                        "join", "leave", "decline",
                        "call_request", "call_accepted", "call_rejected",
                        "webrtc_offer", "webrtc_answer", "webrtc_ice_candidate"];

                    // ── Enregistrer les appels manqués ─────────────────────────
                    if msg_type == "decline" || msg_type == "call_rejected" {
                        let conversation_id = json_val
                            .get("conversationId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(); // Clone to owned String
                        let call_type = json_val
                            .get("callType")
                            .and_then(|v| v.as_str())
                            .unwrap_or("audio")
                            .to_string(); // Clone to owned String
                        
                        if !conversation_id.is_empty() {
                            let pool = state_recv.db.clone();
                            let caller_id = user_id_recv.clone();
                            let callee_id = json_val
                                .get("to_user_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            
                            if !callee_id.is_empty() {
                                let status = if msg_type == "decline" { "declined" } else { "missed" };
                                tokio::spawn(async move {
                                    if let Err(e) = crate::missed_calls::record_missed_call(
                                        &pool,
                                        &conversation_id,
                                        &caller_id,
                                        &callee_id,
                                        &call_type,
                                        status,
                                    ).await {
                                        tracing::error!(error = %e, "Erreur enregistrement appel manqué");
                                    }
                                });
                            }
                        }
                    }

                    if webrtc_types.contains(&msg_type) {
                        let to_user_id = json_val
                            .get("to_user_id")
                            .and_then(|v| v.as_str());

                        match to_user_id {
                            Some(target) if !target.is_empty() => {
                                let guard = state_recv.webrtc_state.user_senders.lock().await;
                                if let Some(target_tx) = guard.get(target) {
                                    let _ = target_tx.send(text_str.clone());
                                    tracing::debug!(
                                        from = %user_id_recv,
                                        to = %target,
                                        msg_type = %msg_type,
                                        "WebRTC signal routé"
                                    );
                                } else {
                                    tracing::warn!(
                                        to = %target,
                                        msg_type = %msg_type,
                                        "WebRTC signal : destinataire non connecté"
                                    );
                                }
                            }
                            _ => {
                                let guard = state_recv.webrtc_state.broadcasts.lock().await;
                                for (_, tx) in guard.iter() {
                                    let _ = tx.send(text_str.clone());
                                }
                            }
                        }
                    } else if msg_type == "sfu_join" {
                        if let Some(conv_id) = json_val.get("conversation_id").and_then(|c| c.as_str()) {
                            if let Some(offer) = json_val.get("sdp").and_then(|o| o.as_str()) {
                                match state_recv.sfu_state.handle_join(&user_id_recv, conv_id, offer).await {
                                    Ok(resp) => {
                                        let resp_msg = serde_json::json!({
                                            "type": "sfu_answer",
                                            "answer": resp.answer,
                                            "peers": resp.peers,
                                            "renegotiate_offer": resp.renegotiate_offer,
                                        }).to_string();
                                        let _ = broadcast_tx_for_receive.send(resp_msg);
                                    }
                                    Err(e) => {
                                        let err_msg = serde_json::json!({
                                            "type": "sfu_error",
                                            "error": e,
                                        }).to_string();
                                        let _ = broadcast_tx_for_receive.send(err_msg);
                                    }
                                }
                            }
                        }
                    } else if msg_type == "sfu_answer" {
                        if let Some(conv_id) = json_val.get("conversation_id").and_then(|c| c.as_str()) {
                            if let Some(answer) = json_val.get("sdp").and_then(|s| s.as_str()) {
                                let _ = state_recv.sfu_state.handle_answer(&user_id_recv, conv_id, answer).await;
                            }
                        }
                    } else if msg_type == "sfu_candidate" {
                        if let Some(conv_id) = json_val.get("conversation_id").and_then(|c| c.as_str()) {
                            if let Some(candidate) = json_val.get("candidate").and_then(|c| c.as_str()) {
                                let _ = state_recv.sfu_state.handle_candidate(&user_id_recv, conv_id, candidate).await;
                            }
                        }
                    } else if msg_type == "sfu_leave" {
                        if let Some(conv_id) = json_val.get("conversation_id").and_then(|c| c.as_str()) {
                            let result = state_recv.sfu_state.remove_peer(&user_id_recv, conv_id).await;
                            if let Ok(remaining) = result {
                                let msg = serde_json::json!({
                                    "type": "sfu_peers",
                                    "peers": remaining,
                                }).to_string();
                                let _ = broadcast_tx_for_receive.send(msg);
                            }
                        }
                    } else {
                        let _ = broadcast_tx_for_receive.send(text_str);
                    }
                }

                Ok(axum::extract::ws::Message::Binary(data)) => {
                    tracing::debug!(bytes = data.len(), "WebSocket : binaire ignoré (P2P direct)");
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

    // Nettoyage des deux maps
    {
        let mut guard = state.webrtc_state.broadcasts.lock().await;
        guard.remove(&id);
    }
    {
        let mut guard = state.webrtc_state.user_senders.lock().await;
        guard.remove(&user_id);
    }
    
    // Marquer l'utilisateur comme hors ligne
    state.presence_state.set_offline(&user_id).await;
    
    tracing::info!(ws_id = %id, user_id = %user_id, "WebSocket déconnecté");
}

// ════════════════════════════════════════════════════════════════
// ROUTES
// ════════════════════════════════════════════════════════════════


// ════════════════════════════════════════════════════════════════
// ICE CONFIG — Returns TURN/STUN credentials (short-lived)
// This replaces the hardcoded TURN_SECRET in the frontend bundle
// ════════════════════════════════════════════════════════════════

/// GET /api/webrtc/ice-config
/// Returns ICE server configuration with short-lived TURN credentials.
/// Requires authentication via cookie (same pattern as verify_ws_auth).
async fn handle_ice_config(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Auth check — same pattern as verify_ws_auth
    let cookie_header = match headers.get(COOKIE).and_then(|v| v.to_str().ok()) {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response(),
    };

    let token_value = match cookie_header
        .split(';')
        .find(|c| c.trim().starts_with("auth_token="))
        .and_then(|c| c.trim().strip_prefix("auth_token="))
    {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response(),
    };

    let (user_id, token) = match token_value.split_once(':') {
        Some((u, t)) if !u.is_empty() && !t.is_empty() => (u, t),
        _ => return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response(),
    };

    // DB verification
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND token = ? AND approved = 1)"
    )
        .bind(user_id)
        .bind(token)
        .fetch_one(&state.db)
        .await
        .map(|v: i64| v == 1)
        .unwrap_or(false);

    if !valid {
        return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response();
    }

    let config = &state.config;

    if config.turn_secret.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(json!({"error": "TURN_SECRET non configuré"})),
        ).into_response();
    }

    // Resolve TURN host: explicit config > request Host header > localhost
    let turn_host = if config.turn_host.is_empty() || config.turn_host == "localhost" {
        headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.split(':').next())
            .unwrap_or("localhost")
            .to_string()
    } else {
        config.turn_host.clone()
    };

    // Generate short-lived TURN credentials (RFC 5389 — 24h validity)
    let validity_hours: u64 = 24;
    let username = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + (validity_hours * 3600);

    // HMAC-SHA1 credential generation (compatible with turn-rs)
    use sha1::Sha1;
    use hmac::{Hmac, Mac};
    type HmacSha1 = Hmac<Sha1>;

    let mut mac = <HmacSha1 as hmac::Mac>::new_from_slice(config.turn_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(username.to_string().as_bytes());
    let credential = base64ct::Base64Unpadded::encode_string(&mac.finalize().into_bytes());

    let response = json!({
        "host": turn_host,
        "port": config.turn_port,
        "username": username.to_string(),
        "credential": credential,
    });

    (StatusCode::OK, AxumJson(response)).into_response()
}

/// Routes API WebRTC — à merger dans protected_routes (auth requise).
pub fn webrtc_api_routes() -> Router<Arc<crate::SharedState>> {
    Router::new()
        .route("/webrtc/ice-config", get(handle_ice_config))
        .route("/webrtc/offer", post(handle_offer))
        .route("/webrtc/answer", post(handle_answer))
}

/// Route WebSocket — authentifiée via cookie dans ws_handler lui-même.
/// Doit rester hors middleware auth pour ne pas bloquer l'upgrade WebSocket.
pub fn webrtc_ws_routes() -> Router<Arc<crate::SharedState>> {
    Router::new()
        .route("/ws", get(ws_handler))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secretbox_keygen_length() {
        let key = crypto_secretbox_keygen();
        assert_eq!(key.len(), CRYPTO_SECRETBOX_KEYBYTES);
    }

    #[test]
    fn test_secretbox_keygen_unique() {
        let key1 = crypto_secretbox_keygen();
        let key2 = crypto_secretbox_keygen();
        assert_ne!(key1, key2, "Keys should be unique");
    }

    #[test]
    fn test_secretbox_nonce_length() {
        let nonce = crypto_secretbox_nonce();
        assert_eq!(nonce.len(), CRYPTO_SECRETBOX_NONCEBYTES);
    }

    #[test]
    fn test_secretbox_encrypt_decrypt() {
        let key = crypto_secretbox_keygen();
        let nonce = crypto_secretbox_nonce();
        let message = b"Hello, Nook!";

        let ciphertext = crypto_secretbox_easy(message, &key, &nonce);
        let plaintext = crypto_secretbox_open_easy(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, message);
    }

    #[test]
    fn test_secretbox_wrong_key_fails() {
        let key1 = crypto_secretbox_keygen();
        let key2 = crypto_secretbox_keygen();
        let nonce = crypto_secretbox_nonce();
        let message = b"Secret message";

        let ciphertext = crypto_secretbox_easy(message, &key1, &nonce);
        let result = crypto_secretbox_open_easy(&ciphertext, &key2);

        assert!(result.is_err(), "Decryption with wrong key should fail");
    }

    #[test]
    fn test_secretbox_too_short_fails() {
        let key = crypto_secretbox_keygen();
        let short = vec![0u8; 10]; // Too short

        let result = crypto_secretbox_open_easy(&short, &key);
        assert!(result.is_err(), "Too short ciphertext should fail");
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"test data for base64";
        let encoded = to_base64(data);
        let decoded = from_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_invalid() {
        let result = from_base64("!!!invalid!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_empty() {
        let encoded = to_base64(b"");
        assert_eq!(encoded, "");
        let decoded = from_base64(&encoded).unwrap();
        assert_eq!(decoded, b"");
    }

    #[test]
    fn test_constants() {
        assert_eq!(CRYPTO_SECRETBOX_NONCEBYTES, 24);
        assert_eq!(CRYPTO_SECRETBOX_KEYBYTES, 32);
        assert_eq!(CRYPTO_SECRETBOX_MACBYTES, 16);
    }
}
