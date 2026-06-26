# Backend Endpoint Patterns — Nook Rust

## Auth Pattern — State<SharedState> + Extension<CurrentUser> (depuis S37/S50)

```rust
// ❌ ANCIEN — cassé sans Extension<SqlitePool> + Extension<String> middleware
Extension(pool): Extension<Arc<SqlitePool>>,
Extension(user_id): Extension<String>,

// ✅ MODERNE — utilise l'état partagé + extracteur CurrentUser
State(state): State<Arc<SharedState>>,
Extension(CurrentUser(user)): Extension<CurrentUser>,

// Accès DB: state.db (Arc<SqlitePool>)
// Accès user: user.id, user.username, etc.
```

Le middleware `require_auth` insère `CurrentUser` dans les extensions. Ne **jamais** utiliser `Extension<SqlitePool>` ni `Extension<String>` pour user_id.

## CRUD Events — Pattern complet (S50)

### Payloads

```rust
// Création — date/time strings frontend-friendly
#[derive(Debug, Deserialize)]
pub struct CreateEventPayload {
    pub title: String,
    pub description: Option<String>,
    pub date: String,      // YYYY-MM-DD
    pub time: Option<String>, // HH:MM
}

// Mise à jour — Option<Option<String>> pour permettre null explicite
#[derive(Debug, Deserialize, Clone)]
pub struct UpdateEventPayload {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub date: Option<String>,
    pub time: Option<Option<String>>,
}
```

### Fix E0382 — partial move lors du binding SQLx

```rust
// ❌ PROBLÈME: .unwrap() move le payload, inutilisable après
.bind(payload.title.unwrap())
handle_update_result(result, state, id, payload).await // payload partially moved

// ✅ FIX: Extraire AVANT le binding (as_ref pour emprunter)
let title_val = payload.title.as_ref().map(|s| s.as_str());
let desc_val = payload.description.as_ref().map(|opt| opt.as_ref().map(|s| s.as_str()));
let has_date_time = payload.date.is_some() || payload.time.as_ref().is_some_and(|o| o.is_some());

let result = sqlx::query("...")
    .bind(title_val)  // &str, pas move
    .execute(&db)
    .await;
```

### Fix E0716 — temporary value dropped while borrowed

```rust
// ❌ PROBLÈME: bloc anonyme dans unwrap_or(&{ ... }) → durée de vie trop courte
let date_str = payload.date.as_deref().unwrap_or(&{
    let (d, _) = timestamp_to_date_time(event.start_time);
    d
});

// ✅ FIX: let binding AVANT pour prolonger la durée de vie
let default_date_time = timestamp_to_date_time(event.start_time);
let date_str = payload.date.as_deref().unwrap_or(&default_date_time.0);
let time_str = payload.time.as_ref().and_then(|o| o.as_deref()).unwrap_or(&default_date_time.1);
```

### CARGO_BUILD_JOBS=1 — prévenir OOM sur GitHub Actions

```yaml
# .github/workflows/Backend.yml
env:
  CARGO_BUILD_JOBS: "1"  # compile mono-thread
  CARGO_PROFILE_RELEASE_LTO: "true"
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "1"
```

Réduit l'utilisation RAM de ~14GB → ~7GB sur arm64, évite le kill OOM (exit 101).

### Router Events

```rust
pub fn events_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/", post(create_event).get(list_events))
        .route("/{id}", get(get_event).patch(update_event).delete(delete_event))
}
```

Recordé depuis la session S50 — fix complet Calendar/Events API.