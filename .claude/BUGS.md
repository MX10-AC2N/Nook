# 🐛 BUGS.md — Suivi des bugs Nook

> Mis à jour : **2026-02-27** (session 13)

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

## ✅ BUGS RÉSOLUS (session 13 — 2026-02-27)

### [R14] Prune supprime `default_global` au démarrage → POST /messages 404
**Session** : 13  
**Fichier** : `backend/src/prune.rs`  
**Symptôme CI** : Test E2E `Login → Chat → Envoi message` échoue systématiquement — `POST /api/conversations/default_global/messages` retourne 404 (3 tentatives, toutes 404).  
**Cause racine** :
- `default_global` est créée au boot (vide, 0 messages)
- Le job prune se déclenche **10 secondes** après le démarrage
- La requête DELETE supprimait **toutes** les conversations sans messages, y compris les groupes
- `default_global` était donc détruite avant que le test e2e puisse l'utiliser
**Fix** : Ajout de `AND is_group = 0` dans la clause DELETE conversations vides.
Les groupes (`is_group = 1`) sont créés intentionnellement par des admins et ne doivent jamais être supprimés automatiquement, même s'ils sont temporairement vides.
**Fichier modifié** : `backend/src/prune.rs`  
**Diff clé** :
```sql
-- AVANT (bug) :
DELETE FROM conversations
WHERE NOT EXISTS (
    SELECT 1 FROM messages WHERE messages.conversation_id = conversations.id
)

-- APRÈS (fix) :
DELETE FROM conversations
WHERE is_group = 0
  AND NOT EXISTS (
    SELECT 1 FROM messages WHERE messages.conversation_id = conversations.id
  )
```

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

---

## Bugs Session 12 — Retour test manuel homeserver

### 🔴 data.token undefined dans generateInvite (RÉSOLU)
Backend `/api/invites` (POST) retourne `{success, message, invite_link: "/invite?token=UUID"}`.
Frontend lisait `data.token` (undefined). Fix: extraction depuis `data.invite_link`.

### 🔴 Thème non fonctionnel (RÉSOLU)
CSS: `.theme-jardin-secret { ... }` (sélecteur de classe)
Code: `documentElement.setAttribute('data-theme', 'jardin-secret')` (attribut HTML)
Ces deux mécanismes ne se correspondent pas. Fix: `body.classList.add('theme-X')`.

### 🔴 Route /api/user/update manquante (RÉSOLU)
Ajouté dans `db.rs` + `main.rs`. Update du champ `name` (et `email` optionnel).

### 🔴 Routes /api/events manquantes (RÉSOLU)
Table `events` ajoutée en migration. GET/POST/DELETE implémentés dans `db.rs`.

### 🟡 sendGif mauvaise URL (RÉSOLU)
`/api/messages` → `/api/conversations/${conversationId}/messages`

### 🟡 Menu incomplet (RÉSOLU)
Chess et Polls ajoutés dans navItems du layout.

### 🟡 Chess pas de rafraîchissement temps réel (EN COURS)
Le WS backend broadcast bien les coups via `webrtc_state.broadcasts`.
Le frontend chess `onmessage` reçoit les events mais ne recharge pas le plateau.
À investiguer dans `chessStore.svelte.ts` → handler onmessage.

### ⚠️ Upload FK constraint (DÉPEND SESSION 11)
Nécessite que la conversation `default_global` existe.
Fix dans `main.rs::check_initial_admin()` fait en session 11 — vérifier déployé.

---

## ✅ BUGS RÉSOLUS (sessions 15-18 — 2026-02-28)

### [R15] e2e_ci absent de conversation_participants → GET /api/conversations retourne []
**Session** : 15
**Cause** : La table `conversation_participants` n'était jamais alimentée automatiquement.
La query conversations utilise INNER JOIN → si pas de ligne participant → résultat vide.
**Fix** :
- `main.rs::check_initial_admin()` : INSERT OR IGNORE tous les users approuvés dans default_global au boot
- `admin.rs::approve_user()` : INSERT OR IGNORE dans default_global lors de chaque approbation
**Idempotent** : INSERT OR IGNORE → sûr pour redémarrages et re-runs CI.

### [R16] Logout button introuvable en E2E (strict mode violation)
**Session** : 15
**Cause** : sélecteur `getByRole('button', { name: /déconnect/i })` ne matchait pas le bouton header
qui n'a que l'icône 🔌 (pas de texte visible).
**Fix** : `button[aria-label="Déconnexion"]` — cible l'attribut aria explicite.

### [R17] Chess page — strict mode violation sur h1
**Session** : 15
**Cause** : `locator('.btn-create, h1')` résolvait 3 éléments : h1 layout "🌱 Nook" + h1 chess "Échecs" + button.btn-create
→ Playwright strict mode refuse toBeVisible() sur multi-match.
**Fix** : `locator('.btn-create')` seul.

### [R18] Admin UI inaccessible : #username disabled (localStorage persistant entre tests)
**Sessions** : 16-18 (3 tentatives)
**Cause racine** : `AuthStore` classe Svelte 5 — son constructeur lit `localStorage` **synchroniquement**
lors du chargement du module ES6. En contexte Playwright (workers:1, même browser context),
le `localStorage` de `localhost:6300` persiste entre les tests → `isAuthenticated=true`
→ `$effect()` du layout redirige /login avant que les inputs soient interactifs.
**Tentatives échouées** :
1. `clearCookies()` seul → localStorage intact, cookie ≠ localStorage
2. `goto('about:blank') + page.evaluate(localStorage.clear)` → about:blank a une origine différente, localStorage isolé
**Fix final** : `page.addInitScript()` — seul hook Playwright qui s'exécute avant les modules ES6 :
```typescript
await page.context().clearCookies();
await page.addInitScript(() => {
  localStorage.removeItem('nook_user');
  localStorage.removeItem('nook_session_id');
  localStorage.removeItem('nook_token');
});
await page.goto('/login');
```

### [R19] git push TEST_REPORT.md rejeté (non-fast-forward)
**Session** : 17
**Cause** : le workflow CI commitait le rapport sans synchroniser avec la branche distante → fast-forward impossible si la branche avait avancé entre le checkout et le push.
**Fix** : `test-nook.yml` — `git pull --rebase origin $ref` avant `git push`.

---

## ✅ BUGS RÉSOLUS (session 19 — 2026-02-28)

### [R20] Admin UI E2E — tests 2-5 toujours en échec malgré addInitScript()

**Sessions** : 19 (fix définitif après 4 tentatives échouées sessions 16-18)

**Cause racine finale** : `addInitScript()` sur un objet `Page` Playwright s'exécute
dans le contexte de la page à sa création — soit `about:blank`. L'origine `about:blank`
est différente de `localhost:6300` → le localStorage de l'app n'est jamais atteint.

**Historique complet des tentatives :**
1. `clearCookies()` seul → localStorage intact → `isAuthenticated=true`
2. `goto('about:blank') + localStorage.clear()` → about:blank = origine isolée
3. `addInitScript()` → s'exécute sur about:blank, pas sur localhost:6300

**Fix définitif** : `page.request.post('/api/auth/login')` — bypass total du browser.
`page.request` partage le cookie store du browser context → le cookie `auth_token`
est posé sans jamais charger `/login` → `page.goto('/admin')` directement.
Le localStorage et le `$effect()` de redirection ne sont plus jamais impliqués.

---

## 🔧 Notes architecture CI (session 20)

### Race condition matrix Backend.yml
**Problème détecté** : la matrix amd64/arm64 génère deux jobs parallèles qui commitent
simultanément sur le même fichier → conflit git → un rapport est systématiquement perdu.
**Solution** : fichiers séparés `BACKEND-BUILD-REPORT-amd64.md` et `BACKEND-BUILD-REPORT-arm64.md`.

### Heredoc indenté dans les workflows GitHub Actions
**Problème** : `cat > file << ENDOFMD` avec indentation YAML produit des lignes avec
des espaces en début → Markdown invalide (listes cassées, code blocks mal détectés).
**Solution** : heredoc toujours au niveau colonne 0, même si le shell run est indenté
dans le YAML (GitHub Actions ignore l'indentation du heredoc lui-même).
