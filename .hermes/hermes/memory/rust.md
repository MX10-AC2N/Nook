# 🦀 Mémoire RUST - Apprentissages & Patterns

> **DERNIÈRE MISE À JOUR** : 2026-05-04
> Patterns Rust/Axum/SQLx pour Nook

## 📦 Crates & Versions

### Backend Actuel (Cargo.toml)
- **Rust Edition** : 2024 (edition = "2024")
- **Axum** : 0.8 (⚠️ Breaking changes vs 0.7)
- **SQLx** : 0.8.6 (avec runtime tokio, features: sqlite, macros, migrate)
- **Tokio** : 1.44.1 (full features)
- **rand** : 0.9 (⚠️ Breaking changes vs 0.8)
- **rustrtc** : 0.3.40 (WebRTC)
- **argon2** : 0.5.3 (Password hashing)
- **jsonwebtoken** : 9.3.1 (JWT)
- **voca_rs** : 0.9.1 (Validation)

## 🔧 Patterns Axum 0.8

### Routing
```rust
// ❌ Ancien (Axum 0.7)
router.route("/users/:id", get(handler));

// ✅ Nouveau (Axum 0.8)
router.route("/users/{id}", get(handler));
```

### Handlers
```rust
// ✅ Axum 0.8 - Utf8Bytes pour le body
async fn handler(body: Utf8Bytes) -> Response<Utf8Bytes> { ... }

// ✅ Extracteurs
async fn handler(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<MyStruct>,
) -> Json<Response> { ... }
```

## 🗄️ Patterns SQLx 0.8

### Requêtes avec transaction
```rust
// ✅ Pattern avec transaction
let mut tx = state.db.begin().await?;

sqlx::query("INSERT INTO users (name) VALUES (?)")
    .bind(&name)
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
```

### Préparation (sqlx prepare)
```bash
# Générer query.sql pour les macros SQL
cargo sqlx prepare --workspace -- --features sqlite
```

⚠️ **Problème connu** : `cargo sqlx prepare` échoue en CI (voir known-issues.md)

## 🎲 Patterns rand 0.9

### Génération aléatoire
```rust
// ❌ Ancien (rand 0.8)
use rand::thread_rng;
let mut rng = thread_rng();

// ✅ Nouveau (rand 0.9)
use rand::rng;
let mut rng = rng();

// ✅ Distributions
use rand::distr::{Alphanumeric, Uniform};
let chars: String = (0..length)
    .map(|_| rng.sample(Alphanumeric))
    .collect();
```

## 🚨 Erreurs fréquentes

### map_err avec closures
```rust
// ❌ Incorrect - parenthèses mal placées
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

// ✅ Correct - accolades pour le bloc
.map_err(|e| { 
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()) 
})?;
```

### lifetime dans les structs
```rust
// ⚠️ Problème avec 'static dans les handlers axum
// Solution: utiliser Clone ou Arc pour partager les données
#[derive(Clone)]
struct AppState {
    db: sqlx::SqlitePool,
}
```

## 🔨 Commandes Utiles

### Build & Test
```bash
# Build release
cargo build --release

# Test avec output
cargo test -- --nocapture

# Clippy strict
cargo clippy -- -D warnings

# Format
cargo fmt --all
```

### SQLx
```bash
# Préparer les requêtes (générer query.sql)
cargo sqlx prepare --workspace -- --features sqlite

# Vérifier les migrations
sqlx migrate info --source backend/migrations
```

## 📝 Notes de Session

- Rust nightly 1.97.0 installé pour CI
- cargo sqlx prepare échoue en CI (à investiguer)
- Axum 0.8 nécessite des ajouts dans Cargo.toml pour Utf8Bytes

---
*Mettre à jour après chaque session de dev backend*
