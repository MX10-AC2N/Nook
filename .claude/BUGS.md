# 🐛 BUGS.md — Nook

> Mis à jour : **2026-03-22** (session 41)

---

## 🔴 BUGS ACTIFS

*Aucun bug actif bloquant.*

---

## 📋 Pièges critiques S41

```
invites.rs  : accept_invite ≠ join — accept_invite prend {token, username, name, password}
              needs_password_change = 0 (le mdp est choisi à l'inscription, pas généré)
chess.rs    : time_limit_secs dans INSERT chess_games → migration 007 requise avant déploiement
admin.rs    : delete_user protège contre auto-suppression et suppression d'un admin
layout      : app-main padding:0 → toutes les pages qui avaient padding doivent le gérer elles-mêmes
chat        : _hoverTimer doit être déclaré en let (pas $state) — c'est un timer JS, pas réactif
```

## 📋 Pièges critiques S40


```
chess.rs  : play_ai moved dans spawn_blocking → reconstruire Game depuis new_fen pour game_json()
chess.rs  : ai_move retourne game_status_str (String), make_move retourne game_status (enum) — ne pas mélanger
layout    : initThemeGlobal() DOIT être dans +layout.svelte onMount — pas seulement dans settings
timer IA  : bascule vers aiColor AVANT fetch IA, pas après data.game
TT chess  : 1M entrées = 40MB réalloués chaque coup → limiter à 64K par défaut (Zimaboard ARM64)
```

## 📋 Règles Svelte 5 (éviter régressions)

```typescript
// ✅ export $state → objet encapsulant, mutation via propriété
export const store = $state<State>({...});
store.prop = newValue;  // ✅
// ❌ jamais : export let x = $state(); x = newVal;
// ❌ jamais : writable() Svelte 4
// ❌ $derived/$effect hors composant .svelte
```

---

## ✅ BUGS RÉSOLUS — Index compact

> Détails complets dans `SESSIONS.md`. Format : `[Session] Titre — Fix en une ligne`

| ID | Session | Titre | Fix |
|----|---------|-------|-----|
| R_RESIGN | 39 | `resign_game` stockait `winner_id="ai"` → UPDATE silencieusement ignoré → status "playing" | `winner_id = None` pour parties IA + propagation erreur UPDATE |
| R_POLLS_ID | 39 | Tests polls lisaient `(await createRes.json()).id` → `undefined` (body = `{poll:{id}}`) | Accès via `.poll?.id` |
| R_POLL_SERDE | 39 | `encrypted`/`is_group` sans `#[serde(default)]` dans db.rs → 422 | `#[serde(default)]` + `fn default_true()` placée AVANT le `#[derive]` de `User` |
| R_AI_MOVE | 39 | POST `/chess/{id}/ai-move` sans body → 415 (Axum exige Json body) | `Option<Json<AiMoveRequest>>` dans le handler |
| R_DECRYPT | 38 | `decrypt_file_from_storage` re-préfixait le nonce → download 500 | `_nonce_base64` ignoré, `ciphertext` contient déjà le nonce intégré par `encrypt_file_for_storage` |
| R_INVITE | 38 | Test `invite/validate` extrayait `{ token }` d'un body qui retourne `{ invite_link }` | Token extrait depuis `invite_link.split('?')[1]` |
| R_COOKIE | 38 | Test flux inscription polluait cookie admin via `adminPage.request.post(login)` | `isolatedPage` isolé pour tous les logins `testUser` |
| R_SERDE | 38 | `encrypted` et `is_group` sans `#[serde(default)]` → 422 si champ absent | `#[serde(default)]` + `fn default_true()` avant les structs |
| R37 | 37 | `waitForSodium()` bloquait `loading=false` → `#username` jamais visible en CI | Sodium lancé en fire-and-forget, `loading=false` après `authStore.init()` uniquement |
| R36a | 36 | Page blanche Zimaboard — base_inject_middleware inutile | Supprimé de main.rs + app.html nettoyé |
| R36b | 36 | Rate limit 429 en CI E2E — NotKeyed global épuisé par les tests | KeyedRateLimiter par IP, quota 30/min |
| R33 | 33 | `clearSession` ne vidait pas localStorage → `isAuthenticated=true` | `page.evaluate(() => localStorage.clear())` |
| R25 | 26 | Polls E2E race condition `waitForResponse` après `goto()` | `Promise.all([waitForResponse, goto()])` |
| R24 | 25 | Layout bloque sur `!cryptoInitialized` → `#username` jamais visible | Crypto failure = mode dégradé non-bloquant |
| R23 | 23 | `fill('#username')` avant layout onMount | `waitFor('#username', visible, 20s)` |
| R22 | 22 | `clearSession` goto('/') → authStore.init avec cookie | `page.request.post(logout)` avant tout goto |
| R21 | 21 | `fullyParallel:true` partage browser context | `fullyParallel: false` |
| R20 | 20 | Race condition matrix amd64/arm64 | Deux fichiers rapport séparés |
| R19 | 19 | git push TEST_REPORT non-fast-forward | Fetch avant push dans workflow |
| R18 | 18 | Admin UI : #username disabled localStorage | `loginAsAdmin` API-first |
| R17 | 17 | Chess page strict mode violation h1 | Un seul h1 par page |
| R16 | 16 | Logout button introuvable E2E | Sélecteur data-testid ajouté |
| R15 | 15 | e2e_ci absent conversation_participants | Ajout dans E2E_SETUP init |
| R14 | 13 | Prune supprime default_global | Exclure conversations système |
| R13 | 12 | Cookie SameSite=Lax bloque WAN | Détecter X-Forwarded-Proto → None;Secure |
| R12 | 11 | CORS bloque LAN + WAN simultanément | Lister origines explicites |
| R11 | 10 | crypto.randomUUID HTTP LAN | Fallback UUID v4 manuel |
| R05 | 5 | SQLite SQLITE_CANTOPEN code 14 | `create_if_missing(true)` |
| R04 | 4 | Linker crash Docker (.cargo/config.toml) | Ne pas COPY .cargo/ dans Docker |
| R03 | 3 | proc-macro async-trait crash | Retirer tower_governor |
| R02 | 2 | rand_core diamond dep | `rand_core = "0.6"` explicite |
| R01 | 2 | axum 0.8 breaking changes | Routes {param}, Message::Text .into() |
| R_THEME | 40 | Thèmes appliqués seulement sur /settings, disparaissent à la navigation | `initThemeGlobal()` dans `+layout.svelte` onMount + CSS via `var(--xxx)` |
| R_CHESS_GAME | 40 | `make_move` et `ai_move` retournaient `{success, fen}` → `data.game` undefined → plateau vide | Réponses wrappées dans `{success, game: GameState}` |
| R_CHESS_IA_TIMER | 40 | Timer décompte joueur mais pas IA — `switchTimer` appelé après `data.game` (side_to_move déjà "white") | Bascule vers `aiColor` avant le fetch, rebascule après |
| R_CHESS_IA_FREEZE | 40 | IA bloquée après ~10 coups — TT 1M entrées (~40MB) allouée à chaque coup sur Zimaboard ARM64 | `spawn_blocking` + `time_limit` par difficulté + TT adaptative (16K→1M selon niveau) |
| R_CHESS_LASTMOVE | 40 | `lastMove` non mis à jour après coup IA (highlight adversaire absent) | `play_ai` retourne `(san, from, to)` ; `triggerAiMove` extrait `from/to` depuis `move_history` |
| R_MAIN_SHARED | 40 | Suppression proxy GIF emportait `SharedState` + `use crate::config` (lignes 51-145) | Suppression chirurgicale lignes 51-124 uniquement (juste la fn `gif_search_proxy`) |
| R_INVITE_ACCEPT | 41 | `/invite/+page.svelte` appelait `/api/invite/accept` inexistant → redirection login | Nouveau handler `accept_invite` dans `invites.rs` + route dans `main.rs` |
| R_MENU_SIDE | 41 | Menu burger à gauche mais panel s'ouvrait à droite | `right:0` → `left:0` + animation `translateX(-100%→0)` |
| R_CHAT_INPUT | 41 | Zone saisie chat coupée par padding app-main | `app-main` padding:0, pages gèrent leur propre hauteur |
| R_CHESS_NAMES | 41 | IDs joueurs affichés au lieu des noms | `get_game` JOIN users → `player1_name`/`player2_name` |
| R_CHESS_TIMER_P2 | 41 | Timer absent pour le joueur qui rejoint | `time_limit_secs` stocké en DB (migration 007), `loadGame` init le timer depuis le serveur |
| R_CHESS_IA_DELAY | 41 | IA jouait instantanément | Délai minimum par difficulté (Easy 700ms … Godlike 4s) avec `std::thread::sleep` |
| R_HOVER_REACTION | 41 | Picker réaction disparaît avant clic souris | `onmouseleave` → 400ms délai avant `hoveredMsgId = null` |
| R_B1 | 26 | `state_invalid_export` conversationStore | Déjà corrigé (objet $state encapsulé) |
| R_B3 | 26 | `connectionError.set()` cassé | Déjà corrigé : `setConnectionError()` |
| R_ISEMOJI | 41 | `isSingleEmoji()` appelée dans le template chat mais jamais déclarée → ReferenceError → aucun message affiché | Fonction ajoutée dans +page.svelte |

---

## 🛡️ Sécurité — État des vulnérabilités (audit S35)

| ID | Vulnérabilité | Statut | Session fix |
|----|---------------|--------|-------------|
| SEC-01 | XSS `{@html}` chat | ✅ Résolu | S35 (DOMPurify) |
| SEC-02 | Rate limit global (non IP) | ✅ Résolu | **S36** (KeyedRateLimiter par IP) |
| SEC-04 | Magic bytes uploads non validés | ✅ Résolu | **S36** (validate_magic_bytes) |
| SEC-05 | Pas de limite taille messages WS | ✅ Résolu | **S36** (64KB limit) |
| SEC-03 | Token session UUID (entropy ok, 256 bits optionnel) | 🟡 Faible risque | S37 optionnel |
| SEC-06 | emergency.rs non connecté | ✅ Résolu | **S40** (route POST /emergency + push VAPID à tous les membres) |

---

## 🌐 Architecture LAN ↔ WAN (référence rapide)

```
LAN : HTTP 192.168.x.x:6300 → SameSite=Lax
WAN : HTTPS via Nginx → X-Forwarded-Proto: https → SameSite=None; Secure
CORS : ALLOWED_ORIGINS env, jamais Any avec credentials
Rate limit : 30 req/min par IP (KeyedRateLimiter, governor)
```

---
*Mis à jour session 37 — ajout R37*
