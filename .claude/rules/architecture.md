# 🏗️ Architecture — Nook

> Référence : architecture, schéma DB, routes API, structure fichiers
> Mis à jour : session 29 (2026-03-07)

---

## Structure du repo

```
Nook/
├── backend/
│   ├── src/
│   │   ├── main.rs        # Router Axum 0.8, middleware, rate limiting governor, init DB
│   │   ├── auth.rs        # Register/Login/Logout/Me/ChangePassword — cookie HttpOnly
│   │   ├── db.rs          # Conversations + messages (SQLx) — nonce E2EE, sender_public_key
│   │   ├── admin.rs       # Approbation users, invites, analytics enrichis
│   │   ├── e2ee.rs        # Clés publiques X25519, message_keys — routes exposées depuis s28
│   │   ├── invites.rs     # Génération/validation liens invitation
│   │   ├── upload.rs      # Upload fichiers (max 50Mo, TTL 48h)
│   │   ├── webrtc.rs      # Signaling WebSocket + XChaCha20-Poly1305
│   │   ├── prune.rs       # Nettoyage DB toutes les 24h
│   │   ├── cleanup.rs     # Nettoyage fichiers expirés
│   │   ├── config.rs      # Config depuis env vars
│   │   ├── emergency.rs   # Mode urgence
│   │   ├── chess.rs       # Jeu d'échecs
│   │   └── polls.rs       # Sondages
│   ├── migrations/
│   │   ├── 001_initial.sql
│   │   ├── 002_chess_fide.sql
│   │   ├── 003_e2ee.sql    # nonce sur messages, message_keys, public_key sur users
│   │   └── 004_polls.sql
│   ├── .sqlx/queries.json     # Cache offline SQLx (CI/Docker)
│   ├── .cargo/config.toml     # Linkers cross + crt-static (Backend.yml seulement !)
│   ├── Cargo.toml             # governor = "0.10" pour rate limiting
│   └── Cargo.lock
│
├── frontend/
│   ├── src/
│   │   ├── lib/
│   │   │   ├── crypto.ts          # Primitives E2EE (dynamic import libsodium)
│   │   │   ├── cryptoStore.svelte.ts # Store E2EE : unlockCrypto, encryptMessage, decryptMessage
│   │   │   ├── chatStore.svelte.ts   # Chat : loadMessages déchiffre E2EE si cryptoStore.ready
│   │   │   ├── sodium.svelte.js      # waitForSodium() — dynamic import, singleton loadingPromise
│   │   │   ├── storage.ts            # Dynamic import libsodium (DT-01)
│   │   │   ├── backup.ts             # Dynamic import libsodium (DT-01)
│   │   │   └── e2ee.ts               # Dynamic import libsodium (DT-01)
│   │   └── routes/            # login, register, chat, admin, calendar, call, settings…
│   ├── tests/e2e.spec.ts      # Tests Playwright E2E (45 tests)
│   ├── playwright.config.ts
│   ├── vite.config.js         # optimizeDeps.exclude libsodium, manualChunks
│   └── package.json
│
├── VERSION                    # Source de vérité : 0.3.0-beta.2
├── Dockerfile                 # Build depuis sources (test-nook.yml + local)
├── Dockerfile.release         # Binaires pré-compilés (Docker.yml)
├── docker-compose.yml         # Production (bind mounts, sans E2E_SETUP)
│
└── .github/workflows/
    ├── Backend.yml            # Manuel — compile Rust amd64 + arm64
    ├── Frontend.yml           # Manuel — build SvelteKit
    ├── test-nook.yml          # Manuel — intégration Docker + E2E
    ├── Docker.yml             # Manuel — assemble → GHCR (dawidd6)
    ├── Release.yml            # Manuel — bump VERSION + tag git
    └── update-frontend-lock.yml
```

---

## 🗄️ Schéma DB (SQLite)

```sql
users(
  id, username, email, password_hash, name, role,
  approved,                    -- 0=en attente, 1=approuvé
  needs_password_change,       -- 1 = forcer changement mdp
  token, public_key,           -- public_key X25519 base64 (E2EE)
  created_at
)
-- Admin initial  : approved=1, needs_password_change=1, mdp "changeme2026"
-- E2E CI user    : approved=1, needs_password_change=0, mdp "E2eTest123!" (si E2E_SETUP=1)

conversations(id, name, is_group, created_at, created_by, updated_at)

conversation_participants(conversation_id, user_id, joined_at)
-- ⚠️ nom réel confirmé : conversation_participants (pas conversation_members)

messages(
  id, conversation_id, sender_id, content, message_type,
  file_id, encrypted, nonce,   -- nonce XSalsa20 base64 si encrypted=true
  timestamp, created_at, edited_at
)

message_keys(                  -- migration 003
  message_id FK→messages,
  recipient_id FK→users,
  encrypted_key TEXT,          -- base64(asymNonce[24]||boxCiphertext)
  PRIMARY KEY (message_id, recipient_id)
)

uploads(
  id, conversation_id, from_user_id, file_name, file_path,
  file_size, content_type, uploaded_at, encrypted, nonce, key_text
)

invites(id, token, created_by, created_at, expires_at, used, used_by, used_at)

-- Tables chess (migration 002) : chess_games, chess_invitations
-- Tables polls (migration 004) : polls, poll_options, poll_votes
-- Tables events (migration 001) : events
```

---

## 🔐 Auth — Cookie HttpOnly

```
Set-Cookie: auth_token=<userId>:<token>; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400
```

- Token stocké en DB → révocable (logout = NULL en DB)
- `require_auth` vérifie `approved=1` ET token valide
- `needs_password_change` **non vérifié** dans `require_auth` (design intentionnel)
- Rate limiting : 10 req/min sur `/auth/login`, `/auth/register`, `/join` (governor 0.10)

---

## 📋 API Endpoints

```
# Auth (rate-limited : 10 req/min)
POST /api/auth/register
POST /api/auth/login          → Set-Cookie auth_token
POST /api/auth/logout         → NULL token en DB
GET  /api/auth/me             → 401 si non auth
POST /api/auth/change-password

# E2EE
POST /api/auth/public-key     → enregistre clé publique X25519
GET  /api/auth/public-keys?conversation_id=xxx → clés membres
GET  /api/conversations/{conv_id}/my-encrypted-key/{msg_id}

# Conversations & Messages
GET  /api/conversations
POST /api/conversations
GET  /api/conversations/{id}
POST /api/conversations/{id}/join
GET  /api/conversations/{id}/messages   → MessageWithSender[] (incl. nonce + sender_public_key)
POST /api/conversations/{id}/messages   → { content, encrypted, nonce?, encrypted_keys? }
GET  /api/conversations/{id}/participants
POST /api/conversations/{id}/participants
POST /api/conversations/{id}/leave

# Uploads
POST /api/upload
POST /api/upload/chat

# Utilisateurs
GET  /api/users/available
POST /api/user/update

# Événements
GET  /api/events
POST /api/events
DELETE /api/events/{id}

# Polls
GET  /api/polls
POST /api/polls
GET  /api/polls/{id}
POST /api/polls/{id}/vote
POST /api/polls/{id}/close
DELETE /api/polls/{id}

# Chess
POST /api/chess/create
GET  /api/chess/list
GET  /api/chess/{id}
POST /api/chess/{id}/join
POST /api/chess/{id}/move
POST /api/chess/{id}/ai-move
POST /api/chess/{id}/resign
GET  /api/chess/{id}/moves
POST /api/chess/{id}/invite
GET  /api/chess/invitations
POST /api/chess/invitations/{id}/accept
POST /api/chess/invitations/{id}/decline

# Admin (require_admin)
GET  /api/users/pending
GET  /api/users
POST /api/users/approve
GET  /api/invites
POST /api/invites
POST /api/invites/delete
GET  /api/analytics           → AnalyticsResponse enrichi (DT-06)

# Santé
GET  /api/health              → "OK" (texte brut)

# Invitations (rate-limited)
GET  /api/invite/validate
POST /api/join

# WebRTC Signaling
WS   /ws
```

---

## 🌐 Architecture LAN ↔ WAN

```
LAN (HTTP) :
  Navigateur (192.168.x.x) → Port 6300 → Backend Axum
  Cookie : SameSite=Lax
  CORS   : http://192.168.x.x:6300 dans ALLOWED_ORIGINS

WAN (HTTPS) :
  Navigateur → Nginx Proxy Manager (443) → Backend Axum (3000)
  Header : X-Forwarded-Proto: https
  Cookie : SameSite=None; Secure
  CORS   : https://nook.mondomaine.com dans ALLOWED_ORIGINS
```

---

## 🎨 Routes Frontend

| Route | Fichier | Description |
|-------|---------|-------------|
| `/` | `+page.svelte` | Redirect auto (admin→/admin, user→/chat, anon→/login) |
| `/login` | `login/+page.svelte` | Inputs `id="username"` + `id="password"` — appelle unlockCrypto() |
| `/chat` | `chat/+page.svelte` | Groupe Global hardcodé — E2EE actif si cryptoStore.ready |
| `/admin` | `admin/+page.svelte` | Gestion users, approbation |
| `/admin/analytics` | `admin/analytics/+page.svelte` | Dashboard enrichi : 6 compteurs + 2 charts |
| `/register` | `register/+page.svelte` | Inscription (approved=0) |
| `/chess` | `chess/+page.svelte` | Jeu d'échecs |
| `/calendar` | `calendar/+page.svelte` | Calendrier familial |
| `/polls` | `polls/+page.svelte` | Sondages — API backend |
| `/settings` | `settings/+page.svelte` | Profil, thème, mot de passe |
| `/call` | `call/+page.svelte` | Appel WebRTC |
| `/join` | `join/+page.svelte` | Inscription via token — génère clé publique E2EE |
