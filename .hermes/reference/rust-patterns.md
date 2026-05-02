# 🦀 Rust Patterns — Nook Backend

> Référence rapide pour le développement backend Nook
> Source : `.hermes/roles/rust-backend.md`

## 🔧 Stack exacte

```toml
axum            = "0.8"         # + features: ws, multipart, tokio
axum-extra      = "0.10"        # typed-header
tokio           = "1.0"         # full
tower-http      = "0.6.8"       # fs, cors, compression-br
sqlx            = "0.8.6"       # runtime-tokio-rustls, sqlite, macros, migrate
argon2          = "0.5"
rand            = "0.9"         # std, std_rng, os_rng
rand_core       = "0.6"         # ⚠️ DIAMOND DEP — std, getrandom
chacha20poly1305= "0.10.1"
```

## ⚡ Pièges critiques

### 1. rand 0.9 API changée
```rust
// ❌ thread_rng() supprimé
let mut rng = rand::thread_rng();
rng.fill_bytes(&mut buf);

// ✅
rand::rng().fill_bytes(&mut buf);

// ❌ rand::rngs::OsRng ne compile pas avec argon2 (rand_core 0.6 vs 0.9)
// ✅ rand_core::OsRng (même crate que argon2)
use rand_core::OsRng;
let salt = SaltString::generate(&mut OsRng);
```

### 2. Axum 0.8 breaking changes
```rust
// ❌ Ancienne syntaxe
.route("/api/users/:id", get(get_user))

// ✅
.route("/api/users/{id}", get(get_user))

// ❌ Message::Text attend String en axum 0.7
Message::Text(String::from("hello"))

// ✅ attend Utf8Bytes en axum 0.8
Message::Text("hello".into())
```

### 3. CORS — allow_credentials strict
```rust
// ❌ PANIC à l'initialisation
CorsLayer::new()
    .allow_origin(Any)
    .allow_credentials(true)  // incompatible avec Any

// ✅ origines explicites depuis config
let origins: Vec<HeaderValue> = config.allowed_origins
    .split(',')
    .filter_map(|s| s.trim().parse().ok())
    .collect();

CorsLayer::new()
    .allow_origin(origins)
    .allow_credentials(true)
```

### 4. SQLite — options obligatoires
```rust
// ❌ SQLITE_CANTOPEN (code 14) si le fichier n'existe pas encore
let pool = SqlitePool::connect(&database_url).await?;

// ✅
let opts = SqliteConnectOptions::from_str(&database_url)?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .synchronous(SqliteSynchronous::Normal);
let pool = SqlitePool::connect_with(opts).await?;
```

### 5. Pattern `.map_err()` correct (FIX admin.rs)
```rust
// ❌ Erreur : mismatched closing delimiter
.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"msg": "err"})))?

// ✅ Syntaxe correcte
.map_err(|_| {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"message": "Erreur DB"})),
    )
})?
```

## 🗄️ Patterns SQLx

```rust
// Query typée avec cache offline
let messages = sqlx::query_as!(
    Message,
    r#"SELECT id, conversation_id, sender_id, content, message_type,
              file_id, encrypted, timestamp, created_at, edited_at
       FROM messages
       WHERE conversation_id = ?
       ORDER BY timestamp ASC
       LIMIT 100"#,
    conversation_id
)
.fetch_all(&pool)
.await?;

// ⚠️ Toujours utiliser r#"..."# pour les requêtes multi-lignes
// ⚠️ Les macros sqlx! nécessitent DATABASE_URL ou .sqlx/queries.json
```

## 🔐 Pattern auth — require_auth

```rust
async fn mon_endpoint(
    State(pool): State<SqlitePool>,
    cookies: CookieJar,
) -> Result<Json<MonType>, StatusCode> {
    let user = require_auth(&pool, &cookies).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    // user.approved == 1 garanti ici
    // ...
}
```

## 🐛 Erreurs fréquentes

| Erreur | Cause probable | Fix |
|--------|----------------|------|
| `SQLITE_CANTOPEN (14)` | Fichier DB inexistant | `create_if_missing(true)` |
| `type annotations needed` | SQLx ambiguïté | Annoter type retour |
| `cannot find value thread_rng` | rand 0.9 | `rand::rng()` |
| `mismatched types: Utf8Bytes` | axum 0.8 | `.into()` sur String |
| `proc-macro panic` | async-trait dans Docker | Retirer tower_governor |
