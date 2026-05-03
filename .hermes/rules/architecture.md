# 🏗️ Architecture — Nook

> Référence : architecture, schéma DB, routes API, structure fichiers

---

## Structure du repo

```
Nook/
├── backend/
│   ├── src/
│   │   ├── main.rs        # Router Axum 0.8, middleware, init DB, E2E_SETUP
│   │   ├── auth.rs        # Register/Login/Logout/Me/ChangePassword — cookie HttpOnly
│   │   ├── db.rs          # Conversations + messages (SQLx)
│   │   ├── admin.rs       # Approbation users, invites, gestion
│   │   ├── invites.rs     # Génération/validation liens invitation
│   │   ├── upload.rs      # Upload fichiers (max 50Mo, TTL 48h)
│   │   ├── webrtc.rs      # Signaling WebSocket + XChaCha20-Poly1305
│   │   ├── prune.rs       # Nettoyage DB toutes les 24h
│   │   ├── cleanup.rs     # Nettoyage fichiers expirés
│   │   ├── config.rs      # Config depuis env vars
│   │   ├── emergency.rs   # Mode urgence
│   │   ├── chess.rs       # Jeu d'échecs
│   │   ├── e2ee.rs        # Chiffrement E2E
│   │   └── polls.rs       # Sondages
│   ├── migrations/
│   │   ├── 001_initial.sql
│   │   ├── 002_chess_fide.sql
│   │   ├── 003_e2ee.sql
│   │   └── 004_polls.sql
│   ├── .sqlx/queries.json     # Cache offline SQLx (CI/Docker)
│   ├── .cargo/config.toml     # Linkers cross + crt-static (Backend.yml seulement !)
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── frontend/
│   ├── src/
│   │   ├── lib/               # Stores Svelte 5, crypto, auth, webrtc
│   │   └── routes/            # login, register, chat, admin, calendar, call, settings…
│   ├── tests/e2e.spec.ts      # Tests Playwright E2E (43 tests)
│   ├── playwright.config.ts
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
  token, created_at
)
-- Admin initial  : approved=1, needs_password_change=1, mdp "changeme2026"
-- E2E CI user    : approved=1, needs_password_change=0, mdp "E2eTest123!" (si E2E_SETUP=1)

conversations(id, name, is_group, created_at, created_by, updated_at)

conversation_participants(conversation_id, user_id, joined_at)
-- ⚠️ nom réel confirmé : conversation_participants (pas conversation_members)

messages(
  id, conversation_id, sender_id, content, message_type,
  file_id, encrypted, timestamp, created_at, edited_at
)

uploads(
  id, conversation_id, from_user_id, file_name, file_path,
  file_size, content_type, uploaded_at, encrypted, nonce, key_text
)

invites(code, created_by, created_at, expires_at, max_uses, current_uses)

-- Tables chess (migration 002)
-- Tables e2ee (migration 003)
-- Tables polls (migration 004)
```

---

## 🔐 Auth — Cookie HttpOnly

```
Set-Cookie: auth_token=<userId>:<token>; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400
```

- Token stocké en DB → révocable (logout = NULL en DB)
- `require_auth` vérifie `approved=1` ET token valide
- `needs_password_change` **non vérifié** dans `require_auth` (design intentionnel)

---

## 📋 API Endpoints

```
# Auth
POST /api/auth/register
POST /api/auth/login         → Set-Cookie auth_token
POST /api/auth/logout        → NULL token en DB
GET  /api/auth/me            → 401 si non auth
POST /api/auth/change-password

# Conversations & Messages
GET  /api/conversations
POST /api/conversations
GET  /api/conversations/{id}
POST /api/conversations/{id}/join
GET  /api/conversations/{id}/messages
POST /api/conversations/{id}/messages

# Uploads
POST /api/upload
POST /api/upload/chat

# Admin
GET  /api/pending-users-json    → SimpleUser[]
GET  /api/all-users-json
POST /api/approve               → { user_id: string }
GET  /api/list-invites
POST /api/delete-invite
POST /api/generate-invite

# Santé
GET  /api/health                → "OK" (texte brut, pas JSON)

# Invitations
GET  /api/invite/validate
POST /api/join

# WebRTC Signaling
WS   /ws
POST /api/webrtc/offer
POST /api/webrtc/answer
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
| `/login` | `login/+page.svelte` | Inputs `id="username"` + `id="password"` |
| `/chat` | `chat/+page.svelte` | Groupe Global hardcodé |
| `/admin` | `admin/+page.svelte` | Gestion users, approbation |
| `/register` | `register/+page.svelte` | Inscription (approved=0) |
| `/chess` | `chess/+page.svelte` | Jeu d'échecs |
| `/calendar` | `calendar/+page.svelte` | Calendrier familial |
| `/polls` | `polls/+page.svelte` | Sondages |
| `/settings` | `settings/+page.svelte` | Profil, thème, mot de passe |
| `/call` | `call/+page.svelte` | Appel WebRTC |
