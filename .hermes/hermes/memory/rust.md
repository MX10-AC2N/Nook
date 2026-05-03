# 🦀 Mémoire Rust - Apprentissages Backend

> Dernière mise à jour: 2026-05-03
> Consulté lors de tout dev backend

## 📦 Crates & Versions

### Core
- **Rust edition** : 2024 (nightly requis pour Nook)
- **Axum** : 0.8
- **SQLx** : 0.8.6
- **Tokio** : 1.x
- **rand** : 0.9+ (⚠️ **IMPORTANT**)
  - ✅ `use rand::rng();` puis `rng()`
  - ❌ NE PLUS UTILISER `thread_rng()`
  - ✅ `use rand::distr::*` (pas `distributions::`)
  - ✅ `rand::distr::Alphanumeric` (pas `rand::distributions::Alphanumeric`)

### WebRTC & Crypto
- **rustrtc** : 0.3.40 (⚠️ ne pas rétrograder en 0.3.39)
- **argon2** : pour hash passwords
- **chacha20poly1305** : pour E2EE

## 🔧 Patterns Axum 0.8

### Routes (⚠️ Changement majeur vs 0.7)
```rust
// ❌ Axum 0.7
.get("/api/messages/:id", handler)

// ✅ Axum 0.8
.get("/api/messages/{id}", handler)
```

### Extracteurs
```rust
// ✅ Axum 0.8 - Utf8Bytes pas String
async fn handler(Path(id): Path<i64>, body: Utf8Bytes) -> impl IntoResponse
```

### Gestion d'erreurs
```rust
// ❌ INCORRECT
.map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Error"))?

// ✅ CORRECT
.map_err(|_| { 
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Error") 
})?
```

## 🗄️ SQLx Patterns

### Migrations
```bash
# Préparer migrations pour CI
cargo sqlx prepare --workspace -- --all-targets
```

### Query patterns
```rust
// Requête simple
let result = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = ?")
    .bind(id)
    .fetch_one(&pool)
    .await?;

// Transaction
let mut tx = pool.begin().await?;
sqlx::query("INSERT INTO ...")
    .bind(...)
    .execute(&mut *tx)
    .await?;
tx.commit().await?;
```

## 🧪 Tests & Clippy

### Exécuter tests
```bash
cd backend
cargo test
```

### Clippy (toujours avant commit)
```bash
cargo clippy -- -D warnings
```

## 📝 Learnings Sessions

### Session 50-53
- ✅ `admin.rs` : fix map_err avec accolades `{}`
- ✅ rand 0.9 migration réussie
- ✅ CI Backend Build : Rust nightly requis (Backend.yml ligne 34)

### Erreurs rencontrées
1. **rand::distributions déprécié** → utiliser `rand::distr`
2. **thread_rng() déprécié** → utiliser `rng()`
3. **Axum 0.8 syntaxe** → `{param}` pas `:param`

## 🔗 Ressources

- [Axum 0.8 Docs](https://docs.rs/axum/0.8.0/axum/)
- [SQLx 0.8 Docs](https://docs.rs/sqlx/0.8.6/sqlx/)
- [Rust Book](https://doc.rust-lang.org/book/)

---
*Ajouter nouveaux apprentissages au fur et à mesure*
