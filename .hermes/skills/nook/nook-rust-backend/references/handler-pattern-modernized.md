# Modernised Handler Pattern — State<SharedState> + CurrentUser

## Contexte
Ancien pattern (fragile) : `Extension<Arc<SqlitePool>> + Extension<String>` pour accès DB + user_id.
Nouveau pattern (recommandé) : `State<Arc<SharedState>> + Extension<CurrentUser>`.

Le middleware `require_auth` insère déjà `CurrentUser` dans les extensions — plus besoin de passer manuellement `user_id` + `pool` depuis le router.

## Avant / Après

```rust
// ❌ ANCIEN
pub async fn my_handler(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<Payload>,
) -> impl IntoResponse { ... }

// ✅ NOUVEAU
pub async fn my_handler(
    State(state): State<Arc<SharedState>>,          // state.db = Arc<SqlitePool>
    Extension(CurrentUser(user)): Extension<CurrentUser>,  // user.id, user.username, etc.
    Json(payload): Json<Payload>,
) -> impl IntoResponse { ... }
```

## Enum Return Type — éviter `impl IntoResponse` mismatches

Quand un handler a plusieurs chemins de retour (update, error, not found, no fields), l'opacité `impl IntoResponse` cause des erreurs E0308.

```rust
// ❌ PROBLÈME — chaque return a un type opaque différent
async fn update_event(...) -> impl IntoResponse {
    if cond1 { return handle1().await }  // type opaque #1
    if cond2 { return handle2().await }  // type opaque #2 (mismatch !)
}

// ✅ SOLUTION — enum concret + match final
#[derive(Debug)]
enum UpdateResult {
    Updated(Event),
    NotFound,
    Error(String),
    NoFields,
}

async fn update_event(...) -> impl IntoResponse {
    let result = match detect_fields(...) {
        Case::All => do_full_update(...).await,
        Case::Title => do_title_only(...).await,
        // ...
    };
    match result {
        UpdateResult::Updated(e) => (StatusCode::OK, Json(e)).into_response(),
        UpdateResult::NotFound => (StatusCode::NOT_FOUND, Json(...)).into_response(),
        UpdateResult::Error(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(...)).into_response(),
        UpdateResult::NoFields => (StatusCode::BAD_REQUEST, Json(...)).into_response(),
    }
}
```

Appliqué dans `backend/src/events.rs` (session 2026-06-13) — compile clean sur arm64 + amd64, build CI OK.