// backend/src/polls.rs
// Sondages familiaux — CRUD + vote + fermeture
//
// Routes montées dans main.rs via polls_routes() :
//   GET    /api/polls             → liste tous les sondages
//   POST   /api/polls             → créer un sondage
//   GET    /api/polls/{id}        → détail + résultats
//   POST   /api/polls/{id}/vote   → voter ou changer son vote
//   POST   /api/polls/{id}/close  → fermer (créateur ou admin)
//   DELETE /api/polls/{id}        → supprimer (créateur ou admin)
//
// Utilise uniquement sqlx::query/query_as non-macro
// (SQLX_OFFLINE=true, queries.json vide → pas de compile-time check)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Extension, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::SharedState;

// ─────────────────────────────────────────────────────────────────────────────
// Types requête
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreatePollRequest {
    pub question: String,
    pub options: Vec<String>, // 2–10 options
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub option_id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Types réponse
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PollOption {
    pub id: String,
    pub text: String,
    pub position: i64,
    pub votes: i64,
    pub voters: Vec<String>, // COALESCE(name, username) pour chaque votant
}

#[derive(Debug, Serialize)]
pub struct PollResult {
    pub id: String,
    pub question: String,
    pub created_by: String,
    pub created_by_name: String,
    pub created_at: i64,
    pub closed_at: Option<i64>,
    pub is_closed: bool,
    pub total_votes: i64,
    pub options: Vec<PollOption>,
    pub my_vote: Option<String>, // option_id voté par l'utilisateur courant
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper : charge un sondage complet (options + votes + créateur)
// ─────────────────────────────────────────────────────────────────────────────

async fn load_poll(
    pool: &sqlx::SqlitePool,
    poll_id: &str,
    current_user_id: &str,
) -> Option<PollResult> {
    // 1. Sondage + nom créateur
    let row: Option<(String, String, String, String, i64, Option<i64>)> = sqlx::query_as(
        r#"SELECT p.id, p.question, p.created_by,
                  COALESCE(u.name, u.username) AS creator_name,
                  p.created_at, p.closed_at
           FROM polls p
           LEFT JOIN users u ON u.id = p.created_by
           WHERE p.id = ?"#,
    )
    .bind(poll_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    let (id, question, created_by, creator_name, created_at, closed_at) = row;
    let is_closed = closed_at.is_some();

    // 2. Options
    let options_raw: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT id, text, position FROM poll_options WHERE poll_id = ? ORDER BY position ASC",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 3. Comptage votes par option
    let vote_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT option_id, COUNT(*) AS cnt FROM poll_votes WHERE poll_id = ? GROUP BY option_id",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 4. Noms des votants par option (transparence familiale)
    let voter_names: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT pv.option_id, COALESCE(u.name, u.username)
           FROM poll_votes pv
           LEFT JOIN users u ON u.id = pv.user_id
           WHERE pv.poll_id = ?"#,
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 5. Mon vote
    let my_vote: Option<(String,)> = sqlx::query_as(
        "SELECT option_id FROM poll_votes WHERE poll_id = ? AND user_id = ?",
    )
    .bind(poll_id)
    .bind(current_user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    // 6. Assemblage
    let mut total_votes: i64 = 0;
    let options = options_raw
        .into_iter()
        .map(|(opt_id, text, position)| {
            let votes = vote_counts
                .iter()
                .find(|(oid, _)| oid == &opt_id)
                .map(|(_, c)| *c)
                .unwrap_or(0);
            total_votes += votes;
            let voters = voter_names
                .iter()
                .filter(|(oid, _)| oid == &opt_id)
                .map(|(_, name)| name.clone())
                .collect();
            PollOption { id: opt_id, text, position, votes, voters }
        })
        .collect();

    Some(PollResult {
        id,
        question,
        created_by,
        created_by_name: creator_name,
        created_at,
        closed_at,
        is_closed,
        total_votes,
        options,
        my_vote: my_vote.map(|(oid,)| oid),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/polls
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_polls(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    let ids: Vec<(String,)> =
        sqlx::query_as("SELECT id FROM polls ORDER BY created_at DESC LIMIT 100")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

    let mut results = Vec::with_capacity(ids.len());
    for (id,) in ids {
        if let Some(poll) = load_poll(&state.db, &id, &user.id).await {
            results.push(poll);
        }
    }
    Json(json!({ "polls": results })).into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/polls/{id}
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_poll(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(poll_id): Path<String>,
) -> impl IntoResponse {
    match load_poll(&state.db, &poll_id, &user.id).await {
        Some(p) => Json(json!({ "poll": p })).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "message": "Sondage introuvable" }))).into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/polls
// ─────────────────────────────────────────────────────────────────────────────

pub async fn create_poll(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Json(req): Json<CreatePollRequest>,
) -> impl IntoResponse {
    let question = req.question.trim().to_string();
    if question.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "message": "Question requise" }))).into_response();
    }
    let options: Vec<String> = req.options.iter()
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .collect();
    if options.len() < 2 {
        return (StatusCode::BAD_REQUEST, Json(json!({ "message": "Au moins 2 options requises" }))).into_response();
    }
    if options.len() > 10 {
        return (StatusCode::BAD_REQUEST, Json(json!({ "message": "Maximum 10 options" }))).into_response();
    }

    let poll_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    if let Err(e) = sqlx::query(
        "INSERT INTO polls (id, question, created_by, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&poll_id).bind(&question).bind(&user.id).bind(now)
    .execute(&state.db).await
    {
        tracing::error!(error = %e, "create_poll INSERT polls");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Erreur création sondage" }))).into_response();
    }

    for (i, text) in options.iter().enumerate() {
        let opt_id = Uuid::new_v4().to_string();
        if let Err(e) = sqlx::query(
            "INSERT INTO poll_options (id, poll_id, text, position) VALUES (?, ?, ?, ?)",
        )
        .bind(&opt_id).bind(&poll_id).bind(text).bind(i as i64)
        .execute(&state.db).await
        {
            tracing::error!(error = %e, "create_poll INSERT poll_options");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Erreur création options" }))).into_response();
        }
    }

    match load_poll(&state.db, &poll_id, &user.id).await {
        Some(p) => (StatusCode::CREATED, Json(json!({ "poll": p }))).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))).into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/polls/{id}/vote
// ─────────────────────────────────────────────────────────────────────────────

pub async fn vote_poll(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(poll_id): Path<String>,
    Json(req): Json<VoteRequest>,
) -> impl IntoResponse {
    // Vérifier existence + statut ouvert
    let row: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT closed_at FROM polls WHERE id = ?")
            .bind(&poll_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    match row {
        None => return (StatusCode::NOT_FOUND, Json(json!({ "message": "Sondage introuvable" }))).into_response(),
        Some((Some(_),)) => return (StatusCode::BAD_REQUEST, Json(json!({ "message": "Sondage fermé" }))).into_response(),
        _ => {}
    }

    // Option valide pour ce sondage ?
    let opt_ok: Option<(String,)> =
        sqlx::query_as("SELECT id FROM poll_options WHERE id = ? AND poll_id = ?")
            .bind(&req.option_id).bind(&poll_id)
            .fetch_optional(&state.db).await.ok().flatten();

    if opt_ok.is_none() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "message": "Option invalide" }))).into_response();
    }

    let now = Utc::now().timestamp();

    // UPSERT : vote initial OU changement de vote
    if let Err(e) = sqlx::query(
        r#"INSERT INTO poll_votes (poll_id, user_id, option_id, voted_at)
           VALUES (?, ?, ?, ?)
           ON CONFLICT(poll_id, user_id) DO UPDATE SET
               option_id = excluded.option_id,
               voted_at  = excluded.voted_at"#,
    )
    .bind(&poll_id).bind(&user.id).bind(&req.option_id).bind(now)
    .execute(&state.db).await
    {
        tracing::error!(error = %e, "vote_poll UPSERT");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Erreur vote" }))).into_response();
    }

    match load_poll(&state.db, &poll_id, &user.id).await {
        Some(p) => Json(json!({ "poll": p })).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))).into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/polls/{id}/close
// ─────────────────────────────────────────────────────────────────────────────

pub async fn close_poll(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(poll_id): Path<String>,
) -> impl IntoResponse {
    let row: Option<(String, Option<i64>)> =
        sqlx::query_as("SELECT created_by, closed_at FROM polls WHERE id = ?")
            .bind(&poll_id)
            .fetch_optional(&state.db).await.ok().flatten();

    match row {
        None => return (StatusCode::NOT_FOUND, Json(json!({ "message": "Sondage introuvable" }))).into_response(),
        Some((_, Some(_))) => return (StatusCode::BAD_REQUEST, Json(json!({ "message": "Déjà fermé" }))).into_response(),
        Some((creator, None)) if creator != user.id && user.role != "admin" => {
            return (StatusCode::FORBIDDEN, Json(json!({ "message": "Accès refusé" }))).into_response();
        }
        _ => {}
    }

    let now = Utc::now().timestamp();
    sqlx::query("UPDATE polls SET closed_at = ? WHERE id = ?")
        .bind(now).bind(&poll_id)
        .execute(&state.db).await.ok();

    match load_poll(&state.db, &poll_id, &user.id).await {
        Some(p) => Json(json!({ "poll": p })).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))).into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /api/polls/{id}
// ─────────────────────────────────────────────────────────────────────────────

pub async fn delete_poll(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(poll_id): Path<String>,
) -> impl IntoResponse {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT created_by FROM polls WHERE id = ?")
            .bind(&poll_id)
            .fetch_optional(&state.db).await.ok().flatten();

    match row {
        None => return (StatusCode::NOT_FOUND, Json(json!({ "message": "Sondage introuvable" }))).into_response(),
        Some((creator,)) if creator != user.id && user.role != "admin" => {
            return (StatusCode::FORBIDDEN, Json(json!({ "message": "Accès refusé" }))).into_response();
        }
        _ => {}
    }

    sqlx::query("DELETE FROM polls WHERE id = ?")
        .bind(&poll_id)
        .execute(&state.db).await.ok();

    Json(json!({ "success": true })).into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

pub fn polls_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/polls",              get(list_polls).post(create_poll))
        .route("/polls/{id}",         get(get_poll).delete(delete_poll))
        .route("/polls/{id}/vote",    post(vote_poll))
        .route("/polls/{id}/close",   post(close_poll))
}
