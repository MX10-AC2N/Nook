# 🐛 BUGS.md — Suivi des bugs Nook

> Mis à jour : **2026-02-24** (session 8)

---

## 🔴 BUGS ACTIFS (Svelte 5 Frontend)

### Bug #1 — `state_invalid_export` dans conversationStore
**Fichier** : `frontend/src/lib/conversationStore.svelte.ts`  
**Erreur CI** :
```
[vite-plugin-svelte:compile-module] Cannot export state from a module if it is reassigned.
```
**Cause** : variables `$state` exportées et réassignées directement.  
**Fix** :
```typescript
// ❌ export let conversations = $state<Conversation[]>([]);
// ✅
export const conversationStore = $state<ConversationState>({
  conversations: [], activeConversationId: null, participants: [], availableUsers: []
});
// puis : conversationStore.conversations = newData;
```
**Status** : 🔴 Non résolu — bloquant CI frontend

---

### Bug #2 — Exports manquants dans authStore
**Fichier** : `frontend/src/lib/authStore.svelte.js`  
**Cause** : refactorisé en classe mais exports attendus absents.  
**Exports manquants** : `authUser`, `isAuthenticated`, `isAdmin`, `needsPasswordChange`, `authLoading`, `initAuth()`, `setAuthenticated()`  
**Fichiers impactés** : layout, login, chat, calendar, call, admin, change-password, conversationStore, chatStore, webrtc-calls, crypto, mediaStore  
**Status** : 🔴 Non résolu

---

### Bug #3 — `connectionError.set()` cassé
**Cause** : `connectionError` n'est plus un writable store Svelte 4, c'est un champ de `chatStore`.  
**Fix** :
```typescript
// ❌ import { connectionError } from './chatStore.svelte.ts';
// ✅ import { setConnectionError } from './chatStore.svelte.ts';
```
**Fichiers** : conversationStore, mediaStore, MediaPlayer, MediaRecorder  
**Status** : 🔴 Non résolu

---

### Bug #4 — `sodiumLoading`/`sodiumError` dans layout
**Cause** : layout utilise syntaxe Svelte 4 (`$sodiumError`, `.subscribe()`) mais `sodium.svelte.js` exporte `sodiumState` (Svelte 5).  
**Fix** :
```svelte
import { sodiumState, waitForSodium } from '$lib/sodium.svelte.js';
await waitForSodium();
sodiumState.error // au lieu de get(sodiumError)
```
**Status** : 🟢 Résolu dans layout actuel (utilise sodiumState.error directement)

---

### Bug #5 — Incohérence nom table SQL
**Status** : 🟢 Résolu — migration `001_initial.sql` et `db.rs` utilisent tous les deux `conversation_participants`

---

## ✅ BUGS RÉSOLUS (session 8 — 2026-02-24)

### [R11] `crypto.randomUUID is not a function` en HTTP LAN
**Session** : 8  
**Cause** : `crypto.randomUUID()` n'existe qu'en contexte sécurisé (HTTPS).
En HTTP LAN, l'appel crash.  
**Cause profonde** : le frontend essayait de gérer le token de session côté client,
alors que le token vit dans le cookie HttpOnly du backend.  
**Fix** :
- `authStore.svelte.js` : suppression de `crypto.randomUUID()` et du champ `token`.
  Remplacement par `sessionId = String(Date.now())` (compatible HTTP + HTTPS).
- `authStore.login(user, token)` → `authStore.login(user)` (signature simplifiée).
- `login/+page.svelte` : suppression de `crypto.randomUUID()` côté login.
**Fichiers modifiés** : `authStore.svelte.js`, `routes/login/+page.svelte`

---

### [R12] CORS bloque LAN + WAN simultanément
**Session** : 8  
**Cause** : origines CORS codées en dur dans `main.rs` (localhost uniquement).  
**Fix** :
- `config.rs` : nouveau champ `allowed_origins` lu depuis `ALLOWED_ORIGINS` env var.
- `main.rs` : CORS construit dynamiquement depuis `config.allowed_origins`.
- `.env` : ajouter `ALLOWED_ORIGINS=http://192.168.x.x:6300,https://ton-domaine.com`
**Fichiers modifiés** : `config.rs`, `main.rs`, `.env.example`

---

### [R13] Cookie `SameSite=Lax` bloque WAN via Nginx Proxy Manager
**Session** : 8  
**Cause** : `SameSite=Lax` fonctionne en LAN HTTP mais bloque les contextes
cross-origin HTTPS (WAN avec reverse proxy).  
**Fix** : `auth.rs` — `build_set_cookie()` détecte `X-Forwarded-Proto: https`
(injecté par Nginx) et utilise `SameSite=None; Secure` en HTTPS,
`SameSite=Lax` en HTTP.  
**Fichiers modifiés** : `auth.rs`

---

## 📋 Règles Svelte 5 (pour éviter les bugs #1-4)

```typescript
// ✅ Pattern store correct
export const monStore = $state<MonState>(createInitialState());
export function setData(data: string[]): void { monStore.data = data; }
export function getData(): string[] { return monStore.data; }
export function reset(): void { Object.assign(monStore, createInitialState()); }

// ❌ JAMAIS — export + réassignation directe
export let items = $state<Item[]>([]);
items = newItems;  // → state_invalid_export

// ❌ JAMAIS — writable/readable Svelte 4
import { writable } from 'svelte/store';

// ❌ JAMAIS — $derived/$effect en dehors des composants .svelte
export const active = $derived(() => ...);  // interdit en .svelte.ts
```

**Stores conformes** : `chatStore`, `sodiumState`, `callStore`, `authStore` ✅  
**Stores à corriger** : `conversationStore` (Bug #1)

---

## 🌐 Architecture LAN ↔ WAN

```
LAN (HTTP) :
  Navigateur (192.168.x.x) → Port 6300 → Backend Axum
  Cookie : auth_token=...; SameSite=Lax
  CORS   : http://192.168.x.x:6300 dans ALLOWED_ORIGINS

WAN (HTTPS) :
  Navigateur → Nginx Proxy Manager (443) → Backend Axum (3000)
  Header injecté par Nginx : X-Forwarded-Proto: https
  Cookie : auth_token=...; SameSite=None; Secure
  CORS   : https://nook.mondomaine.com dans ALLOWED_ORIGINS

LAN ↔ WAN (WebRTC) :
  Signaling via /ws (WebSocket)
  ICE candidates échangés via le WebSocket → connexion P2P directe
  Si P2P impossible : TURN server requis (à implémenter)
```

---

## Bugs Session 11 — Template Literals + Chat

### 🔴 Template literals corrompus (RÉSOLU)
**Pattern** : `${expr}` → `\( {expr} \)` (corruption lors copier-coller entre sessions)
**Impact** : liens d'invitation affichés en clair, GIFs/uploads cassés dans le chat
**Fix** : remplacement byte-level dans admin/+page.svelte et chat/+page.svelte

### 🔴 sendMessage mauvaise URL (RÉSOLU)
**Avant** : `POST /api/messages`  
**Après** : `POST /api/conversations/${conversationId}/messages`

### 🔴 sendMessage mauvais payload (RÉSOLU)
**Avant** : payload chiffré `{content: number[], encrypted_keys, nonce, ...}`  
**Après** : `{content: string, encrypted: false}`  
**Note** : chiffrement E2E à réactiver quand clés par-utilisateur implémentées

### 🔴 loadMessages parsing incorrect (RÉSOLU)
**Avant** : `data.messages ?? []` → toujours vide (backend retourne tableau direct)  
**Après** : `Array.isArray(data) ? data : (data.messages ?? [])`

### 🔴 Conversation default_global manquante (RÉSOLU)
**Symptôme** : POST /api/conversations/default_global/messages → 500 (FK constraint)  
**Fix** : création dans `check_initial_admin()` au démarrage si absente

### ⚠️ Chiffrement E2E désactivé temporairement
Messages envoyés en clair (`encrypted: false`). À réactiver quand :
1. Clés Ed25519 générées et stockées par utilisateur à l'inscription
2. `recipientPublicKeys` disponibles dans la page chat
3. `encryptForRecipients` testé avec des vraies clés
