# 🦀 Rust Resources — Nook

> Référence **opérationnelle** des crates Rust utilisées ou à intégrer dans Nook.
> Versions exactes et patterns validés en production.
> Mis à jour : session 44

---

## ✅ Crates actuellement dans Cargo.toml

```toml
# backend/Cargo.toml — versions de production validées
axum          = { version = "0.8", features = ["ws", "multipart", "tokio"] }
axum-extra    = { version = "0.10", features = ["typed-header"] }
sqlx          = { version = "0.8.6", features = ["sqlite", "migrate", "runtime-tokio-rustls"] }
tokio         = { version = "1.0", features = ["full"] }
tower-http    = { version = "0.6.8", features = ["fs", "cors", "compression-br"] }
serde         = { version = "1.0", features = ["derive"] }
serde_json    = "1.0"
uuid          = { version = "1.0", features = ["v4"] }
chrono        = { version = "0.4", features = ["serde"] }
rand          = "0.9"           # utiliser rand::rng() — PAS thread_rng()
rand_core     = "0.6"           # utiliser rand_core::OsRng — PAS rand::rngs::OsRng
argon2        = "0.5"
chacha20poly1305 = "0.10.1"
base64ct      = "1.6"
tracing       = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## 🔴 Pièges Rust connus dans Nook

### rand 0.9 — API changée
```rust
// ❌ Ancien (rand 0.8) — ne compile pas
use rand::thread_rng;
let mut rng = thread_rng();

// ✅ Nouveau (rand 0.9)
let mut rng = rand::rng();
let val: f64 = rng.random::<f64>();
```

### rand_core::OsRng — diamond dependency
```rust
// ❌ rand::rngs::OsRng — conflit de type avec argon2
// ✅ rand_core::OsRng — compatible argon2 0.5
use rand_core::OsRng;
let password_hash = Argon2::default().hash_password(pwd, &SaltString::generate(&mut OsRng))?;
```

### DefaultBodyLimit — obligatoire pour les uploads
```rust
// main.rs — sans ça, Axum limite à 2MB (cause du bug upload > 7Mo)
use axum::extract::DefaultBodyLimit;

let app = Router::new()
    .nest("/api", api_router)
    .layer(DefaultBodyLimit::max(52 * 1024 * 1024)) // 52MB
    // ...
```

### spawn_blocking — obligatoire pour le code CPU-bound
```rust
// ❌ Bloque le thread Tokio → serveur freeze
let result = play_ai(game, difficulty)?;

// ✅ CPU-bound dans spawn_blocking
let result = tokio::task::spawn_blocking(move || {
    play_ai(game, difficulty)
}).await??;
```

### sqlx sans macro quand queries.json est absent
```rust
// ❌ sqlx! macro → erreur si .sqlx/queries.json pas régénéré
let row = sqlx::query!("SELECT id FROM users WHERE id = ?", id)...

// ✅ query_as sans macro → toujours safe
let row: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE id = ?")
    .bind(&id)
    .fetch_optional(&state.db)
    .await?;
```

---

## 🟡 Crates à ajouter — LOT 3

### sysinfo — Métriques système pour `/api/admin/metrics`
```toml
sysinfo = "0.32"
```
```rust
use sysinfo::System;
let mut sys = System::new_all();
sys.refresh_all();
Json(json!({
    "cpu_usage": sys.global_cpu_usage(),
    "memory_used": sys.used_memory(),
    "memory_total": sys.total_memory(),
}))
```

### bollard — Stats Docker pour l'admin
```toml
bollard = "0.17"
```
```rust
use bollard::Docker;
let docker = Docker::connect_with_local_defaults()?;
let containers = docker.list_containers::<String>(None).await?;
```

---

## 🔵 Crates à ajouter — LOT 4

### turn-rs — Serveur TURN pour appels WAN
Pas une crate — déployer comme service Docker séparé.  
Voir `github-resources.md` pour la config docker-compose.

---

## 📐 Patterns Axum validés dans Nook

### Route avec auth + state
```rust
pub async fn my_handler(
    AxumState(state): AxumState<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(req): Json<MyRequest>,
) -> impl IntoResponse {
    // ...
}
```

### Route PATCH (nouveau en S43)
```rust
// main.rs
.route("/events/{id}", axum::routing::patch(db::update_event))
// ⚠️ Axum 0.8 : {id} et non :id
```

### Broadcast WS avec routage to_user_id (S42)
```rust
// Enregistrer le sender du user
state.webrtc_state.user_senders.lock().await.insert(user_id.clone(), tx);
// Router vers un user spécifique
if let Some(target_tx) = guard.get(target_user_id) {
    let _ = target_tx.send(message);
}
```

---

## 📚 Ressources officielles

- **Axum 0.8 docs :** https://docs.rs/axum/0.8/axum/
- **SQLx 0.8 :** https://docs.rs/sqlx/0.8/sqlx/
- **Tokio :** https://tokio.rs/
- **The Book :** https://doc.rust-lang.org/book/
- **Rust by Example :** https://doc.rust-lang.org/rust-by-example/
