---
name: nook-rust-backend
description: "Skill spécialisé pour tout développement backend Rust du projet Nook"
---

# 🦀 Nook — Backend Rust Skill

## Périmètre

```
backend/src/
├── main.rs            → Router, middleware, init, E2E_SETUP
├── auth.rs            → Cookie HttpOnly, argon2, tokens
├── db.rs              → SQLx queries, conversations, messages
├── admin.rs           → Gestion users, approbation
├── invites.rs         → Liens invitation UUID
├── upload.rs          → Multipart, TTL 48h, 50 Mo max
├── webrtc.rs          → WebSocket signaling, XChaCha20
├── chess.rs           → API parties d'échecs
├── chess_engine/      → Moteur pur Rust (10 fichiers)
├── polls.rs           → Sondages + vote UPSERT
├── reactions.rs       → Réactions emoji aux messages
├── prune.rs           → Nettoyage DB 24h (exclure conversations système !)
├── cleanup.rs         → Fichiers expirés
└── config.rs          → Env vars → struct Config
```

## Stack — versions FIXES (ne pas bump without valider)

```toml
axum = "0.8"                    # routes {param}, Message::Text .into()
sqlx = "0.8.6"                  # offline mode → .sqlx/queries.json obligatoire
rand = "0.9"                    # rand::rng() — PAS thread_rng()
rand_core = "0.6"               # ⚠️ diamond dep argon2 — PAS bumper
argon2 = "0.5"
chacha20poly1305 = "0.10.1"
governor = "0.10"               # PAS tower_governor (tire async-trait → crash Docker)
tower-http = "0.6.8"
```

## Pièges critiques — à vérifier à chaque intervention

### rand 0.9
```rust
// ❌ supprimé
rand::thread_rng().fill_bytes(&mut buf);

// ✅
rand::rng().fill_bytes(&mut buf);

// ❌ rand::rngs::OsRng incompatible avec argon2 (rand_core 0.6 vs 0.9)
// ✅ toujours rand_core::OsRng
use rand_core::OsRng;
let salt = SaltString::generate(&mut OsRng);
```

### Axum 0.8 — syntaxe routes
```rust
// ❌ ancienne syntaxe
.route("/api/users/:id", get(handler))

// ✅
.route("/api/users/{id}", get(handler))

// ❌ Message::Text attend String en axum 0.7
Message::Text(String::from("hello"))

// ✅ Utf8Bytes en axum 0.8
Message::Text("hello".into())
```

### CORS — origins explicites obligatoires
```rust
// ❌ PANIC au démarrage
CorsLayer::new().allow_origin(Any).allow_credentials(true)

// ✅
let origins: Vec<HeaderValue> = config.allowed_origins
    .split(',').filter_map(|s| s.trim().parse().ok()).collect();
CorsLayer::new().allow_origin(origins).allow_credentials(true)
```

### SQLite — options obligatoires
```rust
// ❌ SQLITE_CANTOPEN (14) si fichier inexistant
SqlitePool::connect(&database_url).await?

// ✅
SqliteConnectOptions::from_str(&database_url)?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .synchronous(SqliteSynchronous::Normal)
```

### SQLx sans macros (quand queries.json non régénéré)
```rust
// ❌ les macros sqlx! nécessitent queries.json à jour en CI
let rows = sqlx::query_as!(MyType, "SELECT ...", param).fetch_all(&pool).await?;

// ✅ sans macro, toujours safe
let rows = sqlx::query_as::<_, MyType>("SELECT ...").bind(param).fetch_all(&pool).await?;
```

### Clippy -D warnings — erreurs courantes
```rust
// ❌ imports inutilisés dans un routing module
use axum::routing::{delete, get, post};
// delete/get sont des méthodes sur MethodRouter, pas des fonctions routing
// → utiliser uniquement ce qui est appelé comme fonction

// ❌ struct déclarée mais jamais construite
pub struct MyResponse { ... }
// → soit la construire, soit ajouter #[allow(dead_code)]
```

### prune.rs — règle absolue
```rust
// ❌ Supprimer conversations sans participants → supprime default_global
// ✅ Exclure les conversations système
"DELETE FROM conversations WHERE id NOT IN (SELECT conversation_id FROM conversation_participants) AND id != 'default_global'"
```

### .map_err() — syntaxe pour retour tuple
```rust
// ❌ Erreur : mismatched closing delimiter
.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Erreur DB"})))?

// ✅ Syntaxe correcte avec bloc closure
.map_err(|_| {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"message": "Erreur DB"})),
    )
})?
```

### governor 0.10 — KeyedRateLimiter n'est pas dans governor::state
```rust
// ❌ E0432 — n'existe pas dans governor::state
use governor::state::{keyed::DefaultKeyedStateStore, KeyedRateLimiter};
type IpRateLimiter = KeyedRateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;

// ✅ RateLimiter générique avec 4 paramètres de type
use governor::state::keyed::DefaultKeyedStateStore;
type IpRateLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;
// RateLimiter::keyed(Quota::per_minute(...)) → retourne ce type
```

## ⚠️ BUG Session 56/58 — `/api/events` GET : State<Arc<SharedState>> non propagé

**Symptôme** : Erreur 500 `Missing request extension: Extension of type alloc::sync::Arc<sqlx_core::pool::Pool<sqlx_sqlite::database::Sqlite>>` alors que `/api/polls` fonctionne correctement avec le même pattern.

**Cause réelle** : Le module `events.rs` utilise `State(state): State<Arc<SharedState>>` dans tous ses handlers. Le middleware `from_fn_with_state` injecte le state mais ne le propage **pas** aux sous-routers mergés.

**Fix correct (Session 58 — validé CI release LTO)** → Voir [`references/axum-middleware-state-propagation.md`](references/axum-middleware-state-propagation.md)

### Pattern qui MARCHE (compile en release LTO + runtime OK)

1. **Middleware auth.rs** — utilise `AxumState` extractor + réinsère state :
```rust
pub async fn require_auth(
    AxumState(state): AxumState<Arc<SharedState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // ... validation session ...
    if let Some(user) = user {
        req.extensions_mut().insert(state.clone());  // ← CLÉ
        req.extensions_mut().insert(CurrentUser(user));
        return next.run(req).await;
    }
}
```

2. **main.rs** — enregistre avec **function item** (pas closure) :
```rust
.layer(middleware::from_fn_with_state(
    shared_state.clone(),
    auth::require_auth,  // ← function item, PAS |s,r,n| auth::require_auth(s,r,n)
));
```

3. **Handlers events.rs, polls.rs, etc.** — utilisent `State` extractor normalement.

**Pourquoi ça marche** : `from_fn_with_state` injecte le state via `AxumState`, le middleware le réinsère manuellement dans `req.extensions()`, et les handlers downstream le récupèrent via `State<T>`.

### Ce qui NE marche PAS
- ❌ Closure avec `from_fn_with_state` → compile FAIL en release LTO
- ❌ `.with_state()` seul sur parent sans middleware propagation → runtime 500
- ❌ `Extension(db.clone())` sur sous-router → handlers attendent `State<T>`

## ⚠️ TURN Credential Generation Pattern (RFC 5389)

Never hardcode TURN secrets in the frontend. Generate short-lived credentials server-side.

### Backend endpoint — `/api/webrtc/ice-config`

```rust
use sha1::Sha1;
use hmac::{Hmac, Mac};
type HmacSha1 = Hmac<Sha1>;

async fn handle_ice_config(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Auth — inline SQL query (no validate_session function exists)
    let cookie_header = match headers.get(COOKIE).and_then(|v| v.to_str().ok()) {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response(),
    };
    let token_value = match cookie_header
        .split(';').find(|c| c.trim().starts_with("auth_token="))
        .and_then(|c| c.trim().strip_prefix("auth_token="))
    {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response(),
    };
    let (user_id, token) = match token_value.split_once(':') {
        Some((u, t)) if !u.is_empty() && !t.is_empty() => (u, t),
        _ => return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response(),
    };
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND token = ? AND approved = 1)"
    ).bind(user_id).bind(token).fetch_one(&state.db)
        .await.map(|v: i64| v == 1).unwrap_or(false);
    if !valid {
        return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response();
    }

    // Generate HMAC-SHA1 TURN credentials (24h validity)
    let username = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() + 86400;
    let mut mac = <HmacSha1 as hmac::Mac>::new_from_slice(state.config.turn_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(username.to_string().as_bytes());
    let credential = base64ct::Base64Unpadded::encode_string(&mac.finalize().into_bytes());

    (StatusCode::OK, AxumJson(json!({
        "host": state.config.turn_host,
        "port": state.config.turn_port,
        "username": username.to_string(),
        "credential": credential,
    }))).into_response()
}
```

### Config struct — add TURN fields
```rust
pub struct Config {
    // ... existing fields ...
    pub turn_host: String,   // env: TURN_HOST (fallback: PUBLIC_SITE_URL hostname)
    pub turn_port: u16,      // env: TURN_PORT (default: 3478)
    pub turn_secret: String, // env: TURN_SECRET
}
```

### Cargo.toml deps
```toml
hmac = "0.12"
sha1 = "0.10"
```

### Frontend — fetch don't hardcode
```typescript
// ❌ NEVER — secret exposed in JS bundle
const TURN_SECRET = 'change...cret';

// ✅ CORRECT — fetch from backend
const resp = await fetch('/api/webrtc/ice-config', { credentials: 'include' });
const { host, port, username, credential } = await resp.json();
```

## ⚠️ HSTS Header in Reverse Proxy Setup (Session 64)

**Problem** : Backend behind nginx (TLS termination) envoyait HSTS sur HTTP -> browser force HTTPS -> cert auto-signé -> page blanche.

**Fix** : Check `x-forwarded-proto` header avant d'envoyer HSTS (voir `references/hsts-reverse-proxy.md`).

```rust
// src/main.rs:615 — security headers middleware
if req.headers().get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https") {
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
}
```

**Nginx requis** : `proxy_set_header X-Forwarded-Proto https;`

---

## ⚠️ Auth Pattern — Inline SQL, No validate_session

The `auth` module does NOT export `validate_session`. Session validation is done inline:
```rust
// ❌ WRONG
crate::auth::validate_session(&state.pool, &token)

// ✅ CORRECT — direct SQL with state.db (not state.pool)
sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND token = ? AND approved = 1)")
    .bind(user_id).bind(token).fetch_one(&state.db).await
```

## Checklist ajout endpoint

1. Handler dans le fichier `.rs` approprié
2. Route dans `main.rs` avec syntaxe `{param}`
3. Migration SQL si nouvelle table (incrémenter `005_*.sql`)
4. Régénérer `.sqlx/queries.json` si nouvelles macros sqlx! → `sqlx-prepare.yml`
5. Mettre à jour `rules/api-and-db.md` section endpoints
6. Signaler à 🧪 E2E les nouveaux endpoints testables

## Diagnostics rapides

| Erreur CI | Cause | Fix |
|-----------|-------|-----|
| `unused imports: delete and get` | routing module | Garder uniquement `post`, utiliser `.delete()/.get()` sur MethodRouter |
| `struct X is never constructed` | dead code clippy | `#[allow(dead_code)]` ou supprimer |
| `cannot find value thread_rng` | rand 0.9 | `rand::rng()` |
| `mismatched types: Utf8Bytes` | axum 0.8 | `.into()` |
| `SQLITE_CANTOPEN (14)` | DB inexistante | `create_if_missing(true)` |
| `proc-macro panic` Docker | tower_governor | Retirer, utiliser `governor` seul |
| `Missing request extension: Pool<Sqlite>` | DB non injectée dans router | `.layer(Extension(db.clone()))` sur le router |
| Build arm64 ✅ / amd64 ✅ mais clippy ❌ | warnings promus erreurs | Corriger chaque warning |

## Flux inter-agents

```
→ Après tout endpoint modifié  : signaler URL + payload + codes HTTP à 🎨 SVELTE
→ Après toute migration SQL    : lancer sqlx-prepare.yml, informer 🚀 DEVOPS
→ Après tout changement DB     : mettre à jour rules/api-and-db.md
→ Tout endpoint nouveau        : informer 🧪 E2E avec contrat HTTP
```