# 🐛 BUGS.md — Suivi des bugs Nook

> Mis à jour : **2026-02-23** (session 7)

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
**Status** : 🔴 Non résolu

---

### Bug #5 — Incohérence nom table SQL
**Cause** : `001_initial.sql` crée `conversation_members` mais `db.rs` utilise `conversation_participants`.  
**Fix** : corriger `db.rs` pour utiliser `conversation_members` (évite migration destructive).  
**Status** : 🟡 Non bloquant CI, crash runtime uniquement

---

## ✅ BUGS RÉSOLUS

### [R1] Diamond dependency rand_core 0.6/0.9
**Session** : 2 (2026-02-21)  
**Erreur** : `the trait CryptoRngCore is not implemented for &mut rand::rngs::OsRng`  
**Fix** :
```toml
rand = { version = "0.9", features = ["std", "std_rng", "os_rng"] }
rand_core = { version = "0.6", features = ["std", "getrandom"] }
```
```rust
use rand_core::OsRng;  // auth.rs — rand_core 0.6
rand::rng().fill_bytes(&mut buf);  // webrtc.rs — rand 0.9
```

---

### [R2] axum 0.7 → 0.8 breaking changes
**Session** : 2  
- `Host` supprimé → extraire depuis `HeaderMap`
- `Message::Text(String)` → `Message::Text(msg.into())`
- Routes `:param` → `{param}`
- CORS wildcard + credentials → lists explicites → CORS panic au démarrage

---

### [R3] Cargo.lock désynchronisé
**Session** : 3  
**Fix** : `rm Cargo.lock && cargo update && git commit`

---

### [R4] `home@0.5.12 requires rustc 1.88`
**Session** : 3  
**Fix** : `FROM rust:1.88-bookworm`

---

### [R5] proc-macro async-trait/displaydoc en Docker
**Session** : 3-4  
**Cause racine** : `.cargo/config.toml` copié dans Docker → Cargo détecte linker externe → mode cross-compilation → proc-macros incompatibles  
**Fix** : COPY explicite dans Dockerfile qui exclut `.cargo/`

---

### [R6] SQLite Permission denied (code 14) — distroless
**Session** : 5-6  
**Cause** : volumes Docker créés root + user nonroot 65532 + `SqlitePool::connect()` sans `create_if_missing`  
**Fix** :
1. `SqliteConnectOptions::create_if_missing(true)` dans `init_db()`
2. Init container `alpine:3` qui `chown -R 65532:65532 /app/data`
3. Named volumes dans `docker-compose.ci.yml`

---

### [R7] CORS panic au démarrage
**Session** : 6  
**Erreur** : `Cannot combine Access-Control-Allow-Credentials: true with Access-Control-Allow-Headers: *`  
**Fix** : lister origines, méthodes et headers explicitement dans `CorsLayer`

---

### [R8] Playwright — `reuseExistingServer` inversé
**Session** : 6  
**Erreur** : `http://localhost:6300 is already used`  
**Fix** : `reuseExistingServer: !!process.env.CI` (était `!process.env.CI`)

---

### [R9] Playwright — login admin 401 en CI
**Session** : 7  
**Cause** : extraction cookie curl complexe et fragile ; admin `needs_password_change=1` → flows UI bloqués  
**Fix** : `E2E_SETUP=1` env var → `check_initial_admin` crée `e2e_ci` (approved=1, sans changement mdp) → Playwright se connecte directement sans passer par l'admin

---

### [R10] Docker.yml — artifact cross-workflow introuvable
**Session** : 7  
**Cause** : `actions/download-artifact@v4` cherche uniquement dans le workflow courant  
**Fix** : `dawidd6/action-download-artifact@v6` avec `workflow: Backend.yml` + `branch:`

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

**Stores conformes** : `chatStore`, `sodiumState`, `callStore`  
**Stores à corriger** : `conversationStore` (Bug #1), `authStore` (Bug #2)
