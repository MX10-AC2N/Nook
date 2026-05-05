     1|// backend/src/webrtc.rs
     2|// Signalisation P2P + Chiffrement fichiers (XChaCha20-Poly1305)
     3|// Session 9  — fix sécurité : authentification du WebSocket
     4|//   → le cookie auth_token est vérifié dès la connexion WS
     5|//   → connexion refusée si token invalide ou manquant
     6|// Session 36 — SEC-05 : limite 64 KB sur les messages WS de signaling
     7|
     8|#![allow(clippy::for_kv_map)]
     9|
    10|use axum::{
    11|    extract::{ws::WebSocket, Json as AxumJson, State as AxumState},
    12|    http::{header::COOKIE, StatusCode},
    13|    response::IntoResponse,
    14|    routing::{get, post},
    15|    Router,
    16|};
    17|use base64ct::{Base64Unpadded, Encoding};
    18|use chacha20poly1305::aead::generic_array::GenericArray;
    19|use chacha20poly1305::aead::Aead;
    20|use chacha20poly1305::KeyInit;
    21|use chacha20poly1305::XChaCha20Poly1305;
    22|use futures_util::{SinkExt, StreamExt};
    23|use rand::RngCore;
    24|use serde_json::{json, Value};
    25|use std::{
    26|    collections::HashMap,
    27|    path::PathBuf,
    28|    sync::Arc,
    29|    time::{Duration, SystemTime},
    30|};
    31|use tokio::sync::broadcast;
    32|use tokio::sync::Mutex;
    33|use tokio::time::{interval, sleep};
    34|use uuid::Uuid;
    35|
    36|// ════════════════════════════════════════════════════════════════
    37|// CRYPTO — Compatible libsodium (XChaCha20-Poly1305, nonces 24 bytes)
    38|// ════════════════════════════════════════════════════════════════
    39|
    40|const CRYPTO_SECRETBOX_NONCEBYTES: usize = 24;
    41|const CRYPTO_SECRETBOX_KEYBYTES: usize = 32;
    42|const CRYPTO_SECRETBOX_MACBYTES: usize = 16;
    43|
    44|const FILE_EXPIRATION_HOURS: u64 = 48;
    45|const CLEANUP_INTERVAL_HOURS: u64 = 1;
    46|
    47|fn crypto_secretbox_keygen() -> Vec<u8> {
    48|    let mut key = vec![0u8; CRYPTO_SECRETBOX_KEYBYTES];
    49|    rand::rng().fill_bytes(&mut key);
    50|    key
    51|}
    52|
    53|fn crypto_secretbox_nonce() -> Vec<u8> {
    54|    let mut nonce = vec![0u8; CRYPTO_SECRETBOX_NONCEBYTES];
    55|    rand::rng().fill_bytes(&mut nonce);
    56|    nonce
    57|}
    58|
    59|fn crypto_secretbox_easy(message: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    60|    let mut result = Vec::with_capacity(nonce.len() + message.len() + CRYPTO_SECRETBOX_MACBYTES);
    61|    result.extend_from_slice(nonce);
    62|
    63|    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("Clé invalide");
    64|    let nonce_array = GenericArray::from_slice(nonce);
    65|    let encrypted = cipher
    66|        .encrypt(nonce_array, message)
    67|        .expect("Échec chiffrement");
    68|    result.extend_from_slice(&encrypted);
    69|    result
    70|}
    71|
    72|#[allow(dead_code)]
    73|fn crypto_secretbox_open_easy(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, &'static str> {
    74|    if ciphertext.len() < CRYPTO_SECRETBOX_NONCEBYTES + CRYPTO_SECRETBOX_MACBYTES {
    75|        return Err("Ciphertext trop court");
    76|    }
    77|    let nonce = &ciphertext[0..CRYPTO_SECRETBOX_NONCEBYTES];
    78|    let encrypted = &ciphertext[CRYPTO_SECRETBOX_NONCEBYTES..];
    79|    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| "Clé invalide")?;
    80|    let nonce_array = GenericArray::from_slice(nonce);
    81|    cipher
    82|        .decrypt(nonce_array, encrypted)
    83|        .map_err(|_| "Échec déchiffrement")
    84|}
    85|
    86|#[allow(dead_code)]
    87|pub fn to_base64(data: &[u8]) -> String {
    88|    Base64Unpadded::encode_string(data)
    89|}
    90|
    91|pub fn from_base64(encoded: &str) -> Result<Vec<u8>, &'static str> {
    92|    Base64Unpadded::decode_vec(encoded).map_err(|_| "Base64 invalide")
    93|}
    94|
    95|// ════════════════════════════════════════════════════════════════
    96|// STRUCTURES
    97|// ════════════════════════════════════════════════════════════════
    98|
    99|pub type BroadcastSender = broadcast::Sender<String>;
   100|pub type SharedCallState = Arc<Mutex<HashMap<Uuid, BroadcastSender>>>;
   101|/// Mapping user_id → sender pour router les signaux WebRTC vers le bon destinataire.
   102|pub type UserSenderMap = Arc<Mutex<HashMap<String, BroadcastSender>>>;
   103|
   104|#[derive(Clone)]
   105|pub struct WebRtcState {
   106|    pub broadcasts: SharedCallState,
   107|    /// Index user_id → canal de broadcast pour le routage des signaux d'appel.
   108|    pub user_senders: UserSenderMap,
   109|}
   110|
   111|impl WebRtcState {
   112|    pub fn new() -> Self {
   113|        Self {
   114|            broadcasts: Arc::new(Mutex::new(HashMap::new())),
   115|            user_senders: Arc::new(Mutex::new(HashMap::new())),
   116|        }
   117|    }
   118|}
   119|
   120|#[derive(Clone)]
   121|#[allow(dead_code)]
   122|struct TrackedFile {
   123|    file_id: String,
   124|    path: PathBuf,
   125|    uploaded_at: SystemTime,
   126|    expires_at: SystemTime,
   127|}
   128|
   129|#[derive(Clone)]
   130|pub struct FileManager {
   131|    tracked_files: Arc<Mutex<Vec<TrackedFile>>>,
   132|    uploads_dir: PathBuf,
   133|}
   134|
   135|impl FileManager {
   136|    pub fn new(uploads_dir: PathBuf) -> Self {
   137|        Self {
   138|            tracked_files: Arc::new(Mutex::new(Vec::new())),
   139|            uploads_dir,
   140|        }
   141|    }
   142|
   143|    pub fn get_uploads_dir(&self) -> &PathBuf {
   144|        &self.uploads_dir
   145|    }
   146|
   147|    pub async fn register_file(&self, file_id: &str, path: PathBuf) {
   148|        let now = SystemTime::now();
   149|        let expires_at = now + Duration::from_secs(FILE_EXPIRATION_HOURS * 3600);
   150|        let mut files = self.tracked_files.lock().await;
   151|        files.push(TrackedFile {
   152|            file_id: file_id.to_string(),
   153|            path,
   154|            uploaded_at: now,
   155|            expires_at,
   156|        });
   157|        tracing::debug!(file_id = %file_id, expires_in_hours = FILE_EXPIRATION_HOURS, "Fichier enregistré");
   158|    }
   159|
   160|    pub async fn cleanup_expired_files(&self) -> usize {
   161|        let now = SystemTime::now();
   162|        let mut files = self.tracked_files.lock().await;
   163|        let mut deleted_count = 0;
   164|        let mut i = 0;
   165|        while i < files.len() {
   166|            if files[i].expires_at < now {
   167|                let file = files[i].clone();
   168|                if let Err(e) = tokio::fs::remove_file(&file.path).await {
   169|                    tracing::warn!(file_id = %file.file_id, error = %e, "Échec suppression fichier expiré");
   170|                } else {
   171|                    deleted_count += 1;
   172|                }
   173|                files.remove(i);
   174|            } else {
   175|                i += 1;
   176|            }
   177|        }
   178|        deleted_count
   179|    }
   180|
   181|    pub async fn start_cleanup_task(self) {
   182|        let mut tick = interval(Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600));
   183|        sleep(Duration::from_secs(60)).await;
   184|        loop {
   185|            tick.tick().await;
   186|            let deleted = self.cleanup_expired_files().await;
   187|            if deleted > 0 {
   188|                tracing::info!(count = deleted, "FileManager : fichiers expirés supprimés");
   189|            }
   190|        }
   191|    }
   192|}
   193|
   194|// ════════════════════════════════════════════════════════════════
   195|// FONCTIONS PUBLIQUES POUR UPLOAD.RS
   196|// ════════════════════════════════════════════════════════════════
   197|
   198|pub fn encrypt_file_for_storage(data: &[u8]) -> (Vec<u8>, String, String) {
   199|    let key = crypto_secretbox_keygen();
   200|    let nonce = crypto_secretbox_nonce();
   201|    let ciphertext = crypto_secretbox_easy(data, &key, &nonce);
   202|    (ciphertext, to_base64(&nonce), to_base64(&key))
   203|}
   204|
   205|#[allow(dead_code)]
   206|pub fn decrypt_file_from_storage(
   207|    ciphertext: &[u8],
   208|    _nonce_base64: &str,  // non utilisé : le nonce est déjà intégré dans les premiers bytes du ciphertext
   209|    key_base64: &str,
   210|) -> Result<Vec<u8>, &'static str> {
   211|    // encrypt_file_for_storage stocke nonce||encrypted dans le fichier
   212|    // crypto_secretbox_open_easy sépare lui-même nonce[0..24] du reste
   213|    let key = from_base64(key_base64)?;
   214|    crypto_secretbox_open_easy(ciphertext, &key)
   215|}
   216|
   217|#[allow(dead_code)]
   218|pub async fn broadcast_message(
   219|    state: SharedCallState,
   220|    _conversation_id: String,
   221|    _event: String,
   222|    message: String,
   223|) {
   224|    let guard = state.lock().await;
   225|    for tx in guard.values() {
   226|        let _ = tx.send(message.clone());
   227|    }
   228|}
   229|
   230|// ════════════════════════════════════════════════════════════════
   231|// AUTHENTIFICATION WEBSOCKET
   232|// ════════════════════════════════════════════════════════════════
   233|
   234|/// Extrait et vérifie le cookie auth_token depuis les headers WS.
   235|/// Retourne Some(user_id) si valide, None sinon.
   236|async fn verify_ws_auth(
   237|    headers: &axum::http::HeaderMap,
   238|    state: &Arc<crate::SharedState>,
   239|) -> Option<String> {
   240|    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
   241|
   242|    let token_value = cookie_header
   243|        .split(';')
   244|        .find(|c| c.trim().starts_with("auth_token="))
   245|        .and_then(|c| c.trim().strip_prefix("auth_token="))?;
   246|
   247|    let (user_id, token) = token_value.split_once(':')?;
   248|
   249|    if user_id.is_empty() || token.is_empty() {
   250|        return None;
   251|    }
   252|
   253|    // Vérification en DB
   254|    let result: Option<(String,)> =
   255|        sqlx::query_as("SELECT id FROM users WHERE id = ? AND token = ? AND approved = 1 LIMIT 1")
   256|            .bind(user_id)
   257|            .bind(token)
   258|            .fetch_optional(&state.db)
   259|            .await
   260|            .ok()
   261|            .flatten();
   262|
   263|    result.map(|(id,)| id)
   264|}
   265|
   266|// ════════════════════════════════════════════════════════════════
   267|// HANDLERS HTTP WEBRTC
   268|// ════════════════════════════════════════════════════════════════
   269|
   270|pub async fn handle_offer(
   271|    AxumState(state): AxumState<Arc<crate::SharedState>>,
   272|    AxumJson(payload): AxumJson<Value>,
   273|) -> impl IntoResponse {
   274|    let offer = payload.get("offer").and_then(|o| o.as_str());
   275|    let from_user_id = payload
   276|        .get("from_user_id")
   277|        .and_then(|u| u.as_str())
   278|        .unwrap_or("unknown");
   279|    let conversation_id = payload
   280|        .get("conversation_id")
   281|        .and_then(|c| c.as_str())
   282|        .unwrap_or("general");
   283|
   284|    if let Some(offer_sdp) = offer {
   285|        let response = json!({
   286|            "type": "offer",
   287|            "offer": offer_sdp,
   288|            "from_user_id": from_user_id,
   289|            "conversation_id": conversation_id,
   290|            "timestamp": chrono::Utc::now().timestamp()
   291|        });
   292|
   293|        let guard = state.webrtc_state.broadcasts.lock().await;
   294|        for tx in guard.values() {
   295|            let _ = tx.send(response.to_string());
   296|        }
   297|
   298|        tracing::info!(from = %from_user_id, "Offre WebRTC diffusée");
   299|        (StatusCode::OK, AxumJson(json!({"status": "offer_sent"})))
   300|    } else {
   301|        (
   302|            StatusCode::BAD_REQUEST,
   303|            AxumJson(json!({"error": "Missing offer"})),
   304|        )
   305|    }
   306|}
   307|
   308|pub async fn handle_answer(
   309|    AxumState(state): AxumState<Arc<crate::SharedState>>,
   310|    AxumJson(payload): AxumJson<Value>,
   311|) -> impl IntoResponse {
   312|    let answer = payload.get("answer").and_then(|a| a.as_str());
   313|    let from_user_id = payload
   314|        .get("from_user_id")
   315|        .and_then(|u| u.as_str())
   316|        .unwrap_or("unknown");
   317|    let conversation_id = payload
   318|        .get("conversation_id")
   319|        .and_then(|c| c.as_str())
   320|        .unwrap_or("general");
   321|
   322|    if let Some(answer_sdp) = answer {
   323|        let response = json!({
   324|            "type": "answer",
   325|            "answer": answer_sdp,
   326|            "from_user_id": from_user_id,
   327|            "conversation_id": conversation_id,
   328|            "timestamp": chrono::Utc::now().timestamp()
   329|        });
   330|
   331|        let guard = state.webrtc_state.broadcasts.lock().await;
   332|        for tx in guard.values() {
   333|            let _ = tx.send(response.to_string());
   334|        }
   335|
   336|        tracing::info!(from = %from_user_id, "Réponse WebRTC diffusée");
   337|        (StatusCode::OK, AxumJson(json!({"status": "answer_sent"})))
   338|    } else {
   339|        (
   340|            StatusCode::BAD_REQUEST,
   341|            AxumJson(json!({"error": "Missing answer"})),
   342|        )
   343|    }
   344|}
   345|
   346|// ════════════════════════════════════════════════════════════════
   347|// WEBSOCKET — avec vérification d'authentification
   348|// ════════════════════════════════════════════════════════════════
   349|
   350|pub async fn ws_handler(
   351|    ws: axum::extract::ws::WebSocketUpgrade,
   352|    // Les headers de la requête HTTP d'upgrade contiennent le cookie
   353|    headers: axum::http::HeaderMap,
   354|    AxumState(state): AxumState<Arc<crate::SharedState>>,
   355|) -> impl IntoResponse {
   356|    // Vérification du cookie avant d'upgrader
   357|    let user_id = verify_ws_auth(&headers, &state).await;
   358|
   359|    match user_id {
   360|        Some(uid) => {
   361|            tracing::info!(user_id = %uid, "WebSocket : connexion authentifiée");
   362|            ws.on_upgrade(move |socket| handle_websocket(socket, state, uid))
   363|        }
   364|        None => {
   365|            tracing::warn!("WebSocket : tentative de connexion non authentifiée refusée");
   366|            // on_upgrade ne peut pas retourner une erreur HTTP directement —
   367|            // on refuse l'upgrade en renvoyant 401 sans appeler ws.on_upgrade
   368|            axum::http::Response::builder()
   369|                .status(StatusCode::UNAUTHORIZED)
   370|                .body(axum::body::Body::from(
   371|                    "WebSocket : authentification requise",
   372|                ))
   373|                .unwrap()
   374|                .into_response()
   375|        }
   376|    }
   377|}
   378|
   379|async fn handle_websocket(socket: WebSocket, state: Arc<crate::SharedState>, user_id: String) {
   380|    let (mut sender, mut receiver) = socket.split();
   381|    let id = Uuid::new_v4();
   382|
   383|    let (broadcast_tx, _) = broadcast::channel::<String>(100);
   384|    let broadcast_tx_for_send = broadcast_tx.clone();
   385|    let broadcast_tx_for_receive = broadcast_tx.clone();
   386|
   387|    // Enregistrer dans les deux maps : uuid→sender (broadcast chat) et user_id→sender (signaling)
   388|    {
   389|        let mut guard = state.webrtc_state.broadcasts.lock().await;
   390|        guard.insert(id, broadcast_tx.clone());
   391|    }
   392|    {
   393|        let mut guard = state.webrtc_state.user_senders.lock().await;
   394|        guard.insert(user_id.clone(), broadcast_tx);
   395|    }
   396|
   397|    tracing::info!(ws_id = %id, user_id = %user_id, "WebSocket connecté");
   398|
   399|    let send_task = tokio::spawn(async move {
   400|        let mut rx = broadcast_tx_for_send.subscribe();
   401|        while let Ok(msg) = rx.recv().await {
   402|            if let Err(e) = sender
   403|                .send(axum::extract::ws::Message::Text(msg.into()))
   404|                .await
   405|            {
   406|                tracing::debug!(error = %e, "WebSocket : erreur d'envoi");
   407|                break;
   408|            }
   409|        }
   410|    });
   411|
   412|    let state_recv = state.clone();
   413|    let user_id_recv = user_id.clone();
   414|    let receive_task = tokio::spawn(async move {
   415|        while let Some(result) = receiver.next().await {
   416|            match result {
   417|                Ok(axum::extract::ws::Message::Text(text)) => {
   418|                    let text_str = text.to_string();
   419|
   420|                    // SEC-05 : limite 64 KB
   421|                    if text_str.len() > 65_536 {
   422|                        tracing::warn!(ws_id = %id, bytes = text_str.len(), "WebSocket : message trop volumineux — ignoré");
   423|                        continue;
   424|                    }
   425|
   426|                    // Parser le message pour extraire type et to_user_id
   427|                    let json_val = match serde_json::from_str::<Value>(&text_str) {
   428|                        Ok(v) => v,
   429|                        Err(_) => {
   430|                            // Non-JSON : broadcast global (rétrocompatibilité)
   431|                            let _ = broadcast_tx_for_receive.send(text_str);
   432|                            continue;
   433|                        }
   434|                    };
   435|
   436|                    let msg_type = json_val
   437|                        .get("type")
   438|                        .and_then(|v| v.as_str())
   439|                        .unwrap_or("unknown");
   440|
   441|                    tracing::debug!(ws_id = %id, user_id = %user_id_recv, msg_type = %msg_type, "WebSocket : message reçu");
   442|
   443|                    // ── Routage des signaux WebRTC par to_user_id ─────────────────
   444|                    // Types d'appel : offer, answer, ice, join, leave, decline,
   445|                    //                 call_request, call_accepted, call_rejected
   446|                    // Types P2P file transfer : p2p_file_start, p2p_file_chunk, p2p_file_end
   447|                    let webrtc_types = ["offer", "answer", "ice", "ice_candidate",
   448|                        "join", "leave", "decline",
   449|                        "call_request", "call_accepted", "call_rejected",
   450|                        "webrtc_offer", "webrtc_answer", "webrtc_ice_candidate",
   451|                        "p2p_file_start", "p2p_file_chunk", "p2p_file_end"];
   452|
   453|                    if webrtc_types.contains(&msg_type) {
   454|                        let to_user_id = json_val
   455|                            .get("to_user_id")
   456|                            .and_then(|v| v.as_str());
   457|
   458|                        match to_user_id {
   459|                            Some(target) if !target.is_empty() => {
   460|                                // Routage direct vers le destinataire
   461|                                let guard = state_recv.webrtc_state.user_senders.lock().await;
   462|                                if let Some(target_tx) = guard.get(target) {
   463|                                    let _ = target_tx.send(text_str.clone());
   464|                                    tracing::debug!(
   465|                                        from = %user_id_recv,
   466|                                        to = %target,
   467|                                        msg_type = %msg_type,
   468|                                        "WebRTC signal routé"
   469|                                    );
   470|                                } else {
   471|                                    tracing::warn!(
   472|                                        to = %target,
   473|                                        msg_type = %msg_type,
   474|                                        "WebRTC signal : destinataire non connecté"
   475|                                    );
   476|                                }
   477|                                // call_request : aussi envoyer à l'expéditeur pour confirmation
   478|                            }
   479|                            _ => {
   480|                                // Pas de to_user_id (ex: join/leave) → broadcast global
   481|                                let guard = state_recv.webrtc_state.broadcasts.lock().await;
   482|                                for tx in guard.values() {
   483|                                    let _ = tx.send(text_str.clone());
   484|                                }
   485|                            }
   486|                        }
   487|                    } else {
   488|                        // Messages non-WebRTC (chat, chess, etc.) → broadcast global
   489|                        let _ = broadcast_tx_for_receive.send(text_str);
   490|                    }
   491|                }
   492|                Ok(axum::extract::ws::Message::Binary(data)) => {
   493|                    tracing::debug!(bytes = data.len(), "WebSocket : binaire ignoré (P2P direct)");
   494|                }
   495|                Ok(axum::extract::ws::Message::Close(_)) => {
   496|                    tracing::debug!(ws_id = %id, "WebSocket : fermeture propre");
   497|                    break;
   498|                }
   499|                Err(e) => {
   500|                    tracing::debug!(ws_id = %id, error = %e, "WebSocket : erreur réception");
   501|