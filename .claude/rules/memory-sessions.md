# 📅 Résumés sessions — Nook

> Contexte rapide. Détails complets dans `SESSIONS.md`.

---

## État du projet (après session 37)

**Version** : 0.4.0-beta.1 | **Tests E2E** : 75/75 (stable depuis S37)

### ✅ Fonctionnel et stable
- Backend Rust compile sans erreur (axum 0.8, rand 0.9, sqlx 0.8.6)
- Docker build sources + release, distroless arm64/amd64
- Auth cookie HttpOnly, LAN + WAN (Nginx)
- CI : 5 workflows manuels stables
- E2E : 75/75 depuis S37 (fix sodium fire-and-forget)
- Rate limit : KeyedRateLimiter par IP (30 req/min) — S36
- Sécurité : DOMPurify XSS, magic bytes uploads, WS 64KB limit — S35/S36

### 🟡 Dette technique (non bloquant)
- **DT-01** : libsodium 938 kB → dynamic import (cosmétique, sodium est déjà fire-and-forget)
- **DT-02** : Chess temps réel absent (DT-02 — coups visibles seulement au refresh)
- **SEC-03** : Token session UUID (entropy ok, 256 bits optionnel — faible risque)
- **SEC-06** : emergency.rs non connecté (informationnel — avant activation feature)

### 🔴 Bugs actifs
*Aucun bug actif bloquant — 113/115 tests passent, 2 tests UI en cours de stabilisation*

### 📋 Backlog priorisé
1. **DT-02** : Chess temps réel via WebSocket (ARCHITECT + CHESS requis)
2. **DT-01** : libsodium dynamic import (SVELTE)
3. **E2EE** : Activation complète (CRYPTO + RUST + SVELTE)
4. **Notifications push** : à évaluer avec FOUNDER
5. **Analytics enrichis** : DATA agent

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

## Dernière session (38) — Infrastructure MCP

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
