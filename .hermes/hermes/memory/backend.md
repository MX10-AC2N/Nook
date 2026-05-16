# 🦀 Contexte Backend — Nook

> Mis à jour : 2026-05-16

## Stack Technique
- **Framework** : Axum 0.8
- **Base de données** : SQLite + SQLx 0.8.6 + Migrations
- **Auth** : JWT + HTTP-only cookies (header-only pour WebSocket)
- **E2EE** : X25519 + XChaCha20-Poly1305 (via sodium-wrappers)
- **TURN** : turn-rs
- **Build** : Rust stable, musl target pour Alpine 3.20

## Architecture

### Routes
- `backend/src/routes/` — modules par fonctionnalité
- `auth.rs` — register, login, logout, me
- `rooms.rs` — conversations CRUD
- `messages.rs` — send_message, get_messages, reactions
- `e2ee.rs` — get_my_e2ee_keys, register_public_key, get_member_public_keys
- `files.rs` — upload/download avec E2EE
- `events.rs` — WebSocket SSE pour real-time
- `config.rs` — configuration serveur

### Database (SQLx)
- Chemins : `backend/src/db.rs` (functions SQL)
- Migrations : `backend/src/migrations/` (fichiers .sql)
- Pattern : Query builder explicite, pas d'ORM
- Pool : sqlx::sqlite::SqlitePool (géré dans `main.rs`)

## Points Critiques E2EE Backend

### Fonctions E2EE dans `e2ee.rs`
```rust
// e2ee.rs ~ligne 60+
pub async fn get_member_public_keys(conn, room_id) -> Result<HashMap<Uuid, [u8; 32]>, Error>
// SELECT u.id, u.username, u.public_key FROM users u INNER JOIN room_members rm ...
// Retourne HashMap<user_id, [u8; 32]> (32 bytes X25519, validés avant INSERT/UPDATE)

pub async fn register_public_key(conn, user_id, pubkey: [u8; 32]) -> Result<(), Error>
// UPDATE users SET public_key = ? WHERE id = ?
// Validation 32 bytes avant UPDATE — rejette les clés malformées

pub async fn get_my_e2ee_keys(conn, user_id) -> Result<X25519KeyPair, Error>
// Récupère privkey/pubkey depuis users table (champs priv_key encrypted par Argon2)
// Retourne X25519KeyPair { privateKey: [u8; 32], publicKey: [u8; 32] }
```

### send_message et encrypted_keys dans `db.rs`
```rust
// db.rs ~ligne 345-545
pub async fn send_message(conn, room_id, sender_id, text, content_type, 
                          reply_to, mentions, sender_public_key: [u8; 32]) -> Result<Uuid, Error>

// INSERT INTO messages ... sender_public_key = ? ...
// INSERT INTO encrypted_keys (message_id, user_id, encrypted_key) VALUES (...) 
//   seulement si !encrypted_keys.is_empty() (ligne 456)
// → si HashMap vide, le champ encrypted_keys n'est pas créé en base
```

### get_messages_with_sender dans `db.rs` ~ligne 650+
```rust
// Jointure users.public_key via JOIN pour récupérer sender_public_key
// Renvoie Vec<MessageWithSender> avec tous les champs E2EE présents
```

## Points Critiques Build

### Musl cross-compilation (Alpine Docker)
- CI utilise `musl-unknown-linux-musl` target
- Nécessite `musl-tools` natif sur runner (pas Docker Alpine)
- `CGO_ENABLED=0` dans les builds de release

### Clippy
- `cargo clippy -- -D warnings` — 0 allows acceptable dans `/target/` seulement
- `#[allow(clippy::...)]` dans `tests/` seulement avec justification

### Axum 0.8 migrate
- Routes : `{param}` pas `:param`
- `FromRequestParts` remplace `FromRequest`
- `State<T>` via `State(app_state)` (extract pattern)

### Concurrency dans handlers
- Futures non-await dans handlers Axum → utiliser `tokio::spawn` + canal MPSC ou `join!()`
- Pas de `spawn` dans handler sync (bloqueant)

## Commandes Utiles
```bash
cargo check --target x86_64-unknown-linux-musl
cargo clippy --all-targets -- -D warnings -A clippy::tests-outside-tests-module
cargo sqlx prepare --check  # vérifie les requêtes à jour
```
