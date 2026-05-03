# 🎨 Svelte 5 Patterns — Nook Frontend

> Référence rapide pour le développement frontend Nook
> Source : `.hermes/roles/svelte-frontend.md`

## 🔌 MCP Svelte — Protocole obligatoire

```bash
Avant chaque intervention sur du code Svelte :

1. svelte:list-sections
   → Identifier les sections de doc pertinentes

2. svelte:get-documentation(sections[])
   → Charger la doc Svelte 5 / SvelteKit exacte

3. [Écrire le code en s'appuyant sur la doc fraîche]

4. svelte:svelte-autofixer(code)
   → Analyser le code produit — relancer jusqu'à "no issues"
   → OBLIGATOIRE avant toute livraison
```

## ⚡ Règles Svelte 5 Runes — ABSOLUES

### Règle #1 — $state exporté : jamais de réassignation directe

```typescript
// ❌ ERREUR : state_invalid_export
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
// ❌ Ne pas réassigner
myState = { ...newValue };  // ERREUR

// ✅ Mise à jour propriété par propriété
Object.assign(myState, newValue);
// OU
myState.prop1 = newValue.prop1;
myState.prop2 = newValue.prop2;
```

### Règle #3 — $derived vs $derived.by

```typescript
// ✅ $derived pour expression simple
const filtered = $derived(items.filter(i => i.active));

// ✅ $derived.by pour logique complexe
const summary = $derived.by(() => {
  const active = items.filter(i => i.active);
  return `${active.length} active sur ${items.length}`;
});
```

## 🎯 Périmètre exclusif Nook

```
frontend/src/
├── lib/
│   ├── authStore.svelte.js         → AuthStore classe, cookie HttpOnly
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
├── routes/
│   ├── +layout.svelte              → Loading, sodium init, authStore.init()
│   ├── login/+page.svelte          → id="username" id="password" (Playwright !)
│   ├── chat/+page.svelte
│   ├── call/+page.svelte
│   └── admin/+page.svelte
```

## 🧠 Stores Nook — Patterns

### chatStore.svelte.ts
```typescript
// Pattern WebSocket avec $state
export const chatStore = $state({
  messages: [] as Message[],
  ws: null as WebSocket | null,
  connected: false
});

// Méthode pour envoyer un message
export function sendMessage(content: string) {
  if (chatStore.ws?.readyState === WebSocket.OPEN) {
    chatStore.ws.send(JSON.stringify({ type: 'message', content }));
  }
}
```

### authStore.svelte.js
```typescript
// Pattern classe avec $state
export class AuthStore {
  user = $state(null);
  loading = $state(true);
  
  async init() {
    // vérif cookie HttpOnly
  }
}

export const authStore = new AuthStore();
```

## ⚠️ Pièges connus

1. **Playwright** : login page utilise `id="username"` et `id="password"`
2. **libsodium** : 938 kB, chargement asynchrone dans `+layout.svelte`
3. **E2EE refresh bug** : cryptoStore.ready=false → messages visibles
4. **Mobile** : sidebar overlay, hamburger, 16px font
5. **SVG icons** : pas d'emojis dans l'UI

## 🧪 Tests E2E

- Toujours utiliser `data-testid` ou sélecteurs stables
- Accepter le certificat HTTPS auto-signé (--ignore-https-errors)
- Tests P2P file transfer : conversation 1-to-1 uniquement
- Tests appels : vérifier idle state, pas de vrai media en CI
