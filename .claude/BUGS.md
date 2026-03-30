# 🐛 BUGS.md — Nook

> Mis à jour : **2026-03-29** (session 43)

---

## 🔴 BUGS ACTIFS

*Aucun bug actif bloquant.*

---

## 📋 Pièges critiques S43

```
main.rs     : DefaultBodyLimit DOIT être appliqué AVANT .layer(cors_layer)
              → sinon Axum rejette le body avec 413 avant même le handler
db.rs       : update_event utilise PATCH → ajouter axum::routing::patch dans main.rs
calendar    : PATCH /api/events/{id} nécessite que l'utilisateur soit créateur ou admin
webrtc.rs   : user_senders HashMap<String, BroadcastSender> — clé = user_id (String)
              Si un user ouvre 2 onglets, le 2e écrase le 1er dans user_senders → OK (dernier WS gagne)
themestore  : nuit-douce.css doit être importé dans app.css avant utilisation
polls       : closes_at envoyé au backend → si le backend ne le supporte pas encore, ignorer silencieusement
```

## 📋 Pièges critiques S42

```
webrtc-calls : URL WS était /ws/call → n'existe pas, corriger en /ws
call/[id]    : participants est { value, subscribe } pas un Array → .value.map()
webrtc.rs    : broadcast global → routage to_user_id via user_senders HashMap
chess.rs     : play_ai() sans spawn_blocking dans create_game → freeze frontend
[game_id]    : pageLoading local évite race condition avec chessStore.loading (loadGameList)
cryptoStore  : registerPublicKeyOnServer en arrière-plan (non bloquant) → ready=true immédiatement
chat         : min-height:0 sur messages-container + overflow:hidden sur chat-area → input toujours visible
```

## 📋 Pièges critiques S41

```
invites.rs  : accept_invite ≠ join — accept_invite prend {token, username, name, password}
              needs_password_change = 0 (le mdp est choisi à l'inscription, pas généré)
chess.rs    : time_limit_secs dans INSERT chess_games → migration 007 requise avant déploiement
admin.rs    : delete_user protège contre auto-suppression et suppression d'un admin
layout      : app-main padding:0 → toutes les pages qui avaient padding doivent le gérer elles-mêmes
chat        : _hoverTimer doit être déclaré en let (pas $state) — c'est un timer JS, pas réactif
```

## 📋 Règles Svelte 5 (éviter régressions)

```typescript
// ✅ export $state → objet encapsulant, mutation via propriété
export const store = $state<State>({...});
store.prop = newValue;  // ✅
// ❌ jamais : export let x = $state(); x = newVal;
// ❌ jamais : writable() Svelte 4
// ❌ $derived/$effect hors composant .svelte
// ❌ $derived(() => fn) — retourne la FONCTION, pas le résultat → utiliser IIFE : $derived((() => fn)())
```

---

## ✅ BUGS RÉSOLUS — Index compact

| ID | Session | Titre | Fix |
|----|---------|-------|-----|
| R_UPLOAD_7MO | 43 | Upload > 7Mo échoue — Axum limite body à 2MB par défaut | `DefaultBodyLimit::max(52MB)` dans main.rs |
| R_SCROLL_CHAT | 43 | Scroll chat ne descend pas au dernier message | `$effect` avec tolérance 150px + scroll forcé au changement de conv |
| R_BADGE_MENU | 43 | Badge non-lu absent dans le menu nav | `totalUnread` dérivé depuis `chatStore.unreadCounts` + CSS `.nav-badge` |
| R_CALENDAR_EDIT | 43 | Calendrier : pas de modification/suppression d'événement | Modal détail/édition + PATCH /api/events/{id} dans db.rs + main.rs |
| R_THEME_DARK | 43 | Pas de mode sombre global | Thème `nuit-douce` CSS + ThemeStore étendu + app.css import |
| R_ISEMOJI | 42 | `isSingleEmoji()` non définie → ReferenceError → aucun message affiché | Fonction ajoutée dans chat/+page.svelte |
| R_CHESS_CREATE_FREEZE | 42 | Création partie IA (noirs) bloque la page — play_ai sans spawn_blocking | `tokio::task::spawn_blocking` dans create_game |
| R_CHESS_LOADING | 42 | "Chargement de la partie…" infini — race condition loading singleton | `pageLoading` local à [game_id]/+page.svelte |
| R_CALL_WS_URL | 42 | URL WS appels `/ws/call` n'existe pas → appels impossibles | Corriger en `/ws` + protocole ws/wss dynamique |
| R_CALL_PARTICIPANTS | 42 | `participants.map()` crash — participants est {value,subscribe} pas Array | `participants.value.map()` partout dans call/[id]/+page.svelte |
| R_CALL_ROUTING | 42 | Signaux WebRTC broadcast global → chaos multi-utilisateurs | `user_senders` HashMap + routage `to_user_id` dans webrtc.rs |
| R_CALL_OFFER | 42 | handleOffer refuse si localStream null → appel entrant ignoré | Setup stream automatique avant de répondre |
| R_CALL_TITLE | 42 | Titre page call affiche UUID brut | `convTitle` dérivé des noms participants |
| R_RINGTONE | 42 | Pas de sonnerie à l'appel entrant | `AudioContext` synthétisé + signal `call_request` |
| R_BANNER_E2EE | 42 | Bannière E2EE persistante même après connexion | Conditionné sur `cryptoStore.ready === false` |
| R_INVITE_REDIRECT | 42 | `/invite` redirige vers login | Ajouté dans `publicPaths` du layout |
| R_JOIN_422 | 42 | `/join` → 422 — `public_key` envoyé comme Array d'entiers | Encoder en base64 `btoa(String.fromCharCode(...))` |
| R_CHESS_MODAL | 42 | Modal résultat fin de partie jamais affiché | `$effect` détecte transition vers statut terminal |
| R_CHESS_CHECK | 42 | Roi en échec jamais surligné | `kingInCheckSquare` IIFE fix `$derived((() => ...)())` |
| R_CHAT_SCROLL_MIN | 42 | Zone saisie chat disparaît avec GIFs grands | `min-height:0` + `overflow:hidden` sur chat-area |
| R_RESIGN | 39 | `resign_game` winner_id="ai" → UPDATE ignoré | `winner_id = None` pour IA |
| R_POLLS_ID | 39 | Tests polls lisaient `.id` au lieu de `.poll?.id` | Accès via `.poll?.id` |
| R_B1 | 26 | `state_invalid_export` conversationStore | Objet $state encapsulé |
