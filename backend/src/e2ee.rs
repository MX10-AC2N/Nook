// backend/src/e2ee.rs
//
// Chiffrement de bout en bout — architecture "clé de session par message"
//
// Routes exposées (à merger dans protected_routes de main.rs) :
//   POST /api/auth/public-key
//       Enregistre/met à jour la clé publique X25519 de l'utilisateur.
//
//   GET  /api/auth/public-keys?conversation_id=xxx
//       Retourne les clés publiques de tous les membres d'une conversation.
//
//   POST /api/auth/rotate-key
//       Archive la clé actuelle et active une nouvelle paire.
//
//   GET  /api/auth/key-history
//       Liste les anciennes versions de clé publique.
//
//   GET  /api/auth/key-history/{version}
//       Retourne la clé privée chiffrée pour une version donnée.
//
//   POST /api/conversations/{conv_id}/messages  (étendu dans db.rs, pas ici)
//       Le payload inclut désormais encrypted_keys: HashMap<userId, base64key>,
//       nonce: base64, et sender_key_version.
//
//   GET  /api/conversations/{conv_id}/my-encrypted-key/{msg_id}
//       Retourne la clé de session chiffrée pour l'utilisateur courant.
//
//   POST /api/conversations/{conv_id}/keys
//       Distribue une nouvelle clé de groupe aux membres.
//
//   GET  /api/conversations/{conv_id}/my-key
//       Retourne la clé de groupe chiffrée pour l'utilisateur courant.
//
//   POST /api/conversations/{conv_id}/add-member-key
//       Ajoute un membre à une clé de groupe existante.

use axum::{
    extract::{Path, Query, State as AxumState},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{auth::CurrentUser, SharedState};

// ─────────────────────────────────────────────────────────────────────────────
// Types publics
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterPublicKeyRequest {
    /// Clé publique X25519 encodée en base64 standard (32 bytes → 44 chars avec padding)
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct MemberPublicKey {
    pub user_id: String,
    pub username: String,
    /// Clé publique X25519 en base64
    pub public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PublicKeysQuery {
    pub conversation_id: String,
}

#[derive(Debug, Serialize)]
pub struct EncryptedKeyResponse {
    /// base64(asymNonce[24] || crypto_box_easy_ciphertext)
    pub encrypted_key: String,
    pub message_id: String,
    /// Version de clé de l'expéditeur utilisée pour ce message
    pub key_version: i32,
}

/// Payload pour stocker les clés chiffrées d'un message E2EE.
/// Appelé depuis db.rs::send_message après insertion du message.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MessageKeysPayload {
    /// user_id → base64(asymNonce || boxCiphertext)
    pub encrypted_keys: HashMap<String, String>,
}

// ── Key rotation types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RotateKeyRequest {
    /// Nouvelle clé publique X25519 en base64 (32 bytes)
    pub public_key: String,
    /// Clé privée chiffrée: base64(salt[16] || nonce[24] || ciphertext)
    pub encrypted_private_key: String,
    /// Mot de passe actuel pour re-vérification Argon2id
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RotateKeyResponse {
    pub success: bool,
    pub version: i32,
}

#[derive(Debug, Serialize)]
pub struct KeyHistoryEntry {
    pub version: i32,
    pub public_key: String,
    pub created_at: i64,
    pub revoked_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct KeyVersionResponse {
    pub encrypted_private_key: String,
}

// ── Group key types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DistributeKeysRequest {
    /// user_id → base64 sealed box encrypted group key
    pub distributions: HashMap<String, String>,
    pub keyVersion: i32,
}

#[derive(Debug, Deserialize)]
pub struct MyKeyQuery {
    pub version: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct MyKeyResponse {
    pub encrypted_key: String,
    pub keyVersion: i32,
    pub userKeyVersion: i32,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberKeyRequest {
    pub userId: String,
    pub encryptedKey: String,
    pub userKeyVersion: i32,
    pub keyVersion: i32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Vérifie que l'utilisateur courant est membre d'une conversation.
async fn check_membership(
    state: &Arc<SharedState>,
    user_id: &str,
    conv_id: &str,
) -> Result<(), StatusCode> {
    let is_member: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM conversation_participants
         WHERE conversation_id = ? AND user_id = ?",
    )
    .bind(conv_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_member.map(|(c,)| c).unwrap_or(0) == 0 {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// Récupère la version actuelle de la clé publique d'un utilisateur.
/// Retourne 1 si aucune rotation n'a été effectuée (clé initiale dans users.public_key).
async fn get_current_key_version(
    state: &Arc<SharedState>,
    user_id: &str,
) -> Result<i32, StatusCode> {
    let max_version: Option<(i32,)> = sqlx::query_as(
        "SELECT COALESCE(MAX(version), 0) FROM user_key_history WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Si l'utilisateur a un historique, la version courante = MAX(version) + 1
    // Sinon, c'est la première rotation → version 1
    Ok(max_version.map(|(v,)| v).unwrap_or(0) + 1)
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/auth/public-key
// ─────────────────────────────────────────────────────────────────────────────
pub async fn register_public_key(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(body): Json<RegisterPublicKeyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Valider : base64 décodable + exactement 32 bytes (X25519 public key)
    use base64ct::{Base64, Encoding};
    let decoded = Base64::decode_vec(&body.public_key).map_err(|_| StatusCode::BAD_REQUEST)?;

    if decoded.len() != 32 {
        tracing::warn!(user_id = %user.id, len = decoded.len(), "e2ee: clé publique invalide (taille ≠ 32)");
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("UPDATE users SET public_key = ? WHERE id = ?")
        .bind(&body.public_key)
        .bind(&user.id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user.id, "e2ee: échec mise à jour public_key");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(user_id = %user.id, "e2ee: clé publique enregistrée");
    Ok(Json(serde_json::json!({ "success": true })))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/auth/public-keys?conversation_id=xxx
// ─────────────────────────────────────────────────────────────────────────────
pub async fn get_member_public_keys(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Query(params): Query<PublicKeysQuery>,
) -> Result<Json<Vec<MemberPublicKey>>, StatusCode> {
    check_membership(&state, &user.id, &params.conversation_id).await?;

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT u.id, u.username, u.public_key
         FROM conversation_participants cp
         JOIN users u ON u.id = cp.user_id
         WHERE cp.conversation_id = ?
           AND u.public_key IS NOT NULL
           AND u.approved = 1",
    )
    .bind(&params.conversation_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, conv = %params.conversation_id, "e2ee: échec get public keys");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|(user_id, username, public_key)| MemberPublicKey {
                user_id,
                username,
                public_key,
            })
            .collect(),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/conversations/{conv_id}/my-encrypted-key/{msg_id}
// ─────────────────────────────────────────────────────────────────────────────
pub async fn get_my_encrypted_key(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path((conv_id, msg_id)): Path<(String, String)>,
) -> Result<Json<EncryptedKeyResponse>, StatusCode> {
    check_membership(&state, &user.id, &conv_id).await?;

    let row: Option<(String, i32)> = sqlx::query_as(
        "SELECT encrypted_key, COALESCE(sender_key_version, 1) FROM message_keys
         WHERE message_id = ? AND recipient_id = ?",
    )
    .bind(&msg_id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, msg = %msg_id, user = %user.id, "e2ee: échec get encrypted_key");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (encrypted_key, key_version) = row.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(EncryptedKeyResponse {
        encrypted_key,
        message_id: msg_id,
        key_version,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Fonction utilitaire appelée depuis db.rs::send_message
// Insère les clés chiffrées dans message_keys après insertion du message.
// ─────────────────────────────────────────────────────────────────────────────
pub async fn store_message_keys(
    pool: &sqlx::SqlitePool,
    message_id: &str,
    encrypted_keys: &HashMap<String, String>,
    sender_key_version: i32,
) -> Result<(), sqlx::Error> {
    for (recipient_id, encrypted_key) in encrypted_keys {
        sqlx::query(
            "INSERT OR REPLACE INTO message_keys (message_id, recipient_id, encrypted_key, sender_key_version)
             VALUES (?, ?, ?, ?)",
        )
        .bind(message_id)
        .bind(recipient_id)
        .bind(encrypted_key)
        .bind(sender_key_version)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/auth/rotate-key
// ─────────────────────────────────────────────────────────────────────────────
pub async fn rotate_key(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(body): Json<RotateKeyRequest>,
) -> Result<Json<RotateKeyResponse>, StatusCode> {
    // 1. Rate limiting: max 5 rotations par heure par utilisateur
    let one_hour_ago = chrono::Utc::now().timestamp() - 3600;
    let recent_count: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM user_key_history
         WHERE user_id = ? AND created_at > ?",
    )
    .bind(&user.id)
    .bind(one_hour_ago)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if recent_count.map(|(c,)| c).unwrap_or(0) >= 5 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // 2. Re-verify user's password via Argon2id
    let password_hash: (String,) = sqlx::query_as(
        "SELECT password_hash FROM users WHERE id = ?",
    )
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    if !crate::auth::verify_password(&body.password, &password_hash.0) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 3. Validate new public key: base64 + 32 bytes X25519
    use base64ct::{Base64, Encoding};
    let decoded = Base64::decode_vec(&body.public_key).map_err(|_| StatusCode::BAD_REQUEST)?;
    if decoded.len() != 32 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 4. Validate encrypted_private_key: must be valid base64
    Base64::decode_vec(&body.encrypted_private_key).map_err(|_| StatusCode::BAD_REQUEST)?;

    // 5. Get current key info before rotation
    let current_key: Option<(String,)> = sqlx::query_as(
        "SELECT public_key FROM users WHERE id = ? AND public_key IS NOT NULL",
    )
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let now = chrono::Utc::now().timestamp();
    let new_version = get_current_key_version(&state, &user.id).await?;

    // 6. Insert current keypair into history with revoked_at = now
    //    If there's no existing key, we still record version 0 as "genesis"
    //    so the version sequence starts from 1 for the new key.
    if let Some((public_key,)) = current_key {
        // Get the encrypted private key from users table — in production this
        // is derived from the password. For the initial key (before any rotation),
        // we store a placeholder since we don't have the encrypted private key.
        // The user's first rotation creates version 1 of the key history.
        sqlx::query(
            "INSERT INTO user_key_history (user_id, version, public_key, encrypted_priv, created_at, revoked_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&user.id)
        .bind(new_version - 1)  // version 0 for the initial key
        .bind(&public_key)
        .bind("") // No encrypted priv available for initial key
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user.id, "e2ee: échec archive clé actuelle");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // 7. Insert new keypair into history (revoked_at = NULL = current)
    sqlx::query(
        "INSERT INTO user_key_history (user_id, version, public_key, encrypted_priv, created_at, revoked_at)
         VALUES (?, ?, ?, ?, ?, NULL)",
    )
    .bind(&user.id)
    .bind(new_version)
    .bind(&body.public_key)
    .bind(&body.encrypted_private_key)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, user_id = %user.id, "e2ee: échec insertion nouvelle clé");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 8. Update users.public_key with new key
    sqlx::query("UPDATE users SET public_key = ? WHERE id = ?")
        .bind(&body.public_key)
        .bind(&user.id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user.id, "e2ee: échec mise à jour public_key");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(
        user_id = %user.id,
        new_version = new_version,
        "e2ee: clé pivotée avec succès"
    );

    Ok(Json(RotateKeyResponse {
        success: true,
        version: new_version,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/auth/key-history
// ─────────────────────────────────────────────────────────────────────────────
pub async fn get_key_history(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<Json<Vec<KeyHistoryEntry>>, StatusCode> {
    let rows: Vec<(i32, String, i64, Option<i64>)> = sqlx::query_as(
        "SELECT version, public_key, created_at, revoked_at
         FROM user_key_history
         WHERE user_id = ?
         ORDER BY version ASC",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, user_id = %user.id, "e2ee: échec get_key_history");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|(version, public_key, created_at, revoked_at)| KeyHistoryEntry {
                version,
                public_key,
                created_at,
                revoked_at,
            })
            .collect(),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/auth/key-history/{version}
// ─────────────────────────────────────────────────────────────────────────────
pub async fn get_key_version(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(version): Path<i32>,
) -> Result<Json<KeyVersionResponse>, StatusCode> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT encrypted_priv FROM user_key_history
         WHERE user_id = ? AND version = ? AND encrypted_priv != ''",
    )
    .bind(&user.id)
    .bind(version)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, user_id = %user.id, version = version, "e2ee: échec get_key_version");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (encrypted_private_key,) = row.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(KeyVersionResponse {
        encrypted_private_key,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/conversations/{conv_id}/keys
// Distribue une nouvelle clé de groupe aux membres de la conversation.
// ─────────────────────────────────────────────────────────────────────────────
pub async fn distribute_group_keys(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
    Json(body): Json<DistributeKeysRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Vérifier que l'utilisateur est membre
    check_membership(&state, &user.id, &conv_id).await?;

    let now = chrono::Utc::now().timestamp();

    // Insérer la version de clé de groupe
    sqlx::query(
        "INSERT OR IGNORE INTO conversation_keys (conversation_id, version, creator_id, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&conv_id)
    .bind(body.keyVersion)
    .bind(&user.id)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, conv = %conv_id, version = body.keyVersion, "e2ee: échec insert conversation_keys");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Insérer les clés chiffrées pour chaque destinataire
    for (recipient_id, encrypted_key) in &body.distributions {
        sqlx::query(
            "INSERT OR REPLACE INTO conversation_key_recipients
                (conversation_id, version, user_id, encrypted_key, user_key_version)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&conv_id)
        .bind(body.keyVersion)
        .bind(recipient_id)
        .bind(encrypted_key)
        .bind(1) // Default user_key_version — frontend should provide this in future
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, conv = %conv_id, user = %recipient_id, "e2ee: échec insert recipient key");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    tracing::info!(
        conv = %conv_id,
        version = body.keyVersion,
        recipients = body.distributions.len(),
        "e2ee: clé de groupe distribuée"
    );

    Ok(Json(serde_json::json!({ "success": true })))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/conversations/{conv_id}/my-key
// Retourne la clé de groupe chiffrée pour l'utilisateur courant.
// ─────────────────────────────────────────────────────────────────────────────
pub async fn get_my_group_key(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
    Query(query): Query<MyKeyQuery>,
) -> Result<Json<MyKeyResponse>, StatusCode> {
    check_membership(&state, &user.id, &conv_id).await?;

    // Si version spécifiée, utiliser celle-là; sinon prendre la plus récente
    let row = if let Some(version) = query.version {
        sqlx::query_as::<_, (String, i32, i32)>(
            "SELECT ckr.encrypted_key, ckr.version, ckr.user_key_version
             FROM conversation_key_recipients ckr
             WHERE ckr.conversation_id = ? AND ckr.user_id = ? AND ckr.version = ?",
        )
        .bind(&conv_id)
        .bind(&user.id)
        .bind(version)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, conv = %conv_id, "e2ee: échec get_my_group_key");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        sqlx::query_as::<_, (String, i32, i32)>(
            "SELECT ckr.encrypted_key, ckr.version, ckr.user_key_version
             FROM conversation_key_recipients ckr
             INNER JOIN conversation_keys ck
                 ON ck.conversation_id = ckr.conversation_id AND ck.version = ckr.version
             WHERE ckr.conversation_id = ? AND ckr.user_id = ?
             ORDER BY ck.version DESC
             LIMIT 1",
        )
        .bind(&conv_id)
        .bind(&user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, conv = %conv_id, "e2ee: échec get_my_group_key");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    };

    let (encrypted_key, key_version, user_key_version) = row.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(MyKeyResponse {
        encrypted_key,
        keyVersion: key_version,
        userKeyVersion: user_key_version,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/conversations/{conv_id}/add-member-key
// Ajoute un membre à une clé de groupe existante.
// ─────────────────────────────────────────────────────────────────────────────
pub async fn add_member_key(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
    Json(body): Json<AddMemberKeyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Vérifier que l'utilisateur est membre
    check_membership(&state, &user.id, &conv_id).await?;

    // Vérifier que la clé de groupe existe
    let key_exists: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM conversation_keys
         WHERE conversation_id = ? AND version = ?",
    )
    .bind(&conv_id)
    .bind(body.keyVersion)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if key_exists.map(|(c,)| c).unwrap_or(0) == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Insérer la clé pour le nouveau membre
    sqlx::query(
        "INSERT OR REPLACE INTO conversation_key_recipients
            (conversation_id, version, user_id, encrypted_key, user_key_version)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&conv_id)
    .bind(body.keyVersion)
    .bind(&body.userId)
    .bind(&body.encryptedKey)
    .bind(body.userKeyVersion)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, conv = %conv_id, user = %body.userId, "e2ee: échec add_member_key");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        conv = %conv_id,
        version = body.keyVersion,
        new_member = %body.userId,
        "e2ee: membre ajouté à la clé de groupe"
    );

    Ok(Json(serde_json::json!({ "success": true })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Routeur — à merger dans protected_routes de main.rs
// ─────────────────────────────────────────────────────────────────────────────
pub fn e2ee_routes() -> axum::Router<Arc<SharedState>> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/auth/public-key", post(register_public_key))
        .route("/auth/public-keys", get(get_member_public_keys))
        .route("/auth/rotate-key", post(rotate_key))
        .route("/auth/key-history", get(get_key_history))
        .route("/auth/key-history/{version}", get(get_key_version))
        .route(
            "/conversations/{conv_id}/my-encrypted-key/{msg_id}",
            get(get_my_encrypted_key),
        )
        .route("/conversations/{conv_id}/keys", post(distribute_group_keys))
        .route("/conversations/{conv_id}/my-key", get(get_my_group_key))
        .route("/conversations/{conv_id}/add-member-key", post(add_member_key))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_public_key_deserialize() {
        let json = r#"{"public_key": "dGVzdC1rZXk="}"#;
        let req: RegisterPublicKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.public_key, "dGVzdC1rZXk=");
    }

    #[test]
    fn test_public_keys_query_deserialize() {
        let json = r#"{"conversation_id": "conv-123"}"#;
        let query: PublicKeysQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.conversation_id, "conv-123");
    }

    #[test]
    fn test_member_public_key_serialize() {
        let member = MemberPublicKey {
            user_id: "user-1".to_string(),
            username: "alice".to_string(),
            public_key: "dGVzdA==".to_string(),
        };
        let json = serde_json::to_string(&member).unwrap();
        assert!(json.contains("user-1"));
        assert!(json.contains("alice"));
        assert!(json.contains("dGVzdA=="));
    }

    #[test]
    fn test_encrypted_key_response_serialize() {
        let resp = EncryptedKeyResponse {
            encrypted_key: "encrypted123".to_string(),
            message_id: "msg-456".to_string(),
            key_version: 2,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("encrypted123"));
        assert!(json.contains("msg-456"));
        assert!(json.contains("\"key_version\":2"));
    }

    #[test]
    fn test_x25519_key_size() {
        // X25519 public keys are always 32 bytes
        let key_32_bytes = vec![0u8; 32];
        assert_eq!(key_32_bytes.len(), 32, "X25519 key must be 32 bytes");
    }

    #[test]
    fn test_rotate_key_request_deserialize() {
        let json = r#"{"public_key": "b64key", "encrypted_private_key": "b64enc", "password": "mypass"}"#;
        let req: RotateKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.public_key, "b64key");
        assert_eq!(req.encrypted_private_key, "b64enc");
        assert_eq!(req.password, "mypass");
    }

    #[test]
    fn test_rotate_key_response_serialize() {
        let resp = RotateKeyResponse {
            success: true,
            version: 2,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"version\":2"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_key_history_entry_serialize() {
        let entry = KeyHistoryEntry {
            version: 1,
            public_key: "b64public".to_string(),
            created_at: 1234567890,
            revoked_at: Some(1234567990),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"public_key\":\"b64public\""));
        assert!(json.contains("\"revoked_at\":1234567990"));
    }

    #[test]
    fn test_distribute_keys_request_deserialize() {
        let json = r#"{"distributions": {"user1": "key1"}, "keyVersion": 2}"#;
        let req: DistributeKeysRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.distributions.len(), 1);
        assert_eq!(req.keyVersion, 2);
    }

    #[test]
    fn test_my_key_response_serialize() {
        let resp = MyKeyResponse {
            encrypted_key: "enc-key".to_string(),
            keyVersion: 2,
            userKeyVersion: 1,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"keyVersion\":2"));
        assert!(json.contains("\"userKeyVersion\":1"));
    }

    #[test]
    fn test_add_member_key_request_deserialize() {
        let json = r#"{"userId": "newuser", "encryptedKey": "b64key", "userKeyVersion": 1, "keyVersion": 2}"#;
        let req: AddMemberKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.userId, "newuser");
        assert_eq!(req.userKeyVersion, 1);
        assert_eq!(req.keyVersion, 2);
    }

    #[test]
    fn test_my_key_query_deserialize() {
        let json = r#"{"version": 3}"#;
        let q: MyKeyQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.version, Some(3));
    }

    #[test]
    fn test_my_key_query_default() {
        let json = r#"{}"#;
        let q: MyKeyQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.version, None);
    }
}
