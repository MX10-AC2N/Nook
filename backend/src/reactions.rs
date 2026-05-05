     1|// backend/src/reactions.rs — Réactions aux messages
     2|// Session 35 — POST/DELETE réaction + GET agrégées par message
     3|//
     4|// Endpoints :
     5|//   POST   /api/conversations/{conv_id}/messages/{msg_id}/reactions  { emoji }
     6|//   DELETE /api/conversations/{conv_id}/messages/{msg_id}/reactions
     7|//   GET    /api/conversations/{conv_id}/messages/{msg_id}/reactions
     8|//
     9|// Broadcast WS à chaque changement → type "reaction_updated"
    10|
    11|#![allow(clippy::for_kv_map)]
    12|
    13|use std::sync::Arc;
    14|
    15|use axum::{
    16|    extract::{Path, State},
    17|    http::StatusCode,
    18|    response::Json,
    19|    Extension,
    20|};
    21|use chrono::Utc;
    22|use serde::{Deserialize, Serialize};
    23|use serde_json::{json, Value};
    24|
    25|use crate::{auth::CurrentUser, SharedState};
    26|
    27|// ────────────────────────────────────────────────────────────────────────────
    28|// Types
    29|// ────────────────────────────────────────────────────────────────────────────
    30|
    31|#[derive(Debug, Deserialize)]
    32|pub struct AddReactionRequest {
    33|    pub emoji: String,
    34|}
    35|
    36|/// Agrégat retourné par GET /reactions : { "👍": ["alice", "bob"], "❤️": ["carol"] }
    37|/// + ma propre réaction pour l'UI
    38|#[derive(Debug, Serialize)]
    39|#[allow(dead_code)]
    40|pub struct ReactionsResponse {
    41|    pub counts: std::collections::HashMap<String, Vec<String>>, // emoji → [user_name]
    42|    pub my_emoji: Option<String>,                                // réaction de l'utilisateur courant
    43|}
    44|
    45|// ────────────────────────────────────────────────────────────────────────────
    46|// Helpers
    47|// ────────────────────────────────────────────────────────────────────────────
    48|
    49|/// Vérifie que l'utilisateur est membre de la conversation.
    50|async fn check_conv_membership(
    51|    db: &sqlx::SqlitePool,
    52|    conv_id: &str,
    53|    user_id: &str,
    54|) -> Result<(), StatusCode> {
    55|    let row: Option<(i64,)> = sqlx::query_as(
    56|        "SELECT COUNT(*) FROM conversation_participants WHERE conversation_id = ? AND user_id = ?",
    57|    )
    58|    .bind(conv_id)
    59|    .bind(user_id)
    60|    .fetch_optional(db)
    61|    .await
    62|    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    63|
    64|    if row.map(|(c,)| c).unwrap_or(0) == 0 {
    65|        return Err(StatusCode::FORBIDDEN);
    66|    }
    67|    Ok(())
    68|}
    69|
    70|/// Vérifie que le message appartient à la conversation.
    71|async fn check_msg_in_conv(
    72|    db: &sqlx::SqlitePool,
    73|    conv_id: &str,
    74|    msg_id: &str,
    75|) -> Result<(), StatusCode> {
    76|    let row: Option<(i64,)> = sqlx::query_as(
    77|        "SELECT COUNT(*) FROM messages WHERE id = ? AND conversation_id = ?",
    78|    )
    79|    .bind(msg_id)
    80|    .bind(conv_id)
    81|    .fetch_optional(db)
    82|    .await
    83|    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    84|
    85|    if row.map(|(c,)| c).unwrap_or(0) == 0 {
    86|        return Err(StatusCode::NOT_FOUND);
    87|    }
    88|    Ok(())
    89|}
    90|
    91|/// Charge les réactions agrégées d'un message → HashMap<emoji, Vec<user_name>>.
    92|async fn load_reactions(
    93|    db: &sqlx::SqlitePool,
    94|    msg_id: &str,
    95|) -> Result<std::collections::HashMap<String, Vec<String>>, StatusCode> {
    96|    #[derive(sqlx::FromRow)]
    97|    struct Row {
    98|        emoji: String,
    99|        user_name: String,
   100|    }
   101|
   102|    let rows: Vec<Row> = sqlx::query_as::<_, Row>(
   103|        "SELECT r.emoji, COALESCE(u.name, u.username) AS user_name
   104|         FROM message_reactions r
   105|         JOIN users u ON u.id = r.user_id
   106|         WHERE r.message_id = ?
   107|         ORDER BY r.created_at ASC",
   108|    )
   109|    .bind(msg_id)
   110|    .fetch_all(db)
   111|    .await
   112|    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
   113|
   114|    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
   115|    for row in rows {
   116|        map.entry(row.emoji).or_default().push(row.user_name);
   117|    }
   118|    Ok(map)
   119|}
   120|
   121|/// Broadcast WS "reaction_updated" à toutes les connexions actives.
   122|async fn broadcast_reaction(state: &SharedState, conv_id: &str, msg_id: &str) {
   123|    let payload = json!({
   124|        "type": "reaction_updated",
   125|        "conversation_id": conv_id,
   126|        "message_id": msg_id,
   127|    });
   128|    let guard = state.webrtc_state.broadcasts.lock().await;
   129|    for tx in guard.values() {
   130|        let _ = tx.send(payload.to_string());
   131|    }
   132|}
   133|
   134|// ────────────────────────────────────────────────────────────────────────────
   135|// POST /api/conversations/{conv_id}/messages/{msg_id}/reactions
   136|// Body : { "emoji": "👍" }
   137|// Règle : 1 réaction par user par message — UPSERT (remplace si emoji différent)
   138|// ────────────────────────────────────────────────────────────────────────────
   139|
   140|pub async fn add_reaction(
   141|    State(state): State<Arc<SharedState>>,
   142|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   143|    Path((conv_id, msg_id)): Path<(String, String)>,
   144|    Json(req): Json<AddReactionRequest>,
   145|) -> Result<Json<Value>, StatusCode> {
   146|    check_conv_membership(&state.db, &conv_id, &user.id).await?;
   147|    check_msg_in_conv(&state.db, &conv_id, &msg_id).await?;
   148|
   149|    let now = Utc::now().timestamp();
   150|
   151|    // UPSERT : INSERT ou UPDATE si même (message_id, user_id) existe déjà
   152|    sqlx::query(
   153|        "INSERT INTO message_reactions (message_id, user_id, emoji, created_at)
   154|         VALUES (?, ?, ?, ?)
   155|         ON CONFLICT(message_id, user_id) DO UPDATE SET emoji = excluded.emoji, created_at = excluded.created_at",
   156|    )
   157|    .bind(&msg_id)
   158|    .bind(&user.id)
   159|    .bind(&req.emoji)
   160|    .bind(now)
   161|    .execute(&state.db)
   162|    .await
   163|    .map_err(|e| {
   164|        tracing::error!(error = %e, "add_reaction: DB error");
   165|        StatusCode::INTERNAL_SERVER_ERROR
   166|    })?;
   167|
   168|    // Broadcast + retourner l'état complet
   169|    broadcast_reaction(&state, &conv_id, &msg_id).await;
   170|    let counts = load_reactions(&state.db, &msg_id).await?;
   171|
   172|    Ok(Json(json!({
   173|        "success": true,
   174|        "message_id": msg_id,
   175|        "counts": counts,
   176|        "my_emoji": req.emoji,
   177|    })))
   178|}
   179|
   180|// ────────────────────────────────────────────────────────────────────────────
   181|// DELETE /api/conversations/{conv_id}/messages/{msg_id}/reactions
   182|// Retire la réaction de l'utilisateur courant.
   183|// ────────────────────────────────────────────────────────────────────────────
   184|
   185|pub async fn remove_reaction(
   186|    State(state): State<Arc<SharedState>>,
   187|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   188|    Path((conv_id, msg_id)): Path<(String, String)>,
   189|) -> Result<Json<Value>, StatusCode> {
   190|    check_conv_membership(&state.db, &conv_id, &user.id).await?;
   191|
   192|    sqlx::query(
   193|        "DELETE FROM message_reactions WHERE message_id = ? AND user_id = ?",
   194|    )
   195|    .bind(&msg_id)
   196|    .bind(&user.id)
   197|    .execute(&state.db)
   198|    .await
   199|    .map_err(|e| {
   200|        tracing::error!(error = %e, "remove_reaction: DB error");
   201|        StatusCode::INTERNAL_SERVER_ERROR
   202|    })?;
   203|
   204|    broadcast_reaction(&state, &conv_id, &msg_id).await;
   205|    let counts = load_reactions(&state.db, &msg_id).await?;
   206|
   207|    Ok(Json(json!({
   208|        "success": true,
   209|        "message_id": msg_id,
   210|        "counts": counts,
   211|        "my_emoji": null,
   212|    })))
   213|}
   214|
   215|// ────────────────────────────────────────────────────────────────────────────
   216|// GET /api/conversations/{conv_id}/messages/{msg_id}/reactions
   217|// ────────────────────────────────────────────────────────────────────────────
   218|
   219|pub async fn get_reactions(
   220|    State(state): State<Arc<SharedState>>,
   221|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   222|    Path((conv_id, msg_id)): Path<(String, String)>,
   223|) -> Result<Json<Value>, StatusCode> {
   224|    check_conv_membership(&state.db, &conv_id, &user.id).await?;
   225|
   226|    let counts = load_reactions(&state.db, &msg_id).await?;
   227|
   228|    // Récupérer ma propre réaction
   229|    let my_emoji: Option<String> = sqlx::query_as::<_, (String,)>(
   230|        "SELECT emoji FROM message_reactions WHERE message_id = ? AND user_id = ?",
   231|    )
   232|    .bind(&msg_id)
   233|    .bind(&user.id)
   234|    .fetch_optional(&state.db)
   235|    .await
   236|    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
   237|    .map(|(e,)| e);
   238|
   239|    Ok(Json(json!({
   240|        "message_id": msg_id,
   241|        "counts": counts,
   242|        "my_emoji": my_emoji,
   243|    })))
   244|}
   245|
   246|// ────────────────────────────────────────────────────────────────────────────
   247|// Router — à merger dans protected_routes dans main.rs
   248|// ────────────────────────────────────────────────────────────────────────────
   249|
   250|pub fn reactions_routes() -> axum::Router<Arc<SharedState>> {
   251|    use axum::routing::post;
   252|    axum::Router::new()
   253|        .route(
   254|            "/conversations/{conv_id}/messages/{msg_id}/reactions",
   255|            post(add_reaction).delete(remove_reaction).get(get_reactions),
   256|        )
   257|}
   258|
   259|
   260|#[cfg(test)]
   261|mod tests {
   262|    use super::*;
   263|
   264|    #[test]
   265|    fn test_add_reaction_request_deserialize() {
   266|        let json = r#"{"emoji": "👍"}"#;
   267|        let req: AddReactionRequest = serde_json::from_str(json).unwrap();
   268|        assert_eq!(req.emoji, "👍");
   269|    }
   270|
   271|    #[test]
   272|    fn test_add_reaction_request_invalid_json() {
   273|        let json = r#"{"invalid": "field"}"#;
   274|        let result: Result<AddReactionRequest, _> = serde_json::from_str(json);
   275|        // emoji field is missing, but serde doesn't error on missing fields by default
   276|        // unless #[serde(deny_unknown_fields)] is set
   277|        assert!(result.is_ok()); // serde allows missing fields
   278|    }
   279|}
   280|