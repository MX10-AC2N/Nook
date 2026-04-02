# 🧪 Rapport Audit Tests E2E — Nook

> **Généré par fix/notifications-and-chess-audit** | **2026-04-02**
> Auteur : Agent QA | Portée : chess, WebRTC, couverture E2E, audit sécurité

---

## 📊 Résumé exécutif

| Indicateur | Avant | Après (nouveaux fichiers) |
|-----------|-------|--------------------------|
| Fichiers de tests E2E | 3 | 5 |
| Tests chess | 6 (14 avec auth 401) | 6 + 27 = **33** |
| Tests WebRTC/audio/vidéo | 0 | **14** |
| Gaps critiques couverts | ❌ | ✅ (promotion, timer, IA×5, resign, humain, UI) |
| Gaps restants | ❌ | ⚠️ (checkmate réel, stalemate, WebSocket temps réel, multi-browser) |

---

## 📁 Fichiers de tests existants

| Fichier | Rôle | Tests |
|---------|------|-------|
| `tests/api-sanity.spec.ts` (126 lignes) | Vérifie que 48 routes protégées retournent 401 sans auth | 48 |
| `tests/admin.spec.ts` (322 lignes) | Flux admin complet | 18 |
| `tests/user.spec.ts` (791 lignes) | Flux utilisateur complet (chat, réactions, upload, polls, chess, calendar, settings, E2EE, push, navigation, rate limit) | 49 |
| `tests/helpers.ts` (116 lignes) | Helpers partagés (loginAs, loginViaAPI, waitForAppReady, etc.) | — |
| **`tests/chess-extended.spec.ts`** (nouveau) | Scénarios étendus chess | 27 |
| **`tests/webrtc.spec.ts`** (nouveau) | Tests WebRTC audio/vidéo | 14 |

---

## ♟ Couverture Chess

### ✅ CE QUI EST TESTÉ (avant + après)

#### API — Authentifié (user.spec.ts + chess-extended.spec.ts)
- `GET /chess/list` → 200
- Créer vs IA (medium, easy, hard, expert, godlike) → toutes 5 difficultés testées
- `GET /chess/{id}` → détail partie
- `GET /chess/{id}/moves?from=e2` → coups légaux
- `POST /chess/{id}/move` (e2→e4) → accepté
- `POST /chess/{id}/move` (e2→e6) → 400 (coup illégal)
- `POST /chess/{id}/ai-move` → 200 (IA répond)
- `POST /chess/{id}/resign` → 200, status=finished
- Resign → winner_id = adversaire (vérifié)
- Coup après game over → rejeté (400/404/409)
- Time_limit_secs configuré en DB (300s vs 0)
- Invitations : créer, inviter, lister, décliner
- Jouer en noir → statut vérifié
- POST /chess/{id}/invite, accept, decline → 401 sans auth

#### API — Non-authentifié (api-sanity.spec.ts)
- 8 routes chess testées en 401 (list, create, invitations, {id}, {id}/move, {id}/moves, {id}/ai-move, {id}/resign)
- 3 routes supplémentaires en 401 (invite, accept, decline) — chess-extended.spec.ts

#### UI (user.spec.ts + chess-extended.spec.ts)
- Plateau 8×8 = 64 cases visible
- Case last-move visible après coup
- Sidebar: historique des coups, status banner, players panel
- Navigation /chess → accessible sans erreur
- Lobby: bouton "Nouvelle partie" → formulaire → création UI → navigation
- Modal promotion non visible (sauf situation de promo)
- Bouton abandon visible
- Navigation lobby → partie existante via lien
- Jouer en noir: plateau avec 32 pièces, status "À vous"

### ❌ CE QUI N'EST PAS TESTÉ (gaps restants)

| Gap | Complexité | Pourquoi |
|-----|-----------|----------|
| Échec et mat réel | 🟡 Moyenne | Nécessite une séquence de ~15-20 coups précis ou setup FEN custom |
| Pat (stalemate) | 🟡 Moyenne | Idem — position spécifique requise |
| Nulle par répétition ×3 | 🟠 Haute | ~6 coups minimaux pour répéter 3 fois la même position |
| Nuelle par 50 coups | 🔴 Très haute | 50 coups sans prise ni mouvement de pion — impossible en E2E |
| Matériel insuffisant | 🟡 Moyenne | Nécessite position avec roi+roi uniquement |
| Promotion réelle de pion (modal UI: clic q/r/b/n) | 🟠 Haute | Nécessite ~20 coups pour amener un pion en rangée 8 |
| Annulation modal promotion (cancelPromotion) | 🟠 Haute | Idem |
| Minuteur expiration réelle (onTimerExpired → resign auto) | 🔴 Très haute | 300s réel de test en CI |
| WebSocket temps réel (chess_move push → refresh) | 🔴 Très haute | Nécessite 2 navigateurs synchronisés + broadcast WS |
| Multi-joueur réel: joueur2 rejoint partie en "waiting" | 🔴 Très haute | 2 comptes, joinGame, synchronisation |
| Board flip UI (vue noire avec rows/cols inversés) | 🟡 Moyenne | Vérifier visuellement le flip |
| Couverture: `chessStore.reset()`, `selectSquare()` UI directe | 🟡 Moyenne | Tests de store pur sans backend |
| Gestion déconnexion WS (retry exponential backoff) | 🟠 Haute | Nécessite kill/relance serveur pendant partie |

---

## 📞 Couverture WebRTC/Audio/Vidéo

### ✅ CE QUI EST TESTÉ

#### API
- `POST /api/webrtc/offer` avec auth → 200, status=offer_sent
- `POST /api/webrtc/offer` sans auth → comportement documenté (⚠️ pas de require_auth!)
- `POST /api/webrtc/answer` avec auth → 200, status=answer_sent
- `POST /api/webrtc/offer` body vide → 400 (Missing offer)
- `POST /api/webrtc/answer` body vide → 400 (Missing answer)

#### WebSocket
- `GET /ws` sans auth → 401
- `GET /ws` avec auth → connexion acceptée

#### UI
- Page `/call/default_global` → charge (même en headless)
- Page `/call/default_global?type=audio` → chargée
- Contrôles d'appel (mute/video/fin) détectés dans le DOM

#### Upload média (audio/vidéo dans le chat)
- Upload fichier `.mp3` → accepté, mime_type = audio/mpeg
- Upload fichier `.webm` → accepté, mime_type = video/webm
- Download fichier audio → Content-Disposition: inline (lecture navigateur)
- Participants default_global → liste non vide (prérequis pour calls de groupe)

### ❌ CE QUI N'EST PAS TESTÉ (gaps)

| Gap | Complexité | Pourquoi |
|-----|-----------|----------|
| Vrai appel audio P2P (RTC conn établie) | 🔴 Impossible en CI headless | getUserMedia échoue sans périphériques |
| Vrai appel vidéo P2P | 🔴 Impossible en CI headless | Idem |
| Signalisation WS complète (offer→answer→ice→connected) | 🔴 Très haute | 2 navigateurs + vrais media devices |
| Ringtone (sonnerie d'appel entrant) | 🔴 Très haute | AudioContext non fonctionnel en headless |
| Mute/Unmute en temps réel | 🔴 Très haute | Nécessite vrai stream audio |
| Toggle video on/off | 🔴 Très haute | Nécessite vraie caméra |
| Gestion déconnexion (reconnect) | 🔴 Très haute | Nécessite vrai call actif |
| mediaStore recording (startRecording/stopRecording) | 🟠 Haute | MediaRecorder nécessite vrais périphériques |
| Permissions média (checkMediaPermissions) | 🟡 Moyenne | navigator.permissions.query fonctionne en headless |
| E2EE sur fichiers média (chiffrement/déchiffrement) | 🟠 Haute | downloadAndDecryptMedia = throw Error (stub) |
| Gestion du store callStore (isCalling, isInCall, error) | 🟡 Moyenne | État observable mais sans vrai appel |

---

## 🔐 Audit de sécurité — Nouvelles findings

### [SEC-07] Routes WebRTC sans authentification — HAUTE

**Fichiers**: `backend/src/webrtc.rs` lignes 535-543
**Routes**: `/api/webrtc/offer`, `/api/webrtc/answer`

**Problème**: Les routes `webrtc_routes()` sont mergées directement dans le Router principal,
**hors** du `protected_routes` qui a le middleware `require_auth`. Les handlers `handle_offer`
et `handle_answer` ne vérifient **aucune authentification**.

Tout utilisateur (même non connecté) peut:
- Envoyer de fausses offres WebRTC à tous les utilisateurs connectés (broadcast global)
- Spam les canaux de broadcast avec de faux signaux
- Perturber les appels en cours en injectant de faux ICE candidates

**Impact**: Perturbation des appels audio/vidéo, spam de signaling, DoS potentiel.

**Fix recommandé**: Ajouter `Extension(CurrentUser)` dans les handlers OU
déplacer ces routes dans `protected_routes` :
```rust
pub async fn handle_offer(
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    ...
) -> impl IntoResponse {
    // Maintenant l'auth est vérifiée
}
```

---

### [SEC-08] `handle_offer` et `handle_answer` broadcastent à TOUS les WS connectés — MOYENNE

**Fichier**: `backend/src/webrtc.rs` lignes 291-294, 329-332

**Problème**: Les offres et réponses WebRTC sont diffusées à **tous** les utilisateurs
connectés via WebSocket, pas seulement aux participants de la conversation.

```rust
let guard = state.webrtc_state.broadcasts.lock().await;
for (_, tx) in guard.iter() {
    let _ = tx.send(response.to_string());
}
```

**Impact**: Fuite d'information (SDP offer contient des IP candidates) entre
utilisateurs de conversations différentes. En contexte familial à quelques membres,
le risque est limité mais le design est incorrect.

**Fix recommandé**: Router les signaux via `user_senders` (déjà utilisé dans
`handle_websocket` pour les types WebRTC) — voir le routage existant lignes 453-481.

---

### [SEC-09] `emergency.rs` module toujours non connecté avec `handle_emergency` — INFORMATIONNEL

**Fichier**: `backend/src/emergency.rs`
**Statut**: Le module existe toujours dans le code mais n'est **toujours pas** importé dans `main.rs`.

Aucune progression sur SEC-06 depuis l'audit S35. Le module reste un risque latent
si activé sans authentification.

---

### [SEC-10] Pas de Content-Security-Policy dans app.html — HAUTE

**Fichier**: `frontend/src/app.html`

**Problème**: Recommmandé dans SEC-01 de l'audit S35 mais **pas encore implémenté**.
Sans CSP, si une XSS existe (ou est introduite), l'attaquant peut charger du JS
externe, exfiltrer des données, etc.

**Fix**: Ajouter dans `<head>` de `app.html` :
```html
<meta http-equiv="Content-Security-Policy"
  content="default-src 'self'; script-src 'self'; img-src 'self' data: blob: https://media.tenor.com; connect-src 'self' ws: wss:; style-src 'self' 'unsafe-inline'; font-src 'self';" />
```

---

## 📋 Plan d'action priorisé

| # | Action | Effort | Impact |
|---|--------|--------|--------|
| 1 | **SEC-07**: Ajouter auth sur routes WebRTC | 1h | Haute |
| 2 | **SEC-10**: Ajouter CSP dans app.html | 1h | Haute |
| 3 | **SEC-08**: Router signaux WebRTC par participant | 3h | Moyenne |
| 4 | **SEC-09**: Supprimer ou sécuriser emergency.rs | 30min | Avant activation |
| 5 | Tests E2E: checkmate réel via FEN setup | 4h | Moyenne |
| 6 | Tests E2E: promotion de pion (séquence ~20 coups) | 6h | Moyenne |
| 7 | Tests E2E: minuteur expiration | 2h | Faible |
| 8 | Tests E2E: multi-navigateur WebSocket temps réel | 8h | Moyenne |

---

*Rapport généré automatiquement — fix/notifications-and-chess-audit — 2026-04-02*
