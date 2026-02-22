# 🤖 CLAUDE.md — Référence opérationnelle du projet Nook

> Fichier de référence destiné à Claude. À lire en priorité avant toute intervention.  
> Dernière mise à jour : 2026-02-21

---

## 📍 Localisation du projet

- **Repo** : https://github.com/MX10-AC2N/Nook
- **Branche de travail** : `MX10-AC2N-patch-svelte5-runes`
- **Lire les fichiers** : `https://raw.githubusercontent.com/MX10-AC2N/Nook/MX10-AC2N-patch-svelte5-runes/[chemin]`
- **Lire `.claude/`** en priorité avant toute intervention

---

## 🏗️ Architecture complète

```
Nook/
├── backend/                    # API Rust (Axum 0.8 + SQLite)
│   ├── src/
│   │   ├── main.rs             # Point d'entrée, router Axum, middleware base_inject
│   │   ├── auth.rs             # Register/Login/Logout/Me/ChangePassword + Argon2
│   │   ├── db.rs               # Structs SQLx + handlers conversations/messages
│   │   ├── admin.rs            # Gestion utilisateurs, approbation, invites
│   │   ├── invites.rs          # Génération/validation liens d'invitation
│   │   ├── upload.rs           # Upload fichiers chiffrés (max 50Mo, TTL 48h)
│   │   ├── webrtc.rs           # Signaling WebRTC (WebSocket) + crypto XChaCha20
│   │   ├── cleanup.rs          # Nettoyage fichiers expirés
│   │   ├── prune.rs            # Pruning DB périodique (toutes les 24h)
│   │   └── emergency.rs        # Mode urgence
│   ├── migrations/
│   │   ├── 001_initial.sql
│   │   └── 002_add_file_id_to_messages.sql
│   ├── .sqlx/queries.json      # Cache SQLx offline mode (CI)
│   ├── Cargo.toml
│   └── Cargo.lock              # Régénéré 2026-02-21 — axum 0.8 + rand 0.9
│
├── frontend/                   # SvelteKit 5 + TypeScript strict
│   ├── src/lib/
│   │   ├── authStore.svelte.js         # ⚠️ exports manquants (Bug #2)
│   │   ├── chatStore.svelte.ts         # ✅ pattern correct
│   │   ├── conversationStore.svelte.ts # ⚠️ state_invalid_export (Bug #1)
│   │   ├── mediaStore.svelte.js
│   │   ├── webrtc-calls.svelte.ts
│   │   └── sodium.svelte.js
│   └── package.json             # v0.5.0
│
├── VERSION                      # Source de vérité version (0.5.0)
│
├── .github/workflows/
│   ├── Backend.yml              # Manuel : compile Rust x86_64 + aarch64
│   ├── Frontend.yml             # Manuel : build SvelteKit
│   ├── test-nook.yml            # Manuel : intégration Docker + tests API
│   ├── Docker.yml               # Manuel : assemble artifacts → GHCR
│   ├── ci-new2.yml              # Auto : pipeline complet sur push
│   └── release.yml              # Manuel : bump VERSION + tag git
│
├── Dockerfile                   # Build sources (test-nook.yml + docker-compose)
├── Dockerfile.release           # Binaires pré-compilés (Docker.yml + ci-new2.yml)
└── docker-compose.yml
```

---

## 🦀 Stack Backend (Rust)

| Crate | Cargo.toml | Cargo.lock | Rôle |
|-------|-----------|-----------|------|
| axum | 0.8 | 0.8.8 | HTTP + WebSocket + multipart |
| axum-extra | 0.10 | 0.10.3 | TypedHeader |
| tokio | 1.0 | 1.49 | Runtime async |
| sqlx | 0.8.6 | 0.8.6 | SQLite + migrations |
| argon2 | 0.5 | 0.5.3 | Hash password |
| chacha20poly1305 | 0.10.1 | 0.10.1 | Chiffrement XChaCha20 |
| rand | 0.9 | 0.9.2 | RNG — `rand::rng()` |
| rand_core | 0.6 | 0.6.4 | OsRng pour argon2 |
| base64ct | 1.6 | 1.6.0 | Encodage base64 |
| tower-http | 0.6.8 | 0.6.8 | CORS, ServeDir, Compression |
| tower_governor | 0.8 | 0.8.0 | Rate limiting |
| reqwest | 0.13 | 0.13.2 | Client HTTP |
| headers | 0.4 | 0.4.1 | ContentDisposition |
| tracing | 0.1 | 0.1.44 | Logs |
| chrono | 0.4 | 0.4.43 | Timestamps |

**⚠️ Points critiques :**
- `rand_core` doit être **0.6** — argon2 l'attend, diamond dependency avec rand 0.9
- `use rand_core::OsRng` (pas `rand::rngs::OsRng`)
- `rand::rng()` remplace `rand::thread_rng()` (supprimé en rand 0.9)
- `Message::Text(msg.into())` — axum 0.8 attend `Utf8Bytes`
- `axum::extract::Host` n'existe plus → extraire depuis `headers.get("host")`

---

## ⚠️ RÈGLES CRITIQUES SVELTE 5

### Export de `$state` réassignable = ERREUR DE BUILD
```typescript
// ❌ INTERDIT
export let conversations = $state<Conversation[]>([]);
conversations = newData; // → state_invalid_export

// ✅ CORRECT
export const conversationStore = $state<ConversationState>(createInitialState());
conversationStore.conversations = newData;
```

### `$derived` / `$effect` = uniquement dans les composants `.svelte`
```typescript
// ❌ INTERDIT en module .svelte.ts
export const active = $derived(() => ...);

// ✅ CORRECT en module
export function getActive() { return conversationStore.conversations.find(...); }
```

### Stores Svelte 4 = BANNIS
```typescript
// ❌ INTERDIT
import { writable } from 'svelte/store';
```

---

## 🐳 Docker & CI

> ⚠️ Voir `DOCKER.md` pour le détail complet des règles et pièges.

**Deux Dockerfiles** :

| Fichier | Utilisé par | Stratégie |
|---------|-------------|-----------|
| `Dockerfile` | `test-nook.yml`, `docker-compose` | `cargo-chef` + sources |
| `Dockerfile.release` | `Docker.yml`, `ci-new2.yml` | Binaires pré-compilés |

**Flow manuel** :
```
Backend.yml  ──┐
               ├──▶ test-nook.yml ──▶ Docker.yml ──▶ GHCR
Frontend.yml ──┘
```

**Versioning** : `VERSION` à la racine = source de vérité.  
`release.yml` bumpe VERSION + Cargo.toml + package.json + tag git.

---

## 🗄️ Schéma DB

```sql
users(id, username, email, password_hash, name, role, approved, needs_password_change, token, created_at)
conversations(id, name, is_group, created_at, created_by, updated_at)
conversation_participants(conversation_id, user_id, joined_at)  -- ⚠️ Bug #5
messages(id, conversation_id, sender_id, content, message_type, file_id, encrypted, timestamp, created_at, edited_at)
uploads(id, conversation_id, from_user_id, file_name, file_path, file_size, content_type, uploaded_at, encrypted, nonce, key_text)
invites(code, created_by, created_at, expires_at, max_uses, current_uses)
```

---

## 📋 Commandes utiles

```bash
# Backend
cd backend
cargo check
cargo update                    # régénérer Cargo.lock
SQLX_OFFLINE=true cargo build
cargo clippy -- -D warnings

# Frontend
cd frontend && npm ci && npm run build

# Docker local
docker compose up -d
docker compose logs -f
```

---

## 🔄 Workflow Claude ↔ MX10-AC2N

1. Lire `.claude/` + fichiers GitHub avant d'intervenir
2. Fournir le contenu **complet** des fichiers (jamais de diffs partiels)
3. Toujours livrer en `.txt` pour éviter les bugs de téléchargement
4. Mettre à jour `.claude/` après chaque session
