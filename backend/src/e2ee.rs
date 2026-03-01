// POST /api/auth/public-key
pub async fn register_public_key(
    user: UserExtractor,
    body: axum::extract::RawBody, // ou Bytes
) -> Result<Json<()>, AppError> {
    let bytes = axum::body::to_bytes(body, usize::MAX).await?;
    if bytes.len() != 32 { return Err(AppError::BadRequest); }

    sqlx::query!(
        "INSERT OR REPLACE INTO users (id, public_key) VALUES (?, ?)",
        user.id, bytes.as_ref()
    )
    .execute(&pool)
    .await?;
    Ok(Json(()))
}

// POST /api/conversations/{id}/keys
pub async fn set_conversation_keys(
    user: UserExtractor,
    Path(convo_id): Path<String>,
    Json(payload): Json<DistributionPayload>,
) -> Result<Json<()>, AppError> {
    // Vérif que user est membre + admin/creator si tu veux
    for (uid, sealed_b64) in payload.distributions {
        let sealed = base64::decode(sealed_b64)?;
        sqlx::query!(
            "INSERT OR REPLACE INTO conversation_keys 
             (conversation_id, user_id, key_version, encrypted_group_key) 
             VALUES (?, ?, ?, ?)",
            convo_id, uid, payload.key_version, sealed
        )
        .execute(&pool)
        .await?;
    }
    Ok(Json(()))
}

// GET /api/conversations/{id}/my-key
pub async fn get_my_key(
    user: UserExtractor,
    Path(convo_id): Path<String>,
) -> Result<Json<MyKeyResponse>, AppError> {
    let row = sqlx::query!(
        "SELECT encrypted_group_key, key_version 
         FROM conversation_keys 
         WHERE conversation_id = ? AND user_id = ? 
         ORDER BY key_version DESC LIMIT 1",
        convo_id, user.id
    )
    .fetch_optional(&pool)
    .await?;

    let row = row.ok_or(AppError::NotFound)?;
    Ok(Json(MyKeyResponse {
        encrypted_key: base64::encode(row.encrypted_group_key),
        key_version: row.key_version,
    }))
}

#[derive(Deserialize)]
struct DistributionPayload {
    distributions: HashMap<String, String>,
    key_version: i32,
}

#[derive(Serialize)]
struct MyKeyResponse {
    encrypted_key: String,
    key_version: i32,
}
// POST /api/conversations/{id}/add-member-key
pub async fn add_member_key(
    admin: AdminOrCreatorExtractor, // tu as déjà ce guard
    Path(convo_id): Path<String>,
    Json(payload): Json<AddMemberKeyPayload>,
) -> Result<Json<()>, AppError> {
    sqlx::query!(
        "INSERT INTO conversation_keys 
         (conversation_id, user_id, key_version, encrypted_group_key)
         VALUES (?, ?, ?, ?)",
        convo_id, payload.user_id, payload.key_version, base64::decode(&payload.encrypted_key)?
    ).execute(&pool).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
struct AddMemberKeyPayload {
    user_id: String,
    encrypted_key: String,
    key_version: i32,
}
