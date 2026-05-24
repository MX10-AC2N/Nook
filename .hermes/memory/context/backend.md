# 🦀 Contexte Backend - Nook

> Mis à jour : 2026-05-24

## Stack Technique

- **Framework** : Axum 0.8 + Tower
- **Base de données** : SQLite avec SQLx 0.8 (migrations)
- **Auth** : Cookie HttpOnly `auth_token` = `{user_id}:{token}` + Argon2 password hashing
- **WebSocket** : axum::extract::ws sur `/api/webrtc/ws` (avec vérification d'authentification)
- **E2EE** : XChaCha20-Poly1305 (via webrtc.rs `encrypt_file_for_storage`, `decrypt_file_from_storage`)
- **Push** : VAPID (web-push + Firebase Cloud Messaging)

## Architecture Backend

```
backend/src/
├── main.rs              # Entry point — Router Axum
├── auth.rs              # Login, register, refresh, me, change-password
├── db.rs                # CRUD messages, conversations, reactions, users
├── reactions.rs         # POST/DELETE/GET reactions par message
├── upload.rs            # Upload/download fichiers (avec magic bytes validation + E2EE)
├── admin.rs             # Admin panel (pending count, approve users, global message)
├── conversations.rs     # CRUD conversations + participants
├── webrtc.rs            # WebSocket handler + WebRTC offer/answer/ICE routes
├── push.rs              # Push notifications VAPID
├── e2ee.rs              # E2EE key storage (encrypted_keys par message)
├── chess_engine/        # Moteur d'échecs Rust pur
├── migrations/          # SQLx migrations
└── Cargo.toml
```

## Routes API (préfixe /api/)

```
AUTH
  POST   /api/auth/register
  POST   /api/auth/login
  POST   /api/auth/logout
  POST   /api/auth/refresh
  GET    /api/auth/me
  POST   /api/auth/change-password

USERS
  GET    /api/users              # Liste (admin)
  GET    /api/users/{id}
  PUT    /api/users/{id}
  GET    /api/users/public-key/{userId}
  POST   /api/users/device       # Enregistrement device (push)

CONVERSATIONS
  GET    /api/conversations
  POST   /api/conversations
  GET    /api/conversations/{id}
  PUT    /api/conversations/{id}
  DELETE /api/conversations/{id}
  POST   /api/conversations/{id}/participants
  DELETE /api/conversations/{id}/participants/{userId}
  POST   /api/conversations/{id}/read       # Marquer comme lu

MESSAGES (dans conversations)
  GET    /api/conversations/{id}/messages    # ?limit=50&before={timestamp}
  POST   /api/conversations/{id}/messages    # {content, encrypted, nonce, encrypted_keys[]}
  PATCH  /api/conversations/{id}/messages/{msg_id}
  DELETE /api/conversations/{id}/messages/{msg_id}

REACTIONS (dans messages)
  POST   /api/conversations/{id}/messages/{msg_id}/reactions   # {emoji}
  DELETE /api/conversations/{id}/messages/{msg_id}/reactions
  GET    /api/conversations/{id}/messages/{msg_id}/reactions

FILES
  POST   /api/upload              # multipart: file + conversation_id → {file_id, url}
  GET    /api/download/{id}       # Téléchargement (déchiffrement automatique)
  GET    /api/files/{id}/info     # Métadonnées fichier

CHAT/ADMIN
  POST   /api/admin/send-global   # Broadcast message global (admin only)
  GET    /api/admin/pending-count

WEBRTC (nestées sous webrtc)
  POST   /api/webrtc/offer
  POST   /api/webrtc/answer
  GET    /api/webrtc/ws          # WebSocket upgrade
  GET    /api/webrtc/ice-config  # Config STUN/TURN

HEALTH
  GET    /api/health             # Healthcheck (retourne JSON statut)

CHESS
  GET    /api/chess/games
  POST   /api/chess/games
  GET    /api/chess/games/{id}
  DELETE /api/chess/games/{id}
  POST   /api/chess/games/{id}/move
  POST   /api/chess/games/{id}/resign

POLLS
  GET    /api/polls
  POST   /api/polls
  GET    /api/polls/{id}
  POST   /api/polls/{id}/vote
```

## Points Critiques Backend

### ✅ Corrections Récentes
- **Axum 0.8** : Migration syntaxe `:capture` → `<capture: _>` complétée
- **Clippy** : Warnings `unused_assignments`, `needless_collect` corrigés
- **fix(e2ee/crypto)** : `registerPublicKeyOnServer` est maintenant `await`-é avant `cryptoStore.ready = true`
- **fix(e2ee/encrypt)** : try/catch par destinataire dans `encryptForRecipients` + nettoyage `_FAILED_DECRYPT_IDS`
- **fix(upload/security)** : Validation magic bytes (SEC-04) contre les fichiers déguisés
- **Broadcast WS** : Contraint aux participants uniquement (pas broadcast global)// FIX C4

### ⚠️ Problèmes Connus
- **E2EE anciens messages** : Indéchiffrables après rotation clé X25519 — c'est structurel, pas fixable en code
- ** Prune DB** : `FOREIGN KEY` sur `prune_events.sql` supprimée pour éviter les erreurs de nettoyage
- **Mutating borrow dans reactions.rs** : `send` appelé sur `guard` puis `guard` réutilisé → silencieusement non-bloquant mais à surveiller

### E2EE Backend
```
messages.encrypted        BOOLEAN — si le contenu est chiffré
messages.nonce            TEXT   — nonce XChaCha20
messages.encrypted_keys   JSON   — { recipient_user_id: encrypted_session_key }
users.public_key          TEXT   — clé publique X25519 de chaque utilisateur
users.private_key_enc     TEXT   — clé privée X25519 chiffrée par mot de passe (IndexedDB)
```

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

# Logs backend
RUST_LOG=debug cargo run
```

## Migrations SQL Clés

- `add_e2ee_columns.sql` : ajout `encrypted`, `nonce`, `encrypted_keys`, `message_type`, `file_id`
- `add_session_keys_table.sql` : table `encrypted_session_keys` pour stocker clés de session E2EE

## MCP Servers
- **rust-mcp-server** : Analyse statique, clippy intégré
- **rust-analyzer (lsp-mcp-server)** : Navigation sémantique
