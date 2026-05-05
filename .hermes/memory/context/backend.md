# 🦀 Contexte Backend - Nook

> Mis à jour : 2026-05-05

## Stack Technique

- **Framework** : Axum 0.8
- **Base de données** : SQLite avec SQLx 0.8
- **Auth** : JWT + Argon2 (password hashing)
- **WebRTC** : WebRTC.rs + TURN/STUN (turn-rs)
- **WebSocket** : axum::extract::ws
- **E2EE** : XChaCha20-Poly1305

## Architecture

```
backend/
├── src/
│   ├── main.rs              # Entry point
│   ├── routes/             # Axum routes
│   │   ├── auth.rs         # Login, register, refresh
│   │   ├── messages.rs     # Messages CRUD + E2EE
│   │   ├── files.rs        # Upload/download
│   │   ├── calls.rs        # WebRTC signaling
│   │   ├── chess.rs        # Chess game API
│   │   └── ...
│   ├── models/             # SQLx models
│   ├── middleware/         # Auth, CORS, etc.
│   └── utils/              # Helpers
├── migrations/              # SQLx migrations
└── Cargo.toml
```

## Points Critiques

### ✅ Corrections Récentes
- **clippy warnings** : Nettoyés (unused variables, imports)
- **FOREIGN KEY** : Supprimée dans `prune_events.sql` pour permettre le nettoyage
- **events.rs ligne 316** : `{capture}` ajouté pour les paramètres de capture

### ⚠️ À Surveiller
- **WebRTC** : Vérifier la compatibilité cross-browser
- **E2EE** : Tests de bout en bout nécessaires
- **Performance** : Index SQLite sur les colonnes fréquemment requêtées

## Commandes Utiles

```bash
# Build release avec musl (static)
cargo build --release --target x86_64-unknown-linux-musl

# Tests
cargo test

# Clippy strict
cargo clippy -- -D warnings

# Migration SQLx
sqlx migrate run
```

## Patterns Récurrents

### Ajout de route Axum
```rust
// Dans routes/mod.rs
mod new_module;

// Dans main.rs
app = app.route("/api/new", get(new_module::handler));
```

### Modèle SQLx
```rust
#[derive(FromRow, Serialize, Deserialize)]
struct NewModel {
    id: String,
    created_at: DateTime<Utc>,
    // ...
}
```

## Connexions MCP

- **rust-mcp-server** : Analyse statique, navigation sémantique
- **SocratiCode** : Recherche sémantique dans le codebase
