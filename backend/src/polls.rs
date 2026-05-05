     1|// backend/src/polls.rs
     2|// Sondages familiaux — CRUD + vote + fermeture
     3|//
     4|// Corrections session 27 (build fix) :
     5|//   - Structs #[derive(sqlx::FromRow)] pour tous les types de retour DB
     6|//     (les tuples anonymes complexes ne compilent pas sans turbofish DB explicite)
     7|//   - sqlx::query_as::<_, T>(...) partout (turbofish pour inférence DB)
     8|//   - load_poll : Option<PollRow> correctement unwrappé avec if let Some(row) = ...
     9|//   - Structs #[derive(sqlx::FromRow)] pour tous les types de retour DB
    10|
    11|#![allow(clippy::for_kv_map)]
    12|
    13|use axum::{
    14|    extract::{Path, State},
    15|    http::StatusCode,
    16|    response::{IntoResponse, Json},
    17|    routing::{get, post},
    18|    Extension, Router,
    19|};
    20|use chrono::Utc;
    21|use serde::{Deserialize, Serialize};
    22|use serde_json::json;
    23|use std::sync::Arc;
    24|use uuid::Uuid;
    25|
    26|use crate::auth::CurrentUser;
    27|use crate::SharedState;
    28|
    29|// ─────────────────────────────────────────────────────────────────────────────
    30|// Types requête
    31|// ─────────────────────────────────────────────────────────────────────────────
    32|
    33|#[derive(Debug, Deserialize)]
    34|pub struct CreatePollRequest {
    35|    pub question: String,
    36|    pub options: Vec<String>,
    37|}
    38|
    39|#[derive(Debug, Deserialize)]
    40|pub struct VoteRequest {
    41|    pub option_id: String,
    42|}
    43|
    44|// ─────────────────────────────────────────────────────────────────────────────
    45|// Types réponse (JSON vers le client)
    46|// ─────────────────────────────────────────────────────────────────────────────
    47|
    48|#[derive(Debug, Serialize)]
    49|pub struct PollOption {
    50|    pub id: String,
    51|    pub text: String,
    52|    pub position: i64,
    53|    pub votes: i64,
    54|    pub voters: Vec<String>,
    55|}
    56|
    57|#[derive(Debug, Serialize)]
    58|pub struct PollResult {
    59|    pub id: String,
    60|    pub question: String,
    61|    pub created_by: String,
    62|    pub created_by_name: String,
    63|    pub created_at: i64,
    64|    pub closed_at: Option<i64>,
    65|    pub is_closed: bool,
    66|    pub total_votes: i64,
    67|    pub options: Vec<PollOption>,
    68|    pub my_vote: Option<String>,
    69|}
    70|
    71|// ─────────────────────────────────────────────────────────────────────────────
    72|// Types internes DB — #[derive(sqlx::FromRow)] obligatoire pour query_as
    73|// ─────────────────────────────────────────────────────────────────────────────
    74|
    75|/// Ligne principale du sondage (JOIN users pour le nom du créateur)
    76|#[derive(sqlx::FromRow)]
    77|struct PollRow {
    78|    id: String,
    79|    question: String,
    80|    created_by: String,
    81|    creator_name: String,
    82|    created_at: i64,
    83|    closed_at: Option<i64>,
    84|}
    85|
    86|/// Option de sondage
    87|#[derive(sqlx::FromRow)]
    88|struct PollOptionRow {
    89|    id: String,
    90|    text: String,
    91|    position: i64,
    92|}
    93|
    94|/// Comptage votes par option
    95|#[derive(sqlx::FromRow)]
    96|struct VoteCount {
    97|    option_id: String,
    98|    cnt: i64,
    99|}
   100|
   101|/// Nom de votant par option
   102|#[derive(sqlx::FromRow)]
   103|struct VoterRow {
   104|    option_id: String,
   105|    voter_name: String,
   106|}
   107|
   108|/// Vote de l'utilisateur courant
   109|#[derive(sqlx::FromRow)]
   110|struct MyVoteRow {
   111|    option_id: String,
   112|}
   113|
   114|/// Identifiant seul (pour list_polls)
   115|#[derive(sqlx::FromRow)]
   116|struct IdRow {
   117|    id: String,
   118|}
   119|
   120|/// Statut ouvert/fermé (pour vote_poll)
   121|#[derive(sqlx::FromRow)]
   122|struct ClosedAtRow {
   123|    closed_at: Option<i64>,
   124|}
   125|
   126|/// Option valide (pour vote_poll)
   127|#[derive(sqlx::FromRow)]
   128|struct OptionIdRow {
   129|    #[allow(dead_code)]
   130|    id: String,
   131|}
   132|
   133|/// Créateur + statut (pour close_poll / delete_poll)
   134|#[derive(sqlx::FromRow)]
   135|struct CreatorClosedRow {
   136|    created_by: String,
   137|    closed_at: Option<i64>,
   138|}
   139|
   140|#[derive(sqlx::FromRow)]
   141|struct CreatorRow {
   142|    created_by: String,
   143|}
   144|
   145|// ─────────────────────────────────────────────────────────────────────────────
   146|// Helper : charge un sondage complet depuis la DB
   147|// ─────────────────────────────────────────────────────────────────────────────
   148|
   149|async fn load_poll(
   150|    pool: &sqlx::SqlitePool,
   151|    poll_id: &str,
   152|    current_user_id: &str,
   153|) -> Option<PollResult> {
   154|    // 1. Sondage + nom créateur
   155|    let row = sqlx::query_as::<_, PollRow>(
   156|        r#"SELECT p.id, p.question, p.created_by,
   157|                  COALESCE(u.name, u.username) AS creator_name,
   158|                  p.created_at, p.closed_at
   159|           FROM polls p
   160|           LEFT JOIN users u ON u.id = p.created_by
   161|           WHERE p.id = ?"#,
   162|    )
   163|    .bind(poll_id)
   164|    .fetch_optional(pool)
   165|    .await
   166|    .ok()
   167|    .flatten()?;
   168|
   169|    let is_closed = row.closed_at.is_some();
   170|
   171|    // 2. Options
   172|    let options_raw = sqlx::query_as::<_, PollOptionRow>(
   173|        "SELECT id, text, position FROM poll_options WHERE poll_id = ? ORDER BY position ASC",
   174|    )
   175|    .bind(poll_id)
   176|    .fetch_all(pool)
   177|    .await
   178|    .unwrap_or_default();
   179|
   180|    // 3. Comptage votes par option
   181|    let vote_counts = sqlx::query_as::<_, VoteCount>(
   182|        "SELECT option_id, COUNT(*) AS cnt FROM poll_votes WHERE poll_id = ? GROUP BY option_id",
   183|    )
   184|    .bind(poll_id)
   185|    .fetch_all(pool)
   186|    .await
   187|    .unwrap_or_default();
   188|
   189|    // 4. Noms des votants par option
   190|    let voter_names = sqlx::query_as::<_, VoterRow>(
   191|        r#"SELECT pv.option_id, COALESCE(u.name, u.username) AS voter_name
   192|           FROM poll_votes pv
   193|           LEFT JOIN users u ON u.id = pv.user_id
   194|           WHERE pv.poll_id = ?"#,
   195|    )
   196|    .bind(poll_id)
   197|    .fetch_all(pool)
   198|    .await
   199|    .unwrap_or_default();
   200|
   201|    // 5. Mon vote
   202|    let my_vote = sqlx::query_as::<_, MyVoteRow>(
   203|        "SELECT option_id FROM poll_votes WHERE poll_id = ? AND user_id = ?",
   204|    )
   205|    .bind(poll_id)
   206|    .bind(current_user_id)
   207|    .fetch_optional(pool)
   208|    .await
   209|    .ok()
   210|    .flatten();
   211|
   212|    // 6. Assemblage
   213|    let mut total_votes: i64 = 0;
   214|    let options = options_raw
   215|        .into_iter()
   216|        .map(|opt| {
   217|            let votes = vote_counts
   218|                .iter()
   219|                .find(|vc| vc.option_id == opt.id)
   220|                .map(|vc| vc.cnt)
   221|                .unwrap_or(0);
   222|            total_votes += votes;
   223|            let voters = voter_names
   224|                .iter()
   225|                .filter(|vr| vr.option_id == opt.id)
   226|                .map(|vr| vr.voter_name.clone())
   227|                .collect();
   228|            PollOption {
   229|                id: opt.id,
   230|                text: opt.text,
   231|                position: opt.position,
   232|                votes,
   233|                voters,
   234|            }
   235|        })
   236|        .collect();
   237|
   238|    Some(PollResult {
   239|        id: row.id,
   240|        question: row.question,
   241|        created_by: row.created_by,
   242|        created_by_name: row.creator_name,
   243|        created_at: row.created_at,
   244|        closed_at: row.closed_at,
   245|        is_closed,
   246|        total_votes,
   247|        options,
   248|        my_vote: my_vote.map(|r| r.option_id),
   249|    })
   250|}
   251|
   252|// ─────────────────────────────────────────────────────────────────────────────
   253|// GET /api/polls
   254|// ─────────────────────────────────────────────────────────────────────────────
   255|
   256|pub async fn list_polls(
   257|    State(state): State<Arc<SharedState>>,
   258|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   259|) -> impl IntoResponse {
   260|    let ids = sqlx::query_as::<_, IdRow>("SELECT id FROM polls WHERE (closed_at IS NULL OR closed_at > datetime('now')) ORDER BY created_at DESC LIMIT 100")
   261|        .fetch_all(&state.db)
   262|        .await
   263|        .unwrap_or_default();
   264|
   265|    let mut results = Vec::with_capacity(ids.len());
   266|    for row in ids {
   267|        if let Some(poll) = load_poll(&state.db, &row.id, &user.id).await {
   268|            results.push(poll);
   269|        }
   270|    }
   271|    Json(json!({ "polls": results })).into_response()
   272|}
   273|
   274|// ─────────────────────────────────────────────────────────────────────────────
   275|// GET /api/polls/{id}
   276|// ─────────────────────────────────────────────────────────────────────────────
   277|
   278|pub async fn get_poll(
   279|    State(state): State<Arc<SharedState>>,
   280|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   281|    Path(poll_id): Path<String>,
   282|) -> impl IntoResponse {
   283|    match load_poll(&state.db, &poll_id, &user.id).await {
   284|        Some(p) => Json(json!({ "poll": p })).into_response(),
   285|        None => (
   286|            StatusCode::NOT_FOUND,
   287|            Json(json!({ "message": "Sondage introuvable" })),
   288|        )
   289|            .into_response(),
   290|    }
   291|}
   292|
   293|// ─────────────────────────────────────────────────────────────────────────────
   294|// POST /api/polls
   295|// ─────────────────────────────────────────────────────────────────────────────
   296|
   297|pub async fn create_poll(
   298|    State(state): State<Arc<SharedState>>,
   299|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   300|    Json(req): Json<CreatePollRequest>,
   301|) -> impl IntoResponse {
   302|    let question = req.question.trim().to_string();
   303|    if question.is_empty() {
   304|        return (
   305|            StatusCode::BAD_REQUEST,
   306|            Json(json!({ "message": "Question requise" })),
   307|        )
   308|            .into_response();
   309|    }
   310|    let options: Vec<String> = req
   311|        .options
   312|        .iter()
   313|        .map(|o| o.trim().to_string())
   314|        .filter(|o| !o.is_empty())
   315|        .collect();
   316|    if options.len() < 2 {
   317|        return (
   318|            StatusCode::BAD_REQUEST,
   319|            Json(json!({ "message": "Au moins 2 options requises" })),
   320|        )
   321|            .into_response();
   322|    }
   323|    if options.len() > 10 {
   324|        return (
   325|            StatusCode::BAD_REQUEST,
   326|            Json(json!({ "message": "Maximum 10 options" })),
   327|        )
   328|            .into_response();
   329|    }
   330|
   331|    let poll_id = Uuid::new_v4().to_string();
   332|    let now = Utc::now().timestamp();
   333|
   334|    if let Err(e) =
   335|        sqlx::query("INSERT INTO polls (id, question, created_by, created_at) VALUES (?, ?, ?, ?)")
   336|            .bind(&poll_id)
   337|            .bind(&question)
   338|            .bind(&user.id)
   339|            .bind(now)
   340|            .execute(&state.db)
   341|            .await
   342|    {
   343|        tracing::error!(error = %e, "create_poll INSERT polls");
   344|        return (
   345|            StatusCode::INTERNAL_SERVER_ERROR,
   346|            Json(json!({ "message": "Erreur création sondage" })),
   347|        )
   348|            .into_response();
   349|    }
   350|
   351|    for (i, text) in options.iter().enumerate() {
   352|        let opt_id = Uuid::new_v4().to_string();
   353|        if let Err(e) = sqlx::query(
   354|            "INSERT INTO poll_options (id, poll_id, text, position) VALUES (?, ?, ?, ?)",
   355|        )
   356|        .bind(&opt_id)
   357|        .bind(&poll_id)
   358|        .bind(text)
   359|        .bind(i as i64)
   360|        .execute(&state.db)
   361|        .await
   362|        {
   363|            tracing::error!(error = %e, "create_poll INSERT poll_options");
   364|            return (
   365|                StatusCode::INTERNAL_SERVER_ERROR,
   366|                Json(json!({ "message": "Erreur création options" })),
   367|            )
   368|                .into_response();
   369|        }
   370|    }
   371|
   372|    match load_poll(&state.db, &poll_id, &user.id).await {
   373|        Some(p) => {
   374|            // Broadcast WS notification: new_poll created
   375|            let notif = serde_json::json!({
   376|                "type": "new_poll",
   377|                "poll_id": poll_id,
   378|                "title": question,
   379|                "options": options.len(),
   380|            }).to_string();
   381|            let guard = state.webrtc_state.broadcasts.lock().await;
   382|    for tx in guard.values() {
   383|        let _ = tx.send(notif.clone()); }
   384|            }
   385|            (StatusCode::CREATED, Json(json!({ "poll": p }))).into_response()
   386|        }
   387|        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))).into_response(),
   388|    }
   389|}
   390|
   391|// ─────────────────────────────────────────────────────────────────────────────
   392|// POST /api/polls/{id}/vote
   393|// ─────────────────────────────────────────────────────────────────────────────
   394|
   395|pub async fn vote_poll(
   396|    State(state): State<Arc<SharedState>>,
   397|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   398|    Path(poll_id): Path<String>,
   399|    Json(req): Json<VoteRequest>,
   400|) -> impl IntoResponse {
   401|    // Sondage existe et est ouvert ?
   402|    let status = sqlx::query_as::<_, ClosedAtRow>("SELECT closed_at FROM polls WHERE id = ?")
   403|        .bind(&poll_id)
   404|        .fetch_optional(&state.db)
   405|        .await
   406|        .ok()
   407|        .flatten();
   408|
   409|    match status {
   410|        None => {
   411|            return (
   412|                StatusCode::NOT_FOUND,
   413|                Json(json!({ "message": "Sondage introuvable" })),
   414|            )
   415|                .into_response()
   416|        }
   417|        Some(r) if r.closed_at.is_some() => {
   418|            return (
   419|                StatusCode::BAD_REQUEST,
   420|                Json(json!({ "message": "Sondage fermé" })),
   421|            )
   422|                .into_response()
   423|        }
   424|        _ => {}
   425|    }
   426|
   427|    // Option valide pour ce sondage ?
   428|    let opt_ok = sqlx::query_as::<_, OptionIdRow>(
   429|        "SELECT id FROM poll_options WHERE id = ? AND poll_id = ?",
   430|    )
   431|    .bind(&req.option_id)
   432|    .bind(&poll_id)
   433|    .fetch_optional(&state.db)
   434|    .await
   435|    .ok()
   436|    .flatten();
   437|
   438|    if opt_ok.is_none() {
   439|        return (
   440|            StatusCode::BAD_REQUEST,
   441|            Json(json!({ "message": "Option invalide" })),
   442|        )
   443|            .into_response();
   444|    }
   445|
   446|    let now = Utc::now().timestamp();
   447|    if let Err(e) = sqlx::query(
   448|        r#"INSERT INTO poll_votes (poll_id, user_id, option_id, voted_at)
   449|           VALUES (?, ?, ?, ?)
   450|           ON CONFLICT(poll_id, user_id) DO UPDATE SET
   451|               option_id = excluded.option_id,
   452|               voted_at  = excluded.voted_at"#,
   453|    )
   454|    .bind(&poll_id)
   455|    .bind(&user.id)
   456|    .bind(&req.option_id)
   457|    .bind(now)
   458|    .execute(&state.db)
   459|    .await
   460|    {
   461|        tracing::error!(error = %e, "vote_poll UPSERT");
   462|        return (
   463|            StatusCode::INTERNAL_SERVER_ERROR,
   464|            Json(json!({ "message": "Erreur vote" })),
   465|        )
   466|            .into_response();
   467|    }
   468|
   469|    match load_poll(&state.db, &poll_id, &user.id).await {
   470|        Some(p) => Json(json!({ "success": true, "poll": p })).into_response(),
   471|        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false }))).into_response(),
   472|    }
   473|}
   474|
   475|// ─────────────────────────────────────────────────────────────────────────────
   476|// POST /api/polls/{id}/close
   477|// ─────────────────────────────────────────────────────────────────────────────
   478|
   479|pub async fn close_poll(
   480|    State(state): State<Arc<SharedState>>,
   481|    Extension(CurrentUser(user)): Extension<CurrentUser>,
   482|    Path(poll_id): Path<String>,
   483|) -> impl IntoResponse {
   484|    let row = sqlx::query_as::<_, CreatorClosedRow>(
   485|        "SELECT created_by, closed_at FROM polls WHERE id = ?",
   486|    )
   487|    .bind(&poll_id)
   488|    .fetch_optional(&state.db)
   489|    .await
   490|    .ok()
   491|    .flatten();
   492|
   493|    match row {
   494|        None => {
   495|            return (
   496|                StatusCode::NOT_FOUND,
   497|                Json(json!({ "message": "Sondage introuvable" })),
   498|            )
   499|                .into_response()
   500|        }
   501|