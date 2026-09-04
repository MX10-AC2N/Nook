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

    // 6. Archive current keypair: mark the previous version as revoked.
    //    If the previous version already has a row (from a prior rotation),
    //    UPDATE it. Otherwise (first rotation from legacy user with public_key
    //    but no history entry), INSERT a new archive row.
    if let Some((public_key,)) = current_key {
        let archive_version = new_version - 1;
        let result = sqlx::query(
            "UPDATE user_key_history SET revoked_at = ? WHERE user_id = ? AND version = ?",
        )
        .bind(now)
        .bind(&user.id)
        .bind(archive_version)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user.id, "e2ee: échec archive clé actuelle");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // If no existing row was updated, INSERT the archive row
        // (legacy user without history entry, or version 0 genesis)
        if result.rows_affected() == 0 {
            sqlx::query(
                "INSERT INTO user_key_history (user_id, version, public_key, encrypted_priv, created_at, revoked_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&user.id)
            .bind(archive_version)
            .bind(&public_key)
            .bind("")
            .bind(now)
            .bind(now)
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, user_id = %user.id, "e2ee: échec archive clé actuelle (insert)");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
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

/// Récupère la version actuelle (active) de la clé publique d'un utilisateur.
/// Retourne la version avec revoked_at IS NULL, ou 1 si clé initiale sans historique,
/// ou 0 si aucune clé publique.
async fn get_user_active_key_version(
    state: &Arc<SharedState>,
    user_id: &str,
) -> Result<i32, StatusCode> {
    // Check if there's an active (non-revoked) entry in history
    let active: Option<(i32,)> = sqlx::query_as(
        "SELECT version FROM user_key_history WHERE user_id = ? AND revoked_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((v,)) = active {
        return Ok(v);
    }

    // No history — check if user has a public_key (initial key = version 1)
    let has_key: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE id = ? AND public_key IS NOT NULL",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if has_key.map(|(c,)| c).unwrap_or(0) > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
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
    // SEC-FIX-1: Valider que chaque destinataire est membre de la conversation
    // SEC-FIX-3: Utiliser user_key_version dynamique (version actuelle du destinataire)
    for recipient_id in body.distributions.keys() {
        let is_member: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = ? AND user_id = ?",
        )
        .bind(&conv_id)
        .bind(recipient_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if is_member.map(|(c,)| c).unwrap_or(0) == 0 {
            tracing::warn!(
                conv = %conv_id,
                recipient = %recipient_id,
                "e2ee: distribute_group_keys — destinataire non membre rejeté"
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }

    for (recipient_id, encrypted_key) in &body.distributions {
        // SEC-FIX-3: Récupérer la version actuelle de clé publique du destinataire
        let recipient_key_version = get_user_active_key_version(&state, recipient_id).await?;

        sqlx::query(
            "INSERT OR REPLACE INTO conversation_key_recipients
                (conversation_id, version, user_id, encrypted_key, user_key_version, distribution_status)
             VALUES (?, ?, ?, ?, ?, 'delivered')",
        )
        .bind(&conv_id)
        .bind(body.keyVersion)
        .bind(recipient_id)
        .bind(encrypted_key)
        .bind(recipient_key_version)
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
    // SEC-FIX-1: Valider que le nouveau membre est participant de la conversation
    let is_member: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM conversation_participants
         WHERE conversation_id = ? AND user_id = ?",
    )
    .bind(&conv_id)
    .bind(&body.userId)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_member.map(|(c,)| c).unwrap_or(0) == 0 {
        tracing::warn!(
            conv = %conv_id,
            new_member = %body.userId,
            "e2ee: add_member_key — utilisateur cible non membre rejeté"
        );
        return Err(StatusCode::FORBIDDEN);
    }

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
// SEC-FIX-3: Nouvelles routes pour distribution_status + claim-key + metadata
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MyKeyStatusResponse {
    pub user_id: String,
    pub conversation_id: String,
    pub distribution_status: String,
    pub key_version: i32,
    pub user_key_version: i32,
}

#[derive(Debug, Serialize)]
pub struct GroupKeyMetadataResponse {
    pub conversation_id: String,
    pub version: i32,
    pub creator_id: String,
    pub created_at: i64,
    pub recipient_count: i64,
    pub delivered_count: i64,
    pub pending_count: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/conversations/{conv_id}/claim-key
// Si le membre a un statut 'pending', distribuer la clé courante.
// ─────────────────────────────────────────────────────────────────────────────
pub async fn claim_key(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_membership(&state, &user.id, &conv_id).await?;

    // Vérifier si l'utilisateur a une entrée pending
    let pending: Option<(String, i32, i32)> = sqlx::query_as(
        "SELECT encrypted_key, version, user_key_version
         FROM conversation_key_recipients
         WHERE conversation_id = ? AND user_id = ? AND distribution_status = 'pending'",
    )
    .bind(&conv_id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, conv = %conv_id, user = %user.id, "e2ee: échec claim-key query");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some((encrypted_key, version, user_key_version)) = pending {
        // Marquer comme delivered
        sqlx::query(
            "UPDATE conversation_key_recipients
             SET distribution_status = 'delivered'
             WHERE conversation_id = ? AND user_id = ? AND version = ?",
        )
        .bind(&conv_id)
        .bind(&user.id)
        .bind(version)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, conv = %conv_id, user = %user.id, "e2ee: échec claim-key update");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        tracing::info!(
            conv = %conv_id,
            user = %user.id,
            version = version,
            "e2ee: clé claimée (pending → delivered)"
        );

        Ok(Json(serde_json::json!({
            "success": true,
            "status": "delivered",
            "encrypted_key": encrypted_key,
            "keyVersion": version,
            "userKeyVersion": user_key_version
        })))
    } else {
        // Vérifier s'il y a une clé delivered
        let delivered: Option<(i32,)> = sqlx::query_as(
            "SELECT version FROM conversation_key_recipients
             WHERE conversation_id = ? AND user_id = ? AND distribution_status = 'delivered'
             ORDER BY version DESC LIMIT 1",
        )
        .bind(&conv_id)
        .bind(&user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if delivered.is_some() {
            Ok(Json(serde_json::json!({
                "success": true,
                "status": "already_delivered",
                "message": "Clé déjà distribuée"
            })))
        } else {
            Ok(Json(serde_json::json!({
                "success": false,
                "status": "not_found",
                "message": "Aucune clé en attente pour cette conversation"
            })))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/conversations/{conv_id}/my-key-status
// Retourne le statut de distribution (delivered/pending/failed) pour l'utilisateur courant.
// ─────────────────────────────────────────────────────────────────────────────
pub async fn get_my_key_status(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
) -> Result<Json<Vec<MyKeyStatusResponse>>, StatusCode> {
    check_membership(&state, &user.id, &conv_id).await?;

    let rows: Vec<(String, String, i32, i32)> = sqlx::query_as(
        "SELECT user_id, distribution_status, version, user_key_version
         FROM conversation_key_recipients
         WHERE conversation_id = ? AND user_id = ?
         ORDER BY version DESC",
    )
    .bind(&conv_id)
    .bind(&user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, conv = %conv_id, user = %user.id, "e2ee: échec get_my_key_status");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|(user_id, distribution_status, key_version, user_key_version)| {
                MyKeyStatusResponse {
                    user_id,
                    conversation_id: conv_id.clone(),
                    distribution_status,
                    key_version,
                    user_key_version,
                }
            })
            .collect(),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/conversations/{conv_id}/group-key-metadata
// Retourne les métadonnées de la clé de groupe (version, dernière rotation, stats).
// ─────────────────────────────────────────────────────────────────────────────
pub async fn get_group_key_metadata(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(conv_id): Path<String>,
) -> Result<Json<GroupKeyMetadataResponse>, StatusCode> {
    check_membership(&state, &user.id, &conv_id).await?;

    // Récupérer la dernière version de clé de groupe
    let meta: Option<(i32, String, i64)> = sqlx::query_as(
        "SELECT version, creator_id, created_at
         FROM conversation_keys
         WHERE conversation_id = ?
         ORDER BY version DESC LIMIT 1",
    )
    .bind(&conv_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, conv = %conv_id, "e2ee: échec get_group_key_metadata");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (version, creator_id, created_at) = meta.ok_or(StatusCode::NOT_FOUND)?;

    // Compter les statuts de distribution
    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT distribution_status, COUNT(*)
         FROM conversation_key_recipients
         WHERE conversation_id = ? AND version = ?
         GROUP BY distribution_status",
    )
    .bind(&conv_id)
    .bind(version)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut delivered_count = 0i64;
    let mut pending_count = 0i64;
    let mut recipient_count = 0i64;

    for (status, count) in &counts {
        recipient_count += count;
        match status.as_str() {
            "delivered" => delivered_count = *count,
            "pending" => pending_count = *count,
            _ => {}
        }
    }

    Ok(Json(GroupKeyMetadataResponse {
        conversation_id: conv_id,
        version,
        creator_id,
        created_at,
        recipient_count,
        delivered_count,
        pending_count,
    }))
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
        .route("/conversations/{conv_id}/claim-key", post(claim_key))
        .route("/conversations/{conv_id}/my-key-status", get(get_my_key_status))
        .route("/conversations/{conv_id}/group-key-metadata", get(get_group_key_metadata))
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

    // ═════════════════════════════════════════════════════════════════════
    // Integration tests avec base de données SQLite in-memory
    // ═════════════════════════════════════════════════════════════════════

    use axum::extract::{Path, Query, State as AxumState};
    use base64ct::{Base64, Encoding};
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Arc;
    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::auth::CurrentUser;
    use crate::config::Config;
    use crate::db::User;
    use crate::presence::PresenceState;
    use crate::sfu::SfuState;
    use crate::webrtc::{FileManager, WebRtcState};

    // ─── Helpers ───────────────────────────────────────────────────────

    async fn setup_test_db() -> SqlitePool {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
        let pool = SqlitePool::connect_with(opts).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    fn make_state(pool: SqlitePool) -> Arc<SharedState> {
        let config = Config {
            port: 3000,
            database_url: "sqlite::memory:".to_string(),
            static_dir: "/tmp".to_string(),
            uploads_dir: "/tmp".to_string(),
            gifs_dir: "/tmp".to_string(),
            public_site_url: "http://localhost:6300".to_string(),
            allowed_origins: vec![],
            turn_host: "localhost".to_string(),
            turn_port: 3478,
            turn_secret: String::new(),
            sfu_relay_capacity: 500,
        };
        Arc::new(SharedState {
            db: pool,
            webrtc_state: WebRtcState::new(),
            file_manager: Arc::new(FileManager::new(PathBuf::from("/tmp"))),
            sfu_state: SfuState::new(config.sfu_relay_capacity),
            config,
            presence_state: PresenceState::new(),
        })
    }

    async fn seed_user(pool: &SqlitePool, username: &str, password: &str, public_key: Option<&str>) -> User {
        let id = Uuid::new_v4().to_string();
        let hash = crate::auth::hash_password(password);
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, name, role, approved, needs_password_change, token, public_key, created_at)
             VALUES (?, ?, ?, ?, ?, 'user', 1, 0, NULL, ?, ?)"
        )
        .bind(&id)
        .bind(username)
        .bind(&format!("{}@test.com", username))
        .bind(&hash)
        .bind(username)
        .bind(public_key)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
        User {
            id,
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password_hash: hash,
            name: Some(username.to_string()),
            role: "user".to_string(),
            approved: true,
            needs_password_change: false,
            token: None,
            created_at: now,
            avatar_url: None,
            avatar_style: None,
            avatar_seed: None,
        }
    }

    async fn seed_conversation(pool: &SqlitePool, creator_id: &str) -> String {
        let conv_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO conversations (id, name, is_group, created_by, created_at, updated_at)
             VALUES (?, ?, 1, ?, ?, ?)"
        )
        .bind(&conv_id)
        .bind("Test Group")
        .bind(creator_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO conversation_participants (conversation_id, user_id, joined_at)
             VALUES (?, ?, ?)"
        )
        .bind(&conv_id)
        .bind(creator_id)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
        conv_id
    }

    async fn add_participant(pool: &SqlitePool, conv_id: &str, user_id: &str) {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT OR IGNORE INTO conversation_participants (conversation_id, user_id, joined_at)
             VALUES (?, ?, ?)"
        )
        .bind(conv_id)
        .bind(user_id)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn seed_message(
        pool: &SqlitePool,
        conv_id: &str,
        sender_id: &str,
    ) -> String {
        let msg_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, sender_id, content, timestamp, created_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&msg_id)
        .bind(conv_id)
        .bind(sender_id)
        .bind("test encrypted message")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();
        msg_id
    }

    fn valid_public_key() -> String {
        let key = [42u8; 32];
        Base64::encode_string(&key)
    }

    fn invalid_public_key() -> String {
        Base64::encode_string(&[42u8; 16])
    }

    fn bad_base64() -> String {
        "!!!not-valid-base64!!".to_string()
    }

    fn valid_encrypted_private_key() -> String {
        let data = [42u8; 64];
        Base64::encode_string(&data)
    }

    fn make_user_struct(user_id: &str, username: &str, password_hash: &str, now: i64) -> User {
        User {
            id: user_id.to_string(),
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password_hash: password_hash.to_string(),
            name: Some(username.to_string()),
            role: "user".to_string(),
            approved: true,
            needs_password_change: false,
            token: None,
            created_at: now,
            avatar_url: None,
            avatar_style: None,
            avatar_seed: None,
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // POST /api/auth/rotate-key
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_rotate_key_success_first_rotation() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;

        let body = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let resp = result.expect("rotate_key should succeed");
        assert!(resp.success);
        assert_eq!(resp.version, 1, "First rotation should return version 1");
    }

    #[tokio::test]
    async fn test_rotate_key_wrong_password() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let user = seed_user(&pool, "alice", "correct_password", None).await;

        let body = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: "wrong_password".to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("rotate_key with wrong password should fail");
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_rotate_key_invalid_base64() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;

        let body = RotateKeyRequest {
            public_key: bad_base64(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("rotate_key with invalid base64 key should fail");
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_rotate_key_wrong_size_key() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;

        let body = RotateKeyRequest {
            public_key: invalid_public_key(), // 16 bytes, not 32
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("rotate_key with 16-byte key should fail");
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_rotate_key_invalid_encrypted_private_key() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;

        let body = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: bad_base64(),
            password: password.to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("rotate_key with invalid base64 encrypted_private_key should fail");
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_rotate_key_rate_limit() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;

        // Insert 5 prior key history entries directly (so rate limit triggers on 6th attempt)
        let now = chrono::Utc::now().timestamp();
        for v in 0..5 {
            sqlx::query(
                "INSERT INTO user_key_history (user_id, version, public_key, encrypted_priv, created_at, revoked_at)
                 VALUES (?, ?, ?, ?, ?, NULL)"
            )
            .bind(&user.id)
            .bind(v)
            .bind(valid_public_key())
            .bind(valid_encrypted_private_key())
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        }

        let body = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("6th rotation in an hour should be rate limited");
        assert_eq!(err, StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_rotate_key_archives_previous() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let old_pub_key = valid_public_key();
        // Create user WITH an existing public key
        let user = seed_user(&pool, "alice", password, Some(&old_pub_key)).await;
        let now = chrono::Utc::now().timestamp();

        // First rotation — should archive version 0 in user_key_history
        let new_key = valid_public_key();
        let body = RotateKeyRequest {
            public_key: new_key.clone(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user.clone())),
            Json(body),
        )
        .await;

        let resp = result.expect("rotate_key should succeed");
        assert_eq!(resp.version, 1);

        // Verify: old key was archived as version 0
        let archived: Vec<(i32, String, Option<i64>)> = sqlx::query_as(
            "SELECT version, public_key, revoked_at FROM user_key_history
             WHERE user_id = ? AND version = 0"
        )
        .bind(&user.id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(archived.len(), 1, "Archived entry should exist");
        assert_eq!(archived[0].1, old_pub_key, "Archived public_key should match old key");
        assert!(archived[0].2.is_some(), "Archived entry should have revoked_at set");

        // Verify: new key was inserted as version 1
        let current: Vec<(i32, String, Option<i64>)> = sqlx::query_as(
            "SELECT version, public_key, revoked_at FROM user_key_history
             WHERE user_id = ? AND version = 1"
        )
        .bind(&user.id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(current.len(), 1, "New entry should exist");
        assert_eq!(current[0].1, new_key, "New public_key should match");
        assert!(current[0].2.is_none(), "Current entry should have revoked_at = NULL");

        // Verify: users.public_key was updated
        let updated: Option<(String,)> = sqlx::query_as(
            "SELECT public_key FROM users WHERE id = ?"
        )
        .bind(&user.id)
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(updated.unwrap().0, new_key);
    }

    #[tokio::test]
    async fn test_rotate_key_increments_version() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;

        let body1 = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };
        let resp1 = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user.clone())),
            Json(body1),
        )
        .await
        .unwrap();
        assert_eq!(resp1.version, 1);

        // Second rotation — now we have a previous key to archive
        let body2 = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };
        let resp2 = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user.clone())),
            Json(body2),
        )
        .await
        .unwrap();
        assert_eq!(resp2.version, 2, "Second rotation should give version 2");

        // Third rotation
        let body3 = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };
        let resp3 = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user.clone())),
            Json(body3),
        )
        .await
        .unwrap();
        assert_eq!(resp3.version, 3, "Third rotation should give version 3");
    }

    // ═════════════════════════════════════════════════════════════════════
    // GET /api/auth/key-history
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_key_history_empty() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let user = seed_user(&pool, "alice", "pass", None).await;

        let result = get_key_history(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
        )
        .await;

        let entries = result.expect("get_key_history should succeed");
        assert!(entries.is_empty(), "No rotations should give empty history");
    }

    #[tokio::test]
    async fn test_get_key_history_after_rotation() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;

        // Rotate twice
        let pub_key_1 = valid_public_key();
        let body1 = RotateKeyRequest {
            public_key: pub_key_1.clone(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };
        rotate_key(AxumState(state.clone()), Extension(CurrentUser(user.clone())), Json(body1))
            .await
            .unwrap();

        let pub_key_2 = valid_public_key();
        let body2 = RotateKeyRequest {
            public_key: pub_key_2.clone(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };
        rotate_key(AxumState(state.clone()), Extension(CurrentUser(user.clone())), Json(body2))
            .await
            .unwrap();

        let result = get_key_history(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
        )
        .await;

        let entries = result.expect("get_key_history should succeed");
        // Rotation 1: no previous key → skip archive, inserts version 1 = 1 entry
        // Rotation 2: archives version 1 by marking revoked_at, inserts version 2 = 2 entries
        // Total: 2 entries (version 1 revoked, version 2 current)
        assert_eq!(entries.len(), 2, "Should have 2 history entries after 2 rotations");
        assert_eq!(entries[0].version, 1);
        assert_eq!(entries[1].version, 2);
        // Only the latest should have revoked_at = None
        assert!(entries[0].revoked_at.is_some(), "Version 1 should be revoked");
        assert!(entries[1].revoked_at.is_none(), "Version 2 should be current");
    }

    // ═════════════════════════════════════════════════════════════════════
    // GET /api/auth/key-history/{version}
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_key_version_ok() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let user = seed_user(&pool, "alice", password, None).await;
        let expected_enc = valid_encrypted_private_key();

        let body = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: expected_enc.clone(),
            password: password.to_string(),
        };
        rotate_key(AxumState(state.clone()), Extension(CurrentUser(user.clone())), Json(body))
            .await
            .unwrap();

        let result = get_key_version(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Path(1),
        )
        .await;

        let resp = result.expect("get_key_version should succeed");
        assert_eq!(resp.encrypted_private_key, expected_enc);
    }

    #[tokio::test]
    async fn test_get_key_version_not_found() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let user = seed_user(&pool, "alice", "pass", None).await;

        let result = get_key_version(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Path(99),
        )
        .await;

        let err = result.expect_err("Non-existent version should return 404");
        assert_eq!(err, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_key_version_empty_priv_returns_not_found() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let password = "testpassword123";
        let old_pub_key = valid_public_key();
        let user = seed_user(&pool, "alice", password, Some(&old_pub_key)).await;

        // First rotation archives version 0 with empty encrypted_priv (the handler
        // archives with encrypted_priv = "" because initial key has no encrypted priv).
        // Then try to get version 0 — should be NOT_FOUND because encrypted_priv = ''.
        let body = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: password.to_string(),
        };
        rotate_key(AxumState(state.clone()), Extension(CurrentUser(user.clone())), Json(body))
            .await
            .unwrap();

        let result = get_key_version(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Path(0),
        )
        .await;

        let err = result.expect_err("version with empty encrypted_priv should return 404");
        assert_eq!(err, StatusCode::NOT_FOUND);
    }

    // ═════════════════════════════════════════════════════════════════════
    // POST /api/auth/public-key
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_register_public_key_success() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let user = seed_user(&pool, "alice", "pass", None).await;

        let body = RegisterPublicKeyRequest {
            public_key: valid_public_key(),
        };

        let result = register_public_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user.clone())),
            Json(body),
        )
        .await;

        assert!(result.is_ok(), "register_public_key should succeed");

        // Verify the key was stored
        let stored: Option<(String,)> = sqlx::query_as(
            "SELECT public_key FROM users WHERE id = ?"
        )
        .bind(&user.id)
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(stored.unwrap().0.len() > 0);
    }

    #[tokio::test]
    async fn test_register_public_key_wrong_size() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let user = seed_user(&pool, "alice", "pass", None).await;

        let body = RegisterPublicKeyRequest {
            public_key: invalid_public_key(), // 16 bytes, not 32
        };

        let result = register_public_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("register_public_key with wrong size should fail");
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_public_key_invalid_base64() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let user = seed_user(&pool, "alice", "pass", None).await;

        let body = RegisterPublicKeyRequest {
            public_key: bad_base64(),
        };

        let result = register_public_key(
            AxumState(state.clone()),
            Extension(CurrentUser(user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("register_public_key with invalid base64 should fail");
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    // ═════════════════════════════════════════════════════════════════════
    // GET /api/auth/public-keys?conversation_id=xxx
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_member_public_keys_ok() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", Some(&valid_public_key())).await;
        let bob = seed_user(&pool, "bob", "pass", Some(&valid_public_key())).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;

        let query = PublicKeysQuery {
            conversation_id: conv_id,
        };

        let result = get_member_public_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Query(query),
        )
        .await;

        let keys = result.expect("get_member_public_keys should succeed");
        assert_eq!(keys.len(), 2, "Should return 2 members with public keys");
        let user_ids: Vec<&str> = keys.iter().map(|k| k.username.as_str()).collect();
        assert!(user_ids.contains(&"alice"));
        assert!(user_ids.contains(&"bob"));
    }

    #[tokio::test]
    async fn test_get_member_public_keys_not_member() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        // Bob is NOT a participant

        let query = PublicKeysQuery {
            conversation_id: conv_id,
        };

        let result = get_member_public_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(bob)),
            Query(query),
        )
        .await;

        let err = result.expect_err("Non-member should get FORBIDDEN");
        assert_eq!(err, StatusCode::FORBIDDEN);
    }

    // ═════════════════════════════════════════════════════════════════════
    // POST /api/conversations/{conv_id}/keys
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_distribute_group_keys_success() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;

        let mut dist = HashMap::new();
        dist.insert(bob.id.clone(), valid_public_key());
        let body = DistributeKeysRequest {
            distributions: dist,
            keyVersion: 1,
        };

        let result = distribute_group_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Path(conv_id.clone()),
            Json(body),
        )
        .await;

        assert!(result.is_ok(), "distribute_group_keys should succeed");

        // Verify the key was stored
        let stored: Vec<(String, i32)> = sqlx::query_as(
            "SELECT user_id, version FROM conversation_key_recipients
             WHERE conversation_id = ?"
        )
        .bind(&conv_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].1, 1);
    }

    #[tokio::test]
    async fn test_distribute_group_keys_not_member() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        // Bob is NOT in the conversation

        let mut dist = HashMap::new();
        dist.insert(bob.id.clone(), valid_public_key());
        let body = DistributeKeysRequest {
            distributions: dist,
            keyVersion: 1,
        };

        let result = distribute_group_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(bob)),
            Path(conv_id),
            Json(body),
        )
        .await;

        let err = result.expect_err("Non-member should get FORBIDDEN");
        assert_eq!(err, StatusCode::FORBIDDEN);
    }

    // ═════════════════════════════════════════════════════════════════════
    // GET /api/conversations/{conv_id}/my-key
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_my_group_key_found() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", Some(&valid_public_key())).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;

        // Distribute a key so bob has one
        let mut dist = HashMap::new();
        dist.insert(bob.id.clone(), "bob-encrypted-key".to_string());
        let dist_body = DistributeKeysRequest {
            distributions: dist,
            keyVersion: 1,
        };
        distribute_group_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Path(conv_id.clone()),
            Json(dist_body),
        )
        .await
        .unwrap();

        let query = MyKeyQuery { version: None };
        let result = get_my_group_key(
            AxumState(state.clone()),
            Extension(CurrentUser(bob.clone())),
            Path(conv_id.clone()),
            Query(query),
        )
        .await;

        let resp = result.expect("get_my_group_key should succeed");
        assert_eq!(resp.encrypted_key, "bob-encrypted-key");
        assert_eq!(resp.keyVersion, 1);
        assert_eq!(resp.userKeyVersion, 1);
    }

    #[tokio::test]
    async fn test_get_my_group_key_specific_version() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;

        // Distribute key version 1
        let mut dist = HashMap::new();
        dist.insert(bob.id.clone(), "enc-key-v1".to_string());
        distribute_group_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(alice.clone())),
            Path(conv_id.clone()),
            Json(DistributeKeysRequest { distributions: dist, keyVersion: 1 }),
        )
        .await
        .unwrap();

        // Distribute key version 2
        let mut dist2 = HashMap::new();
        dist2.insert(bob.id.clone(), "enc-key-v2".to_string());
        distribute_group_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Path(conv_id.clone()),
            Json(DistributeKeysRequest { distributions: dist2, keyVersion: 2 }),
        )
        .await
        .unwrap();

        // Request version 1 specifically
        let query = MyKeyQuery { version: Some(1) };
        let result = get_my_group_key(
            AxumState(state.clone()),
            Extension(CurrentUser(bob)),
            Path(conv_id),
            Query(query),
        )
        .await;

        let resp = result.expect("get_my_group_key with version should succeed");
        assert_eq!(resp.encrypted_key, "enc-key-v1");
        assert_eq!(resp.keyVersion, 1);
    }

    #[tokio::test]
    async fn test_get_my_group_key_not_found() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;

        let query = MyKeyQuery { version: None };
        let result = get_my_group_key(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Path(conv_id),
            Query(query),
        )
        .await;

        let err = result.expect_err("No key should return 404");
        assert_eq!(err, StatusCode::NOT_FOUND);
    }

    // ═════════════════════════════════════════════════════════════════════
    // POST /api/conversations/{conv_id}/add-member-key
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_add_member_key_success() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;

        // First distribute a group key version 1
        let mut dist = HashMap::new();
        dist.insert(alice.id.clone(), "alice-key".to_string());
        distribute_group_keys(
            AxumState(state.clone()),
            Extension(CurrentUser(alice.clone())),
            Path(conv_id.clone()),
            Json(DistributeKeysRequest { distributions: dist, keyVersion: 1 }),
        )
        .await
        .unwrap();

        // Now add bob to the same key version
        let body = AddMemberKeyRequest {
            userId: bob.id.clone(),
            encryptedKey: "bob-encrypted-key".to_string(),
            userKeyVersion: 1,
            keyVersion: 1,
        };

        let result = add_member_key(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Path(conv_id.clone()),
            Json(body),
        )
        .await;

        assert!(result.is_ok(), "add_member_key should succeed");

        // Verify bob can retrieve it
        let query = MyKeyQuery { version: Some(1) };
        let bob_result = get_my_group_key(
            AxumState(state.clone()),
            Extension(CurrentUser(bob)),
            Path(conv_id),
            Query(query),
        )
        .await
        .unwrap();
        assert_eq!(bob_result.encrypted_key, "bob-encrypted-key");
    }

    #[tokio::test]
    async fn test_add_member_key_no_such_group_version() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;

        let body = AddMemberKeyRequest {
            userId: bob.id.clone(),
            encryptedKey: "bob-key".to_string(),
            userKeyVersion: 1,
            keyVersion: 999, // non-existent version
        };

        let result = add_member_key(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Path(conv_id),
            Json(body),
        )
        .await;

        let err = result.expect_err("Non-existent group key version should return 404");
        assert_eq!(err, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_add_member_key_sender_not_member() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let mallet = seed_user(&pool, "mallet", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;
        // mallet is NOT in the conversation

        let body = AddMemberKeyRequest {
            userId: bob.id.clone(),
            encryptedKey: "bob-key".to_string(),
            userKeyVersion: 1,
            keyVersion: 1,
        };

        let result = add_member_key(
            AxumState(state.clone()),
            Extension(CurrentUser(mallet)),
            Path(conv_id),
            Json(body),
        )
        .await;

        let err = result.expect_err("Non-member should get FORBIDDEN");
        assert_eq!(err, StatusCode::FORBIDDEN);
    }

    // ═════════════════════════════════════════════════════════════════════
    // GET /api/conversations/{conv_id}/my-encrypted-key/{msg_id}
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_my_encrypted_key_ok() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;

        // Insert a message with encrypted keys
        let msg_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, sender_id, content, message_type, encrypted, timestamp, created_at)
             VALUES (?, ?, ?, 'encrypted-content', 'text', 1, ?, ?)"
        )
        .bind(&msg_id)
        .bind(&conv_id)
        .bind(&alice.id)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // Store encrypted key for bob
        let mut keys = HashMap::new();
        keys.insert(bob.id.clone(), "base64encryptedkey".to_string());
        store_message_keys(&pool, &msg_id, &keys, 1).await.unwrap();

        let result = get_my_encrypted_key(
            AxumState(state.clone()),
            Extension(CurrentUser(bob)),
            Path((conv_id, msg_id.clone())),
        )
        .await;

        let resp = result.expect("get_my_encrypted_key should succeed");
        assert_eq!(resp.encrypted_key, "base64encryptedkey");
        assert_eq!(resp.message_id, msg_id);
    }

    #[tokio::test]
    async fn test_get_my_encrypted_key_not_found() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;

        let result = get_my_encrypted_key(
            AxumState(state.clone()),
            Extension(CurrentUser(alice)),
            Path((conv_id, "nonexistent-msg-id".to_string())),
        )
        .await;

        let err = result.expect_err("Non-existent message should return 404");
        assert_eq!(err, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_my_encrypted_key_not_member() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        // bob is NOT a participant

        let result = get_my_encrypted_key(
            AxumState(state.clone()),
            Extension(CurrentUser(bob)),
            Path((conv_id, "any-msg".to_string())),
        )
        .await;

        let err = result.expect_err("Non-member should get FORBIDDEN");
        assert_eq!(err, StatusCode::FORBIDDEN);
    }

    // ═════════════════════════════════════════════════════════════════════
    // store_message_keys (appelée depuis db.rs::send_message)
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_store_message_keys_persists() {
        let pool = setup_test_db().await;
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;
        let msg_id = seed_message(&pool, &conv_id, &alice.id).await;

        let mut keys = HashMap::new();
        keys.insert(alice.id.clone(), "alice-enc-key".to_string());
        keys.insert(bob.id.clone(), "bob-enc-key".to_string());

        store_message_keys(&pool, &msg_id, &keys, 2).await.unwrap();

        // Verify entries
        let rows: Vec<(String, String, i32)> = sqlx::query_as(
            "SELECT recipient_id, encrypted_key, sender_key_version FROM message_keys WHERE message_id = ?"
        )
        .bind(&msg_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].2, 2);
        assert_eq!(rows[1].2, 2);

        // Verify sender_key_version was stored
        let versions: Vec<i32> = rows.iter().map(|r| r.2).collect();
        assert!(versions.iter().all(|&v| v == 2));
    }

    #[tokio::test]
    async fn test_store_message_keys_upsert() {
        let pool = setup_test_db().await;
        let alice = seed_user(&pool, "alice", "pass", None).await;
        let bob = seed_user(&pool, "bob", "pass", None).await;
        let conv_id = seed_conversation(&pool, &alice.id).await;
        add_participant(&pool, &conv_id, &bob.id).await;
        let msg_id = seed_message(&pool, &conv_id, &alice.id).await;

        let mut keys = HashMap::new();
        keys.insert(alice.id.clone(), "old-key".to_string());
        keys.insert(bob.id.clone(), "bob-key".to_string());
        store_message_keys(&pool, &msg_id, &keys, 1).await.unwrap();

        // Upsert alice's key only
        let mut new_keys = HashMap::new();
        new_keys.insert(alice.id.clone(), "new-key".to_string());
        store_message_keys(&pool, &msg_id, &new_keys, 2).await.unwrap();

        let rows: Vec<(String, String, i32)> = sqlx::query_as(
            "SELECT recipient_id, encrypted_key, sender_key_version FROM message_keys WHERE message_id = ? ORDER BY recipient_id"
        )
        .bind(&msg_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 2, "Should still have 2 rows after upsert");
        let alice_row = rows.iter().find(|r| r.0 == alice.id).unwrap();
        assert_eq!(alice_row.1, "new-key", "Alice's key should be upserted");
        assert_eq!(alice_row.2, 2, "Alice's sender_key_version should be updated");
        let bob_row = rows.iter().find(|r| r.0 == bob.id).unwrap();
        assert_eq!(bob_row.1, "bob-key", "Bob's key should be untouched");
        assert_eq!(bob_row.2, 1, "Bob's sender_key_version should be untouched");
    }

    // ═════════════════════════════════════════════════════════════════════
    // Authentification: utilisateur non approuvé / inexistant
    // ═════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_rotate_key_user_not_found() {
        let pool = setup_test_db().await;
        let state = make_state(pool.clone());
        // Construct a User whose ID doesn't exist in the DB
        let phantom_user = User {
            id: "nonexistent-id".to_string(),
            username: "ghost".to_string(),
            email: "ghost@test.com".to_string(),
            password_hash: String::new(),
            name: Some("ghost".to_string()),
            role: "user".to_string(),
            approved: true,
            needs_password_change: false,
            token: None,
            created_at: 0,
            avatar_url: None,
            avatar_style: None,
            avatar_seed: None,
        };

        let body = RotateKeyRequest {
            public_key: valid_public_key(),
            encrypted_private_key: valid_encrypted_private_key(),
            password: "any".to_string(),
        };

        let result = rotate_key(
            AxumState(state.clone()),
            Extension(CurrentUser(phantom_user)),
            Json(body),
        )
        .await;

        let err = result.expect_err("Non-existent user should fail");
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }
}
