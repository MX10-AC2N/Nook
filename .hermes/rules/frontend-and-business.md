# 🎨 Frontend Stores & Règles métier — Nook

> Ce fichier documente les 16 stores/modules de `frontend/src/lib/`
> et les contraintes métier qui ne sont pas dans le code.

---

## 📦 Inventaire `frontend/src/lib/`

| Fichier | Type | Expose | État |
|---------|------|--------|------|
| `authStore.svelte.js` | Store $state | `authStore.user`, `.isAuthenticated`, `.isAdmin`, `.loading`, `.init()`, `.logout()` | ✅ Stable |
| `chatStore.svelte.ts` | Store $state | `chatStore.messages[]`, `.conversations[]`, `.currentConv`, `.send()`, `.loadMessages()` | ✅ Stable |
| `conversationStore.svelte.ts` | Store $state | Export $state — objet encapsulant | ✅ Stable |
| `chessStore.svelte.ts` | Store $state | `chessStore.game`, `.board`, `.myColor`, `.makeMove()`, `.loadGame()` | ✅ Stable |
| `cryptoStore.svelte.ts` | Store $state | Clés E2EE en mémoire + IndexedDB, `unlockCrypto()`, `encryptMessage()` | ✅ Actif S39 |
| `mediaStore.svelte.js` | Store $state | Enregistrement audio/vidéo, upload progress | ✅ |
| `webrtc-calls.svelte.ts` | Store $state | `callStore.isInCall`, `.isMuted`, `.localStream`, `.peerConnections` | ⚠️ WAN instable |
| `sodium.svelte.js` | Module | `waitForSodium()` — fire-and-forget depuis S37 (938 kB WASM) | ✅ Non-bloquant |
| `e2ee.ts` | Module | Fonctions crypto E2EE legacy (libsodium) — remplacé par `crypto.ts` + `cryptoStore` | ✅ Remplacé |
| `crypto.ts` | Module | Wrap XChaCha20-Poly1305 pour fichiers | ✅ |
| `api.ts` | Module | Helpers auth (`changePassword`, `getUserInfo`, `logout`, `validateInviteToken`…) — pas d'`apiFetch` | ✅ |
| `auth.js` | Module | Helpers auth legacy | ✅ |
| `types.ts` | Types | Interfaces TS : User, Conversation, Message, Poll, ChessGame... | ✅ |
| `device.ts` | Module | Détection mobile/desktop | ✅ |
| `storage.ts` | Module | LocalStorage helpers | ✅ |
| `backup.ts` | Module | Export données utilisateur | ✅ |
| `emergency.ts` | Module | Mode urgence (panic button) | ✅ |
| `push.ts` | Module | `subscribeToPush()`, `unsubscribePush()`, `getPushState()` — S39 | ✅ S39 |
| `ui/` | Composants | ThemeStore, composants partagés | ✅ |

### Pattern $state — Règle absolue
```typescript
// ✅ Export via objet encapsulant
export const authStore = $state<AuthState>({ isAuthenticated: false, ... });
authStore.user = newUser;  // mutation via propriété

// ❌ Jamais
export let x = $state(0);
x = 1;  // state_invalid_export → Bug #1
```

### Séquence critique du layout (onMount)
```
+layout.svelte onMount() :
  1. waitForSodium()          ~500ms (charge 938 kB WASM)
  2. initCrypto()             clés IndexedDB
  3. authStore.init()         GET /api/auth/me
  → Avant ça : #username n'existe PAS dans le DOM
  → E2E : toujours waitFor('#username', visible, 20s) avant fill()
```

### apiFetch — pattern standard
```typescript
import { apiFetch } from '$lib/api';

const res = await apiFetch('/api/conversations', {
  method: 'POST',
  body: JSON.stringify({ name: 'Famille' })
});
// credentials: 'include' automatique → cookie envoyé
// Sur 401 : authStore.logout() automatique
```

---

## 🎨 Routes frontend

| Route | Fichier | Accès |
|-------|---------|-------|
| `/` | `+page.svelte` | Redirect auto : admin→`/admin`, user→`/chat`, anon→`/login` |
| `/login` | `login/` | Inputs : `id="username"` + `id="password"` (⚠️ id=, pas name=) |
| `/chat` | `chat/` | Groupe Global hardcodé (`default_global`) |
| `/admin` | `admin/` | `require_admin` — redirect si non admin |
| `/admin/analytics` | `admin/analytics/` | Chart.js doughnut — GET /api/analytics |
| `/register` | `register/` | Inscription → `approved=0`, en attente admin |
| `/chess` | `chess/` | Liste des parties + créer |
| `/chess/[game_id]` | `chess/[game_id]/` | Partie en cours |
| `/calendar` | `calendar/` | Événements familiaux |
| `/polls` | `polls/` | Sondages |
| `/settings` | `settings/` | Profil + thème + mdp |
| `/call` | `call/` | Appel WebRTC audio/vidéo |
| `/invite` | `invite/` | Page lien invitation |
| `/join` | `join/` | Rejoindre via token |
| `/help` | `help/` | Aide |
| `/change-password` | `change-password/` | Forcé si `needs_password_change=1` |

### Thèmes disponibles
```typescript
type Theme = 'jardin-secret' | 'space-hub' | 'maison-chaleureuse'
// Persisté localStorage, appliqué via CSS variables sur :root
// Ne jamais hardcoder des couleurs — utiliser les variables du thème
```

---

## 📐 Règles métier (non écrites dans le code)

### Utilisateurs & Auth
- Cookie `auth_token` : `Max-Age=86400` (24h)
- Argon2id pour les mots de passe (via `rand_core::OsRng`, **pas** `rand::rng()`)
- E2EE : `unlockCrypto(userId, password)` appelée au login + change-password → `cryptoStore.ready = true`
- Messages chiffrés si `cryptoStore.ready`, en clair sinon (mode dégradé transparent)
- `approved=0` à l'inscription → l'admin doit approuver manuellement
- `needs_password_change=1` → redirect forcé vers `/change-password`
- Seul l'admin peut approuver/rejeter des membres
- Un seul admin possible (rôle `admin` unique)

### Uploads & Fichiers
- Max **50 Mo** par fichier (`upload.rs`)
- TTL **48h** → `prune.rs` nettoyage toutes les 24h
- Chiffrement XChaCha20-Poly1305 disponible (nonce 24B, key 32B)
- Types acceptés : images, vidéos, audio, documents

### Invitations
- TTL **48h** (expires_at = now + 48h)
- Usage unique (`used INTEGER DEFAULT 0`)
- Seul l'admin peut générer des invitations

### Polls
- 1 vote par user par sondage (`UNIQUE(poll_id, user_id)`)
- Vote modifiable via UPSERT avant fermeture
- Seul le créateur ou l'admin peut fermer un sondage
- 409 retourné si tentative double vote sans modification

### Chess
- `ai_difficulty = NULL` → humain vs humain
- `ai_difficulty = 'easy'|'medium'|'hard'` → vs IA minimax
- FEN stocké dans `board_state` (format standard FIDE)
- Positions légales calculées côté serveur (pas côté client)
- `status : 'waiting' | 'playing' | 'finished'`

### WebRTC
- Signaling via WS `/ws` (authentifié — token vérifié à la connexion)
- Connexion P2P directe (serveur ne voit pas les flux)
- Stable en LAN — instable WAN sans serveur TURN
- `SameSite=Lax` en HTTP/LAN → `SameSite=None; Secure` en HTTPS/WAN (détecté via `X-Forwarded-Proto`)

---

## 🚀 Déploiement Zimaboard (référence)

```bash
# 1. Cloner ou puller
git clone https://github.com/MX10-AC2N/Nook.git && cd Nook
# ou : git pull && docker compose pull

# 2. Configurer .env (Zimaboard — une seule fois)
cat > .env << EOF
NOOK_IMAGE=ghcr.io/mx10-ac2n/nook:latest
HOST_PORT=6300
DATA_DIR=/media/ac2n-cloud/volume_docker_Nook/nook-data
LOGS_DIR=/media/ac2n-cloud/volume_docker_Nook/nook-logs
PUBLIC_SITE_URL=http://192.168.X.X:6300
ALLOWED_ORIGINS=http://192.168.X.X:6300,https://nook.mondomaine.com
RUST_LOG=info
TZ=Europe/Paris
E2E_SETUP=0
EOF

# 3. Premier lancement (init container chown 65532 → distroless)
docker compose up -d

# 4. Mise à jour
docker compose pull && docker compose up -d
```

### Debug logs Docker
```bash
docker compose logs -f nook          # logs temps réel
docker compose logs --tail=50 nook   # 50 dernières lignes
docker compose ps                    # état des containers
docker exec -it nook sh              # ⚠️ impossible — distroless sans shell
```

### Nginx Proxy Manager → Nook
```
Forward Hostname/IP : localhost (ou IP Zimaboard)
Forward Port : 6300
Websockets Support : ON  ← obligatoire pour /ws
SSL : Let's Encrypt recommandé
```

---

## 🧩 Dépendances clés — Versions exactes

### Backend (Cargo.toml)
```toml
axum = "0.8" + features ws, multipart, tokio
sqlx = "0.8.6" + features sqlite, migrate, runtime-tokio-rustls
rand = "0.9"       # webrtc.rs — rand::rng() pas thread_rng()
rand_core = "0.6"  # argon2 — rand_core::OsRng (diamond dep fix)
argon2 = "0.5"
chacha20poly1305 = "0.10.1"
tokio = "1.0" + full
chrono = "0.4" + serde
uuid = "1.0" + v4
tower-http = "0.6.8" + fs, cors, compression-br
```

### Frontend (package.json)
```
svelte: ^5.46.3 | @sveltejs/kit: ^2.49.4 | typescript: ^5.9.3  ← vérifier package.json
tailwindcss: ^4.1.18 | vite: ^7.3.1
libsodium-wrappers: ^0.8.0  ← 938 kB WASM (DT-01)
chart.js: ^4.5.1 | simple-peer: ^9.11.1
@playwright/test: ^1.40.0
```


## 🌐 SFU (Selective Forwarding Unit) — Appels groupe

### CallState — Champs SFU

```typescript
// Dans CallState interface
useSfu: boolean;              // Mode SFU actif (auto pour 3+ participants)
sfuAnswer: string | null;     // SDP answer du backend SFU
sfuRenegotiateOffer: null;    // Offre de renegotiation du SFU
sfuPeers: string[];           // Liste des autres participants dans la room SFU
sfuPendingOffer: string | null; // Offre en attente de traitement
```

### WebRTCCallManager — Méthodes SFU

```typescript
// Démarrer un appel SFU (backend relaye les medias)
await callManager.startSfuCall(conversationId, participantIds, type);

// Gérer la réponse du backend
callManager.handleSfuJoinResponse({ answer, peers, renegotiate_offer });

// Gérer une offre de renegotiation (nouvelles tracks ajoutées)
callManager.handleSfuRenegotiateOffer(offer);

// Mettre à jour la liste des peers
callManager.handleSfuPeers({ peers });

// Retourner au mode P2P mesh
await callManager.stopSfuMode();
```

### Auto-activation SFU

```typescript
// Dans startCall() — bascule automatique si 3+ participants
if (participantIds.length >= 3) {
  callStore.useSfu = true;
  return this.startSfuCall(conversationId, participantIds, type);
}
// Sinon: continue en mode P2P mesh normal
```

### Signalisation SFU via WebSocket

Types de messages WS ajoutés dans `handleSignal`:
- `sfu_answer` → handleSfuJoinResponse
- `sfu_renegotiate_offer` → handleSfuRenegotiateOffer
- `sfu_peers` → handleSfuPeers
- `sfu_error` → set error state

### Call Page — UI SFU

```svelte
<!-- Badge SFU dans le header -->
{#if callStore.useSfu}
  <span class="sfu-badge">🌐 SFU · {callStore.sfuPeers.length} pairs</span>
{/if}

<!-- Toggle P2P ↔ SFU (visible si 3+ participants) -->
{#if callStore.sfuPeers.length >= 2}
  <button onclick={toggleSfuMode} class:active={callStore.useSfu}>
    {callStore.useSfu ? '🌐 P2P' : '🔗 SFU'}
  </button>
{/if}
```

### Toggle SFU/P2P

```typescript
function toggleSfuMode() {
  if (callStore.useSfu) {
    callManager.stopSfuMode(); // Retour en P2P mesh
  } else {
    const nonSelf = participants.value.filter(p => p.id !== authStore.user?.id);
    if (nonSelf.length >= 2) {
      callManager.startSfuCall(conversationId, nonSelf.map(p => p.id), callType);
    }
  }
}
```

### Architecture backend SFU

```
SfuState (singleton dans SharedState)
  └─> rooms: HashMap<conversation_id, Room>
       └─> Room
            ├─ peers: HashMap<user_id, Arc<Peer>>
            │    └─ Peer { pc: PeerConnection, added_sources: HashSet, negotiation_pending }
            └─ tracks: Vec<Arc<TrackInfo>>
                 └─ TrackInfo { relay: MediaRelay, remote_track, user_id, kind, params }
```

### Flow SFU complet

1. Client appelle `startSfuCall()` → WS `sfu_join` avec offer SDP
2. Backend `handle_join()`:
   - Crée `PeerConnection::new(RtcConfiguration::default())`
   - `pc.set_remote_description(offer)` puis `pc.create_answer()`
   - Ajoute les tracks existantes de la room au nouveau peer
   - Retourne `SfuJoinResponse { answer, peers, renegotiate_offer? }`
3. Client reçoit `sfu_answer` → stocke dans `callStore.sfuAnswer`
4. Quand un participant envoie une track:
   - Backend reçoit `PeerConnectionEvent::Track(transceiver)`
   - Crée `MediaRelay::with_capacity(track, 500)`
   - Ajoute la track relayée aux autres peers
   - Forward PLI/RTCP + PLI periodique 3s
5. Si nouvelles tracks ajoutées → backend envoie `sfu_renegotiate_offer`
