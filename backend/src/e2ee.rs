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
//       L'expéditeur en a besoin pour chiffrer sa clé de session pour chacun.
//
//   POST /api/conversations/{id}/messages  (étendu dans db.rs, pas ici)
//       Le payload inclut désormais encrypted_keys: HashMap<userId, base64key>
//       et nonce: base64. Ce handler est dans db.rs::send_message.
//
//   GET  /api/conversations/{conv_id}/my-encrypted-key/{msg_id}
//       Retourne la clé de session chiffrée pour l'utilisateur courant.

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
}

/// Payload pour stocker les clés chiffrées d'un message E2EE.
/// Appelé depuis db.rs::send_message après insertion du message.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MessageKeysPayload {
    /// user_id → base64(asymNonce || boxCiphertext)
    pub encrypted_keys: HashMap<String, String>,
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
    // L'appelant doit être membre de la conversation
    let is_member: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM conversation_participants
         WHERE conversation_id = ? AND user_id = ?",
    )
    .bind(&params.conversation_id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_member.map(|(c,)| c).unwrap_or(0) == 0 {
        return Err(StatusCode::FORBIDDEN);
    }

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
    // Vérifier appartenance
    let is_member: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM conversation_participants
         WHERE conversation_id = ? AND user_id = ?",
    )
    .bind(&conv_id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_member.map(|(c,)| c).unwrap_or(0) == 0 {
        return Err(StatusCode::FORBIDDEN);
    }

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT encrypted_key FROM message_keys
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

    let (encrypted_key,) = row.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(EncryptedKeyResponse {
        encrypted_key,
        message_id: msg_id,
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
) -> Result<(), sqlx::Error> {
    for (recipient_id, encrypted_key) in encrypted_keys {
        sqlx::query(
            "INSERT OR REPLACE INTO message_keys (message_id, recipient_id, encrypted_key)
             VALUES (?, ?, ?)",
        )
        .bind(message_id)
        .bind(recipient_id)
        .bind(encrypted_key)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Routeur — à merger dans protected_routes de main.rs
// ─────────────────────────────────────────────────────────────────────────────
pub fn e2ee_routes() -> axum::Router<Arc<SharedState>> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/auth/public-key", post(register_public_key))
        .route("/auth/public-keys", get(get_member_public_keys))
        .route(
            "/conversations/{conv_id}/my-encrypted-key/{msg_id}",
            get(get_my_encrypted_key),
        )
}


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
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("encrypted123"));
        assert!(json.contains("msg-456"));
    }

    #[test]
    fn test_x25519_key_size() {
        // X25519 public keys are always 32 bytes
        // Base64 encoded: 32 * 4/3 = 42.67 → 44 chars with padding
        let key_32_bytes = vec![0u8; 32];
        let encoded = base64::encode(&key_32_bytes);
        assert_eq!(encoded.len(), 44, "32 bytes base64 = 44 chars");
    }
}
