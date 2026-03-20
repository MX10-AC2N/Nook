# 📅 Résumés sessions — Nook

> Contexte rapide. Détails complets dans `SESSIONS.md`.

---

## État du projet (après session 39)

**Version** : 0.4.0-beta.2 | **Tests E2E** : 115/115 ✅ (100% depuis S39)

### ✅ Fonctionnel et stable
- Backend Rust : axum 0.8, rand 0.9, sqlx 0.8.6, ring (VAPID)
- Docker build sources + release, distroless arm64/amd64
- Auth cookie HttpOnly, LAN + WAN (Nginx), E2EE X25519 actif
- CI : 115/115 tests E2E verts, rapport MD auto, Docker multi-arch
- E2EE : unlockCrypto au login → chiffrement/déchiffrement transparent dans le chat
- Push notifications : VAPID réel via ring + reqwest, SW frontend complet
- Rate limit : KeyedRateLimiter par IP (60 req/min) — S36/S38
- Sécurité : DOMPurify XSS, magic bytes uploads, WS 64KB limit — S35/S36
- Chess temps réel : WS broadcast côté serveur + refreshGame côté client (DT-02 résolu)

### 🟡 Dette technique (non bloquant)
- **DT-01** : libsodium 938 kB — cosmétique (sodium est déjà fire-and-forget depuis S37)
- **SEC-03** : Token session UUID (entropy ok, 256 bits optionnel — faible risque)
- **SEC-06** : emergency.rs non connecté (informationnel — avant activation feature)

### 🔴 Bugs actifs
*Aucun bug actif bloquant — 115/115 tests verts*

### 📋 Backlog priorisé (post S39)
1. **Analytics enrichis** : graphiques détaillés par membre, exports — DATA agent
2. **DT-01** : libsodium chunk size (cosmétique)
3. **SEC-03** : Token 256 bits (faible risque)
4. **Tests IRL Zimaboard** : valider push sur device réel, E2EE multi-device
5. **Version 0.4.0** : retirer le tag beta après validation prod

---

## Chronologie condensée

| Sessions | Thème principal | Résultat |
|----------|----------------|---------|
| 1 | Analyse initiale, identification bugs | CLAUDE.md créé |
| 2-5 | Rust upgrades, Docker, SQLite, CORS | Backend stable ✅ |
| 6-7 | CI Playwright infra, E2E_SETUP | CI infra stable ✅ |
| 8-14 | Bugs prod : UUID, CORS, SameSite, prune | Prod stable ✅ |
| 15-19 | E2E stabilisation (sélecteurs, admin, git) | 12/43 → progrès |
| 20 | Race condition matrix amd64/arm64 | Build reports séparés ✅ |
| 21 | fullyParallel:true → localStorage partagé | fullyParallel:false ✅ |
| 22 | clearSession goto('/') → init avec cookie | request.post(logout) ✅ |
| 23 | fill() avant layout onMount | waitFor(visible) ✅ |
| 24 | Refonte .claude/ v4 : orchestration + agents | Structure v4 ✅ |
| 25-32 | Polls E2E, crypto dégradé, admin UI | 75 tests définis ✅ |
| 33 | localStorage non vidé → 55 tests échouaient | localStorage.clear() ✅ |
| 34 | [Voir SESSIONS.md] | — |
| 35 | Audit sécurité (XSS, rate limit, magic bytes) | SEC-01/02/04/05 fixés ✅ |
| 36 | Rate limit IP, magic bytes, WS 64KB, CORS panic | Sécurité S36 ✅ |
| 37 | waitForSodium() bloquait loading=false en CI | Sodium fire-and-forget ✅ |
| 38 | MCP Svelte + Rust + Lightpanda dans .claude | Structure MCP ✅ |
| 38 | Fix decrypt_file_from_storage (nonce double-préfixé → 500) | Download fichiers ✅ |
| 38 | Fix serde(default) sur encrypted/is_group, isolatedPage admin tests | Tests E2E 75→113 ✅ |
| 39 | Fix resign winner_id IA, polls id, ai-move 415, react UI hover | 115/115 tests ✅ |
| 39 | Activation Push VAPID (ring + reqwest), frontend SW + settings | Push actif ✅ |
| 39 | Mise à jour documentation complète (.claude/) | Docs à jour ✅ |

---

## Dernière session (38) — Points clés (mise à jour)

### Bugs critiques résolus
- **R_DECRYPT** : `decrypt_file_from_storage` re-préfixait le nonce → download 500
  - `crypto_secretbox_easy` stocke `nonce||ciphertext` ensemble → `_nonce_base64` maintenant ignoré dans decrypt
- **R_INVITE** : test `invite/validate` → token extrait depuis `invite_link` (backend retourne `{invite_link}` pas `{token}`)
- **R_COOKIE** : `isolatedPage` pour tous les logins `testUser` dans admin.spec.ts
- **R_SERDE** : `#[serde(default)]` sur `encrypted` et `is_group` dans `db.rs`

### Pièges à retenir (agent 🦀 RUST)
- `encrypt_file_for_storage` intègre TOUJOURS le nonce dans les premiers bytes du fichier stocké
  → `decrypt_file_from_storage` doit appeler `crypto_secretbox_open_easy(ciphertext, key)` directement
  → ne jamais re-préfixer le nonce depuis la DB dans decrypt

## Dernière session (39) — 115/115 + Push VAPID

### Bugs corrigés S39
- **R_RESIGN** : `resign_game` → `winner_id="ai"` violait la FK users → `None` pour IA + propagation erreur
- **R_POLLS_ID** : `(await createRes.json()).id` → `undefined` → `.poll?.id`
- **R_AI_MOVE** : POST `/chess/{id}/ai-move` → 415 → `Option<Json<AiMoveRequest>>`
- **R_REACT_UI** : hover CI headless → `dispatchEvent('mouseenter')` + `waitForResponse('/reactions')`

### Push VAPID S39
- `push.rs` : envoi VAPID réel (ring ECDSA P-256 + reqwest POST)
- `frontend/src/lib/push.ts` : `subscribeToPush()` / `unsubscribePush()`
- `frontend/src/routes/settings/+page.svelte` : bouton activation notifications
- `.env.example` : `VAPID_PRIVATE_KEY`, `VAPID_PUBLIC_KEY`, `VAPID_SUBJECT`
- `db.rs` : appel `send_push_notification()` après `send_message()`

### E2EE — état réel S39
L'E2EE était déjà fonctionnel depuis S37-38 :
- `login/+page.svelte` → `unlockCrypto(userId, password)` après chaque login
- `change-password/+page.svelte` → `unlockCrypto(userId, newPassword)` au changement initial
- `chatStore.svelte.ts` → chiffre si `cs.ready`, déchiffre à la réception
- Backend → retourne `sender_public_key` avec chaque message
- Mode dégradé automatique si clé absente (message envoyé en clair sans erreur)

### Pièges à retenir S39
- `winner_id` dans chess_games a une FK vers users → ne jamais stocker de valeur arbitraire ("ai")
- `#[derive(...)]` doit être immédiatement suivi du `struct` qu'il annote — une fonction libre entre les deux casse la compilation
- VAPID JWT : algorithme ES256, audience = origin de l'endpoint, expiry < 12h

## Session 38 — Infrastructure MCP

- **Intégration MCP Svelte** : `https://mcp.svelte.dev/mcp` — remote, 4 outils
  - `list-sections`, `get-documentation`, `svelte-autofixer`, `playground-link`
  - `svelte-autofixer` obligatoire avant toute livraison de code Svelte
- **Intégration MCP Rust** : `rust-mcp-server` (cargo tools) + `mcp-language-server` (LSP)
  - Local uniquement, ne pas référencer dans GitHub Actions
- **Lightpanda** : navigateur headless Zig, beta, noté pour monitoring futur
- **Fichiers créés/mis à jour** :
  - `.claude/CLAUDE.md` — section MCP ajoutée, session 38
  - `.claude/settings.json` — version 0.4.0-beta.1, mcpServers, règle autofixer
  - `.claude/rules/mcp-servers.md` — **NOUVEAU** — référence MCP complète
  - `.claude/roles/svelte-frontend.md` — protocole MCP Svelte en tête
  - `.claude/skills/nook-svelte-frontend/SKILL.md` — checklist + workflow MCP
  - `.claude/rules/memory-sessions.md` — ce fichier, état S38
