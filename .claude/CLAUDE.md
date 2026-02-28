# 🤖 CLAUDE.md — Référence opérationnelle Nook

> Lire ce fichier EN PREMIER avant toute intervention.  
> Dernière mise à jour : **2026-02-28** (session 20)

---

## 📍 Localisation

- **Repo** : https://github.com/MX10-AC2N/Nook
- **Branche active** : `main` ← tout le travail est sur main désormais
- **Raw** : `https://raw.githubusercontent.com/MX10-AC2N/Nook/main/[chemin]`
- **Lire aussi** : `DOCKER.md`, `BUGS.md`, `SESSIONS.md` dans ce dossier

## 📊 Rapports CI disponibles dans `.claude/`

| Fichier | Généré par | Contenu |
|---------|-----------|---------|
| `FRONTEND-BUILD-REPORT.md` | `Frontend.yml` | Erreurs TS/Vite, warnings a11y svelte, tailles chunks gzip, npm audit |
| `BACKEND-BUILD-REPORT-amd64.md` | `Backend.yml` (job amd64) | cargo check/clippy/build, warnings avec fichier:ligne, taille binaire |
| `BACKEND-BUILD-REPORT-arm64.md` | `Backend.yml` (job arm64) | idem pour arm64 |
| `DOCKER-BUILD-REPORT.md` | `Docker.yml` | digest GHCR, tags publiés, tailles artifacts intégrés |
| `TEST_REPORT.md` | `test-nook.yml` | Résultats E2E Playwright (38 tests), logs Docker |

> ⚠️ **Deux fichiers backend** — la matrix compile amd64 et arm64 en parallèle.
> Un seul fichier partagé causerait une race condition (les deux jobs commitent simultanément).
> Lire les deux rapports pour avoir le statut complet.

---

## 🏗️ Architecture

```
Nook/
├── backend/
│   ├── src/
│   │   ├── main.rs        # Router Axum 0.8, middleware base_inject, init DB, E2E_SETUP
│   │   ├── auth.rs        # Register/Login/Logout/Me/ChangePassword — cookie HttpOnly
│   │   ├── db.rs          # Conversations + messages (SQLx)
│   │   ├── admin.rs       # Approbation users, invites, gestion
│   │   ├── invites.rs     # Génération/validation liens invitation
│   │   ├── upload.rs      # Upload fichiers (max 50Mo, TTL 48h)
│   │   ├── webrtc.rs      # Signaling WebSocket + XChaCha20-Poly1305
│   │   ├── prune.rs       # Nettoyage DB toutes les 24h
│   │   ├── cleanup.rs     # Nettoyage fichiers expirés
│   │   ├── config.rs      # Config depuis env vars
│   │   └── emergency.rs   # Mode urgence
│   ├── migrations/
│   │   ├── 001_initial.sql
│   │   └── 002_add_file_id_to_messages.sql
│   ├── .sqlx/queries.json # Cache offline SQLx (CI/Docker)
│   ├── .cargo/config.toml # Linkers cross + crt-static (Backend.yml seulement !)
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── frontend/
│   ├── src/
│   │   ├── lib/           # Stores Svelte 5, crypto, auth, webrtc
│   │   └── routes/        # login, register, chat, admin, calendar, call, settings…
│   ├── tests/e2e.spec.ts  # Tests Playwright E2E
│   ├── playwright.config.ts
│   └── package.json       # v0.5.0 — @playwright/test inclus
│
├── VERSION                # Source de vérité : 0.1.0
├── Dockerfile             # Build depuis sources (test-nook.yml + docker-compose local)
├── Dockerfile.release     # Binaires pré-compilés (Docker.yml)
├── docker-compose.yml     # Production (bind mounts, sans E2E_SETUP)
├── docker-compose.ci.yml  # Override CI (named volumes, init container, E2E_SETUP=1)
│
└── .github/workflows/
    ├── Backend.yml        # Manuel — compile Rust amd64 + arm64 → artifacts 7j
    ├── Frontend.yml       # Manuel — build SvelteKit → artifact 7j
    ├── test-nook.yml      # Manuel — intégration Docker + E2E Playwright
    ├── Docker.yml         # Manuel — assemble artifacts → GHCR (dawidd6)
    ├── Release.yml        # Manuel — bump VERSION + tag git
    └── update-frontend-lock.yml  # Manuel — régénère package-lock.json
```

---

## 🦀 Stack Backend

| Crate | Version | Rôle |
|-------|---------|------|
| axum | 0.8 | HTTP + WebSocket + multipart |
| sqlx | 0.8.6 | SQLite + migrations offline |
| argon2 | 0.5 | Hash password |
| rand | 0.9 | RNG (`rand::rng()`) |
| rand_core | **0.6** | OsRng pour argon2 ⚠️ diamond dep |
| tower-http | 0.6.8 | CORS, ServeDir, Compression |
| chacha20poly1305 | 0.10.1 | Chiffrement fichiers |
| tokio | 1.0 | Runtime async |
| reqwest | 0.13 | Client HTTP (rustls) |

### ⚠️ Points critiques Rust

```rust
// rand_core 0.6 OBLIGATOIRE pour argon2 (pas rand::rngs::OsRng)
use rand_core::OsRng;

// rand 0.9 : thread_rng() supprimé
rand::rng().fill_bytes(&mut buf);  // ✅

// axum 0.8 : routes avec {param} (plus :param)
.route("/conversations/{id}", get(handler))  // ✅

// axum 0.8 : Message::Text attend Utf8Bytes
Message::Text(msg.into())  // ✅

// axum 0.8 : Host supprimé → extraire depuis HeaderMap
headers.get("host").and_then(|v| v.to_str().ok())

// CORS : allow_credentials(true) incompatible avec Any
// → lister origines, méthodes et headers explicitement

// SQLite : toujours SqliteConnectOptions avec create_if_missing(true)
// SqlitePool::connect() refuse d'ouvrir un fichier inexistant → SQLITE_CANTOPEN (code 14)
```

---

## 🎨 Stack Frontend

- **SvelteKit 5** + TypeScript strict
- **Svelte 5 Runes** — voir règles dans `BUGS.md`
- Port dev : 5173 | Port prod : 6300

### Routes principales

| Route | Fichier | Description |
|-------|---------|-------------|
| `/` | `+page.svelte` | Redirect auto (admin→/admin, user→/chat, anon→/login) |
| `/login` | `login/+page.svelte` | Inputs `id="username"` + `id="password"` |
| `/chat` | `chat/+page.svelte` | Groupe Global hardcodé, textarea "Envoyer un message..." |
| `/admin` | `admin/+page.svelte` | Gestion users, approbation |
| `/register` | `register/+page.svelte` | Inscription (approved=0 par défaut) |

---

## 🗄️ Schéma DB

```sql
users(id, username, email, password_hash, name, role, approved,
      needs_password_change, token, created_at)
-- Admin initial : approved=1, needs_password_change=1, mdp "changeme2026"
-- E2E CI user   : approved=1, needs_password_change=0, mdp "E2eTest123!" (si E2E_SETUP=1)

conversations(id, name, is_group, created_at, created_by, updated_at)
conversation_members(conversation_id, user_id, joined_at)   -- NOM RÉEL EN DB
-- ⚠️ db.rs utilise "conversation_participants" → incohérence à corriger (Bug #5)
messages(id, conversation_id, sender_id, content, message_type,
         file_id, encrypted, timestamp, created_at, edited_at)
uploads(id, conversation_id, from_user_id, file_name, file_path,
        file_size, content_type, uploaded_at, encrypted, nonce, key_text)
invites(code, created_by, created_at, expires_at, max_uses, current_uses)
```

---

## 🔐 Auth : Cookie HttpOnly

```
Set-Cookie: auth_token=<userId>:<token>; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400
```

- Token stocké en DB → révocable
- `require_auth` vérifie `approved=1` ET token valide
- `needs_password_change` **non vérifié** dans `require_auth` → admin peut appeler les API même avec ce flag

---

## 📋 API Endpoints

```
POST /api/auth/register      — inscription (approved=0)
POST /api/auth/login         — login → cookie
POST /api/auth/logout        — révoque token
GET  /api/auth/me            — infos user courant (401 si non auth)
POST /api/auth/change-password

GET  /api/conversations      — liste conversations (auth)
POST /api/conversations      — créer conversation
GET  /api/conversations/{id}
POST /api/conversations/{id}/join
GET  /api/conversations/{id}/messages
POST /api/conversations/{id}/messages

POST /api/upload
POST /api/upload/chat

GET  /api/pending-users-json  — admin : users en attente (SimpleUser[])
GET  /api/all-users-json      — admin : tous les users
POST /api/approve             — admin : { user_id: string }
GET  /api/list-invites
POST /api/delete-invite
POST /api/generate-invite

GET  /api/health              — "OK" (texte, pas JSON)
GET  /api/invite/validate
POST /api/join

WS   /ws                      — WebSocket signaling WebRTC
POST /api/webrtc/offer
POST /api/webrtc/answer
```

---

## 🔄 Workflow de collaboration

1. **Toujours lire le repo avant d'intervenir** — fetcher les fichiers concernés via Raw GitHub
2. **Fournir le contenu complet** des fichiers (jamais de diffs partiels)
3. **Livrer en `.txt`** pour éviter les bugs de téléchargement Claude.ai
4. **Mettre à jour `.claude/`** après chaque session
5. **Lire `BUGS.md`** pour ne pas réintroduire des bugs déjà résolus
