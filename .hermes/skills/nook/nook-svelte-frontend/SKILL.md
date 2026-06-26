---
name: nook-svelte-frontend
description: Skill spécialisé pour tout développement frontend SvelteKit 5 du projet Nook. Utilise cette skill dès qu'un fichier .svelte, .svelte.ts ou .svelte.js est impliqué, qu'un store $state est créé ou modifié, qu'une route frontend est ajoutée, ou que le rapport FRONTEND-BUILD-REPORT.md signale des erreurs. Couvre : SvelteKit 5 Runes, stores $state, composants, routes, thèmes CSS, apiFetch, authStore, chatStore, conversationStore, chessStore, webrtc-calls, mediaStore. Depuis S38 : intégrer le MCP Svelte (mcp.svelte.dev) avant toute intervention.
---

# 🎨 Nook — Frontend SvelteKit 5 Skill

## 🔌 MCP Svelte — Protocole obligatoire (depuis S38)

**Avant toute intervention sur du code Svelte, appeler le MCP dans cet ordre :**

```
1. svelte:list-sections
   → Découvrir toutes les sections de documentation disponibles

2. svelte:get-documentation(sections[])
   → Charger le contenu exact des sections pertinentes
   → Accepte plusieurs sections en un seul appel

3. [Écrire le code en s'appuyant sur la doc fraîche]

4. svelte:svelte-autofixer(code)
   → OBLIGATOIRE avant livraison — analyser et corriger
   → Relancer jusqu'à "no issues returned"
```

**Pourquoi ?** La doc Svelte 5 Runes évolue. Le MCP évite les régressions
dues à de la mémoire périmée sur les runes, $state, $derived, SvelteKit routing.

## Périmètre

```
frontend/src/
├── lib/
│   ├── authStore.svelte.js         → AuthStore, isAuthenticated, init(), logout()
│   ├── chatStore.svelte.ts         → Messages, WebSocket temps réel
│   ├── conversationStore.svelte.ts → Liste conversations (objet $state encapsulé)
│   ├── chessStore.svelte.ts        → État partie d'échecs
│   ├── cryptoStore.svelte.ts       → Clés E2EE en mémoire + IndexedDB
│   ├── mediaStore.svelte.js        → Upload audio/vidéo, progress
│   ├── webrtc-calls.svelte.ts      → Appels WebRTC, état P2P
│   ├── sodium.svelte.js            → libsodium-wrappers 938 kB (DT-01 ⚠️)
│   ├── api.ts                      → apiFetch avec credentials:include
│   └── types.ts                    → User, Conversation, Message, Poll, ChessGame...
│
└── routes/
    ├── +layout.svelte              → sodium fire-and-forget → authStore.init()
    ├── login/+page.svelte          → id="username" + id="password" (E2E critique !)
    ├── chat/+page.svelte           → default_global hardcodé
    └── admin/+page.svelte          → require_admin
```

## Règle absolue — $state exporté

```typescript
// ❌ state_invalid_export → erreur CI vite-plugin-svelte:compile-module
export let conversations = $state<Conversation[]>([]);
conversations = newData;

// ✅ Pattern correct : $state sur objet encapsulant
interface ConvState { conversations: Conversation[]; activeId: string | null; }
export const conversationStore = $state<ConvState>({ conversations: [], activeId: null });

// Mutation via propriété :
conversationStore.conversations = newData;
conversationStore.activeId = id;

// Reset via Object.assign (préserve la référence réactive) :
Object.assign(conversationStore, { conversations: [], activeId: null });
```

## Séquence critique du layout (onMount) — R37

**⚠️ Ne JAMAIS `await waitForSodium()` dans onMount du layout.**
Sodium en fire-and-forget → authStore.init() en priorité.

```
+layout.svelte onMount() :
  1. waitForSodium().then(...).catch(...)  ← FIRE-AND-FORGET (pas await !)
  2. await authStore.init()               ← GET /api/auth/me (bloquant)
  3. loading = false                      ← #username accessible maintenant

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

## Checklist livraison (mise à jour S38)

- [ ] **MCP `svelte:svelte-autofixer` passé — 0 issues** ← NOUVEAU S38
- [ ] Pas de `writable()` / `readable()` Svelte 4
- [ ] `$state` exporté → mutations via propriété (pas réassignation)
- [ ] `$derived` / `$effect` uniquement dans `.svelte` (pas `.svelte.ts` pur)
- [ ] `id="username"` / `id="password"` dans les formulaires de login
- [ ] `credentials: 'include'` sur tous les `fetch()`
- [ ] Template literals vérifiés manuellement
- [ ] Responsive vérifié (mobile 375px minimum)
- [ ] Thème appliqué (pas de couleurs hardcodées)
- [ ] Sodium en fire-and-forget dans onMount (pas await)

## Diagnostics rapides

| Erreur | Cause | Fix |
|--------|-------|-----|
| `state_invalid_export` | `export let x = $state()` | Encapsuler dans objet |
| `rune outside component` | `$derived`/`$effect` dans `.ts` pur | Déplacer dans `.svelte.ts` |
| `#username` non trouvé (E2E) | Layout pas encore chargé | `waitFor('#username', visible, 20s)` |
| `#username` jamais visible (CI) | `await waitForSodium()` bloque loading | Sodium en fire-and-forget |
| 401 inattendu | `apiFetch` sans credentials | Utiliser `apiFetch` (jamais `fetch` direct) |
| Thème non appliqué | Couleur hardcodée | Passer par CSS variables |

## Flux inter-agents

```
← Après tout endpoint 🦀 RUST   : récupérer URL + payload + codes HTTP
→ Après tout nouveau sélecteur  : informer 🧪 E2E (id= ou data-testid= ajouté)
→ Après tout nouveau store      : documenter dans rules/frontend-and-business.md
```
