---
name: nook-rust-backend
description: Skill spécialisé pour tout développement backend Rust du projet Nook. Utilise cette skill dès qu'un fichier .rs est impliqué, qu'un endpoint API est ajouté/modifié, qu'une migration SQL est nécessaire, qu'un build Rust échoue en CI, ou que le rapport BACKEND-BUILD-REPORT-*.md signale des erreurs. Couvre : Axum 0.8, SQLx 0.8.6, SQLite, auth cookies, upload, WebSocket signaling, chess engine, migrations, clippy, rand 0.9, rand_core 0.6.
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

## Stack — versions FIXES (ne pas bumper sans valider)

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
| Build arm64 ✅ / amd64 ✅ mais clippy ❌ | warnings promus erreurs | Corriger chaque warning |

## Flux inter-agents

```
→ Après tout endpoint modifié  : signaler URL + payload + codes HTTP à 🎨 SVELTE
→ Après toute migration SQL    : lancer sqlx-prepare.yml, informer 🚀 DEVOPS
→ Après tout changement DB     : mettre à jour rules/api-and-db.md
→ Tout endpoint nouveau        : informer 🧪 E2E avec contrat HTTP
```
