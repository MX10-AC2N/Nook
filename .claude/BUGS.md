# 🐛 BUGS.md — Nook

> Mis à jour : **2026-03-30** (session 44)

---

## 🔴 BUGS ACTIFS

*Aucun bug actif bloquant.*

---

## 📋 Pièges critiques S44

```
calendar    : Classes CSS DOIVENT correspondre aux sélecteurs E2E du test
              → .calendar-grid (pas .cal-grid)
              → .add-event-btn (pas .btn-add)
              Règle générale : tout composant refait → vérifier user.spec.ts avant de livrer
```

## 📋 Pièges critiques S43

```
main.rs     : DefaultBodyLimit DOIT être appliqué AVANT .layer(cors_layer)
db.rs       : update_event utilise PATCH → ajouter axum::routing::patch dans main.rs
calendar    : PATCH /api/events/{id} nécessite que l'utilisateur soit créateur ou admin
polls       : closes_at envoyé au backend → si backend ne le supporte pas, ignorer silencieusement
themestore  : nuit-douce.css doit être importé dans app.css avant utilisation
```

## 📋 Pièges critiques S42

```
webrtc-calls : URL WS était /ws/call → corriger en /ws
call/[id]    : participants est { value, subscribe } pas un Array → .value.map()
webrtc.rs    : broadcast global → routage to_user_id via user_senders HashMap
chess.rs     : play_ai() sans spawn_blocking dans create_game → freeze frontend
[game_id]    : pageLoading local évite race condition avec chessStore.loading
```

## 📋 Pièges critiques S41

```
invites.rs  : accept_invite ≠ join — prend {token, username, name, password}
chess.rs    : time_limit_secs dans INSERT → migration 007 requise
admin.rs    : delete_user protège contre auto-suppression
layout      : app-main padding:0 → les pages gèrent leur propre hauteur
```

## 📋 Règles Svelte 5 (éviter régressions)

```typescript
// ✅ export $state → objet encapsulant, mutation via propriété
export const store = $state<State>({...});
store.prop = newValue;
// ❌ jamais : export let x = $state(); x = newVal;
// ❌ $derived(() => fn) → retourne la FONCTION → utiliser IIFE
// ❌ onclick|stopPropagation → syntaxe Svelte 4 invalide
//    → utiliser onclick={(e) => { e.stopPropagation(); fn(); }}
```

---

## ✅ BUGS RÉSOLUS — Index compact

| ID | Session | Titre | Fix |
|----|---------|-------|-----|
| R_CALENDAR_CLASSES | 44 | `.calendar-grid` absent → test E2E échoue | Renommer `.cal-grid` → `.calendar-grid`, `.btn-add` → `.add-event-btn` |
| R_ISEMOJI_S44 | 44 | `isSingleEmoji` non définie dans le zip 51 | Ajout fonction après `ALL_EMOJIS` dans chat/+page.svelte |
| R_UPLOAD_7MO | 43 | Upload > 7Mo échoue | `DefaultBodyLimit::max(52MB)` dans main.rs |
| R_SCROLL_CHAT | 43 | Scroll chat ne descend pas | `$effect` avec tolérance 150px |
| R_BADGE_MENU | 43 | Badge non-lu absent du menu | `totalUnread` dérivé + CSS `.nav-badge` |
| R_CALENDAR_EDIT | 43 | Calendrier : pas de modification/suppression | Modal détail/édition + PATCH /api/events/{id} |
| R_THEME_DARK | 43 | Pas de mode sombre | Thème `nuit-douce` + ThemeStore + app.css |
| R_ISEMOJI | 42 | `isSingleEmoji()` non définie → chat vide | Fonction ajoutée dans chat/+page.svelte |
| R_CHESS_CREATE_FREEZE | 42 | Création partie IA bloque la page | `tokio::task::spawn_blocking` |
| R_CALL_WS_URL | 42 | URL WS appels `/ws/call` inexistante | Corriger en `/ws` + ws/wss dynamique |
| R_CALL_PARTICIPANTS | 42 | `participants.map()` crash | `participants.value.map()` |
| R_CALL_ROUTING | 42 | Signaux WebRTC broadcast global | `user_senders` HashMap + routage `to_user_id` |
| R_RESIGN | 39 | `resign_game` winner_id="ai" → FK violation | `winner_id = None` pour IA |
| R_POLLS_ID | 39 | Tests polls `.id` au lieu de `.poll?.id` | Accès via `.poll?.id` |

| R_NOTIF_SW_S45 | 45 | Service Worker jamais enregistré → notifications ne marchent nulle part | +layout.ts: register manuel au load + reply icon path fixe |
