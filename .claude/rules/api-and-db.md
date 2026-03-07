# 📡 API & DB — Référence complète Nook

> Source de vérité extraite du code source (sessions 1-24).
> Ce fichier évite de fetcher `main.rs` + modules pour chaque intervention.

---

## 🗄️ Schéma DB complet (SQLite)

### users
```sql
id TEXT PK | username TEXT UNIQUE | email TEXT UNIQUE | password_hash TEXT
name TEXT | role TEXT DEFAULT 'user'
approved INTEGER DEFAULT 0      -- 0=en attente, 1=approuvé
needs_password_change INTEGER DEFAULT 0  -- 1=forcé au login
token TEXT                      -- session token (NULL = déconnecté)
public_key TEXT                 -- clé publique E2EE X25519 (base64)
created_at INTEGER              -- Unix timestamp (secondes)
```
Comptes spéciaux :
- `admin` : `approved=1`, `needs_password_change=1`, mdp initial `changeme2026`
- `e2e_ci` : `approved=1`, `needs_password_change=0`, mdp `E2eTest123!` (uniquement si `E2E_SETUP=1`)

### conversations
```sql
id TEXT PK | name TEXT | is_group INTEGER DEFAULT 1 | created_by TEXT FK→users
created_at INTEGER | updated_at INTEGER
```
Conversation spéciale : `default_global` (groupe global, toujours présent)

### conversation_participants
```sql
-- ⚠️ nom exact : conversation_participants (pas conversation_members)
conversation_id TEXT FK→conversations | user_id TEXT FK→users
joined_at INTEGER
PRIMARY KEY (conversation_id, user_id)
```

### messages
```sql
id TEXT PK | conversation_id TEXT FK | sender_id TEXT FK | content TEXT
message_type TEXT DEFAULT 'text' | file_id TEXT FK→uploads
encrypted INTEGER DEFAULT 0 | timestamp INTEGER | created_at INTEGER
edited_at INTEGER | nonce TEXT  -- NULL=clair, base64 XSalsa20 24B si E2EE
```

### uploads
```sql
id TEXT PK | conversation_id TEXT FK | from_user_id TEXT FK
file_name TEXT | file_path TEXT | file_size INTEGER | content_type TEXT
uploaded_at INTEGER | encrypted INTEGER DEFAULT 0
nonce TEXT | key_text TEXT          -- XChaCha20 si chiffré
```
Contraintes : max 50 Mo, TTL 48h (nettoyage par `prune.rs` + `cleanup.rs`)

### invites
```sql
id TEXT PK | token TEXT UNIQUE | created_by TEXT FK | created_at INTEGER
expires_at INTEGER               -- toujours +48h
used INTEGER DEFAULT 0 | used_by TEXT FK | used_at INTEGER
```

### events (calendrier)
```sql
id TEXT PK | title TEXT | date TEXT  -- format ISO "YYYY-MM-DD"
time TEXT                             -- format "HH:MM" optionnel
description TEXT | created_by TEXT FK | created_at INTEGER
```

### chess_games
```sql
id TEXT PK | created_by TEXT FK | player1_id TEXT FK | player2_id TEXT FK
player1_color TEXT | player2_color TEXT
status TEXT  -- 'waiting'|'playing'|'finished'
board_state TEXT DEFAULT '[]'    -- FEN string (standard FIDE)
move_history TEXT DEFAULT '[]'   -- JSON SAN moves
winner_id TEXT FK | ai_difficulty TEXT  -- NULL=humain vs humain
created_at INTEGER | updated_at INTEGER
```

### chess_invitations
```sql
id TEXT PK | game_id TEXT FK→chess_games ON DELETE CASCADE
invited_user_id TEXT FK | slot INTEGER | status TEXT DEFAULT 'pending'
-- status: 'pending'|'accepted'|'declined'
created_at INTEGER
```

### message_keys (E2EE — migration 003)
```sql
message_id TEXT FK→messages ON DELETE CASCADE
recipient_id TEXT FK→users ON DELETE CASCADE
encrypted_key TEXT   -- base64(asymNonce[24] || box_ciphertext) ~96 chars
PRIMARY KEY (message_id, recipient_id)
```

### polls
```sql
id TEXT PK | question TEXT | created_by TEXT FK ON DELETE CASCADE
created_at INTEGER | closed_at INTEGER  -- NULL=ouvert
```

### poll_options
```sql
id TEXT PK | poll_id TEXT FK ON DELETE CASCADE
text TEXT | position INTEGER DEFAULT 0
```

### poll_votes
```sql
poll_id TEXT FK | user_id TEXT FK | option_id TEXT FK | voted_at INTEGER
PRIMARY KEY (poll_id, user_id)   -- 1 vote par user par sondage (modifiable via UPSERT)
```

---

## 📋 API Endpoints — Surface complète

> Préfixe : `/api` | Cookie auth : `auth_token=<userId>:<token>; HttpOnly`

### 🔓 Routes publiques
```
POST /api/auth/register       → { username, email, password, name? }
POST /api/auth/login          → { username, password } → Set-Cookie auth_token
GET  /api/health              → "OK" (texte brut)
GET  /api/invite/validate     → ?token=xxx → { valid, invite }
POST /api/join                → { token, username, password, name? }
```

### 🔐 Routes protégées (require_auth)
```
# Auth
GET  /api/auth/me             → User courant
POST /api/auth/logout         → NULL token en DB
POST /api/auth/change-password → { current_password, new_password }

# Conversations
GET  /api/conversations       → Conversation[] (avec unread_count)
POST /api/conversations       → { name?, participant_ids[] }
GET  /api/conversations/{id}
POST /api/conversations/{id}/join
GET  /api/conversations/{id}/messages   → Message[]
POST /api/conversations/{id}/messages   → { content, message_type?, file_id?, nonce?, encrypted_keys? }
GET  /api/conversations/{id}/participants
POST /api/conversations/{id}/participants → { user_id }
POST /api/conversations/{id}/leave

# Uploads
POST /api/upload              → multipart: file + encryption_key? + nonce?
POST /api/upload/chat         → multipart: file + conversation_id

# Utilisateurs
GET  /api/users/available     → User[] (non membres de la conv)
POST /api/user/update         → { name?, username? }

# Événements calendrier
GET  /api/events              → Event[]
POST /api/events              → { title, date, time?, description? }
DELETE /api/events/{id}

# E2EE
POST /api/auth/public-key     → { public_key: base64 }
GET  /api/auth/public-keys    → ?conversation_id=xxx → { [userId]: publicKey }
GET  /api/conversations/{conv_id}/my-encrypted-key/{msg_id}

# Chess
POST /api/chess/create        → { color?, ai_difficulty? }
GET  /api/chess/list          → ChessGame[]
GET  /api/chess/{id}          → ChessGame + board FEN
POST /api/chess/{id}/join     → { color? }
POST /api/chess/{id}/move     → { from, to, promotion? }
POST /api/chess/{id}/ai-move
POST /api/chess/{id}/resign
GET  /api/chess/{id}/moves    → ?from=e2 → coups légaux
POST /api/chess/{id}/invite   → { user_id }
GET  /api/chess/invitations   → ChessInvitation[]
POST /api/chess/invitations/{id}/accept
POST /api/chess/invitations/{id}/decline

# Polls
GET  /api/polls               → Poll[]
POST /api/polls               → { question, options: string[] }
GET  /api/polls/{id}          → Poll + résultats
POST /api/polls/{id}/vote     → { option_id }
POST /api/polls/{id}/close
DELETE /api/polls/{id}        → admin seulement
```

### 🛡️ Routes admin (require_admin)
```
GET  /api/users/pending       → User[] (approved=0)
GET  /api/users               → User[] (tous)
POST /api/users/approve       → { user_id }
GET  /api/invites             → Invite[]
POST /api/invites             → { } → { token, expires_at }
POST /api/invites/delete      → { invite_id }
```

### 🔌 WebSocket
```
WS   /ws                      → Signaling (chess + appels WebRTC + chat temps réel)
```
Messages WS reçus par le client :
- `chess_move` — coup joué par l'adversaire
- `chess_player_joined` — adversaire rejoint la partie
- `chess_ai_move` — coup IA
- `webrtc_offer/answer/ice_candidate` — signaling P2P
- `call_request/call_accepted/call_rejected` — contrôle d'appel
- `new_message` — nouveau message chat (si polling WS actif)

---

## ⚙️ Variables d'environnement

| Variable | Défaut | Rôle |
|----------|--------|------|
| `PUBLIC_SITE_URL` | `http://localhost:6300` | URL principale — toujours dans CORS |
| `ALLOWED_ORIGINS` | *(vide)* | Origines CORS supplémentaires (virgule-séparées) |
| `HOST_PORT` | `6300` | Port exposé sur l'hôte (docker-compose) |
| `PORT` | `3000` | Port interne backend Axum |
| `DATABASE_URL` | `sqlite:///app/data/nook.db` | Chemin SQLite |
| `STATIC_FILES_DIR` | `/app/static` | Dossier build SvelteKit |
| `UPLOADS_DIR` | `/app/data/uploads` | Dossier fichiers uploadés |
| `DATA_DIR` | `./data` | Volume données (docker-compose) |
| `LOGS_DIR` | `./logs` | Volume logs (docker-compose) |
| `NOOK_IMAGE` | `ghcr.io/mx10-ac2n/nook:latest` | Image Docker |
| `RUST_LOG` | `info` | Niveau logs (`debug`/`info`/`warn`/`error`) |
| `RUST_BACKTRACE` | `1` | Backtrace Rust en erreur |
| `TZ` | `Europe/Paris` | Fuseau horaire |
| `E2E_SETUP` | `0` | `1` = crée user `e2e_ci` (CI uniquement, jamais en prod) |

> **Zimaboard** : `DATA_DIR=/media/ac2n-cloud/volume_docker_Nook/nook-data` | `LOGS_DIR=.../nook-logs`
