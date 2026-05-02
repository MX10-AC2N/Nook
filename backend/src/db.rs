// db.rs - Structures et handlers DB avec Extension<CurrentUser>

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::CurrentUser;

// === STRUCTURES DE DONNÉES ===

fn default_true() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub name: Option<String>,
    pub role: String,
    pub approved: bool,
    pub needs_password_change: bool,
    pub token: Option<String>,
    pub created_at: i64,
    pub avatar_url: Option<String>,
    pub avatar_style: Option<String>,
    pub avatar_seed: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: String,
    pub name: Option<String>,
    pub is_group: bool,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
}

/// Message brut en base (sans sender_name) — référence structurelle
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub message_type: String,
    pub file_id: Option<String>,
    pub encrypted: bool,
    pub timestamp: i64,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

/// Message enrichi avec le nom de l'expéditeur (JOIN users)
/// Retourné par GET /conversations/{id}/messages
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageWithSender {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String, // COALESCE(users.name, users.username)
    pub sender_avatar_style: Option<String>, // DiceBear style of the sender
    pub sender_avatar_seed: Option<String>, // DiceBear seed chosen by the sender
    pub sender_public_key: Option<String>, // Clé publique X25519 de l'expéditeur (base64)
    pub content: String,
    pub message_type: String,
    pub file_id: Option<String>,
    pub encrypted: bool,
    pub nonce: Option<String>, // Nonce XSalsa20 base64 si encrypted=true
    pub timestamp: i64,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Upload {
    pub id: String,
    pub conversation_id: Option<String>,
    pub from_user_id: Option<String>,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub content_type: Option<String>,
    pub uploaded_at: i64,
    pub encrypted: bool,
    pub nonce: Option<String>,
    pub key_text: Option<String>,
}

// === REQUEST STRUCTURES ===

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub name: Option<String>,
    #[serde(default = "default_true")] // true si absent (groupe par défaut)
    pub is_group: bool,
    #[serde(default)]
    pub participant_ids: Vec<String>, // membres ajoutés à la création
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(default)] // false si absent (message non chiffré)
    pub encrypted: bool,
    /// Nonce XSalsa20 en base64 (24 bytes) — présent si encrypted=true
    #[serde(default)]
    pub nonce: Option<String>,
    /// Clé de session chiffrée pour chaque destinataire : user_id → base64
    #[serde(default)]
    pub encrypted_keys: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct MessageQueryParams {
    pub limit: Option<i64>,
    pub before: Option<i64>,
}

// === HANDLERS ===

pub async fn create_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, StatusCode> {
    let now = chrono::Utc::now().timestamp();

    // ── DM : réutiliser une conversation existante si elle existe déjà ──
    // Un DM est une conv is_group=0 avec exactement 2 participants : moi + l'autre.
    // Évite les doublons quand Alice recrée une conv avec Bob.
    if !req.is_group && req.participant_ids.len() == 1 {
        let other_id = &req.participant_ids[0];
        let existing: Option<(String,)> = sqlx::query_as::<_, (String,)>(
            r#"SELECT c.id FROM conversations c
               INNER JOIN conversation_participants cp1
                 ON cp1.conversation_id = c.id AND cp1.user_id = ?
               INNER JOIN conversation_participants cp2
                 ON cp2.conversation_id = c.id AND cp2.user_id = ?
               WHERE c.is_group = 0
               AND (SELECT COUNT(*) FROM conversation_participants
                    WHERE conversation_id = c.id) = 2
               LIMIT 1"#,
        )
        .bind(&user.id)
        .bind(other_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

        if let Some((existing_id,)) = existing {
            // Retourner la conv existante plutôt qu'en créer une nouvelle
            let conv =
                sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
                    .bind(&existing_id)
                    .fetch_one(&state.db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(conv));
        }
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO conversations (id, name, is_group, created_at, created_by, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.is_group)
    .bind(now)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Créateur toujours ajouté
    sqlx::query(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Autres participants fournis à la création
    for pid in &req.participant_ids {
        if pid == &user.id {
            continue;
        }
        sqlx::query(
            "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
             VALUES (?, ?, ?)",
        )
        .bind(&id)
        .bind(pid)
        .bind(now)
        .execute(&state.db)
        .await
        .ok();
    }

    Ok(Json(Conversation {
        id,
        name: req.name,
        is_group: req.is_group,
        created_at: now,
        created_by: user.id,
        updated_at: now,
    }))
}

pub async fn get_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
) -> Result<Json<Conversation>, StatusCode> {
    let conv = sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(conv))
}

// PATCH /api/conversations/{id}/rename
// Autorisé : créateur du groupe ou admin. Interdit sur default_global.
#[derive(Debug, serde::Deserialize)]
pub struct RenameConversationRequest {
    pub name: String,
}

pub async fn rename_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Extension(crate::auth::CurrentUser(user)): Extension<crate::auth::CurrentUser>,
    Path(id): Path<String>,
    Json(req): Json<RenameConversationRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    use axum::http::StatusCode;
    use serde_json::json;

    // default_global est intouchable
    if id == "default_global" {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Impossible de renommer le groupe Nook"}))));
    }

    let name = req.name.trim().to_string();
    if name.is_empty() || name.len() > 60 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Nom invalide (1-60 caractères)"}))));
    }

    // Vérifier que la conv existe et que l'utilisateur est le créateur ou admin
    #[derive(sqlx::FromRow)]
    struct ConvMeta { created_by: String, is_group: bool }

    let meta = sqlx::query_as::<_, ConvMeta>(
        "SELECT created_by, is_group FROM conversations WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur DB"}))))?
    .ok_or((StatusCode::NOT_FOUND, Json(json!({"error": "Conversation introuvable"}))))?;

    if !meta.is_group {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Impossible de renommer un DM"}))));
    }

    if meta.created_by != user.id && user.role != "admin" {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Seul le créateur ou l'admin peut renommer"}))));
    }

    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE conversations SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur DB"}))))?;

    tracing::info!(conv_id = %id, new_name = %name, "Groupe renommé");
    Ok(Json(json!({"success": true, "name": name})))
}

pub async fn get_user_conversations(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<Vec<Conversation>>, StatusCode> {
    let convs = sqlx::query_as::<_, Conversation>(
        "SELECT c.* FROM conversations c
         INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
         WHERE cp.user_id = ?
         ORDER BY c.updated_at DESC",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(convs))
}

pub async fn join_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<StatusCode, StatusCode> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
         VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn send_message(
    State(state): State<Arc<crate::SharedState>>,
    Path(conversation_id): Path<String>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    // FIX C5: Verifier que la conversation existe ET que l'utilisateur y est participant
    let is_participant: Option<(i64,)> =
        sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM conversations c
             INNER JOIN conversation_participants p ON c.id = p.conversation_id
             WHERE c.id = ? AND p.user_id = ?"
        )
            .bind(&conversation_id)
            .bind(&user.id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    if is_participant.map(|(c,)| c).unwrap_or(0) == 0 {
        // La conversation n'existe pas OU l'utilisateur n'en est pas participant
        let conv_exists: Option<(i64,)> =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM conversations WHERE id = ?")
                .bind(&conversation_id)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();
        if conv_exists.map(|(c,)| c).unwrap_or(0) == 0 {
            return Err(StatusCode::NOT_FOUND);
        }
        eprintln!(
            "[send_message] Utilisateur {} n'est pas participant de la conversation {}",
            user.id, conversation_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // FIX M2: limiter la taille du message a 8000 caracteres
    if req.content.len() > 8000 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    sqlx::query(
        "INSERT INTO messages
            (id, conversation_id, sender_id, content, encrypted, nonce, timestamp, created_at, message_type)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(&user.id)
    .bind(&req.content)
    .bind(req.encrypted)
    .bind(&req.nonce)
    .bind(now)
    .bind(now)
    .bind("text")
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("[send_message] Erreur DB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // ── Notifications push aux autres membres de la conversation ──────────
    // Fire-and-forget : on ne bloque pas la réponse HTTP pour les push
    {
        let pool = state.db.clone();
        let sender_id = user.id.clone();
        let sender_name = user.username.clone();
        let msg_preview = req.content.chars().take(80).collect::<String>();
        let conv_id = conversation_id.clone();
        tokio::task::spawn(async move {
            // Récupérer tous les membres sauf l'expéditeur
            let members: Vec<(String,)> = sqlx::query_as(
                "SELECT user_id FROM conversation_participants WHERE conversation_id = ? AND user_id != ?",
            )
            .bind(&conv_id)
            .bind(&sender_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            let payload = crate::push::PushPayload {
                title: format!("Nook · {}", sender_name),
                body: msg_preview,
                icon: Some("/icon-192.png".to_string()),
                url: Some("/chat".to_string()),
                tag: Some(format!("nook-msg-{}", conv_id)),
            };

            for (member_id,) in members {
                if let Err(e) = crate::push::send_push_notification(&pool, &member_id, &payload).await {
                    tracing::debug!(error = %e, member_id = %member_id, "Push non envoyé");
                }
            }
        });
    }

    // Stocker les clés de session chiffrées pour chaque destinataire (E2EE)
    if req.encrypted && !req.encrypted_keys.is_empty() {
        if let Err(e) = crate::e2ee::store_message_keys(&state.db, &id, &req.encrypted_keys).await {
            tracing::warn!(error = %e, msg_id = %id, "E2EE: échec store_message_keys (non bloquant)");
        }
    }

    // Retourner le message enrichi (avec sender_name) pour cohérence frontend
    let sender_name: String =
        sqlx::query_as::<_, (String,)>("SELECT COALESCE(name, username) FROM users WHERE id = ?")
            .bind(&user.id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .map(|(n,)| n)
            .unwrap_or_else(|| user.username.clone());

    // Récupérer le style et seed d'avatar de l'expéditeur
    let avatar_data: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT avatar_style, avatar_seed FROM users WHERE id = ?"
    )
        .bind(&user.id)
        .fetch_one(&state.db)
        .await
        .ok();
    let sender_avatar_style = avatar_data.as_ref().and_then(|(s, _)| s.clone());
    let sender_avatar_seed = avatar_data.as_ref().and_then(|(_, s)| s.clone());

    let msg_json = serde_json::json!({
        "id": id,
        "conversation_id": conversation_id,
        "sender_id": user.id,
        "sender_name": sender_name,
        "sender_avatar_style": sender_avatar_style,
        "sender_avatar_seed": sender_avatar_seed,
        "sender_public_key": null,
        "content": req.content,
        "message_type": "text",
        "file_id": null,
        "encrypted": req.encrypted,
        "nonce": req.nonce,
        "timestamp": now,
        "created_at": now,
        "edited_at": null
    });

    // ── C4 FIX : Broadcast WS uniquement aux participants de la conversation ──
    {
        let ws_payload = serde_json::json!({
            "type": "new_message",
            "conversation_id": conversation_id,
            "message": msg_json.clone(),
        });
        // Recuperer les participants de la conversation
        let participant_ids: Vec<(String,)> = sqlx::query_as(
            "SELECT user_id FROM conversation_participants WHERE conversation_id = ?"
        )
            .bind(&conversation_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

        // Envoyer uniquement aux participants connectes
        let guard = state.webrtc_state.user_senders.lock().await;
        for (user_id,) in participant_ids {
            // L'expe?iteur recoit aussi sa confirmation via le WS
            if let Some(tx) = guard.get(&user_id) {
                let _ = tx.send(ws_payload.to_string());
            }
        }
    }

    Ok(Json(msg_json))
}

// ── PATCH /api/conversations/{conv_id}/messages/{msg_id} ────────────────────
// Seul l'expéditeur peut éditer son propre message (max 4000 chars).
#[derive(Debug, serde::Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

pub async fn edit_message(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path((conv_id, msg_id)): Path<(String, String)>,
    Json(req): Json<EditMessageRequest>,
) -> impl axum::response::IntoResponse {
    use axum::http::StatusCode;
    use serde_json::json;

    let content = req.content.trim().to_string();
    if content.is_empty() || content.len() > 4000 {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Contenu invalide (1-4000 chars)"}))).into_response();
    }

    #[derive(sqlx::FromRow)]
    struct MsgMeta { sender_id: String }

    let meta = match sqlx::query_as::<_, MsgMeta>(
        "SELECT sender_id FROM messages WHERE id = ? AND conversation_id = ?"
    )
    .bind(&msg_id).bind(&conv_id)
    .fetch_optional(&state.db).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Message introuvable"}))).into_response(),
        Err(_)    => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur DB"}))).into_response(),
    };

    if meta.sender_id != user.id {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Seul l'auteur peut modifier"}))).into_response();
    }

    let now = chrono::Utc::now().timestamp();
    if sqlx::query("UPDATE messages SET content = ?, edited_at = ? WHERE id = ?")
        .bind(&content).bind(now).bind(&msg_id)
        .execute(&state.db).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur DB"}))).into_response();
    }

    {
        let ws = serde_json::json!({"type": "message_edited", "conversation_id": conv_id, "message_id": msg_id, "content": content, "edited_at": now});
        let participant_ids: Vec<(String,)> = sqlx::query_as(
            "SELECT user_id FROM conversation_participants WHERE conversation_id = ?"
        ).bind(&conv_id).fetch_all(&state.db).await.unwrap_or_default();
        let guard = state.webrtc_state.user_senders.lock().await;
        for (user_id,) in &participant_ids {
            if let Some(tx) = guard.get(user_id) {
                let _ = tx.send(ws.to_string());
            }
        }
    }

    tracing::info!(msg_id = %msg_id, user_id = %user.id, "Message édité");
    Json(json!({"success": true, "content": content, "edited_at": now})).into_response()
}

// ── DELETE /api/conversations/{conv_id}/messages/{msg_id} ──────────────────
// Autorisé : expéditeur ou admin.
pub async fn delete_message(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path((conv_id, msg_id)): Path<(String, String)>,
) -> impl axum::response::IntoResponse {
    use axum::http::StatusCode;
    use serde_json::json;

    #[derive(sqlx::FromRow)]
    struct MsgMeta { sender_id: String }

    let meta = match sqlx::query_as::<_, MsgMeta>(
        "SELECT sender_id FROM messages WHERE id = ? AND conversation_id = ?"
    )
    .bind(&msg_id).bind(&conv_id)
    .fetch_optional(&state.db).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Message introuvable"}))).into_response(),
        Err(_)    => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur DB"}))).into_response(),
    };

    if meta.sender_id != user.id && user.role != "admin" {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Seul l'auteur ou l'admin peut supprimer"}))).into_response();
    }

    if sqlx::query("DELETE FROM messages WHERE id = ?")
        .bind(&msg_id).execute(&state.db).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur DB"}))).into_response();
    }

    {
        let ws = serde_json::json!({"type": "message_deleted", "conversation_id": conv_id, "message_id": msg_id});
        let participant_ids: Vec<(String,)> = sqlx::query_as(
            "SELECT user_id FROM conversation_participants WHERE conversation_id = ?"
        ).bind(&conv_id).fetch_all(&state.db).await.unwrap_or_default();
        let guard = state.webrtc_state.user_senders.lock().await;
        for (user_id,) in &participant_ids {
            if let Some(tx) = guard.get(user_id) {
                let _ = tx.send(ws.to_string());
            }
        }
    }

    tracing::info!(msg_id = %msg_id, user_id = %user.id, "Message supprimé");
    StatusCode::NO_CONTENT.into_response()
}

pub async fn get_conversation_messages(
    State(state): State<Arc<crate::SharedState>>,
    Path(id): Path<String>,
    Query(params): Query<MessageQueryParams>,
) -> Result<Json<Vec<MessageWithSender>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);

    // JOIN sur users pour récupérer sender_name en une requête
    // ORDER BY ASC : les plus anciens en premier (ordre d'affichage naturel)
    let messages = if let Some(before) = params.before {
        sqlx::query_as::<_, MessageWithSender>(
            "SELECT
                m.id, m.conversation_id, m.sender_id,
                COALESCE(u.name, u.username) AS sender_name,
                u.avatar_style AS sender_avatar_style,
                u.avatar_seed AS sender_avatar_seed,
                u.public_key AS sender_public_key,
                m.content, m.message_type, m.file_id,
                m.encrypted, m.nonce, m.timestamp, m.created_at, m.edited_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.sender_id
             WHERE m.conversation_id = ? AND m.created_at < ?
             ORDER BY m.created_at ASC
             LIMIT ?",
        )
        .bind(&id)
        .bind(before)
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map_err(|e| { 
            eprintln!("[get_conversation_messages] Erreur DB: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        sqlx::query_as::<_, MessageWithSender>(
            "SELECT
                m.id, m.conversation_id, m.sender_id,
                COALESCE(u.name, u.username) AS sender_name,
                u.avatar_style AS sender_avatar_style,
                u.avatar_seed AS sender_avatar_seed,
                u.public_key AS sender_public_key,
                m.content, m.message_type, m.file_id,
                m.encrypted, m.nonce, m.timestamp, m.created_at, m.edited_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.sender_id
             WHERE m.conversation_id = ?
             ORDER BY m.created_at DESC
             LIMIT ?",
        )
        .bind(&id)
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map(|mut msgs| {
            // Reverse to get ASC order (oldest first) for frontend compatibility
            msgs.reverse();
            msgs
        })
        .map_err(|e| { 
            eprintln!("[get_conversation_messages] Erreur DB: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    };
    }
    .map_err(|e| {
        eprintln!("[get_conversation_messages] Erreur DB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(messages))
}

// === PROFIL UTILISATEUR ===

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_style: Option<String>,
    pub avatar_seed: Option<String>,
}

pub async fn update_user_profile(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<UpdateProfileRequest>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;

    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Le nom ne peut pas être vide" })),
            );
        }
        if sqlx::query("UPDATE users SET name = ? WHERE id = ?")
            .bind(name.trim())
            .bind(&user.id)
            .execute(&state.db)
            .await
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur de mise à jour" })),
            );
        }
    }

    if let Some(ref email) = req.email {
        if sqlx::query("UPDATE users SET email = ? WHERE id = ?")
            .bind(email.trim())
            .bind(&user.id)
            .execute(&state.db)
            .await
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur de mise à jour de l'email" })),
            );
        }
    }

    if let Some(ref avatar_url) = req.avatar_url {
        if sqlx::query("UPDATE users SET avatar_url = ? WHERE id = ?")
            .bind(avatar_url.trim())
            .bind(&user.id)
            .execute(&state.db)
            .await
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur de mise à jour de l'avatar" })),
            );
        }
    }

    if let Some(ref avatar_style) = req.avatar_style {
        let valid_styles = ["adventurer", "avataaars", "open-peeps", "notionists", "fun-emoji", "big-smile", "lorelei", "personas", "bottts", "initials"];
        let style = if valid_styles.contains(&avatar_style.as_str()) {
            avatar_style.trim()
        } else {
            "adventurer"
        };
        if sqlx::query("UPDATE users SET avatar_style = ? WHERE id = ?")
            .bind(style)
            .bind(&user.id)
            .execute(&state.db)
            .await
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur de mise à jour du style d'avatar" })),
            );
        }
    }

    if let Some(ref avatar_seed) = req.avatar_seed {
        let seed = avatar_seed.trim();
        if seed.len() > 64 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Seed d'avatar trop long" })),
            );
        }
        if sqlx::query("UPDATE users SET avatar_seed = ? WHERE id = ?")
            .bind(seed)
            .bind(&user.id)
            .execute(&state.db)
            .await
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur de mise à jour du seed d'avatar" })),
            );
        }
    }

    (
        StatusCode::OK,
        Json(json!({ "success": true, "message": "Profil mis à jour" })),
    )
}

// === ÉVÉNEMENTS (CALENDRIER) ===

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub date: String,
    pub time: Option<String>,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub date: String,
    pub time: Option<String>,
    pub description: Option<String>,
}

pub async fn get_events(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(_)): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let events = sqlx::query_as::<_, Event>("SELECT * FROM events ORDER BY date ASC, time ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("[events] GET error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({ "events": events })))
}

pub async fn create_event(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreateEventRequest>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;

    if req.title.trim().is_empty() || req.date.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "message": "Titre et date requis" })),
        );
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    match sqlx::query(
        "INSERT INTO events (id, title, date, time, description, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(req.title.trim())
    .bind(req.date.trim())
    .bind(&req.time)
    .bind(&req.description)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            // Broadcast WS: new_event
            let notif = serde_json::json!({
                "type": "new_event",
                "event_id": id,
                "title": req.title.trim(),
                "date": req.date.trim(),
                "creator": user.username,
            }).to_string();
            let guard = state.webrtc_state.broadcasts.lock().await;
            for (_, tx) in guard.iter() { let _ = tx.send(notif.clone()); }

            (StatusCode::OK, Json(json!({ "success": true, "id": id })))
        },
        Err(e) => {
            eprintln!("[events] INSERT error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "message": "Erreur création" })),
            )
        }
    }
}

// ═════════════════════════════════════════════════════════════════
// PATCH /events/{id}  — modifier un événement
// ═════════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct UpdateEventRequest {
    pub title:       Option<String>,
    pub date:        Option<String>,
    pub time:        Option<String>,
    pub description: Option<String>,
}

pub async fn update_event(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(req): Json<UpdateEventRequest>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;

    // Vérifier existence + propriétaire
    let row: Option<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT created_by, title, date, time, description FROM events WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let (created_by, cur_title, cur_date, cur_time, cur_desc) = match row {
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "message": "Événement introuvable" }))).into_response(),
        Some(r) if r.0 != user.id && user.role != "admin" => return (StatusCode::FORBIDDEN, Json(json!({ "success": false, "message": "Accès refusé" }))).into_response(),
        Some(r) => r,
    };
    let _ = created_by;

    let new_title = req.title.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or(cur_title);
    let new_date  = req.date.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or(cur_date);
    let new_time  = req.time.unwrap_or(cur_time);
    let new_desc  = req.description.unwrap_or(cur_desc);

    match sqlx::query(
        "UPDATE events SET title = ?, date = ?, time = ?, description = ? WHERE id = ?"
    )
    .bind(&new_title).bind(&new_date).bind(&new_time).bind(&new_desc).bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({
            "success": true,
            "event": { "id": id, "title": new_title, "date": new_date, "time": new_time, "description": new_desc }
        }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false }))).into_response(),
    }
}

pub async fn delete_event(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;

    let row: Option<(String,)> =
        sqlx::query_as::<_, (String,)>("SELECT created_by FROM events WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    match row {
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "message": "Événement introuvable" })),
            )
        }
        Some((created_by,)) if created_by != user.id && user.role != "admin" => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "success": false, "message": "Accès refusé" })),
            )
        }
        _ => {}
    }

    match sqlx::query("DELETE FROM events WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": "Erreur suppression" })),
        ),
    }
}

// ═════════════════════════════════════════════════════════════════
// GET /conversations/{id}/participants
// ═════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ParticipantInfo {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub role: String,
}

pub async fn get_conversation_participants(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(_user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
) -> impl IntoResponse {
    let participants = sqlx::query_as::<_, ParticipantInfo>(
        r#"SELECT u.id, u.username, u.name, u.role
           FROM users u
           INNER JOIN conversation_participants cp ON cp.user_id = u.id
           WHERE cp.conversation_id = ?
           ORDER BY COALESCE(u.name, u.username) ASC"#,
    )
    .bind(&conv_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(serde_json::json!({ "participants": participants }))
}

// ═════════════════════════════════════════════════════════════════
// POST /conversations/{id}/participants  — ajouter un membre
// ═════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct AddParticipantRequest {
    pub user_id: String,
}

pub async fn add_conversation_participant(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
    Json(req): Json<AddParticipantRequest>,
) -> impl IntoResponse {
    // Le demandeur doit être membre
    let is_member: (i64,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM conversation_participants WHERE conversation_id = ? AND user_id = ?",
    )
    .bind(&conv_id)
    .bind(&user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    if is_member.0 == 0 {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "message": "Non membre de cette conversation" })),
        )
            .into_response();
    }

    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
         VALUES (?, ?, ?)",
    )
    .bind(&conv_id)
    .bind(&req.user_id)
    .bind(now)
    .execute(&state.db)
    .await
    .ok();

    Json(serde_json::json!({ "success": true })).into_response()
}

// ═════════════════════════════════════════════════════════════════
// POST /conversations/{id}/leave
// ═════════════════════════════════════════════════════════════════

pub async fn leave_conversation(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
) -> impl IntoResponse {
    if conv_id == "default_global" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "message": "Impossible de quitter le groupe global" })),
        )
            .into_response();
    }
    sqlx::query("DELETE FROM conversation_participants WHERE conversation_id = ? AND user_id = ?")
        .bind(&conv_id)
        .bind(&user.id)
        .execute(&state.db)
        .await
        .ok();
    Json(serde_json::json!({ "success": true })).into_response()
}

// ═════════════════════════════════════════════════════════════════
// GET /users/available — membres approuvés pour créer des convs
// ═════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AvailableUser {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub avatar_style: Option<String>,
    pub avatar_seed: Option<String>,
}

pub async fn get_available_users(
    State(state): State<Arc<crate::SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    let users = sqlx::query_as::<_, AvailableUser>(
        r#"SELECT id, username, name, avatar_style, avatar_seed FROM users
           WHERE approved = 1 AND id != ?
           ORDER BY COALESCE(name, username) ASC"#,
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(serde_json::json!({ "users": users }))
}
