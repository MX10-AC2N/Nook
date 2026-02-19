# 🤖 CLAUDE.md — Référence opérationnelle du projet Nook

> Fichier de référence destiné à Claude. À lire en priorité avant toute intervention.  
> Dernière mise à jour : 2026-02-19

---

## 📍 Localisation du projet

- **Repo** : https://github.com/MX10-AC2N/Nook
- **Branche de travail** : `develop`
- **Branche PR active** : `MX10-AC2N-patch-svelte5-runes`
- **Lire les fichiers** : `https://raw.githubusercontent.com/MX10-AC2N/Nook/develop/[chemin]`

---

## 🏗️ Architecture complète

```
Nook/
├── backend/                    # API Rust (Axum 0.7 + SQLite)
│   ├── src/
│   │   ├── main.rs             # Point d'entrée, router Axum, middleware base_inject
│   │   ├── auth.rs             # Register/Login/Logout/Me/ChangePassword + Argon2
│   │   ├── db.rs               # Structs SQLx + handlers conversations/messages
│   │   ├── admin.rs            # Gestion utilisateurs, approbation, invites
│   │   ├── invites.rs          # Génération/validation liens d'invitation
│   │   ├── upload.rs           # Upload fichiers chiffrés (max 50Mo, TTL 7j)
│   │   ├── webrtc.rs           # Signaling WebRTC (WebSocket)
│   │   ├── cleanup.rs          # Nettoyage fichiers expirés
│   │   ├── prune.rs            # Pruning DB périodique (toutes les 24h)
│   │   └── emergency.rs        # Mode urgence
│   ├── migrations/
│   │   ├── 001_initial.sql     # Schéma complet
│   │   └── 002_add_file_id_to_messages.sql
│   ├── .sqlx/queries.json      # Cache SQLx offline mode (CI)
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── frontend/                   # SvelteKit 5 + TypeScript strict
│   ├── src/
│   │   ├── lib/
│   │   │   ├── authStore.svelte.js         # Auth (classe $state)
│   │   │   ├── chatStore.svelte.ts         # Messages, GIFs, erreurs (objet $state)
│   │   │   ├── conversationStore.svelte.ts # Conversations, participants (⚠️ BUG)
│   │   │   ├── mediaStore.svelte.js        # Enregistrement audio/vidéo
│   │   │   ├── webrtc-calls.svelte.ts      # État appels WebRTC (objet $state)
│   │   │   ├── sodium.svelte.js            # Chargement libsodium (objet $state)
│   │   │   ├── ui/
│   │   │   │   ├── ThemeStore.svelte.ts    # Thèmes (let $state privé + getter)
│   │   │   │   └── themes/                # CSS des 3 thèmes
│   │   │   ├── crypto.ts                  # Chiffrement E2EE (libsodium)
│   │   │   ├── types.ts                   # Tous les types TypeScript
│   │   │   └── storage.ts                 # IndexedDB (clés crypto)
│   │   ├── routes/
│   │   │   ├── +layout.svelte             # Layout global (⚠️ imports cassés)
│   │   │   ├── +page.svelte
│   │   │   ├── login/, register/, chat/, call/
│   │   │   ├── calendar/, admin/, settings/
│   │   │   ├── change-password/, invite/, join/
│   │   │   └── events/, polls/, help/
│   │   └── components/
│   │       ├── MediaPlayer.svelte          # (⚠️ import cassé connectionError)
│   │       └── MediaRecorder.svelte        # (⚠️ import cassé connectionError)
│   ├── svelte.config.js         # adapter-static, runes: true
│   ├── vite.config.js           # Proxy /api → :3000, chunks libsodium
│   ├── tsconfig.json            # strict, moduleResolution: bundler
│   └── package.json             # v0.5.0, Svelte 5.46.3, Vite 7.3.1
│
├── .github/workflows/ci-new2.yml  # CI principal
├── Dockerfile                      # Distroless multi-arch (amd64/arm64)
└── docker-compose.yml
```

---

## 🦀 Stack Backend (Rust)

| Crate | Version | Rôle |
|-------|---------|------|
| axum | 0.7 | Framework HTTP + WebSocket + multipart |
| tokio | 1.0 | Runtime async |
| sqlx | 0.8.6 | SQLite async + migrations |
| argon2 | 0.5 | Hachage mot de passe |
| chacha20poly1305 | 0.10.1 | Chiffrement symétrique |
| uuid | 1.0 | Génération IDs |
| serde / serde_json | 1.0 | Sérialisation |
| tower-http | 0.6.8 | CORS, ServeDir, Compression (br) |
| tracing | 0.1 | Logs structurés |
| tower_governor | 0.4.3 | Rate limiting |
| chrono | 0.4 | Timestamps Unix |
| reqwest | 0.12 | Client HTTP (proxy GIFs) |

**Auth** : Cookie `auth_token=userId:token` (HttpOnly, SameSite=Lax, Max-Age=86400)  
**DB** : SQLite à `/app/data/nook.db` — migrations sqlx au démarrage  
**Fichiers** : `/app/data/uploads/`, servis via route `/files/`  
**Frontend static** : `/app/static/` (build SvelteKit)  
**Port** : 3000

---

## 🧡 Stack Frontend (Svelte 5)

| Outil | Version | Rôle |
|-------|---------|------|
| svelte | ^5.46.3 | Framework UI (runes) |
| @sveltejs/kit | ^2.49.4 | Routing, SSG |
| vite | ^7.3.1 | Build tool |
| typescript | ^5.9.3 | Typage strict |
| @sveltejs/adapter-static | ^3.0.10 | SPA statique |
| libsodium-wrappers | ^0.8.0 | Chiffrement client |
| simple-peer | ^9.11.1 | WebRTC |

**Build** : SPA statique → `frontend/build/`, fallback `index.html`  
**PWA** : manifest.json présent, Service Worker désactivé (`register: false`)  
**Dev proxy** : `/api` → `http://127.0.0.1:3000`, `/ws` → `ws://127.0.0.1:3000`

---

## ⚠️ RÈGLES CRITIQUES SVELTE 5 — FONDAMENTALES

### Règle 1 : Export de `$state` réassignable = ERREUR DE BUILD
```typescript
// ❌ INTERDIT dans un module .svelte.ts — provoque l'erreur CI actuelle
export let conversations = $state<Conversation[]>([]);
conversations = newData; // réassignation → compile error "state_invalid_export"

// ✅ CORRECT — objet state unique, mutation de propriétés uniquement
export const conversationStore = $state<ConversationState>(createInitialState());
conversationStore.conversations = newData; // mutation → OK
```

### Règle 2 : `$derived` et `$effect` uniquement dans les composants `.svelte`
```typescript
// ❌ INTERDIT en module .svelte.ts
export const activeConv = $derived(() => ...);
$effect(() => { ... });

// ✅ CORRECT en module — fonctions getter pures
export function getActiveConversation(): Conversation | null {
  return conversationStore.conversations.find(
    c => c.id === conversationStore.activeConversationId
  ) ?? null;
}
// ✅ CORRECT en module — init explicite appelée depuis onMount()
export async function initConversationStore(): Promise<void> { ... }
```

### Règle 3 : Syntaxe runes dans les composants `.svelte`
```svelte
<script lang="ts">
  // Props — jamais "export let"
  interface Props { name: string; onClose?: () => void; value?: number; }
  let { name, onClose, value = 0 }: Props = $props();

  // State local
  let count = $state(0);
  
  // Derived local (OK dans .svelte)
  let doubled = $derived(count * 2);
  
  // Effect local (OK dans .svelte)
  $effect(() => {
    console.log(count);
    return () => console.log('cleanup');
  });
</script>
```

### Règle 4 : Stores Svelte 4 = BANNIS
```typescript
// ❌ INTERDIT — toute l'API Svelte 4 store
import { writable, readable, derived } from 'svelte/store';
export const myStore = writable(0);
myStore.set(1);
myStore.update(n => n + 1);
$myStore; // auto-subscription
```

### Règle 5 : Pattern module partagé Svelte 5
```typescript
// fichier: src/lib/monStore.svelte.ts
interface MonState { data: string[]; loading: boolean; error: string | null; }

function createInitialState(): MonState {
  return { data: [], loading: false, error: null };
}

export const monStore = $state<MonState>(createInitialState());

// Setters
export function setData(data: string[]): void { monStore.data = data; }
export function setError(err: string | null): void { monStore.error = err; }
export function reset(): void { Object.assign(monStore, createInitialState()); }

// Getters
export function getData(): string[] { return monStore.data; }
```

---

## 🔑 État actuel des stores (exports réels vs utilisés)

### `authStore.svelte.js`

**Exports RÉELS** :
```javascript
export const authStore         // instance AuthStore (user, token, isAuthenticated, authHeaders)
export function getIsAuthenticated()
export function getAuthHeaders()
export function getCurrentUser()
```

**Exports MANQUANTS** (utilisés dans tout le projet) :
```javascript
// Utilisé dans : layout, chat, conversationStore, chatStore, webrtc-calls, crypto, calendar, call
export const authUser         // → authStore.user
export const isAuthenticated  // → authStore.isAuthenticated
export const isAdmin          // → authStore.user?.role === 'admin'
export const needsPasswordChange // → authStore.user?.needs_password_change
export const authLoading      // → état de chargement init
export function initAuth()    // → initialisation au mount
export function setAuthenticated() // → utilisé dans login page
```

### `sodium.svelte.js`

**Exports RÉELS** :
```javascript
export const sodiumState      // { isReady, isLoading, error }
export async function waitForSodium()
export async function preloadSodium()
export function isSodiumReady()
export function getSodiumInstance()
```

**Exports MANQUANTS** (utilisés dans layout) :
```javascript
export const sodiumLoading    // writable store utilisé avec .subscribe()
export const sodiumError      // writable store utilisé avec $sodiumError
```

### `chatStore.svelte.ts`

**Exports RÉELS** :
```typescript
export const chatStore        // objet $state { messages, connectionError, gifResults, showGifs, gifLoading }
export function setMessages(), addMessage(), setConnectionError()
export function setGifResults(), toggleGifs(), setGifLoading(), resetChat()
export function getMessages(), getConnectionError(), getGifResults(), getShowGifs(), getGifLoading()
export function formatTimestamp(), loadMessages(), sendMessage(), sendGif()
export function searchGifs(), addReaction(), decryptMessageContent(), initUserKeys()
```

**Problème** : `connectionError` était un writable store Svelte 4 avant la migration.  
`conversationStore`, `mediaStore`, `MediaPlayer`, `MediaRecorder` font encore `connectionError.set(...)` → cassé.  
**Fix** : utiliser `setConnectionError(...)` à la place.

### `conversationStore.svelte.ts` — ❌ BUG BLOQUANT

**Problème** : variables `$state` exportées et réassignées.  
**Fix complet** : voir LEARNING.md section "Bug bloquant #1".

---

## 🗄️ Schéma Base de Données

```sql
-- Authentification
users(id TEXT PK, username TEXT UNIQUE, email TEXT UNIQUE, password_hash TEXT,
      name TEXT, role TEXT DEFAULT 'user', approved INT DEFAULT 0,
      needs_password_change INT DEFAULT 0, token TEXT, created_at INT)

-- Messaging
conversations(id TEXT PK, name TEXT, is_group BOOL, created_at INT, created_by TEXT, updated_at INT)
conversation_participants(conversation_id TEXT, user_id TEXT, joined_at INT, PK(conv_id, user_id))
messages(id TEXT PK, conversation_id TEXT, sender_id TEXT, content TEXT,
         message_type TEXT, file_id TEXT, encrypted BOOL, timestamp INT, created_at INT, edited_at INT)

-- Fichiers
uploads(id TEXT PK, conversation_id TEXT, from_user_id TEXT, file_name TEXT,
        file_path TEXT, file_size INT, content_type TEXT, uploaded_at INT,
        encrypted INT, nonce TEXT, key_text TEXT)

-- Invitations
invites(code TEXT PK, created_by TEXT, created_at INT, expires_at INT,
        max_uses INT, current_uses INT DEFAULT 0)
```

⚠️ `001_initial.sql` crée `conversation_members` mais `db.rs` utilise `conversation_participants` → migration incohérente à corriger.

---

## 🚀 Routes API Backend

```
POST /api/auth/register          → register()
POST /api/auth/login             → login() → cookie auth_token
GET  /api/auth/me                → me() → user info si cookie valide
POST /api/auth/logout            → logout() → clear cookie
POST /api/auth/change-password   → change_password()
POST /api/join                   → join par code d'invitation
GET  /api/invite/validate?code=  → valider un code
GET  /api/conversations          → liste des conversations user
POST /api/conversations          → créer conversation
GET  /api/conversations/:id      → détail conversation
POST /api/conversations/:id/join → rejoindre
GET  /api/conversations/:id/messages → liste messages (limit, before)
POST /api/conversations/:id/messages → envoyer message
POST /api/upload                 → upload fichier générique
POST /api/upload/chat            → upload fichier chat (chiffré)
GET  /api/pending-users-json     → [ADMIN] utilisateurs en attente
GET  /api/all-users-json         → [ADMIN] tous les utilisateurs
POST /api/approve                → [ADMIN] approuver un user
GET  /api/list-invites           → [ADMIN] liste invitations
POST /api/generate-invite        → [ADMIN] créer invitation
POST /api/delete-invite          → [ADMIN] supprimer invitation
GET  /api/health                 → "OK"
WS   /ws/*                       → WebRTC signaling
GET  /files/*                    → fichiers uploadés
```

---

## 🐳 Docker & CI

**Image finale** : `gcr.io/distroless/cc-debian12:nonroot`  
**Multi-arch** : linux/amd64 + linux/arm64  
**Taille cible** : ~5-10 MB  

**Flux CI** (`ci-new2.yml`) :
```
push/PR → develop ou main
  ↓
fmt (cargo fmt auto-commit)
  ↓
backend (x86_64 + aarch64 en parallèle)  ←→  frontend (npm build)
  ↓                                                 ↓
  └──────────── docker (assemble + push ghcr.io) ───┘
```

**Middlewares Axum** : `base_inject_middleware` (injecte `<base href="..."/>` dans index.html) + `CompressionLayer`

---

## 📋 Commandes de développement

```bash
# Backend
cd backend
cargo check                                    # vérif rapide sans compiler
cargo build / cargo build --release
cargo clippy -- -D warnings                    # lint strict (requis en CI)
cargo fmt                                      # auto-format
SQLX_OFFLINE=true cargo build                  # build CI sans DB live
DATABASE_URL=sqlite:./dev.db cargo sqlx prepare # régénérer .sqlx/queries.json

# Frontend
cd frontend
npm ci                    # install strict depuis package-lock.json
npm run dev               # dev server sur :5173 (proxy → :3000)
npm run build             # prod build → build/
npm run check             # svelte-check TypeScript
npm run lint              # ESLint

# Docker local
docker compose up -d
docker compose logs -f
```

---

## 🔄 Workflow de collaboration Claude ↔ MX10-AC2N

1. **Avant tout** : lire les fichiers concernés (raw GitHub ou ZIP fourni)
2. **Fournir** : contenu **complet** de chaque fichier modifié (jamais de diffs partiels)
3. **Signaler** : bugs connexes détectés + opportunités d'amélioration architecture
4. **Demander** : toute info manquante avant de coder
5. **Documenter** : mettre à jour LEARNING.md après chaque session
