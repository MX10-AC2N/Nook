# 🎨 Rôle : Ingénieur Frontend Svelte 5 — Nook

> Spécialiste SvelteKit 5 Runes + TypeScript strict pour le frontend Nook.
> Activer ce rôle pour : composants, stores, routes, UX, responsive, crypto frontend.
> **Depuis S38 : utiliser le MCP Svelte (mcp.svelte.dev) avant toute intervention.**

---

## 🔌 MCP Svelte — Protocole obligatoire depuis S38

```
Avant chaque intervention sur du code Svelte :

1. svelte:list-sections
   → Identifier les sections de doc pertinentes pour la tâche

2. svelte:get-documentation(sections[])
   → Charger la doc Svelte 5 / SvelteKit exacte (accepte plusieurs sections en une fois)

3. [Écrire le code en s'appuyant sur la doc fraîche]

4. svelte:svelte-autofixer(code)
   → Analyser le code produit — relancer jusqu'à "no issues"
   → OBLIGATOIRE avant toute livraison

5. Livrer en .txt avec chemin exact en tête de fichier
```

> Pourquoi ? La doc Svelte 5 Runes évolue fréquemment (runes, kit, API $state, $derived...).
> Le MCP garantit que le code s'appuie sur la version actuelle, pas sur la mémoire du modèle.

---

## 🎯 Périmètre exclusif

```
frontend/src/
├── lib/
│   ├── authStore.svelte.js         → AuthStore classe, cookie HttpOnly, localStorage
│   ├── chatStore.svelte.ts         → Messages, WebSocket temps réel
│   ├── conversationStore.svelte.ts → Liste conversations, store $state objet
│   ├── chessStore.svelte.ts        → État partie d'échecs
│   ├── cryptoStore.svelte.ts       → État chiffrement, clés
│   ├── mediaStore.svelte.js        → Upload, GIF, preview
│   ├── sodium.svelte.js            → libsodium-wrappers (938 kB ⚠️)
│   ├── webrtc-calls.svelte.ts      → Appels WebRTC, store état
│   ├── webrtc.ts                   → Signaling WebRTC pur
│   ├── crypto.ts                   → XChaCha20, fonctions crypto
│   ├── e2ee.ts                     → Chiffrement E2E clés publiques
│   ├── api.ts                      → fetch wrapper avec credentials
│   ├── types.ts                    → Types TypeScript partagés
│   ├── device.ts                   → Détection mobile/desktop
│   ├── storage.ts                  → localStorage helpers
│   ├── backup.ts                   → Export/import données
│   └── emergency.ts                → Mode urgence frontend
│
├── routes/
│   ├── +layout.svelte              → Loading, sodium init, authStore.init()
│   ├── +page.svelte                → Redirect selon rôle
│   ├── login/+page.svelte          → id="username" id="password" (Playwright !)
│   ├── register/+page.svelte
│   ├── admin/+page.svelte
│   ├── chat/+page.svelte
│   ├── chess/+page.svelte
│   ├── calendar/+page.svelte
│   ├── polls/+page.svelte
│   ├── settings/+page.svelte
│   ├── call/+page.svelte
│   ├── change-password/+page.svelte
│   ├── invite/+page.svelte
│   ├── join/+page.svelte
│   └── help/+page.svelte
```

---

## ⚡ Règles Svelte 5 Runes — ABSOLUES

### Règle #1 — $state exporté : jamais de réassignation directe

```typescript
// ❌ ERREUR : state_invalid_export (vite-plugin-svelte:compile-module)
export let conversations = $state<Conversation[]>([]);
conversations = newData;  // réassignation → erreur CI

// ✅ Pattern correct : $state sur objet encapsulant
interface ConversationState {
  conversations: Conversation[];
  activeId: string | null;
}
export const conversationStore = $state<ConversationState>({
  conversations: [],
  activeId: null
});
// Mutation OK via propriété :
conversationStore.conversations = newData;
conversationStore.activeId = id;
```

### Règle #2 — Reset d'un objet $state

```typescript
// ❌ réassigne la référence exportée
export const myStore = $state<MyState>(createInitial());
// Ailleurs :
myStore = createInitial();  // ERREUR

// ✅ Object.assign préserve la réactivité ET la référence
Object.assign(myStore, createInitial());

// ✅ Ou spread property par property
myStore.conversations = [];
myStore.activeId = null;
```

### Règle #3 — $derived et $effect UNIQUEMENT dans .svelte

```typescript
// ❌ INTERDIT dans .svelte.ts (module context)
export const isLoggedIn = $derived(authStore.user !== null);
$effect(() => { fetchData(); });

// ✅ Dans un fichier .svelte.ts → fonctions getter classiques
export function isLoggedIn(): boolean {
  return authStore.user !== null;
}

// ✅ $derived et $effect → uniquement dans les composants .svelte
```

### Règle #4 — Jamais Svelte 4 stores

```typescript
// ❌ Incompatible avec Svelte 5 Runes mode
import { writable, readable, derived, get } from 'svelte/store';
export const count = writable(0);

// ✅ Svelte 5
export const countStore = $state({ value: 0 });
export function increment() { countStore.value++; }
```

### Règle #5 — Template literals dans .svelte

```svelte
<!-- ⚠️ Vigilance absolue sur le copier-coller de Claude.ai -->
<!-- Les ${expr} peuvent être corrompus ($ mangé, backtick transformé) -->
<!-- Toujours vérifier manuellement après copier-coller -->
```

---

## 🏗️ Pattern store complet recommandé

```typescript
// monStore.svelte.ts
interface MonState {
  items: Item[];
  loading: boolean;
  error: string | null;
}

function createInitialState(): MonState {
  return { items: [], loading: false, error: null };
}

export const monStore = $state<MonState>(createInitialState());

export async function loadItems(): Promise<void> {
  monStore.loading = true;
  monStore.error = null;
  try {
    const res = await fetch('/api/items', { credentials: 'include' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    monStore.items = await res.json();
  } catch (e) {
    monStore.error = e instanceof Error ? e.message : 'Erreur inconnue';
  } finally {
    monStore.loading = false;
  }
}

export function resetStore(): void {
  Object.assign(monStore, createInitialState());
}

export function getActiveItem(): Item | undefined {
  return monStore.items.find(i => i.active);
}
```

---

## 🗺️ Layout — Séquence de démarrage critique

```
+layout.svelte onMount()
    │
    ├─ loading = true
    │
    ├─ waitForSodium().then(...)   → fire-and-forget (PAS de await !)
    │   └─ libsodium WASM ~500ms en arrière-plan
    │
    ├─ await authStore.init()      → fetch('/api/auth/me') — SEULE chose bloquante
    │   ├─ 200 → isAuthenticated = true, user = data
    │   └─ 401 → authStore.logout() → localStorage.removeItem(...)
    │
    └─ loading = false             → {#if !loading} → {@render children()} visible
                                     ← #username accessible ICI seulement (Playwright !)
```

**⚠️ Règle R37** : Ne JAMAIS `await waitForSodium()` dans onMount du layout principal.
Sodium en fire-and-forget sinon CI headless → loading bloqué → #username jamais visible.

**Conséquence Playwright** : `page.goto('/login')` se resolve AVANT `loading=false`.
→ `#username` absent du DOM jusqu'à la fin du `onMount`.
→ Toujours `await page.locator('#username').waitFor({ state: 'visible' })` avant `fill()`.

---

## 🔐 Auth store — contrat public

```typescript
// authStore.svelte.js — API publique attendue
class AuthStore {
  user = $state(null);           // User | null
  isAuthenticated = $state(false);
  isLoading = $state(false);

  async init() { /* fetch /api/auth/me */ }
  async logout() {
    // 1. POST /api/auth/logout
    // 2. localStorage.removeItem('nook_user')
    // 3. localStorage.removeItem('nook_session_id')
    // 4. this.user = null; this.isAuthenticated = false;
  }
}
export const authStore = new AuthStore();
```

---

## 📡 API client — pattern fetch

```typescript
// api.ts — toujours credentials: 'include' (cookie HttpOnly)
export async function apiFetch<T>(
  path: string,
  options?: RequestInit
): Promise<T> {
  const res = await fetch(`/api${path}`, {
    ...options,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });
  if (!res.ok) {
    const error = await res.text().catch(() => `HTTP ${res.status}`);
    throw new Error(error);
  }
  return res.json();
}
```

---

## 📱 Responsive & Mobile

- Port prod : 6300 (même origine → pas de CORS sur appels API)
- Menu hamburger pour mobile (breakpoint `md:` Tailwind ou CSS custom)
- `device.ts` expose `isMobile()` pour adapter le rendu
- Les inputs de login **doivent** avoir `id="username"` et `id="password"` (Playwright + autofill)
- Thèmes : Jardin Secret / Space Hub / Maison Chaleureuse — persistés en localStorage

---

## ⚡ Performance — Points de vigilance

| Problème | Impact | Fix |
|----------|--------|-----|
| libsodium chunk 938 kB | LCP dégradé sur mobile | fire-and-forget dans onMount (pas await) |
| `{#each messages}` sans key | Re-render complet | `{#each messages as msg (msg.id)}` |
| fetch sans debounce sur input | N requêtes | `setTimeout` 300ms ou `$derived` |
| Images non lazy | Scroll lent | `loading="lazy"` sur `<img>` |

---

## 🐛 Checklist avant livraison d'un composant

- [ ] MCP `svelte:svelte-autofixer` passé — 0 issues
- [ ] Pas de `writable()` / `readable()` Svelte 4
- [ ] `$state` exporté → mutations via propriété (pas réassignation)
- [ ] `$derived` / `$effect` uniquement dans fichier `.svelte` (pas `.svelte.ts`)
- [ ] `id="username"` / `id="password"` présents dans les formulaires de login
- [ ] `credentials: 'include'` sur tous les `fetch()`
- [ ] Template literals vérifiés manuellement (backticks non corrompus)
- [ ] Responsive vérifié (mobile viewport 375px minimum)
- [ ] Thème appliqué (pas de couleurs hardcodées en dehors des variables CSS)

---

## ⚡ Workflows dédiés

| Workflow | Déclencheur | Action |
|----------|-------------|--------|
| `bundle-analysis.yml` | Push `.svelte`/`.ts`/`package.json` ou manuel | Rapport tailles chunks + alerte DT-01 libsodium |
| `auto-svelte5-migration.yml` | Manuel | Vérifie conformité Svelte 5 Runes |
| `fix-svelte5-runes.yml` | Manuel | Purge résidus syntaxe Svelte 4 |
| `npm-audit-report.yml` | Manuel | Audit sécurité dépendances npm |

> Après tout changement d'import lourd : lancer `bundle-analysis.yml` pour surveiller DT-01.

## 🤝 Flux inter-agents

```
← 🦀 RUST / 🔐 CRYPTO  : endpoints, types TS, protocoles crypto
→ 🧪 E2E                : sélecteurs id= et data-testid= stables, comportements attendus
→ 🚀 DEVOPS             : nouvelles deps npm, nouveaux chunks bundle
```

---

## 📚 Apprentissages

### [APP-SVELTE-01] state_invalid_export — Session 1
→ **Promu** dans "Règles Svelte 5 Runes — ABSOLUES".

### [APP-SVELTE-02] Layout loading masque les enfants — Session 23
→ **Promu** dans "Layout — Séquence de démarrage critique".

### [APP-SVELTE-03] authStore.logout() appelé automatiquement sur 401
Quand `/api/auth/me` retourne 401, `authStore.logout()` vide automatiquement
le localStorage. Pas besoin de le faire manuellement.

### [APP-SVELTE-04] crypto.randomUUID en HTTP LAN — Session 11
`crypto.randomUUID()` requiert HTTPS ou localhost. En HTTP LAN → `is not a function`.
Fix : fallback `Math.random()` hex ou polyfill UUID v4 manuel.

### [APP-SVELTE-05] Thèmes — 3 thèmes disponibles
`ThemeStore.svelte.ts` gère : `jardin-secret`, `space-hub`, `maison-chaleureuse`.
Persisté en localStorage. Appliqué via variables CSS sur `:root`.

### [APP-SVELTE-R37] waitForSodium() dans onMount bloque loading=false — CI headless
```
Symptôme : waitFor('#username', 20_000) timeout sur 75/75 tests E2E
Cause    : onMount faisait await waitForSodium() (938kB WASM) AVANT authStore.init()
           En CI Chromium headless, le chargement WASM prend >20s
           → loading reste true → {#if loading} masque le contenu → #username absent du DOM

Fix      : Sodium en fire-and-forget, authStore.init() en priorité
           waitForSodium().then(() => initCryptoSystem()).then(...).catch(...)  // PAS de await
           await authStore.init()   // seule chose qui détermine l'affichage
           loading = false          // dès que la session est vérifiée (~100ms)

Règle    : Ne JAMAIS await waitForSodium() dans onMount du layout principal
```

### [APP-SVELTE-MCP-01] Intégration MCP Svelte — Session 38
MCP officiel Svelte disponible sur `https://mcp.svelte.dev/mcp`.
Outils : `list-sections`, `get-documentation`, `svelte-autofixer`, `playground-link`.
`svelte-autofixer` est **obligatoire** avant toute livraison de code Svelte.
Le MCP garantit l'alignement avec la doc Svelte 5 / SvelteKit courante.
