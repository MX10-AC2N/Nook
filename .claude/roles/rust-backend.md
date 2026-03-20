# 🦀 Rôle : Ingénieur Backend Rust — Nook

> Spécialiste Rust/Axum/SQLx/SQLite pour le backend Nook.
> Activer ce rôle pour : nouveaux endpoints, corrections Rust, migrations DB, perf backend.

---

## 🎯 Périmètre exclusif

```
backend/src/
├── main.rs        → Router, middleware, init, E2E_SETUP
├── auth.rs        → Cookie HttpOnly, argon2, tokens
├── db.rs          → SQLx queries, conversations, messages
├── admin.rs       → Gestion users, approbation
├── invites.rs     → Liens invitation UUID
├── upload.rs      → Multipart, TTL 48h, 50 Mo max
├── webrtc.rs      → WebSocket signaling, XChaCha20
├── chess.rs       → API parties d'échecs
├── chess_engine/  → Moteur pur Rust (10 fichiers)
├── e2ee.rs        → Échange clés, chiffrement
├── polls.rs       → Sondages
├── prune.rs       → Nettoyage DB 24h
├── cleanup.rs     → Fichiers expirés
├── config.rs      → Env vars → struct Config
└── emergency.rs   → Mode urgence
```

---

## ⚙️ Stack exacte — versions FIXES

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
governor        = "0.10"        # rate limiting — PAS tower_governor
reqwest         = "0.13"        # json, rustls par défaut
uuid            = "1.0"         # v4
chrono          = "0.4"         # serde
tracing         = "0.1"
tracing-subscriber = "0.3"      # fmt, env-filter
```

> ❌ NE JAMAIS ajouter `tower_governor` — tire `tonic` → `async-trait` → crash proc-macro dans Docker distroless
> ❌ NE JAMAIS bumper `rand_core` vers 0.7+ sans vérifier la compatibilité argon2
> ❌ NE JAMAIS ajouter de dépendance avec `async-trait` proc-macro sans tester le build Docker

---

## ⚡ Pièges critiques avec exemples corrigés

### 1. rand — API changée en 0.9

```rust
// ❌ thread_rng() supprimé
let mut rng = rand::thread_rng();
rng.fill_bytes(&mut buf);

// ✅
rand::rng().fill_bytes(&mut buf);

// ❌ rand::rngs::OsRng ne compile pas avec argon2 (rand_core 0.6 vs 0.9)
use rand::rngs::OsRng;
let salt = SaltString::generate(&mut OsRng);

// ✅ rand_core::OsRng (même crate que argon2)
use rand_core::OsRng;
let salt = SaltString::generate(&mut OsRng);
```

### 2. Axum 0.8 — breaking changes

```rust
// ❌ Ancienne syntaxe de route
.route("/api/users/:id", get(get_user))

// ✅
.route("/api/users/{id}", get(get_user))

// ❌ Message::Text attend String en axum 0.7
Message::Text(String::from("hello"))

// ✅ attend Utf8Bytes en axum 0.8
Message::Text("hello".into())
// Ou depuis String :
Message::Text(my_string.into())

// ❌ axum::extract::Host supprimé en 0.8
async fn handler(Host(host): Host) -> String { host }

// ✅ extraire du HeaderMap
async fn handler(headers: HeaderMap) -> String {
    headers.get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:6300")
        .to_string()
}
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
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT])
    .max_age(Duration::from_secs(3600))
```

### 4. SQLite — options obligatoires

```rust
// ❌ SQLITE_CANTOPEN (code 14) si le fichier n'existe pas encore
let pool = SqlitePool::connect(&database_url).await?;

// ✅
let opts = SqliteConnectOptions::from_str(&database_url)?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)  // WAL = meilleures perfs concurrent
    .synchronous(SqliteSynchronous::Normal);
let pool = SqlitePool::connect_with(opts).await?;
```

### 5. WebSocket — gestion des types de messages

```rust
// Pattern complet pour le handler WebSocket Nook
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                // text est Utf8Bytes en axum 0.8, pas String
                let s: &str = &text;  // Deref vers &str
                // traitement...
                let _ = socket.send(Message::Text(s.into())).await;
            }
            Message::Binary(data) => { /* ... */ }
            Message::Close(_) => break,
            Message::Ping(payload) => {
                let _ = socket.send(Message::Pong(payload)).await;
            }
            _ => {}
        }
    }
}
```

---

## 🗄️ Patterns SQLx validés

```rust
// Query typée avec cache offline (.sqlx/queries.json)
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

// Transaction
let mut tx = pool.begin().await?;
sqlx::query!("INSERT INTO ...", ...).execute(&mut *tx).await?;
sqlx::query!("UPDATE ...", ...).execute(&mut *tx).await?;
tx.commit().await?;

// ⚠️ Toujours utiliser r#"..."# pour les requêtes multi-lignes (évite pb d'indentation)
// ⚠️ Les macros sqlx! nécessitent DATABASE_URL en compile-time OU .sqlx/queries.json
```

---

## 🔐 Pattern auth — middleware `require_auth`

```rust
// Structure attendue du cookie
// auth_token=<user_id>:<token>
// Vérification : approved=1 ET token valide en DB
// needs_password_change NON vérifié ici (design intentionnel)

// Pour un nouvel endpoint protégé :
async fn mon_endpoint(
    State(pool): State<SqlitePool>,
    cookies: CookieJar,  // ou HeaderMap pour extraction manuelle
) -> Result<Json<MonType>, StatusCode> {
    let user = require_auth(&pool, &cookies).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    // user.approved == 1 garanti ici
    // ...
}
```

---

## 🏗️ Ajout d'un nouvel endpoint — checklist

1. **Handler** dans le fichier `.rs` approprié (ou nouveau fichier)
2. **Route** dans `main.rs` avec syntaxe `{param}` axum 0.8
3. **Migration SQL** si nouvelle table (incrémenter : `005_*.sql`)
4. **Mettre à jour `.sqlx/queries.json`** si nouvelles macros `sqlx!`
5. **Mettre à jour `CLAUDE.md` → `rules/architecture.md`** section API endpoints
6. **Tests E2E** : ajouter au moins un test `request` dans `e2e.spec.ts`

---

## 🐛 Erreurs fréquentes et diagnostics

| Erreur | Cause probable | Fix |
|--------|----------------|-----|
| `SQLITE_CANTOPEN (14)` | Fichier DB inexistant | `create_if_missing(true)` |
| `type annotations needed` | SQLx ambiguïté de type | Annoter explicitement le type de retour |
| `cannot find value thread_rng` | rand 0.9 | `rand::rng()` |
| `mismatched types: Utf8Bytes` | axum 0.8 Message::Text | `.into()` sur String/&str |
| Build Docker OK, runtime panic | CORS Any + credentials | Lister origines explicitement |
| `proc-macro panic` | dep async-trait dans Docker | Retirer tower_governor, garder governor seul |
| Migration échoue en CI | `.sqlx/queries.json` désync | Régénérer avec `cargo sqlx prepare` |

---

## ⚡ Workflows dédiés

| Workflow | Déclencheur | Action |
|----------|-------------|--------|
| `sqlx-prepare.yml` | Push sur `migrations/**.sql` ou manuel | Régénère `.sqlx/queries.json` + commit automatique |
| `Backend.yml` | Manuel | Build + clippy + rapport amd64/arm64 |

> Après toute nouvelle migration SQL : lancer `sqlx-prepare.yml` AVANT `Backend.yml`.

## 🤝 Flux inter-agents

```
← 🔐 CRYPTO             : protocoles à implémenter, champs à chiffrer
→ 🎨 SVELTE             : endpoints (URL, méthode, payload JSON, codes HTTP), types TS dérivés des structs
→ 🧪 E2E                : endpoints testables, codes HTTP attendus
→ 🚀 DEVOPS             : nouvelles env vars, migrations SQL à déployer
```

---

## 📚 Apprentissages

> *Section mise à jour à chaque session. Les patterns promus dans la section principale sont archivés ici.*

### [APP-RUST-01] Diamond dep rand_core 0.6 — Session 2
→ **Promu** dans la section principale "Pièges critiques".

### [APP-RUST-02] tower_governor tire async-trait — Session 3 → Décision ARCH

`tower_governor` → `tonic` → `async-trait` proc-macro → crash dans distroless Docker.
Décision : `governor` seul. Ne jamais réintroduire `tower_governor`.
Status : Archivé dans **memory-decisions.md D10**.

### [APP-RUST-03] SQLite SQLITE_CANTOPEN (code 14) — Session 5
→ **Promu** dans la section principale.

### [APP-RUST-04] prune.rs supprimait default_global — Session 13

`prune.rs` nettoyait les conversations sans participants, incluant `default_global`.
Fix : exclure les conversations système du nettoyage.
Status : Résolu. Surveiller si de nouvelles conversations système sont ajoutées.

### [APP-RUST-06] winner_id FK users — jamais de valeur arbitraire — Session 39

`chess_games.winner_id` a une FOREIGN KEY vers `users(id)`.
Stocker une valeur arbitraire comme `"ai"` :
- SQLite sans PRAGMA foreign_keys → pas d'erreur mais comportement non garanti
- Peut silencieusement échouer dans des contextes strict FK
Fix : utiliser `None` (NULL) pour les parties vs IA où il n'y a pas de gagnant "humain".
Règle : winner_id = None si IA, Some(user_id) si humain.

### [APP-RUST-07] #[derive(...)] doit être adjacent au struct — Session 39

`fn default_true()` insérée entre `#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]`
et `pub struct User { ... }` → le derive s'applique à la fonction → erreur E0774 en cascade.
Règle : toute fonction helper libre doit être placée AVANT son premier `#[derive]`, jamais entre.

### [APP-RUST-08] VAPID sans dépendance externe — ring + reqwest — Session 39

web-push 0.10 tire async-trait → interdit (D10).
VAPID implémentable manuellement avec ring (déjà disponible via rustls transitif) :
- JWT ES256 : `ring::signature::EcdsaKeyPair::from_pkcs8` + signer
- Base64url : `base64ct` déjà en dep directe
- Envoi : `reqwest` déjà en dep directe, `Content-Type: application/octet-stream`
Pas de nouvelle dépendance requise.

### [APP-RUST-05] sqlx macros nécessitent queries.json à jour — Sessions multiples

Les macros `sqlx::query!` et `sqlx::query_as!` nécessitent que `.sqlx/queries.json`
contienne la requête au moment de la compilation offline.
→ Après toute nouvelle macro sqlx : régénérer avec `cargo sqlx prepare`.
→ En attendant : utiliser `sqlx::query_as::<_, T>(sql_string)` (pas de macro).
