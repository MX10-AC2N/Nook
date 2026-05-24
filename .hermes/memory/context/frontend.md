# 🎨 Contexte Frontend - Nook

> Mis à jour : 2026-05-24

## Stack Technique

- **Framework** : SvelteKit 5 (Runes mode)
- **State** : Svelte 5 Runes ($state, $derived, $effect)
- **Styling** : CSS variables, dark/light themes
- **Build** : Vite + adapter-static
- **E2EE** : XChaCha20-Poly1305 (cryptoStore, sessionStorage)

## Architecture

```
frontend/
├── src/
│   ├── routes/             # SvelteKit routes (+layout.svelte, +page.svelte)
│   │   ├── chat/           # Chat pages
│   │   ├── calls/          # WebRTC calls
│   │   ├── chess/          # Chess game
│   │   └── ...
│   ├── lib/
│   │   ├── chatStore.svelte.ts      # État chat + WebSocket
│   │   ├── conversationStore.svelte.ts # Conversations list + WebSocket messages
│   │   ├── cryptoStore.svelte.ts     # E2EE (IndexedDB + sessionStorage)
│   │   ├── authStore.svelte.js       # Auth (localStorage + sessionStorage)
│   │   ├── webrtc-calls.svelte.ts    # Appels WebRTC
│   │   ├── chessStore.svelte.ts      # Jeu d'échecs + WebSocket
│   │   ├── emergency.ts              # Alerte d'urgence (WebSocket)
│   │   ├── device.ts                 # Détection d'appareil (WebSocket)
│   │   └── e2ee.ts                   # Chiffrement E2EE messages/fichiers
│   └── app.html
├── static/                 # Assets statiques
└── package.json
```

## WebSocket — CRITIQUE

### Endpoint Backend
Le backend expose le WebSocket sur **`/api/webrtc/ws`** (routes nestées sous `/api`).
Le frontend utilisait `/ws` ou `/ws/messages` sans le préfixe `/api` → connexion échouait.

### Corrections Appliquées (2026-05-24)
| Fichier | Avant | Après |
|---------|-------|-------|
| `chatStore.svelte.ts` | `/webrtc/ws` | `/api/webrtc/ws` ✅ |
| `chessStore.svelte.ts` | `/webrtc/ws` | `/api/webrtc/ws` ✅ |
| `webrtc-calls.svelte.ts` | `/webrtc/ws` | `/api/webrtc/ws` ✅ |
| `emergency.ts` | `/webrtc/ws` | `/api/webrtc/ws` ✅ |
| `device.ts` | `/webrtc/ws` | `/api/webrtc/ws` ✅ |
| `conversationStore.svelte.ts` | `/ws/messages` | `/api/webrtc/ws` ✅ |

### Pattern correct
```typescript
const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
const ws = new WebSocket(`${proto}://${window.location.host}/api/webrtc/ws`);
```

### Message types routés par le WS
- `new_message` → broadcast chat
- `message_edited` / `message_deleted` → broadcast chat
- `reaction_updated` → broadcast chat
- `new_poll` / `poll_voted` → notifications
- `offer` / `answer` / `ice_candidate` → WebRTC signaling
- `call_request` / `call_accepted` / `call_rejected` → appel
- `join` / `leave` / `decline` → présence WebRTC
- `p2p_file_start` / `p2p_file_chunk` / `p2p_file_end` → transfert P2P

## E2EE ( Chiffrement de bout en bout )

### Architecture
```
cryptoStore.svelte.ts  ← Store principal E2EE
├── IndexedDB          : stockage persistant clés privées (chiffrées par mot de passe)
├── sessionStorage     : clé de session dérivée (volatile, par onglet) pour rechargement
├── users API          : /api/users/public-key/{userId} pour récupérer les clés publiques
└── XChaCha20-Poly1305 : chiffrement messages et fichiers
```

### Problème critique résolu (2026-05-15)
- **Bug** : `cryptoStore.ready` était mis à `true` avant que `registerPublicKeyOnServer` ne termine
- **Fix** : `await` sur l'appel API avant d'activer le store → `ready = true` seulement quand clé publique bien synchronisée
- **Résidu** : sessionStorage peuplé avant cet appel pour permettre le rechargement sans re-saisir le mot de passe

### Messages anciens après rotation clé
- Les messages envoyés avec une ancienne clé X25519 restent indéchiffrables après rotation
- C'est **structurel** (pas un bug code) : la clé de session utilisée pour chiffrer est perdue

## Commandes Utiles

```bash
# Dev server
npm run dev

# Build production
npm run build

# Check Svelte
npx svelte-check

# Tests Playwright
npx playwright test

# Lint
npm run lint
```

## Points Critiques Actuels

### Testé et Fonctionnel (2026-05-24)
- ✅ Chat HTTP : envoyer/récupérer messages
- ✅ WebSocket temps réel : broadcast `new_message`
- ✅ Réactions : ajouter/supprimer/récupérer
- ✅ Upload fichier : POST /api/upload → download /api/download/{id}
- ✅ Éditer/supprimer message (si auteur)
- ✅ E2EE : cryptoStore unlock avec mot de passe

### À Valider
- ⏳ Pagination messages (avant/après curseur)
- ⏳ E2EE après reload (cryptoStore.ready reste true via sessionStorage)
- ⏳ Anciens messages chiffrés (indéchiffrables après rotation clé — structural)
- ⏳ Playwright tests complets

### Patterns Critiques
- **Svelte 5 form submit** : `<form onsubmit={handler}>` ne tire pas → utiliser `<button type="button" onclick={handler}>`
- **WebSocket URL** : toujours `/api/webrtc/ws`, jamais juste `/webrtc/ws`
- **E2EE unlock** : `await cryptoStore.unlock(password)` puis vérifier `cryptoStore.ready`
