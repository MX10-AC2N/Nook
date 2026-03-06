# 🐛 BUGS.md — Suivi des bugs Nook

> Mis à jour : **2026-03-05** (session 21)

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
**Status** : 🟡 Non bloquant CI (build réussit malgré ce warning latent)

---

### Bug #2 — Exports manquants dans authStore
**Fichier** : `frontend/src/lib/authStore.svelte.js`  
**Cause** : refactorisé en classe mais exports attendus absents.  
**Status** : 🟢 Résolu — authStore expose tous les champs nécessaires via la classe AuthStore

---

### Bug #3 — `connectionError.set()` cassé
**Status** : 🟡 Non bloquant pour les tests E2E actuels

---

### Bug #4 — `sodiumLoading`/`sodiumError` dans layout
**Status** : 🟢 Résolu dans layout actuel (utilise sodiumState.error directement)

---

### Bug #5 — Incohérence nom table SQL
**Status** : 🟢 Résolu — migration `001_initial.sql` et `db.rs` utilisent tous les deux `conversation_participants`

---

## ✅ BUGS RÉSOLUS (session 21 — 2026-03-05)

### [R21] Tests E2E — 31/43 en timeout sur `waiting for locator('#username')`

**Session** : 21  
**Fichiers modifiés** : `frontend/tests/e2e.spec.ts`, `frontend/playwright.config.ts`, `.github/workflows/test-nook.yml`

**Symptôme CI** :
```
Error: page.fill: Test timeout of 30000ms exceeded.
Call log:
  - waiting for locator('#username')
```
31 tests sur 43 échouaient avec ce timeout. Seuls les tests `request` (API purs) passaient.

**Cause racine** :
- `playwright.config.ts` avait `fullyParallel: true` avec `workers: 1` en CI
- En mode `fullyParallel: true`, tous les tests du fichier partagent le **même browser context** (et donc le même `localStorage`)
- `AuthStore` constructeur lit `localStorage` synchroniquement : `nook_user` + `nook_session_id`
- Le 1er test qui fait `loginAs()` → login réussi → `localStorage` pollué avec session active
- Tous les tests suivants : goto('/login') → `AuthStore` constructeur trouve localStorage plein → `isAuthenticated = true` → `$effect()` de `login/+page.svelte` redirige instantanément vers `/chat` → les inputs `#username`/`#password` existent une fraction de seconde puis la page est démontée → Playwright timeout

**Pourquoi les 12 tests passaient** : tous des tests `request` (API purs, pas de browser) ou `loginAsAdmin` (API-first, jamais de page `/login`).

**Fix 1 — `playwright.config.ts`** :
```typescript
// ❌ fullyParallel: true  → partage browser context entre tests
// ✅ fullyParallel: false → chaque test peut avoir son propre contexte
fullyParallel: false,
```

**Fix 2 — `e2e.spec.ts`** : ajout de `clearSession()` appelé en début de chaque test utilisant `loginAs()` :
```typescript
async function clearSession(page: Page) {
  // Étape 1 : navigation vers l'app pour accéder à son localStorage
  await page.goto('/', { waitUntil: 'domcontentloaded', timeout: 10_000 });
  // Étape 2 : vider le localStorage de l'app (même origine)
  await page.evaluate(() => {
    localStorage.removeItem('nook_user');
    localStorage.removeItem('nook_session_id');
    localStorage.removeItem('nook_token');
  });
  // Étape 3 : révoquer le cookie auth_token côté browser
  await page.context().clearCookies();
}
```
`loginAs()` appelle désormais `clearSession()` en premier.

**Fix 3 — `test-nook.yml`** : correction de l'URL healthcheck :
```bash
# ❌ /health → retourne index.html (ServeDir fallback → toujours 200)
until curl -sf http://localhost:6300/health; do sleep 3; done
# ✅ /api/health → retourne "OK" depuis le handler Axum
until curl -sf http://localhost:6300/api/health | grep -q "OK"; do sleep 3; done
```

**Effets de bord** : `clearSession()` ajoute ~2-3 secondes par test (goto + evaluate). Sur 43 tests, impact acceptable (~2min ajoutées). Les tests `request` (pas de page) ne sont pas affectés.

---

## ✅ BUGS RÉSOLUS (session 20)

### Race condition matrix Backend.yml
**Problème** : matrix amd64/arm64 génère deux jobs parallèles qui commitent simultanément → conflit git.
**Solution** : fichiers séparés `BACKEND-BUILD-REPORT-amd64.md` et `BACKEND-BUILD-REPORT-arm64.md`.

---

## ✅ BUGS RÉSOLUS (session 19 — 2026-02-28)

### [R20] Admin UI E2E — tests 2-5 toujours en échec malgré addInitScript()

**Sessions** : 19 (fix définitif)
**Cause racine finale** : `addInitScript()` sur un objet `Page` Playwright s'exécute
dans le contexte `about:blank` — origine différente de `localhost:6300`.
**Fix définitif** : `page.request.post('/api/auth/login')` — bypass total du browser.

---

## ✅ BUGS RÉSOLUS (sessions 15-18 — 2026-02-28)

### [R15] e2e_ci absent de conversation_participants → GET /api/conversations retourne []
### [R16] Logout button introuvable en E2E
### [R17] Chess page — strict mode violation sur h1
### [R18] Admin UI inaccessible : #username disabled (localStorage persistant entre tests)
### [R19] git push TEST_REPORT.md rejeté (non-fast-forward)

---

## ✅ BUGS RÉSOLUS (sessions 8-13)

### [R11] `crypto.randomUUID is not a function` en HTTP LAN
### [R12] CORS bloque LAN + WAN simultanément
### [R13] Cookie `SameSite=Lax` bloque WAN via Nginx Proxy Manager
### [R14] Prune supprime `default_global` au démarrage → POST /messages 404

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
```

---

## ✅ BUGS RÉSOLUS (session 22 — 2026-03-05)

### [R22] clearSession() goto('/') déclenche authStore.init() avec cookie valide

**Session** : 22  
**Fichier** : `frontend/tests/e2e.spec.ts`

**Symptôme** : 31/43 tests en timeout sur `waiting for locator('#username')` — identique à session 21 malgré les corrections apportées.

**Cause racine** : `clearSession()` implémentée en session 21 faisait `goto('/')` en PREMIER.
- `goto('/')` monte le layout → `onMount` → `waitForSodium()` + `initCryptoSystem()` + `authStore.init()`
- `authStore.init()` fait `fetch('/api/auth/me')` avec le cookie encore présent (clearCookies() pas encore appelé)
- `/api/auth/me` → 200 (token encore valide en DB) → `isAuthenticated=true`
- `$effect()` redirige vers `/chat`
- `clearCookies()` appelé APRÈS ne sert plus à rien
- `goto('/login')` suivant → `isAuthenticated=true` déjà → redirect → `#username` inaccessible

**Fix définitif** : `clearSession()` sans navigation browser préalable :
```typescript
async function clearSession(page: Page) {
  // 1. Révoquer le token côté serveur (page.request = API sans browser)
  try { await page.request.post(`${BASE}/auth/logout`); } catch {}
  // 2. Vider les cookies du browser context
  await page.context().clearCookies();
}
```
Ensuite `loginAs` fait `goto('/login')` → layout monte → `authStore.init()` → `/api/auth/me` → 401 (token NULL en DB + cookie absent) → `authStore.logout()` → `isAuthenticated=false` + localStorage vidé → `$effect()` ne redirige PAS → `#username` interactif ✅

**Pourquoi ça marche sans localStorage.clear() explicite** : `authStore.logout()` est appelé automatiquement quand `/api/auth/me` retourne 401, et cette méthode fait `localStorage.removeItem(...)` elle-même.

**Chronologie des tentatives** :
| Session | Approche | Résultat |
|---------|----------|---------|
| 17 | `clearCookies()` seul | ❌ localStorage intact → isAuthenticated=true |
| 18 | `goto('about:blank') + localStorage.clear()` | ❌ about:blank ≠ localhost:6300, localStorage isolé |
| 18 | `addInitScript()` (sur Page) | ❌ s'exécute sur about:blank, pas sur localhost:6300 |
| 19 | `loginAsAdmin` API-first | ✅ (admin seulement) |
| 21 | `goto('/') + evaluate(localStorage) + clearCookies` | ❌ goto('/') déclenche init() avec cookie valide |
| **22** | **`page.request.post('/api/auth/logout') + clearCookies()`** | **✅ token révoqué avant toute navigation** |

---

## ✅ Bug #23 — loginAs() fill('#username') avant que le layout finisse de charger

**Session** : 23
**Fichier** : `frontend/tests/e2e.spec.ts` — fonction `loginAs()`
**Impact** : 31/43 tests en échec depuis la session 1 (tous les tests utilisant `loginAs`)

**Symptôme** : `page.fill: Test timeout of 30000ms exceeded. waiting for locator('#username')`
Persistait malgré les corrections des sessions 21 et 22 (fullyParallel, clearSession).

**Cause racine** :
Le layout Svelte (`+layout.svelte`) démarre avec `loading = $state(true)` et ne passe
à `loading=false` qu'à la fin de `onMount` :
```
onMount → waitForSodium() → initCryptoSystem() → authStore.init() → loading = false
```
Tant que `loading=true`, le bloc `{#if loading}` affiche l'écran de chargement et
`{@render children()}` **n'est pas dans le DOM** — la page `/login` et son `#username`
sont donc absents.

`page.goto('/login')` se resolve à l'événement `load` (HTML + scripts reçus par le
browser), qui arrive AVANT que `onMount` ait terminé ses opérations async.
→ `page.fill('#username')` cherche un élément inexistant → timeout.

**Pourquoi les 12 tests passaient** : tous des tests `request` (API purs, pas de browser).
Les tests `loginAsAdmin` échouaient aussi mais pour la même raison — ils réussissaient
leur `goto('/admin')` car le contenu attendu (`admin-header`) était vérifié avec
`toBeVisible({timeout:8000})` qui attend implicitement.

**Fix** :
```typescript
async function loginAs(page: Page, username: string, password: string) {
  await clearSession(page);
  await page.goto('/login');
  // Attendre que le layout finisse de charger (loading=false → slot rendu → #username visible)
  await page.locator('#username').waitFor({ state: 'visible', timeout: 20_000 });
  await page.fill('#username', username);
  // ...
}
```

**Tableau chronologique des corrections E2E (sessions 17-23)** :

| Session | Problème | Fix |
|---------|----------|-----|
| 17 | Cookie persistant entre tests | `clearCookies()` — insuffisant |
| 18 | localStorage persistant | `goto(about:blank)` — origine isolée |
| 19 | Admin needs_password_change | `loginAsAdmin()` API-first ✅ |
| 21 | `fullyParallel:true` + shared context | `fullyParallel:false` ✅ |
| 22 | `clearSession goto('/')` → authStore.init() avec cookie | `request.post(logout)` ✅ |
| **23** | **`fill()` avant que layout finisse onMount** | **`waitFor('#username', visible)`** ✅ |
