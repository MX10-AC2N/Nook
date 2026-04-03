// backend/src/polls.rs
// Sondages familiaux — CRUD + vote + fermeture
//
// Corrections session 27 (build fix) :
//   - Structs #[derive(sqlx::FromRow)] pour tous les types de retour DB
//     (les tuples anonymes complexes ne compilent pas sans turbofish DB explicite)
//   - sqlx::query_as::<_, T>(...) partout (turbofish pour inférence DB)
//   - load_poll : Option<PollRow> correctement unwrappé avec if let Some(row) = ...
//   - Pas de macros sqlx (SQLX_OFFLINE=true, queries.json vide)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
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
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub option_id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Types réponse (JSON vers le client)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PollOption {
    pub id: String,
    pub text: String,
    pub position: i64,
    pub votes: i64,
    pub voters: Vec<String>,
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
    pub my_vote: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Types internes DB — #[derive(sqlx::FromRow)] obligatoire pour query_as
// ─────────────────────────────────────────────────────────────────────────────

/// Ligne principale du sondage (JOIN users pour le nom du créateur)
#[derive(sqlx::FromRow)]
struct PollRow {
    id: String,
    question: String,
    created_by: String,
    creator_name: String,
    created_at: i64,
    closed_at: Option<i64>,
}

/// Option de sondage
#[derive(sqlx::FromRow)]
struct PollOptionRow {
    id: String,
    text: String,
    position: i64,
}

/// Comptage votes par option
#[derive(sqlx::FromRow)]
struct VoteCount {
    option_id: String,
    cnt: i64,
}

/// Nom de votant par option
#[derive(sqlx::FromRow)]
struct VoterRow {
    option_id: String,
    voter_name: String,
}

/// Vote de l'utilisateur courant
#[derive(sqlx::FromRow)]
struct MyVoteRow {
    option_id: String,
}

/// Identifiant seul (pour list_polls)
#[derive(sqlx::FromRow)]
struct IdRow {
    id: String,
}

/// Statut ouvert/fermé (pour vote_poll)
#[derive(sqlx::FromRow)]
struct ClosedAtRow {
    closed_at: Option<i64>,
}

/// Option valide (pour vote_poll)
#[derive(sqlx::FromRow)]
struct OptionIdRow {
    #[allow(dead_code)]
    id: String,
}

/// Créateur + statut (pour close_poll / delete_poll)
#[derive(sqlx::FromRow)]
struct CreatorClosedRow {
    created_by: String,
    closed_at: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct CreatorRow {
    created_by: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper : charge un sondage complet depuis la DB
// ─────────────────────────────────────────────────────────────────────────────

async fn load_poll(
    pool: &sqlx::SqlitePool,
    poll_id: &str,
    current_user_id: &str,
) -> Option<PollResult> {
    // 1. Sondage + nom créateur
    let row = sqlx::query_as::<_, PollRow>(
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

    let is_closed = row.closed_at.is_some();

    // 2. Options
    let options_raw = sqlx::query_as::<_, PollOptionRow>(
        "SELECT id, text, position FROM poll_options WHERE poll_id = ? ORDER BY position ASC",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 3. Comptage votes par option
    let vote_counts = sqlx::query_as::<_, VoteCount>(
        "SELECT option_id, COUNT(*) AS cnt FROM poll_votes WHERE poll_id = ? GROUP BY option_id",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 4. Noms des votants par option
    let voter_names = sqlx::query_as::<_, VoterRow>(
        r#"SELECT pv.option_id, COALESCE(u.name, u.username) AS voter_name
           FROM poll_votes pv
           LEFT JOIN users u ON u.id = pv.user_id
           WHERE pv.poll_id = ?"#,
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 5. Mon vote
    let my_vote = sqlx::query_as::<_, MyVoteRow>(
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
        .map(|opt| {
            let votes = vote_counts
                .iter()
                .find(|vc| vc.option_id == opt.id)
                .map(|vc| vc.cnt)
                .unwrap_or(0);
            total_votes += votes;
            let voters = voter_names
                .iter()
                .filter(|vr| vr.option_id == opt.id)
                .map(|vr| vr.voter_name.clone())
                .collect();
            PollOption {
                id: opt.id,
                text: opt.text,
                position: opt.position,
                votes,
                voters,
            }
        })
        .collect();

    Some(PollResult {
        id: row.id,
        question: row.question,
        created_by: row.created_by,
        created_by_name: row.creator_name,
        created_at: row.created_at,
        closed_at: row.closed_at,
        is_closed,
        total_votes,
        options,
        my_vote: my_vote.map(|r| r.option_id),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/polls
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_polls(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    let ids = sqlx::query_as::<_, IdRow>("SELECT id FROM polls WHERE (closed_at IS NULL OR closed_at > datetime('now')) ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut results = Vec::with_capacity(ids.len());
    for row in ids {
        if let Some(poll) = load_poll(&state.db, &row.id, &user.id).await {
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
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Sondage introuvable" })),
        )
            .into_response(),
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
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": "Question requise" })),
        )
            .into_response();
    }
    let options: Vec<String> = req
        .options
        .iter()
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .collect();
    if options.len() < 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": "Au moins 2 options requises" })),
        )
            .into_response();
    }
    if options.len() > 10 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": "Maximum 10 options" })),
        )
            .into_response();
    }

    let poll_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    if let Err(e) =
        sqlx::query("INSERT INTO polls (id, question, created_by, created_at) VALUES (?, ?, ?, ?)")
            .bind(&poll_id)
            .bind(&question)
            .bind(&user.id)
            .bind(now)
            .execute(&state.db)
            .await
    {
        tracing::error!(error = %e, "create_poll INSERT polls");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Erreur création sondage" })),
        )
            .into_response();
    }

    for (i, text) in options.iter().enumerate() {
        let opt_id = Uuid::new_v4().to_string();
        if let Err(e) = sqlx::query(
            "INSERT INTO poll_options (id, poll_id, text, position) VALUES (?, ?, ?, ?)",
        )
        .bind(&opt_id)
        .bind(&poll_id)
        .bind(text)
        .bind(i as i64)
        .execute(&state.db)
        .await
        {
            tracing::error!(error = %e, "create_poll INSERT poll_options");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Erreur création options" })),
            )
                .into_response();
        }
    }

    match load_poll(&state.db, &poll_id, &user.id).await {
        Some(p) => {
            // Broadcast WS notification: new_poll created
            let notif = serde_json::json!({
                "type": "new_poll",
                "poll_id": poll_id,
                "title": question,
                "options": options.len(),
            }).to_string();
            let guard = state.webrtc_state.broadcasts.lock().await;
            for (_, tx) in guard.iter() {
                let _ = tx.send(notif.clone());
            }
            (StatusCode::CREATED, Json(json!({ "poll": p }))).into_response()
        }
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
    // Sondage existe et est ouvert ?
    let status = sqlx::query_as::<_, ClosedAtRow>("SELECT closed_at FROM polls WHERE id = ?")
        .bind(&poll_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    match status {
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "Sondage introuvable" })),
            )
                .into_response()
        }
        Some(r) if r.closed_at.is_some() => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "message": "Sondage fermé" })),
            )
                .into_response()
        }
        _ => {}
    }

    // Option valide pour ce sondage ?
    let opt_ok = sqlx::query_as::<_, OptionIdRow>(
        "SELECT id FROM poll_options WHERE id = ? AND poll_id = ?",
    )
    .bind(&req.option_id)
    .bind(&poll_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if opt_ok.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": "Option invalide" })),
        )
            .into_response();
    }

    let now = Utc::now().timestamp();
    if let Err(e) = sqlx::query(
        r#"INSERT INTO poll_votes (poll_id, user_id, option_id, voted_at)
           VALUES (?, ?, ?, ?)
           ON CONFLICT(poll_id, user_id) DO UPDATE SET
               option_id = excluded.option_id,
               voted_at  = excluded.voted_at"#,
    )
    .bind(&poll_id)
    .bind(&user.id)
    .bind(&req.option_id)
    .bind(now)
    .execute(&state.db)
    .await
    {
        tracing::error!(error = %e, "vote_poll UPSERT");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Erreur vote" })),
        )
            .into_response();
    }

    match load_poll(&state.db, &poll_id, &user.id).await {
        Some(p) => Json(json!({ "success": true, "poll": p })).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false }))).into_response(),
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
    let row = sqlx::query_as::<_, CreatorClosedRow>(
        "SELECT created_by, closed_at FROM polls WHERE id = ?",
    )
    .bind(&poll_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match row {
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "Sondage introuvable" })),
            )
                .into_response()
        }
        Some(ref r) if r.closed_at.is_some() => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "message": "Déjà fermé" })),
            )
                .into_response()
        }
        Some(ref r) if r.created_by != user.id && user.role != "admin" => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "Accès refusé" })),
            )
                .into_response();
        }
        _ => {}
    }

    let now = Utc::now().timestamp();
    sqlx::query("UPDATE polls SET closed_at = ? WHERE id = ?")
        .bind(now)
        .bind(&poll_id)
        .execute(&state.db)
        .await
        .ok();

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
    let row = sqlx::query_as::<_, CreatorRow>("SELECT created_by FROM polls WHERE id = ?")
        .bind(&poll_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    match row {
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "Sondage introuvable" })),
            )
                .into_response()
        }
        Some(ref r) if r.created_by != user.id && user.role != "admin" => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "Accès refusé" })),
            )
                .into_response();
        }
        _ => {}
    }

    sqlx::query("DELETE FROM polls WHERE id = ?")
        .bind(&poll_id)
        .execute(&state.db)
        .await
        .ok();

    Json(json!({ "success": true })).into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

pub fn polls_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/polls", get(list_polls).post(create_poll))
        .route("/polls/{id}", get(get_poll).delete(delete_poll))
        .route("/polls/{id}/vote", post(vote_poll))
        .route("/polls/{id}/close", post(close_poll))
}