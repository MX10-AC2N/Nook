---
name: nook-svelte-frontend
description: Skill spécialisé pour tout développement frontend SvelteKit 5 du projet Nook. Utilise cette skill dès qu'un fichier .svelte, .svelte.ts ou .svelte.js est impliqué, qu'un store $state est créé ou modifié, qu'une route frontend est ajoutée, ou que le rapport FRONTEND-BUILD-REPORT.md signale des erreurs. Couvre : SvelteKit 5 Runes, stores $state, composants, routes, thèmes CSS, apiFetch, authStore, chatStore, conversationStore, chessStore, webrtc-calls, mediaStore.
---

# 🎨 Nook — Frontend SvelteKit 5 Skill

## Périmètre

```
frontend/src/
├── lib/
│   ├── authStore.svelte.js         → AuthStore, isAuthenticated, init(), logout()
│   ├── chatStore.svelte.ts         → Messages, WebSocket temps réel
│   ├── conversationStore.svelte.ts → Liste conversations (⚠️ Bug #1 résolu — objet $state)
│   ├── chessStore.svelte.ts        → État partie d'échecs
│   ├── cryptoStore.svelte.ts       → Clés E2EE en mémoire + IndexedDB
│   ├── mediaStore.svelte.js        → Upload audio/vidéo, progress
│   ├── webrtc-calls.svelte.ts      → Appels WebRTC, état P2P
│   ├── sodium.svelte.js            → libsodium-wrappers 938 kB (DT-01 ⚠️)
│   ├── api.ts                      → apiFetch avec credentials:include
│   └── types.ts                    → User, Conversation, Message, Poll, ChessGame...
│
└── routes/
    ├── +layout.svelte              → waitForSodium → initCrypto → authStore.init()
    ├── login/+page.svelte          → id="username" + id="password" (E2E critique !)
    ├── chat/+page.svelte           → default_global hardcodé
    └── admin/+page.svelte          → require_admin
```

## Règle absolue — $state exporté

C'est la cause du Bug #1 historique. Ne jamais régresser.

```typescript
// ❌ state_invalid_export → erreur CI vite-plugin-svelte:compile-module
export let conversations = $state<Conversation[]>([]);
conversations = newData;  // réassignation directe = ERREUR

// ✅ Pattern correct : $state sur objet encapsulant
interface ConvState { conversations: Conversation[]; activeId: string | null; }
export const conversationStore = $state<ConvState>({ conversations: [], activeId: null });

// Mutation OK via propriété :
conversationStore.conversations = newData;
conversationStore.activeId = id;
```

### Reset d'un objet $state
```typescript
// ❌ réassigne la référence exportée
myStore = createInitial();

// ✅ Object.assign mute les propriétés en place
Object.assign(myStore, createInitial());
```

### $derived et $effect — portée stricte
```typescript
// ❌ $derived/$effect hors d'un composant .svelte ou d'un .svelte.ts
// → erreur "rune outside component"

// ✅ uniquement dans .svelte ou .svelte.ts (pas .ts pur)
```

## Séquence critique du layout (onMount)

Avant que `authStore.init()` termine, `#username` n'existe pas dans le DOM.

```
+layout.svelte onMount() :
  1. waitForSodium()     ~500ms (charge 938 kB WASM libsodium)
  2. initCrypto()        clés IndexedDB
  3. authStore.init()    GET /api/auth/me → isAuthenticated

→ E2E : toujours waitFor('#username', { state: 'visible', timeout: 20000 })
→ NE PAS appeler fill() avant que ce bloc soit terminé
```

## apiFetch — pattern standard

```typescript
import { apiFetch } from '$lib/api';

// credentials:include automatique → cookie auth_token envoyé
// Sur 401 : authStore.logout() déclenché automatiquement
const res = await apiFetch('/api/conversations', {
  method: 'POST',
  body: JSON.stringify({ name: 'Ma conv' })
});
```

## Sélecteurs E2E — règles strictes

Les tests Playwright cherchent des attributs précis. Les respecter à chaque nouveau composant.

```html
<!-- ✅ Utiliser id= ou data-testid= -->
<input id="username" ... />
<button data-testid="logout-btn">...</button>

<!-- ❌ name=, class= ne sont pas utilisés comme sélecteurs E2E -->
```

## Thèmes — ne pas hardcoder les couleurs

```typescript
type Theme = 'jardin-secret' | 'space-hub' | 'maison-chaleureuse'
// Persisté localStorage, appliqué via CSS variables sur :root
// ✅ utiliser var(--couleur-primaire), var(--bg-surface)...
// ❌ ne jamais écrire color: #3b82f6 directement dans un composant
```

## Routes frontend

| Route | Accès | Notes |
|-------|-------|-------|
| `/` | Tous | Redirect : admin→`/admin`, user→`/chat`, anon→`/login` |
| `/login` | Anon | `id="username"` + `id="password"` (E2E strict) |
| `/chat` | Auth | `default_global` hardcodé |
| `/admin` | Admin | `require_admin` middleware |
| `/change-password` | Auth | Forcé si `needs_password_change=1` |
| `/chess/[game_id]` | Auth | `chessStore.loadGame(game_id)` |

## Diagnostics rapides

| Erreur | Cause | Fix |
|--------|-------|-----|
| `state_invalid_export` | `export let x = $state()` | Encapsuler dans objet |
| `rune outside component` | `$derived`/`$effect` dans `.ts` pur | Déplacer dans `.svelte.ts` |
| `#username` non trouvé (E2E) | Layout pas encore chargé | `waitFor('#username', visible, 20s)` |
| 401 inattendu | `apiFetch` sans credentials | Utiliser `apiFetch` (jamais `fetch` direct) |
| Thème non appliqué | Couleur hardcodée | Passer par CSS variables |

## Flux inter-agents

```
← Après tout endpoint 🦀 RUST   : récupérer URL + payload + codes HTTP
→ Après tout nouveau sélecteur  : informer 🧪 E2E (id= ou data-testid= ajouté)
→ Après tout nouveau store      : documenter dans rules/frontend-and-business.md
```

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
           Sodium s'initialise en arrière-plan, cryptoStore.ready devient true
           quand il est prêt → unlockCrypto() l'utilisera au moment du login
```
