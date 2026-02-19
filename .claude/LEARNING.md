# 📚 LEARNING.md — Mémoire technique du projet Nook

> Fichier vivant maintenu par Claude. Contient bugs résolus, décisions architecturales,
> patterns validés et tout ce qui évite de réapprendre les mêmes choses.  
> **Mise à jour à chaque session.**

---

## 🐛 BUGS ACTIFS (à résoudre)

### Bug #1 — BUILD FRONTEND CASSÉ ❌ (PRIORITÉ HAUTE)

**Erreur CI** :
```
[vite-plugin-svelte:compile-module] src/lib/conversationStore.svelte.ts:4:0
Cannot export state from a module if it is reassigned.
Either export a function returning the state value or only mutate the state value's properties
https://svelte.dev/e/state_invalid_export
```

**Cause** : `conversationStore.svelte.ts` exporte des variables `$state` qui sont ensuite réassignées.
```typescript
// LIGNE 4-7 — CASSÉ
export let conversations = $state<Conversation[]>([]);
export let activeConversationId = $state<string | null>(null);
export let participants = $state<Participant[]>([]);
export let availableUsers = $state<Participant[]>([]);
// Plus loin : conversations = [...]; activeConversationId = id; → ERREUR
```

**Fix** : Convertir en objet `$state` unique (même pattern que `chatStore.svelte.ts`).
```typescript
// CORRECT
export interface ConversationState {
  conversations: Conversation[];
  activeConversationId: string | null;
  participants: Participant[];
  availableUsers: Participant[];
}
export const conversationStore = $state<ConversationState>({
  conversations: [], activeConversationId: null, participants: [], availableUsers: []
});
// Puis partout : conversationStore.conversations = [...] au lieu de conversations = [...]
```

**Fichier concerné** : `frontend/src/lib/conversationStore.svelte.ts`  
**Status** : 🔴 Non résolu — bloque le build et donc le CI/CD complet

---

### Bug #2 — IMPORTS CASSÉS dans authStore ❌

**Cause** : `authStore.svelte.js` a été refactorisé en classe mais n'exporte plus les noms attendus par le reste du projet.

**Fichiers qui importent des noms inexistants** :

| Fichier | Imports cassés |
|---------|---------------|
| `src/routes/+layout.svelte` | `isAuthenticated`, `isAdmin`, `needsPasswordChange`, `initAuth` |
| `src/routes/+page.svelte` | `isAuthenticated`, `isAdmin`, `authLoading` |
| `src/routes/login/+page.svelte` | `isAuthenticated`, `needsPasswordChange`, `setAuthenticated` |
| `src/routes/chat/+page.svelte` | `isAuthenticated`, `authUser` |
| `src/routes/calendar/+page.svelte` | `isAuthenticated` |
| `src/routes/call/+page.svelte` | `isAuthenticated` |
| `src/routes/admin/+page.svelte` | `isAdmin` |
| `src/routes/change-password/+page.svelte` | `needsPasswordChange` |
| `src/lib/conversationStore.svelte.ts` | `authUser`, `isAuthenticated` |
| `src/lib/chatStore.svelte.ts` | `authUser`, `isAuthenticated` |
| `src/lib/webrtc-calls.svelte.ts` | `authUser` |
| `src/lib/crypto.ts` | `authUser` |
| `src/lib/mediaStore.svelte.js` | `authUser` |

**Fix** : Ajouter les exports de compatibilité dans `authStore.svelte.js` :
```javascript
// Ajouter à la fin de authStore.svelte.js

// Compatibilité avec les anciens imports
export const authUser = $derived(() => authStore.user);
export const isAuthenticated = $derived(() => authStore.isAuthenticated);
export const isAdmin = $derived(() => authStore.user?.role === 'admin');
export const needsPasswordChange = $derived(() => 
  authStore.user?.needs_password_change ?? false
);
export let authLoading = $state(true);

export function setAuthenticated(user, token) {
  authStore.login(user, token);
}

export async function initAuth() {
  authLoading = true;
  try {
    const resp = await fetch('/api/auth/me', { credentials: 'include' });
    const data = await resp.json();
    if (data.authenticated && data.user) {
      authStore.user = data.user;
    }
  } catch (e) {
    console.error('initAuth error:', e);
  } finally {
    authLoading = false;
  }
}
```

⚠️ **Attention** : `$derived` dans un module `.svelte.js` — à tester. Si non supporté, utiliser des fonctions getter à la place.

**Status** : 🔴 Non résolu

---

### Bug #3 — `connectionError.set()` cassé ❌

**Cause** : `connectionError` n'est plus un writable Svelte 4 store, c'est maintenant un champ dans l'objet `chatStore`.

**Fichiers cassés** :

| Fichier | Pattern cassé | Fix |
|---------|--------------|-----|
| `src/lib/conversationStore.svelte.ts` | `connectionError.set(null)` | `setConnectionError(null)` |
| `src/lib/mediaStore.svelte.js` | `connectionError.set('...')` | `setConnectionError('...')` |
| `src/components/MediaPlayer.svelte` | `connectionError.set('...')` | `setConnectionError('...')` |
| `src/components/MediaRecorder.svelte` | `connectionError.set('...')` | `setConnectionError('...')` |

**Import à changer** :
```typescript
// ❌ AVANT
import { connectionError } from './chatStore.svelte.ts';
connectionError.set('Erreur...');

// ✅ APRÈS
import { setConnectionError } from './chatStore.svelte.ts';
setConnectionError('Erreur...');
```

**Status** : 🔴 Non résolu

---

### Bug #4 — `sodiumLoading` / `sodiumError` cassés dans layout ❌

**Cause** : Le layout utilise `sodiumLoading.subscribe(...)` et `$sodiumError` (syntaxe store Svelte 4) mais `sodium.svelte.js` n'exporte que `sodiumState` (objet `$state`).

**Fix dans `+layout.svelte`** :
```svelte
// ❌ AVANT
import { sodiumLoading, sodiumError } from '$lib/sodium.svelte.js';
// ... sodiumLoading.subscribe($loading => { ... })
// ... get(sodiumError)
// ... {$sodiumError.message}

// ✅ APRÈS
import { sodiumState, waitForSodium } from '$lib/sodium.svelte.js';
// Attendre directement
await waitForSodium();
// Lire l'état
sodiumState.error // au lieu de get(sodiumError)
```

**Status** : 🔴 Non résolu

---

### Bug #5 — Incohérence schéma SQL ⚠️

**Cause** : `001_initial.sql` crée `conversation_members` mais `db.rs` utilise `conversation_participants`.

```sql
-- 001_initial.sql crée :
CREATE TABLE conversation_members (...)

-- db.rs utilise :
"INSERT INTO conversation_participants ..."  -- ← table inexistante !
```

**Fix** : Soit renommer la table dans la migration, soit corriger `db.rs`.
Recommandation : corriger `db.rs` pour utiliser `conversation_members` (évite migration destructive).

**Status** : 🟡 Non bloquant en CI (pas de test d'intégration DB) mais bug runtime critique

---

## ✅ DÉCISIONS ARCHITECTURALES

### Pattern stores Svelte 5 retenu pour Nook

**Validé** : objet `$state` unique par domaine fonctionnel, exposé via fonctions getter/setter.

```typescript
// Pattern à suivre partout
export const xxxStore = $state<XxxState>(createInitialState());
export function getXxx(): XxxType { return xxxStore.xxx; }
export function setXxx(val: XxxType): void { xxxStore.xxx = val; }
export function resetXxx(): void { Object.assign(xxxStore, createInitialState()); }
```

Stores existants qui suivent ce pattern : `chatStore`, `callStore`, `sodiumState`, `recordingState`  
Stores à corriger : `conversationStore` (bug #1), `authStore` (bug #2)

---

### Chiffrement E2EE — Architecture retenue

```
Côté client uniquement :
  1. Génération paire de clés Curve25519 (libsodium crypto_box_keypair)
  2. Clé privée chiffrée avec password (ChaCha20-Poly1305) → stockée IndexedDB
  3. Clé publique envoyée au backend (en clair)
  
Envoi d'un message :
  1. Génération clé de session symétrique aléatoire
  2. Chiffrement du message avec la clé de session (XSalsa20-Poly1305)
  3. Chiffrement de la clé de session avec la clé publique de chaque destinataire (crypto_box)
  4. Envoi : { content: encrypted_bytes, encrypted_keys: {userId: encrypted_session_key}, nonce }
  
Réception :
  1. Récupération clé de session chiffrée pour l'userId courant
  2. Déchiffrement clé de session avec clé privée
  3. Déchiffrement message avec clé de session
```

**Modules** : `src/lib/crypto.ts` (fonctions) + `src/lib/storage.ts` (IndexedDB) + `src/lib/sodium.svelte.js` (init)

---

### Auth : Cookie vs JWT

**Choix retenu** : Cookie HttpOnly `auth_token=userId:token`  
**Raison** : Simplicité, protection XSS automatique, pas de refresh token nécessaire (24h)  
**Token stocké** en DB dans `users.token` → révocable côté serveur (logout)  
**Changement de mdp** → nouveau token généré → cookie mis à jour → session continuée

---

### Docker : Image Distroless

**Choix** : `gcr.io/distroless/cc-debian12:nonroot`  
**Raison** : Surface d'attaque minimale (~5-10 MB), pas de shell, pas de package manager  
**Librairies copiées manuellement** : libsqlite3, libsodium, libssl, libcrypto  
**User** : `nonroot` (uid 65532), pas de root  
**Build multi-stage** : libs-extractor → app-prep → distroless

---

## 📝 SESSIONS DE TRAVAIL

### Session 1 — 2026-02-19

**Contexte** : Première analyse complète du projet. Build frontend cassé en CI.

**Analyse réalisée** :
- Lecture de tous les stores, routes, config, backend, CI
- Identification de 5 bugs actifs (voir section BUGS ACTIFS)
- Bug #1 est le seul bloquant le CI (erreur `state_invalid_export`)

**Fichiers analysés** :
- `frontend/src/lib/` : tous les stores `.svelte.ts` et `.svelte.js`
- `frontend/src/routes/+layout.svelte`
- `frontend/svelte.config.js`, `vite.config.js`, `tsconfig.json`, `package.json`
- `backend/src/main.rs`, `auth.rs`, `db.rs`, `Cargo.toml`
- `backend/migrations/001_initial.sql`
- `.github/workflows/ci-new2.yml`, `Dockerfile`

**Fichiers créés** : `CLAUDE.md`, `LEARNING.md`

**À faire prochaine session** :
1. Corriger `conversationStore.svelte.ts` (bug #1 — bloquant)
2. Compléter `authStore.svelte.js` avec les exports manquants (bug #2)
3. Corriger imports `connectionError` → `setConnectionError` (bug #3)
4. Corriger le layout pour `sodiumLoading`/`sodiumError` (bug #4)
5. Corriger incohérence nom table SQL (bug #5)

---

## 💡 OPPORTUNITÉS D'AMÉLIORATION

> Idées notées pour discussion future avec MX10-AC2N

### Frontend

- **`authStore.svelte.js` → `.svelte.ts`** : migrer en TypeScript pour avoir le typage complet
- **Service Worker** : actuellement désactivé (`register: false`) → à activer quand stable pour vrai PWA offline
- **`get()` de svelte/store dans layout** : présence de `import { get } from 'svelte/store'` dans le layout → code legacy à nettoyer
- **Gestion d'erreurs globale** : actuellement en try/catch éparpillé → centraliser dans un errorStore
- **`$derived` in modules** : vérifier si Svelte 5.46+ supporte `$derived` dans `.svelte.ts` (comportement changeant)

### Backend

- **Middleware d'auth manquant** : `Extension(user_id)` dans `db.rs` mais pas de middleware qui l'injecte dans `main.rs` → bug runtime probable sur les routes `/conversations`
- **Gestion d'erreurs typées** : `StatusCode::INTERNAL_SERVER_ERROR` partout → implémenter `thiserror` pour des erreurs plus informatives
- **Token storage** : token auth en clair en DB → à hasher (même Argon2 light ou SHA256)
- **Rate limiting** : `tower_governor` en dépendance mais pas encore branché dans le router
- **Pagination messages** : `/conversations/:id/messages` fait un LIMIT 50 DESC → les messages arrivent dans le mauvais ordre côté client

### Infrastructure

- **Health check** : `/api/health` existe → l'utiliser dans `docker-compose.yml` (`healthcheck:`)
- **Logs structurés** : `tracing` installé mais utilisé via `eprintln!()` partout → migrer vers `tracing::info!()`, `tracing::error!()`
- **SQLX_OFFLINE** : `.sqlx/queries.json` est commité → s'assurer qu'il est à jour à chaque changement de requête SQL

---

## 🔧 SNIPPETS DE RÉFÉRENCE

### Store Svelte 5 complet (module)
```typescript
// src/lib/exempleStore.svelte.ts
import { browser } from '$app/environment';

interface ExempleState {
  items: string[];
  loading: boolean;
  error: string | null;
}

function createInitialState(): ExempleState {
  return { items: [], loading: false, error: null };
}

export const exempleStore = $state<ExempleState>(createInitialState());

// Mutateurs (appelables depuis n'importe où)
export function setItems(items: string[]): void { exempleStore.items = items; }
export function setLoading(v: boolean): void { exempleStore.loading = v; }
export function setError(err: string | null): void { exempleStore.error = err; }
export function reset(): void { Object.assign(exempleStore, createInitialState()); }

// Getters
export function getItems(): string[] { return exempleStore.items; }

// Initialisation (à appeler depuis onMount dans un composant)
export async function initExempleStore(): Promise<void> {
  if (!browser) return;
  setLoading(true);
  try {
    const resp = await fetch('/api/exemples', { credentials: 'include' });
    const data = await resp.json();
    setItems(data.items ?? []);
    setError(null);
  } catch (err) {
    setError('Erreur de chargement');
  } finally {
    setLoading(false);
  }
}
```

### Composant Svelte 5 qui consomme un store module
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { exempleStore, initExempleStore } from '$lib/exempleStore.svelte.ts';
  
  // Props
  interface Props { title?: string; }
  let { title = 'Ma liste' }: Props = $props();
  
  // State local
  let selected = $state<string | null>(null);
  
  // Derived local depuis le store
  let hasItems = $derived(exempleStore.items.length > 0);
  
  onMount(async () => {
    await initExempleStore();
  });
</script>

{#if exempleStore.loading}
  <p>Chargement...</p>
{:else if exempleStore.error}
  <p class="error">{exempleStore.error}</p>
{:else}
  <h2>{title}</h2>
  {#each exempleStore.items as item}
    <button onclick={() => selected = item}>{item}</button>
  {/each}
{/if}
```

### Handler Axum avec extraction cookie auth
```rust
pub async fn mon_handler(
    State(state): State<Arc<SharedState>>,
    headers: HeaderMap,
    Json(payload): Json<MonPayload>,
) -> impl IntoResponse {
    // Récupérer user depuis cookie
    let user_id = match crate::auth::get_cookie(&headers, "auth_token") {
        Some(cookie) => {
            let parts: Vec<&str> = cookie.split(':').collect();
            if parts.len() == 2 { parts[0].to_string() } 
            else { return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid token"}))).into_response(); }
        }
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Not authenticated"}))).into_response(),
    };

    // ... logique métier ...
    Json(json!({"success": true})).into_response()
}
```

### Migration SQLx
```sql
-- backend/migrations/003_ma_migration.sql
ALTER TABLE messages ADD COLUMN reactions TEXT DEFAULT '{}';
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id);
```
```bash
# Après modification SQL, régénérer le cache :
DATABASE_URL=sqlite:./dev.db cargo sqlx prepare --bin nook-backend
```
