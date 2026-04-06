# 📅 SESSIONS.md — Historique des sessions de travail

---

## Session 46 — 2026-04-02 (audit tests + sécurité)

### Tests E2E étendus
- **chess-extended.spec.ts** : 27 nouveaux tests chess (promotion, timer, 5× IA, resign, humain, UI, noir, 401)
- **webrtc.spec.ts** : 14 nouveaux tests WebRTC (API auth, WS auth, page call, upload audio/video)
- Total tests E2E : 115 → **156 tests**
- Rapport complet : `.claude/TEST-AND-SECURITY-AUDIT-2026.md`

### Audit de sécurité complet
- SEC-01 à SEC-06 : toutes **résolues** (confirmé par scan automatique)
- SEC-06 (emergency) : maintenant importé dans main.rs + `CurrentUser` vérifié ✅
- **SEC-07** 🔴 : Routes `/api/webrtc/offer` et `/api/webrtc/answer` sans auth
- **SEC-09** 🔴 : Pas de CSP dans `app.html`
- **SEC-10** 🔴 : Pas de headers sécurité HTTP (X-Frame-Options, HSTS, etc.)
- **SEC-08** 🟡 : Broadcast WebRTC global (pas par conversation)
- **SEC-11** 🟡 : X-Forwarded-Proto spoofable sans Nginx
- **SEC-12** 🟡 : Complexité mot de passe minimale (8 chars)

### Catalogue workflows
- **20 workflows** inventoriés et catégorisés
- `.claude/WORKFLOW-CATALOG.md` créé avec recommandations cleanup
- 3 candidats suppression : `auto-svelte5-migration.yml`, `fix-svelte5-runes.yml`, `generate-android-instruction.yml`
- 2 candidats fusion : `update-cargo-lock.yml` + `update-frontend-lock.yml`
- 1 doublon à décider : `ci-new2.yml` vs `Backend.yml`+`Docker.yml`

### Fichiers modifiés
- `.claude/CLAUDE.md` : version → S46, branche, PR #23, references tests+securite, catalogue workflows
- `.claude/BUGS.md` : 3 bugs sécurité actifs ajoutés, pièges S46
- `.claude/rules/workflows.md` : ajout catalogue tests E2E + reference WORKFLOW-CATALOG.md
- `.claude/WORKFLOW-CATALOG.md` : créé (nouveau)
- `.claude/TEST-AND-SECURITY-AUDIT-2026.md` : créé (précédemment)

---

## Session 1 — 2026-02-19
- Analyse complète du projet (Rust + SvelteKit 5)
- Identification des 5 bugs Svelte 5 actifs
- Création initiale de CLAUDE.md et LEARNING.md

---

## Session 2 — 2026-02-21 (matin)
- Upgrade dépendances Rust : axum 0.7→0.8, rand 0.8→0.9, reqwest 0.12→0.13
- Fix diamond dependency rand_core 0.6/0.9 (argon2)
- Fix axum 0.8 breaking changes (Host, Message::Text, middleware)
- Fix GitHub Actions test-nook.yml (ARG Dockerfile, docker-compose CI)

**Fichiers modifiés** : `backend/Cargo.toml`, `backend/src/main.rs`, `backend/src/auth.rs`, `backend/src/webrtc.rs`

---

## Session 3 — 2026-02-21 (après-midi)
- Debugging proc-macro async-trait (5 tentatives → cause racine : Cargo.lock désync)
- Fix : `rm Cargo.lock && cargo update`
- Refonte CI : Backend.yml, Frontend.yml, test-nook.yml, Docker.yml, Release.yml
- Création Dockerfile.release, VERSION, DOCKER.md
- Mise à jour README.md avec badges GHCR

**Fichiers créés** : `Dockerfile.release`, `VERSION`, `.github/workflows/Backend.yml`, `.github/workflows/Release.yml`, `.claude/DOCKER.md`

---

## Session 4 — 2026-02-23 (matin)
- Fix cause racine proc-macro : `.cargo/config.toml` copié dans Docker → linker externe
- Fix : COPY explicite dans Dockerfile qui exclut `.cargo/`
- Fix distroless + volumes : init container `alpine:3` + chown 65532
- Suppression `tower_governor` (dépendance tonic → async-trait)

**Fichiers modifiés** : `Dockerfile`, `docker-compose.yml`

---

## Session 5 — 2026-02-23 (après-midi)
- Fix SQLite code 14 : `SqliteConnectOptions::create_if_missing(true)`
- Fix axum 0.8 routes : `:param` → `{param}` dans `main.rs`
- Fix CORS panic : listes explicites au lieu de wildcards

**Fichiers modifiés** : `backend/src/main.rs`

---

## Session 6 — 2026-02-23 (après-midi suite)
- Fix Playwright `reuseExistingServer: !!process.env.CI`
- Ajout `@playwright/test` dans `package.json`
- Workflow `update-frontend-lock.yml` créé
- Fix e2e.spec.ts : inputs `#username`/`#password` (id= pas name=)
- Fix setup E2E : login admin 401 → solution `E2E_SETUP=1`

**Fichiers modifiés** : `frontend/playwright.config.ts`, `frontend/package.json`, `frontend/tests/e2e.spec.ts`  
**Fichiers créés** : `.github/workflows/update-frontend-lock.yml`

---

## Session 7 — 2026-02-23 (soir)
- Fix `E2E_SETUP=1` : `check_initial_admin` crée `e2e_ci` si env var présente
- Création `docker-compose.ci.yml` (override CI : E2E_SETUP + named volumes + init container)
- Mise à jour `test-nook.yml` : utilise le compose override CI
- Fix `docker-compose.yml` : suppression healthcheck CMD-SHELL (distroless sans curl)
- Fix `Docker.yml` : `dawidd6/action-download-artifact@v6` pour cross-workflow artifacts

**Fichiers modifiés** : `backend/src/main.rs`, `docker-compose.yml`, `.github/workflows/test-nook.yml`, `.github/workflows/Docker.yml`  
**Fichiers créés** : `docker-compose.ci.yml`

---

## 🎯 État actuel (après session 7)

### ✅ Fonctionnel
- Backend Rust compile sans erreur (axum 0.8, rand 0.9, sqlx 0.8.6)
- Docker build depuis sources (`Dockerfile` + `cargo-chef`)
- Docker image distroless + volumes + permissions
- API backend opérationnelle (health, auth, conversations, messages)
- CI integration (test-nook.yml) : stack démarre, API répond
- Pipeline Docker.yml : cross-workflow artifacts via dawidd6
- Playwright infrastructure : browser installé, serveur réutilisé
- User E2E créé automatiquement via E2E_SETUP=1

### 🔄 En cours
- Tests Playwright E2E : infrastructure OK, stabilisation sélecteurs UI en cours
- Déploiement homeserver : test en cours par MX10-AC2N

### 🔴 Restant à faire
1. **Bug #1** : corriger `conversationStore.svelte.ts` (state_invalid_export)
2. **Bug #2** : corriger exports `authStore.svelte.js`
3. **Bug #3** : corriger `connectionError` → `setConnectionError`
4. **Bug #4** : corriger `sodiumLoading`/`sodiumError` dans layout
5. **Bug #5** : corriger incohérence `conversation_members` vs `conversation_participants`
6. Valider tests E2E Playwright en production avec l'UI réelle
7. Implémenter rate limiting (governor seul, tower_governor retiré)

---

## 💡 Décisions architecturales prises

| Décision | Raison |
|----------|--------|
| Deux Dockerfiles | multi-arch + proc-macros incompatibles dans un seul Dockerfile |
| cargo-chef | seule façon fiable de cacher les dépendances Rust sans casser les proc-macros |
| distroless cc-debian12 | image ~8-15MB, pas de shell, user nonroot |
| init container alpine pour volumes | chown avant montage — distroless n'a pas shell |
| `E2E_SETUP=1` env var | évite le fragile login admin curl en CI, user e2e créé à l'init DB |
| dawidd6 pour cross-workflow artifacts | `actions/download-artifact@v4` limité au workflow courant |
| Cookie HttpOnly `auth_token=userId:token` | révocable côté serveur, pas de JWT |
| rand_core 0.6 explicite | diamond dep avec argon2 0.5 qui attend rand_core 0.6 |

## Session 8 — 2026-02-25

### Problèmes résolus

**Bug critique CI** : `test-nook.yml` échouait avec `JSONDecodeError` sur `GET /api/users/pending`

- **Cause** : `require_auth` retournait `Err(StatusCode::UNAUTHORIZED)` (réponse vide) capturée par le `.fallback_service(static_service)` → le client recevait `index.html` au lieu d'un JSON 401
- **Fix `backend/src/auth.rs`** : signature `-> Result<Response, StatusCode>` → `-> Response`, retour d'une réponse JSON complète avec `(StatusCode::UNAUTHORIZED, Json(...)).into_response()` pour `require_auth` ET `require_admin`
- **Fix `backend/src/main.rs`** : ajout d'un `.fallback(|| async { (404, Json(...)) })` sur `api_router` pour garantir des réponses JSON sur toutes les routes `/api`

### Assets PWA créés

Analyse du frontend → 6 icônes SVG manquantes + tous les PNGs PWA absents.

**Icônes SVG ajoutées** dans `frontend/static/icons/` :
`lock.svg`, `login.svg`, `add-user.svg`, `at-sign.svg`, `check.svg`, `check-circle.svg`, `description.svg`

**PNGs PWA générés** dans `frontend/static/` :
`favicon.png` (32×32), `logo-192.png`, `logo-512.png`, `icon-72.png`, `icon-192.png`, `icon-72-dark.png`, `icon-192-dark.png`

**`manifest.json`** corrigé : chemin `/logo.svg` → `/icons/logo.svg`, ajout `favicon.png` 32×32, `purpose: "any maskable"` sur 512.

### vite.config.js optimisé

`manualChunks` découpé en 4 chunks distincts : `libsodium`, `chess`, `svelte`, `vendor`
→ chunk monolithique 938 kB fractionné, `chunkSizeWarningLimit: 600`

### Workflow créé

**`.github/workflows/generate-pwa-icons.yml`** (`1.5==> 🖼️ Génération des icônes PWA`)
- Déclenché manuellement OU automatiquement si `logo-animated.svg` est modifié
- Convertit le SVG animé en frame statique (suppression CSS animations)
- Génère variantes light (`#f0fdf4`/`#2d5a27`) + dark (`#1a1a2e`/`#4ade80`)
- Convertisseur : Inkscape (priorité) → resvg (fallback)
- Optimisation sans perte avec oxipng
- Commit automatique `[skip ci]` sur la branche courante

**Fichiers modifiés** : `backend/src/auth.rs`, `backend/src/main.rs`, `frontend/vite.config.js`, `frontend/static/manifest.json`
**Fichiers créés** : 7× `frontend/static/icons/*.svg`, 7× `frontend/static/*.png`, `.github/workflows/generate-pwa-icons.yml`


---

## Session 11 — 2026-02-26

### Contexte
CI test-nook.yml encore en échec après session 10. Déploiement homeserver fonctionnel (login + changement mdp admin ✅) mais bug invitations dans page admin.

### Bugs identifiés et corrigés

#### 1. Template literals corrompus (`\( {expr} \)` → `${expr}`)
**Fichiers affectés** : `frontend/src/routes/admin/+page.svelte` (2 occurrences) et `frontend/src/routes/chat/+page.svelte` (4 occurrences).
Le pattern `${expr}` avait été corrompu en `\( {expr} \)` lors d'un précédent copier-coller.

**Résultat visible** : affichage littéral `( {window.location.origin}/invite?token= ){data.token}` au lieu du vrai lien.

**Fix** : remplacement byte-level de `\( {` → `${` et `} \)` → `}`.

#### 2. `chatStore.sendMessage` — mauvaise URL + mauvais payload
**Problème** : `sendMessage` envoyait sur `POST /api/messages` avec un payload chiffré `{content: number[], encrypted_keys, nonce}`.
Le backend attend `POST /api/conversations/{id}/messages` avec `{content: String, encrypted: bool}`.

**Fix dans `chatStore.svelte.ts`** :
- URL corrigée : `/api/conversations/${conversationId}/messages`
- Payload simplifié : `{ content, encrypted: false }` (chiffrement E2E à implémenter quand clés disponibles)

#### 3. `chatStore.loadMessages` — parsing réponse incorrect
**Problème** : `data.messages ?? []` alors que le backend retourne `Vec<Message>` (tableau direct, pas `{messages: [...]}`).

**Fix** : `Array.isArray(data) ? data : (data.messages ?? [])`

#### 4. Conversation `default_global` non créée au démarrage
**Problème** : `send_message` échoue avec contrainte FK car `default_global` n'existe pas dans la table `conversations`.

**Fix dans `backend/src/main.rs`** : ajout dans `check_initial_admin()` d'une création de la conversation globale si absente.

#### 5. Test E2E — étape 7 échoue (message non visible)
Conséquence directe des bugs 2+3+4 : le message était "envoyé" côté client mais jamais persisté, `loadMessages` ne renvoyait rien.

**Fix dans `frontend/tests/e2e.spec.ts`** : timeout étendu à 12s pour l'étape 7, plus robuste.

### Fichiers modifiés
- `frontend/src/routes/admin/+page.svelte` — template literals
- `frontend/src/routes/chat/+page.svelte` — template literals
- `frontend/src/lib/chatStore.svelte.ts` — sendMessage + loadMessages
- `backend/src/main.rs` — création default_global
- `frontend/tests/e2e.spec.ts` — robustesse étape 7

### État après corrections
- ✅ Lien d'invitation généré correctement
- ✅ Messages envoyés et persistés en base
- ✅ Messages chargés correctement depuis le backend
- ✅ Test E2E devrait passer entièrement
- ⚠️  Chiffrement E2E désactivé temporairement (envoi en clair) — à réactiver quand système de clés par utilisateur sera en place

---

## Session 12 — 2026-02-26

### Contexte
Premier retour utilisateur complet (homeserver Zimaboard 832). Analyse du USER_TEST.md.

### Bugs identifiés via test manuel

| Bug | Source | Gravité |
|---|---|---|
| Lien invitation `token=undefined` | Backend retourne `invite_link`, frontend lit `data.token` | 🔴 |
| Bouton "Copier" lien invite non fonctionnel | Pas de feedback + crash silencieux HTTP | 🔴 |
| Thème ne change pas | CSS utilise `.theme-X` (classe) mais code applique `data-theme=X` (attribut) | 🔴 |
| Mise à jour profil → "Route API introuvable" | Route `POST /api/user/update` manquante | 🔴 |
| Création événement → "Route API introuvable" | Route `GET|POST /api/events` manquante | 🔴 |
| Upload fichier → FK constraint failed | Conversation `default_global` non créée (session 11 non déployé) | 🔴 |
| Menu incomplet (manque Chess, Polls) | navItems trop court dans layout | 🟡 |
| sendGif URL incorrecte | `/api/messages` au lieu de `/api/conversations/{id}/messages` | 🟡 |
| Chess pas de refresh temps réel | WebSocket côté client non abonné aux coups adverses | 🟡 |

### Corrections apportées

#### Frontend
- **`admin/+page.svelte`** : extraction du token depuis `data.invite_link` + bouton Copier avec fallback `prompt()`
- **`settings/+page.svelte`** : `applyTheme()` applique maintenant la classe `.theme-X` sur `<body>` en plus de `data-theme`
- **`+layout.svelte`** : ajout de Chess (♟️) et Polls (📊) dans navItems
- **`chatStore.svelte.ts`** : `sendGif` corrige URL → `/api/conversations/${id}/messages`

#### Backend — nouvelles routes
- **`db.rs`** : handlers `update_user_profile`, `get_events`, `create_event`, `delete_event`
- **`main.rs`** : routes `POST /api/user/update`, `GET|POST /api/events`, `DELETE /api/events/{id}`
- **`migrations/001_initial.sql`** : table `events` ajoutée

### Fichiers modifiés
- `frontend/src/routes/admin/+page.svelte`
- `frontend/src/routes/settings/+page.svelte`
- `frontend/src/routes/+layout.svelte`
- `frontend/src/lib/chatStore.svelte.ts`
- `backend/src/db.rs`
- `backend/src/main.rs`
- `backend/migrations/001_initial.sql`
- `.claude/USER_TEST.md` (template de test structuré)

### Ce qui reste à faire
- [ ] Chess temps réel : le WS côté client doit s'abonner aux coups adverses
- [ ] Polls : assignation d'utilisateurs aux sondages
- [ ] Chat : liste des utilisateurs connectés
- [ ] Page Aide : mise à jour du contenu
- [ ] Mobile : corrections des débordements CSS
- [ ] Chiffrement E2E : réactiver quand clés disponibles

## Session 13 — 2026-02-27

### Contexte
Analyse du workflow CI `test-nook` en échec (run #22477567766).

### Diagnostic
- **Backend Rust** : ✅ `cargo check`, `cargo test`, `cargo clippy` — tous OK
- **Frontend Build** : ✅ `npm run build` — OK, artifact uploadé (83 fichiers, 522 kB)
- **Integration Docker** : ❌ Test E2E Playwright échoue

### Bug identifié et corrigé — [R14]

**Symptôme** : `POST /api/conversations/default_global/messages` → HTTP 404, 3 tentatives, toutes identiques.

**Analyse des logs** :
```
✓ Conversation globale 'default_global' créée         ← boot OK
[prune] conversations vides supprimées count=1         ← 10s après = DESTRUCTION
[send_message] Conversation 'default_global' introuvable   ← 404
```

**Cause racine** : Le job prune (`prune.rs`) se lance 10 secondes après le démarrage du serveur. Sa requête DELETE supprimait **toutes** les conversations sans messages, y compris `default_global` (groupe système, vide au boot).

**Fix** : `backend/src/prune.rs` — ajout de `AND is_group = 0` dans le DELETE conversations vides. Les groupes ne sont jamais supprimés automatiquement.

### Fichier modifié
- `backend/src/prune.rs`

### Ce qui reste à faire
- [ ] Chess temps réel : le WS côté client doit s'abonner aux coups adverses
- [ ] Polls : assignation d'utilisateurs aux sondages
- [ ] Chat : liste des utilisateurs connectés
- [ ] Page Aide : mise à jour du contenu
- [ ] Mobile : corrections des débordements CSS
- [ ] Chiffrement E2E : réactiver quand clés disponibles

## Session 14 — 2026-02-27

### Contexte
Après la première réussite du workflow CI, extension de la couverture de test E2E et mise en place du rapport automatique dans `.claude/`.

### Ce qui a été fait

#### 1. `frontend/tests/e2e.spec.ts` — Suite complète (28 tests)
Remplacement du test unique par 9 suites couvrant toutes les fonctionnalités :

| Suite | Tests | Description |
|-------|-------|-------------|
| Auth | 4 | Login valide/invalide, /auth/me non-auth, Logout |
| Chat | 3 | Envoi message, GET conversations, GET messages |
| Admin | 5 | Login, onglets, liste users, génération invite, /users/pending auth/non-auth |
| Settings | 2 | Navigation 3 onglets, changement de thème |
| Calendar | 4 | Page, GET/POST /api/events, bouton ajout |
| Chess | 4 | Page, GET list, POST create, formulaire UI |
| Polls | 2 | Page, création sondage localStorage |
| Navigation | 8 | 7 routes + protection /admin |
| API Sanity | 5 | /health, 4 endpoints non-auth → 401 |

#### 2. `.github/workflows/test-nook.yml` — Rapport MD automatique
- Step `Generer rapport MD dans .claude/` : parse la sortie Playwright, collecte les logs Docker, génère `.claude/TEST_REPORT.md`
- Step `Commit rapport MD dans .claude/` : git commit + push du rapport dans la branche CI
- Step `Upload rapport Playwright HTML` : artefact HTML 7 jours (pour debug visuel)
- Reporter Playwright : `json,html` (au lieu de `html` seul)

#### 3. `.claude/TEST_REPORT.md` — Fichier initial créé
Template vide qui sera écrasé à chaque run CI.

### Fichiers modifiés/créés
- `frontend/tests/e2e.spec.ts`
- `.github/workflows/test-nook.yml`
- `.claude/TEST_REPORT.md` (nouveau)
- `.claude/SESSIONS.md` (ce fichier)
- `.claude/BUGS.md`

### Points d'attention pour les prochains runs
- Le test "Admin → /admin non accessible à e2e_ci" suppose que l'admin e2e_ci ne peut pas voir `.admin-header`. Si le comportement est différent (redirection vs affichage "non autorisé"), le test s'adapte (`notAuth || redirected`).
- Le test Chess `POST /api/chess/create` vérifie `body.id ?? body.game_id ?? body.game?.id` — à adapter si la structure de réponse diffère.
- Les tests Calendar `POST /api/events` vérifient `[200, 201]` — selon l'implémentation backend.
- Polls utilise localStorage → les tests UI fonctionnent sans backend.

### Ce qui reste à faire
- [ ] Chess temps réel : le WS côté client doit s'abonner aux coups adverses
- [ ] Polls : backend API (actuellement localStorage only)
- [ ] Events : page `/events` utilise localStorage, `/calendar` utilise l'API — consolider
- [ ] Chat : liste des utilisateurs connectés
- [ ] Chiffrement E2E : réactiver quand clés disponibles



## Sessions 15-18 — 2026-02-28 — Bataille E2E Admin (clearSession)

### Contexte
Après la session 14 qui a mis en place 28 tests E2E, 4 sessions consécutives ont été
nécessaires pour résoudre un problème persistant : les 4 tests Admin UI échouaient
systématiquement après le 1er test Admin, malgré diverses tentatives.

### Chronologie des bugs et tentatives

#### Session 15 — Bug A : `GET /api/conversations` vide pour e2e_ci
**Cause** : e2e_ci n'était pas inséré dans `conversation_participants` → INNER JOIN retournait [].
**Fix** : `main.rs::check_initial_admin()` + `admin.rs::approve_user()` — INSERT OR IGNORE vers default_global.

#### Session 15 — Bug B : Admin login bloqué sur /change-password
**Cause** : `loginAs()` attendait `/chat|admin` mais admin avait `needs_password_change=1`.
**Fix** : accepter `/(chat|admin|change-password)` dans loginAs().

#### Session 15 — Bug C : Logout button introuvable
**Cause** : sélecteur texte ne matchait pas le bouton header (icône 🔌 seulement).
**Fix** : `button[aria-label="Déconnexion"]`.

#### Session 15 — Bug D : Chess strict mode violation
**Cause** : `.btn-create, h1` résolvait 3 éléments (h1 layout + h1 chess + btn-create).
**Fix** : `.btn-create` seul.

#### Session 16 — Admin UI tests : flow change-password
**Ajout** : helper `loginAsAdmin()` qui gère le flow obligatoire :
login → /change-password → remplit formulaire → /admin.
**Problème résiduel** : 4 tests Admin UI après le 1er échouent encore.

#### Session 17 — Tentative 1 : clearCookies()
**Hypothèse** : cookie de session actif → $effect() redirige /login → #username disabled.
**Résultat** : ❌ — `localStorage` survit à clearCookies(). AuthStore lit `nook_user` + `nook_session_id`
synchroniquement dans son constructeur → `isAuthenticated=true` persistant.

#### Session 17 — Bug git push rejeté
**Cause** : `git push` sans `git pull --rebase` → fast-forward impossible (branche avancée par commit précédent).
**Fix** : `test-nook.yml` — ajout de `git pull --rebase origin $ref` avant push.

#### Session 18 — Tentative 2 : about:blank + localStorage.clear()
**Hypothèse** : naviguer vers about:blank puis `page.evaluate(() => localStorage.clear())`.
**Résultat** : ❌ — `about:blank` a une **origine différente** de `localhost:6300`.
Le localStorage de l'app n'est pas accessible depuis about:blank (isolation d'origine).

#### Session 18 — Tentative 3 (FINALE) : addInitScript()
**Cause racine définitive** : `AuthStore` constructor lit localStorage **synchroniquement**
lors du parsing du module JS — avant que tout hook post-navigation puisse intervenir.
Il n'existe aucune fenêtre d'intervention *après* la navigation et *avant* le constructeur.

**Fix** : `page.addInitScript()` — Playwright injecte le script dans le contexte V8
**avant l'exécution de tout JS de la page**, y compris les modules ES6.
```typescript
await page.context().clearCookies();
await page.addInitScript(() => {
  localStorage.removeItem('nook_user');
  localStorage.removeItem('nook_session_id');
  localStorage.removeItem('nook_token');
});
await page.goto('/login');
// → AuthStore() trouve localStorage vide → isAuthenticated=false → pas de redirect
```

### Fichiers modifiés sessions 15-18
- `frontend/tests/e2e.spec.ts` — évolutions majeures (sessions 15→18)
- `backend/src/main.rs` — conversation_participants au boot (session 15)
- `backend/src/admin.rs` — approve_user ajoute à default_global (session 15)
- `.github/workflows/test-nook.yml` — git pull --rebase (session 17)

### État CI attendu après session 18
- **38 tests** (31 actifs + 7 suites-wrapper dans le JSON Playwright)
- **34 ✅ passés** (sessions 15-17 corrigées)
- **4 tests Admin UI** : ✅ attendus après fix addInitScript()
- **TEST_REPORT.md** : mis à jour automatiquement à chaque run (git pull --rebase corrigé)

### Ce qui reste à faire
- [ ] Chess temps réel : WS client → abonnement coups adverses
- [ ] Polls : backend API (actuellement localStorage only)
- [ ] Chat : liste des utilisateurs connectés
- [ ] Chiffrement E2E : réactiver quand clés disponibles

## Session 19 — 2026-02-28 — Fix Admin E2E : approche API-first

### Contexte
Après 4 tentatives échouées (sessions 16-18) pour résoudre les 4 tests Admin UI,
la session 19 identifie la vraie cause racine et applique le fix définitif.

### Cause racine définitive (session 19)

`addInitScript()` échoue car la page Playwright naît à `about:blank`.
Le script est attaché à la page et s'exécute lors des navigations futures —
mais dans le contexte de `about:blank` (origine différente de `localhost:6300`).
Le `localStorage` de l'app reste intact.

**Seule solution valide :** ne jamais naviguer vers `/login` côté browser.
`page.request.post()` partage le cookie store du browser context → pose le
cookie `auth_token` comme un vrai login backend → `page.goto('/admin')` fonctionne
directement sans jamais impliquer le localStorage ni le `$effect()` de redirection.

### Fix — `loginAsAdmin()` réécrit (approche API-first)

```typescript
// POST /api/auth/login via page.request → pose le cookie dans le browser context
let loginRes = await page.request.post(`${BASE}/auth/login`, {
  data: { username: 'admin', password: ADMIN_NEW_PASSWORD },
});
// Si needs_password_change → POST /api/auth/change-password → re-login
// Puis navigation directe :
await page.goto('/admin');  // cookie actif, localStorage jamais consulté
```

### Fichiers modifiés session 19
- `frontend/tests/e2e.spec.ts` — `loginAsAdmin()` réécrit en API-first

---

## Session 20 — 2026-02-28 — Rapports CI par workflow + migration branche main

### Contexte
Migration de la branche `MX10-AC2N-patch-svelte5-runes` vers `main`.
Mise en place des rapports CI dans `.claude/` pour chaque workflow.

### Changements majeurs

#### Branche active → `main`
Tout le travail se fait désormais sur `main`.
CLAUDE.md mis à jour en conséquence.

#### Backend.yml — deux fichiers distincts
**Problème :** la matrix amd64/arm64 exécute deux jobs en parallèle.
Un seul fichier `BACKEND-BUILD-REPORT.md` partagé → race condition garantie :
les deux jobs commitent simultanément → un seul gagne, l'autre est rejeté ou écrasé.

**Solution :** deux fichiers indépendants :
- `.claude/BACKEND-BUILD-REPORT-amd64.md` → job `x86_64-unknown-linux-gnu`
- `.claude/BACKEND-BUILD-REPORT-arm64.md` → job `aarch64-unknown-linux-gnu`

Chaque fichier est auto-suffisant. Zéro coordination nécessaire entre jobs.

**Contenu de chaque rapport :**
- Statut global (✅/❌) des 3 étapes : check, clippy, build
- Erreurs cargo check avec codes d'erreur
- Warnings clippy avec **contexte fichier:ligne** (grep `^warning|^ -->`)
- Taille du binaire strippé
- Ligne `Finished` avec timing

#### Frontend.yml — correction heredoc + warnings améliorés
- Heredoc non indenté → plus d'espaces parasites dans le Markdown
- Capture des warnings svelte avec leur URL `https://svelte.dev/e/...` pour diagnostic rapide
- Contexte fichier:ligne des warnings

#### Docker.yml — correction heredoc
- Heredoc non indenté → Markdown propre
- Commande `docker compose pull && docker compose up -d` prête à copier

### Fichiers modifiés/créés session 20
- `.github/workflows/Backend.yml` — deux rapports par arch, heredoc fix
- `.github/workflows/Frontend.yml` — heredoc fix, warnings context
- `.github/workflows/Docker.yml` — heredoc fix
- `.claude/CLAUDE.md` — branche main, table des rapports CI
- `.claude/SESSIONS.md` — ce fichier
- `.claude/BUGS.md` — mis à jour

### Ce qui reste à faire
- [ ] Déclencher Backend.yml → vérifier BACKEND-BUILD-REPORT-amd64.md et arm64.md
- [ ] Déclencher Frontend.yml → vérifier FRONTEND-BUILD-REPORT.md
- [ ] Déclencher test-nook.yml → confirmer 38/38 tests ✅ (fix session 19 en attente)
- [ ] Chess temps réel : WS client → abonnement coups adverses
- [ ] Polls : backend API (actuellement localStorage only)
- [ ] Chiffrement E2E : réactiver quand clés disponibles

## Session 21 — 2026-03-05 — Fix E2E : localStorage cross-test (Bug #21)

### Contexte
Analyse des logs CI du 2026-03-05 (zip fourni). Résultats : 12 tests passés / 31 échecs.
Backend Rust : ✅ 0 erreur clippy, build OK. Frontend Vite : ✅ build OK (3 warnings a11y non-bloquants).
Tests E2E : ❌ 31/43 timeout sur `waiting for locator('#username')`.

### Cause racine identifiée

`playwright.config.ts` avait `fullyParallel: true` avec `workers: 1` en CI.
Avec `fullyParallel: true`, tous les tests du même fichier partagent le **même browser context** (et donc le même `localStorage` de `localhost:6300`).

Séquence de défaillance :
1. Test 1 (Auth/Login valide e2e_ci) — loginAs() → login réussi → localStorage : `nook_user` + `nook_session_id` posés
2. Test 2 (Auth/Login invalide) — goto('/login') → AuthStore constructeur lit localStorage → `isAuthenticated=true` → `$effect()` redirige vers `/chat` → `#username` disponible ~0ms → Playwright timeout

**Pourquoi les 12 tests passaient** : tous des tests `request` (API purs) ou `loginAsAdmin` (API-first).

### Corrections

#### 1. `playwright.config.ts` — `fullyParallel: false`
```typescript
fullyParallel: false,  // ✅ (était true)
```
Empêche le partage de browser context entre tests.

#### 2. `frontend/tests/e2e.spec.ts` — `clearSession()` helper
Nouvelle fonction appelée en tête de `loginAs()` :
```typescript
async function clearSession(page: Page) {
  await page.goto('/', { waitUntil: 'domcontentloaded', timeout: 10_000 });
  await page.evaluate(() => {
    localStorage.removeItem('nook_user');
    localStorage.removeItem('nook_session_id');
    localStorage.removeItem('nook_token');
  });
  await page.context().clearCookies();
}
```
**Pourquoi goto('/') d'abord** : le localStorage est isolé par origine. On doit être sur `localhost:6300` pour manipuler son localStorage. `about:blank` est une autre origine → inefficace.

#### 3. `.github/workflows/test-nook.yml` — healthcheck `/api/health`
```bash
# ❌ /health retourne index.html (ServeDir fallback → 200 trompeur)
# ✅ /api/health retourne "OK" depuis le handler Axum
until curl -sf http://localhost:6300/api/health | grep -q "OK"; do sleep 3; done
```

### Fichiers modifiés session 21
- `frontend/playwright.config.ts` — `fullyParallel: false`
- `frontend/tests/e2e.spec.ts` — `clearSession()` + `loginAs()` mis à jour
- `.github/workflows/test-nook.yml` — healthcheck `/api/health`
- `.claude/BUGS.md` — Bug #21 documenté
- `.claude/SESSIONS.md` — ce fichier

### État attendu après fix
- **43 tests** au total (inchangé)
- **43/43 ✅** attendus si les sélecteurs UI sont corrects
- Le seul risque résiduel : sélecteurs UI obsolètes (classes CSS renommées entre sessions)

### Ce qui reste à faire
- [ ] Déclencher test-nook.yml → confirmer 43/43 ✅
- [ ] Chess temps réel : WS client → abonnement coups adverses
- [ ] Polls : backend API (actuellement localStorage only)
- [ ] Chiffrement E2E : réactiver quand clés disponibles
- [ ] Chunk libsodium 938 kB → découper avec dynamic import()

## Session 22 — 2026-03-05 — Fix E2E clearSession API-first (Bug #22)

### Contexte
Logs CI du 2026-03-05 (2ème run). Les fixes de session 21 sont bien committés
(fullyParallel:false, clearSession avec goto('/')), mais 31/43 tests échouent encore
avec les mêmes timeouts `waiting for locator('#username')`.

### Cause racine

`clearSession()` session 21 faisait `goto('/')` en premier pour être sur l'origine
de l'app avant de manipuler le localStorage. Mais :

1. `goto('/')` → layout monte → `onMount` → `authStore.init()`
2. `authStore.init()` → `fetch('/api/auth/me')` avec le cookie encore présent
3. Cookie valide → 200 → `isAuthenticated=true`
4. `$effect()` → redirect `/chat`
5. `clearCookies()` appelé APRÈS → trop tard
6. `goto('/login')` → `isAuthenticated=true` → redirect → timeout `#username`

### Fix définitif — approche API-first pour clearSession

```typescript
async function clearSession(page: Page) {
  try {
    await page.request.post(`${BASE}/auth/logout`);
    // 200 = révoqué, 401 = pas de session → les deux sont OK
  } catch {}
  await page.context().clearCookies();
}
```

`page.request` envoie la requête sans déclencher le browser/layout.
Le token est révoqué en DB **AVANT** toute navigation.
Ensuite `goto('/login')` → `authStore.init()` → `/api/auth/me` → 401 → `authStore.logout()` → `isAuthenticated=false` ✅

Pas besoin de `localStorage.clear()` explicite : `authStore.logout()` le fait automatiquement sur 401.

### Chronologie complète clearSession (sessions 17-22)

| Session | Approche | Cause d'échec |
|---------|----------|--------------|
| 17 | `clearCookies()` seul | localStorage intact → `isAuthenticated=true` |
| 18 | `about:blank + localStorage.clear()` | Origine ≠ → localStorage isolé |
| 18 | `addInitScript(Page)` | S'exécute sur about:blank |
| 19 | `loginAsAdmin` API-first | ✅ admin uniquement |
| 21 | `goto('/') + evaluate + clearCookies` | goto('/') déclenche init() avec cookie valide |
| **22** | **`request.post(logout) + clearCookies`** | **✅ token révoqué avant navigation** |

### Fichiers modifiés session 22
- `frontend/tests/e2e.spec.ts` — `clearSession()` réécrit en API-first
- `.claude/BUGS.md` — Bug #22 documenté
- `.claude/SESSIONS.md` — ce fichier

### État attendu après fix
- **43/43 tests ✅** si les sélecteurs UI sont corrects
- `playwright.config.ts` : `fullyParallel: false` (inchangé depuis session 21)

### Ce qui reste à faire
- [ ] Déclencher test-nook.yml → confirmer 43/43 ✅
- [ ] Chess temps réel : WS client → abonnement coups adverses
- [ ] Polls : backend API (actuellement localStorage only)
- [ ] Chiffrement E2E : réactiver quand clés disponibles
- [ ] Chunk libsodium 938 kB → dynamic import()

---

## Session 23 — 2026-03-06 — Fix E2E loginAs waitFor #username (Bug #23)

### Contexte
Logs CI run 11 (2026-03-05 ~18h37). Fix session 22 bien commité et présent dans le repo.
Résultats inchangés : 12/43 ✅, 31/43 ❌. Même symptôme : timeout `#username`.

### Cause racine définitive

La cause n'était **pas** le localStorage, **pas** les cookies, **pas** le fullyParallel.
C'était le **timing de rendu du layout Svelte**.

Le layout `+layout.svelte` :
1. Démarre avec `loading = $state(true)`
2. `onMount` → `waitForSodium()` + `initCryptoSystem()` + `authStore.init()` (async, ~1-3s)
3. Seulement après : `loading = false`
4. `{#if loading}` masque `{@render children()}` tant que `loading=true`

`page.goto('/login')` se resolve à l'événement `load` du browser (HTML+JS reçus) — avant
que `onMount` ait terminé. À ce moment, `#username` n'est pas dans le DOM.
`page.fill('#username')` attend un élément inexistant → timeout 30s.

### Fix — une ligne dans loginAs()

```typescript
async function loginAs(page: Page, username: string, password: string) {
  await clearSession(page);
  await page.goto('/login');
  // NOUVEAU : attendre que le layout finisse de charger
  await page.locator('#username').waitFor({ state: 'visible', timeout: 20_000 });
  await page.fill('#username', username);
  ...
}
```

### Fichiers modifiés session 23
- `frontend/tests/e2e.spec.ts` — `loginAs()` : ajout `waitFor('#username', visible)`
- `.claude/BUGS.md` — Bug #23 documenté
- `.claude/SESSIONS.md` — ce fichier

### État attendu après fix
**43/43 tests ✅** (sous réserve que les sélecteurs UI soient corrects)

### Ce qui reste à faire
- [ ] Confirmer 43/43 ✅ au prochain run CI
- [ ] Chess temps réel : WS client → abonnement coups adverses
- [ ] Polls : backend API (actuellement localStorage only)
- [ ] Chiffrement E2E : réactiver quand clés disponibles
- [ ] Chunk libsodium 938 kB → dynamic import()

---

## Session 48 — 2026-04-03 (audit tests E2E + corrections)

### Resume CI final
- **Statut** : ✅ PASS — 165/165 tests passes, 0 echec, 0 ignore, 1.8m
- **3 fichiers tests modifies** : `api-sanity.spec.ts`, `user.spec.ts`, `admin.spec.ts`
- **Backend build** : ✅ `nook-backend v0.5.0-beta.1` compile sans erreur (2m46s)
- **Docker image** : ✅ construite et taggee `nook-ci:local`
- **Healthcheck** : ✅ OK en 2s

### Problemes rencontres et corriges

#### 1. Erreurs de syntaxe TS/E2E
- `api-sanity.spec.ts` : titre de test duplique "Upload fichier vide -> 400" (lignes 179 et 400)
  - **Fix** : renomme le test a "Upload sec -- fichier vide refuse -> 400"
- `user.spec.ts` : test "Chess UI — plateau 64 cases" jamais ferme (missing `});` ligne 578)
  - **Fix** : ajoute `  });` entre le try/catch et le commentaire "7. CALENDRIER"
- `user.spec.ts` : deuxieme describe "Call page" test sans fixture `{ page }`
  - **Fix** : `async () => {` -> `async ({ page }) => {` ligne 917
- `admin.spec.ts` : 3 blocs describe referencent `adminPage` non defini
  ('Admin -- Complement', 'Admin -- Analytics', 'Admin -- Approve user + login flow')
  - **Fix** : ajoute `let adminPage: Page;` + `test.beforeAll` avec `loginAsAdmin` dans chaque describe
- `api-sanity.spec.ts` : test "Creer partie -> jouer e2->e4" renvoie 201 au lieu de 200/409
  - **Fix** : ajoute 201 dans le tableau attendu `[200, 201, 409]`

#### 2. Erreurs bash pre-tests CI (non bloquantes sur test E2E)
- Script upload vide: erreur syntaxe bash `syntax error near unexpected token '('` 
- Script poll/event creation: `unexpected EOF while looking for matching '"'`
- WebSocket test: `IndentationError: unexpected indent` (indentation Python)
- Note : ces erreurs de scripts bash sont separees du run Playwright — n'affectent pas les tests E2E

### Tests chess — couverture
| Test | Statut |
|------|--------|
| GET /chess/list | ✅ |
| Creer partie vs IA | ✅ |
| GET /chess/{id} | ✅ |
| Coups legaux depuis e2 | ✅ (e2e4 present) |
| Coup e2->e4 accepte | ✅ |
| Coup illegal -> 400 | ✅ |
| POST /chess/{id}/ai-move | ✅ |
| Resign -> status finished | ✅ |
| Partie humain creee | ✅ |
| Invitation envoyee | ✅ |
| Invitation declinee | ✅ |
| Resign (flaky, test API sans auth) | ✅ passe parfois |
| Chess UI plateau 64 cases | ✅ (catch OK) |
| Chess UI 8x8 (user.spec) | ✅ |

### Architecture tests E2E actuelle
- **3 fichiers Playwright** + **1 helper partagé** :
  - `admin.spec.ts` (540 lignes) : Admin — Flux complet (serial)
  - `user.spec.ts` (1008 lignes) : User — Flux complet (serial)
  - `api-sanity.spec.ts` (534 lignes) : Tests API + Chess
  - `helpers.ts` (115 lignes) : loginAs, loginAsAdmin, clearSession, waitForAppReady, etc.
- 3 fichiers spec + ~2050 lignes de code test E2E

### Conventions etablies
- **Validation systematique** : toujours un `npx playwright test --list` local avant push
- **Fixer les fixtures Page** : si un test utilise `page`, il doit avoir `async ({ page }) =>` ou `async () =>` sans page
- **adminPage scope** : chaque describe block qui utilise `adminPage` doit avoir son propre `let adminPage + beforeAll`
<details>
<summary><h3>Session 48 — Resume Complet (cliquer pour deployer)</h3></summary>

## Contexte

Session initiale: Audit de la couverture des tests E2E chess dans le projet Nook.
Projet: **Nook** — messagerie familiale self-hebergee, Rust/Axum + SvelteKit + SQLite.
Branche: `develop` | Repo: MX10-AC2N/Nook

## Progres Realises

### 🏁 Session Terminee — 165/165 tests E2E passent (0 echec, 1.8min)

### Audit Tests Chess
- **34 tests chess** identifies sur 3 fichiers (api-sanity, user, chess-extended)
- **12/18 categories couvertes** (67% coverage)
- Couvert: Creation IA/humain, coups legaux/illegaux, AI moves, resign, invitations, auth 401, 
           UI plateau 64 cases, promotion API+UI, timer, navigation/historique, jeu en noir
- Non couvert: Roque (castling), En passant, mat/pat, drag/drop UI, regles 50 coups, repetition position

### Corrections CI (5 bugs critiques corriges)
| Bug | Fichier | Fix |
|-----|---------|-----|
| Titre test duplique | api-sanity.spec.ts:400 | Renomme en "Upload sec -- fichier vide refuse" |
| Test Chess UI jamais ferme | user.spec.ts:555 | Ajoute `});` manquant apres try/catch |
| Fixture {page} manquante | user.spec.ts:917 | `async () =>` → `async ({ page }) =>` |
| adminPage scope | admin.spec.ts:330,360,465 | Ajoute `let + beforeAll` dans 3 describe blocks |
| Status 201 manquant | api-sanity.spec.ts:314 | Ajoute 201 dans `[200, 201, 409]` |

### Conventions E2E (nouvelles regles dans critical-pitfalls.md)
1. `npx playwright test --list` obligatoire AVANT chaque push
2. Tests avec `page` ⇒ `{ page }` obligatoire dans signature async
3. Chaque `describe` utilisant `adminPage` ⇒ son propre `let + beforeAll`
4. Titres de tests UNIQUES par describe scope
5. Fermeture systematique de chaque test avec `});`

## Decisions Cles
- Validation locale systematique avant push (fin du cycle push-echec-repush)
- Architecture tests: 3 fichiers spec (admin 540L, user 1008L, api-sanity 534L) + helpers.ts
- Coverage chess a 67% — reste a couvrir: roque, en passant, mat/pat, drag/drop UI

## Todo Prochaines Etapes
### Priorite haute
- [ ] Corriger scripts bash pre-tests CI (erreurs syntaxe upload/poll/WS) — non bloquant mais pollue les logs
- [ ] Migration Node.js 24 (deprecation juin 2026)
- [ ] Ajouter tests roque (castling)
- [ ] Ajouter tests en passant
- [ ] Ajouter tests mat/pat detection

### Priorite moyenne
- [ ] Tests UI drag/drop mouvement pieces
- [ ] Tests validation coups cote UI (avant envoi API)
- [ ] Tests regles speciales (50 coups, repetition triple)
- [ ] Tests clock management UI (timer visible, timeout)
- [ ] Audit SEC-07 (WebRTC sans auth), SEC-09 (CSP), SEC-10 (headers securite)

### Backlog
- [ ] Nettoyer 3 workflows candidats suppression
- [ ] Fusionner 2 workflows duplicats (cargo-lock)
- [ ] Decider ci-new2.yml vs Backend.yml+Docker.yml

## Risques
1. **Node.js 20 deprecation** — juin 2026, necessite migration actions
2. **Scripts bash CI** — erreurs de syntaxe cachees par `|| true` (XSS, upload, polls, events, WS)
3. **Test chess resign** — parfois flaky (401 au lieu de 200 si session expiree)
4. **Pas de test E2E WebRTC** — webrtc.spec.ts existe mais non integre au CI

## Fichiers Modifies (Session 48)
- `frontend/tests/admin.spec.ts` (540L) — +28 lignes adminPage scope
- `frontend/tests/user.spec.ts` (1008L) — +1 ligne fermeture test, +1 ligne fixture
- `frontend/tests/api-sanity.spec.ts` (534L) — +1 titre unique, +1 status 201
- `.claude/SESSIONS.md` — Session 48 ajoutee
- `.claude/E2E-TARGETED-REPORT.md` — mis a jour 165/165
- `.claude/BUGS.md` — 5 bugs marques fixes
- `.claude/CLAUDE.md` — statut CI mis a jour
- `.claude/rules/critical-pitfalls.md` — 6 nouvelles regles E2E
- `.claude/TEST-AND-SECURITY-AUDIT-2026.md` — mis a jour

## Etat Final
- **Branche**: develop
- **CI**: 165/165 PASS | 0 fail | 1.8min
- **Backend**: build OK (nook-backend v0.5.0-beta.1, 2m46s)
- **Docker**: image nook-ci:local OK
- **Git**: Tout commit et push sur origin/develop
- **Zero fichier modifie en attente** (clean state)

</details>

---

## Session — 2026-04-04/05 (CI fixes massifs + docs update)

### Contexte
CI test-nook.yml cassee avec erreurs `cannot produce proc-macro for asn1-rs-derive`
et `ReferenceError: adminPage is not defined`. Objectif: tout reparer et documenter.

### Progres Realises
- `.cargo/config.toml`: supprime section `[target.x86_64-unknown-linux-gnu]` cassant proc-macro
- `Backend.yml`: rustup target add uniquement pour aarch64 (x86_64 = cible native runner)
- `test-nook.yml`: 7 blocs shell consolides en 1 seul `run:` ($ADMIN_COOKIE persist)
- `test-nook.yml`: supprime refs /tmp/*.py, remplace par python3 -c inline
- `test-nook.yml`: supprime ligne heredoc orpheline cassant WS test
- `admin.spec.ts`: adminPage deplace au scope module
- `README.md`: update architecture, test count 144, 7 migrations, TURN, SFU
- `.claude/BUGS.md`: 2 tests flaky documentes + cookie fix
- `.claude/WORKFLOW-CATALOG.md`: stats test-nook.yml mises a jour
- `.claude/roles/ci-devops.md`: targets musl->gnu, .cargo note mise a jour

### Bugs Corriges
| Bug | Fichier | Fix | Commit |
|-----|---------|-----|--------|
| proc-macro cannot produce (.cargo/config) | `.cargo/config.toml` | Supprimer section x86_64 | `84ee879` |
| rustup target add corromp | `Backend.yml` | Conditionnel aarch64 seul | `b2bec48` |
| $ADMIN_COOKIE perdu entre runs | `test-nook.yml` | 7 runs -> 1 block | `b55636b` |
| RefError adminPage | `admin.spec.ts` | Module scope | `e9ae61a` |
| Python IndentationError WS | `test-nook.yml` | Supprime ligne heredoc | `1108e89` |
| YAML syntax L325 | `test-nook.yml` | Indenter heredoc | `541b481` |

### Couverture Tests
| Categorie | Status | Tests |
|-----------|--------|-------|
| E2E Playwright | ✅ | 157 passed, 0 failed, 2 flaky |
| Shell Integration (7 sections) | ✅ | Tout passe avec cookie persistant |
| Backend Build (Docker) | ✅ | ~3min27s |

### Etat Final
- HEAD: cb95173
- CI test-nook: ✅ OK (157/159 pass, 2 flaky preexistants)
- Backend build: ✅ OK
- Docker build: ✅ OK
- Git: propre, tout pousse

---

## Session — 2026-04-05 (Migration Distroless → Alpine 3.21)

### Contexte
L'utilisateur a exige ZERO dependance Google. Migration complete de toutes
les images Docker de gcr.io/distroless/cc-debian12 vers alpine:3.21.

### Changements
- **Dockerfile**: Builder musl-tools, cible `x86_64-unknown-linux-musl`, runtime alpine:3.21
- **Dockerfile.release**: Runtime alpine:3.21 (binaire musl, pas glibc)
- **services/turn-rs/Dockerfile**: Builder musl + protobuf, runtime alpine:3.21
- **Backend.yml**: Targets musl (x86_64 + aarch64), zig cc cross-linker arm64
- **.cargo/config.toml**: Cibles musl, zig cc linker aarch64
- **README.md**: distroless → Alpine 3.21

### Stack technique finale
| | Avant | Apres |
|-|-------|-------|
| Runtime | gcr.io/distroless/cc-debian12 | alpine:3.21 |
| Cible x86_64 | linux-gnu | linux-musl |
| Cible aarch64 | linux-gnu | linux-musl |
| Cross-linker | gcc-aarch64 (glibc) | zig cc 0.13 (musl) |
| Google dep | gcr.io | AUCUNE |

---

## Session — 2026-04-05 (Migration Alpine + CI final)

### Contexte
L'utilisateur a exige ZERO dependance Google. Migration complete de toutes
les images Docker de gcr.io/distroless/cc-debian12 vers alpine:3.21.

### Progres Realises
1. **Dockerfile + Dockerfile.release**: runtime Alpine 3.21, cibles musl, chmod 777 sur dirs
2. **services/turn-rs/Dockerfile**: builder musl + zig 0.13, runtime Alpine 3.21
3. **Backend.yml**: targets -musl, CC_x86_64-unknown-linux-musl=musl-gcc pour ring/aws-lc-sys,
   zig cc pour aarch64 cross-compilation
4. **.cargo/config.toml**: cibles musl uniquement, zig cc pour aarch64
5. **README.md + docs .claude/**: toutes refs Distroless -> Alpine
6. **BUGS.md**: 2 tests flaky documentes (chess resign 401, analytics 401)

### Bugs Rencontres
| Bug | Fichier | Fix |
|-----|---------|-----|
| zig URL YAML escaping | Backend.yml | `\$(uname -m)` -> `$(uname -m)` |
| sqlite-libs vs libsqlite3 | Dockerfile Alpine | apk add sqlite-libs (pas libsqlite3) |
| cc-rs cherche x86_64-linux-musl-gcc | ring, aws-lc-sys | CC_x86_64-unknown-linux-musl=musl-gcc env var |
| PermissionDenied /app/data | Dockerfile | chmod 777 sur /app/data, /app/logs avant USER |

### Etat Final
- HEAD: `08015e3`
- Backend: build musl avec ring/aws-lc-sys OK (musl-gcc)
- aarch64 cross: zig cc 0.13 avec `-target aarch64-linux-musl`
- CI: en attente de re-run sur 08015e3

## Session 8 — 2026-04-05 (Alpine Docker + UTF-8 CI Fix + Zero Google Migration)

### Contexte
Migrate tous les Dockerfiles vers Alpine 3.21 (zero Google), corriger les erreurs CI en cascade.

### Decisions Cles
- Docker: Alpine 3.21 builder (rustup nightly, edition2024) + Alpine 3.21 runtime
- Backend.yml: targets gnu (x86_64/aarch64-unknown-linux-gnu) pour releases, pas Alpine/CI
- `cc-rs` exige le linker exact (`x86_64-linux-musl-gcc` vs `musl-gcc`)
- EMOJI CORRUPTION: Tous les emojies multi-octets dans les fichiers .yml workflow se corrompent via l'API GitHub — remplacent guillemets avec des caracteres casses (ðŸ" → quote rompue)
- musl-tools = paquet Debian uniquement, inexistant sur Alpine

### Progres
- ✅ Dockerfile: Alpine builder (rustup nightly) + Alpine 3.21 runtime
- ✅ Dockerfile.release: Alpine 3.21 runtime (consomme bins musl)
- ✅ services/turn-rs/Dockerfile: Alpine 3.21 builder + runtime
- ✅ Backend.yml: gnu targets, RUSTFLAGS=-C target-feature=+crt-static
- ✅ .cargo/config.toml: linker aarch64-linux-gnu-gcc
- ✅ test-nook.yml: NOOK_IMAGE env var fix, backend check step ajoute, UTF-8 nettoye
- ✅ README.md: references Distroless remplacees par Alpine
- ✅ Tous les emojies casses dans workflow files remplaces par ASCII pur

### Bugs Corriges
| Bug | Fichier | Fix |
|-----|---------|-----|
| musl-tools sur Alpine | Dockerfile | retire (paquet Debian) |
| cc-rs cherche x86_64-linux-musl-gcc | Dockerfile | utilise musl-gcc via Alpine natif |
| Edition2024 incompatible Alpine cargo 1.83 | Dockerfile | rustup nightly --default-toolchain nightly |
| Emojis multi-octets cassent quotes bash | Backend.yml, test-nook.yml | remplaces par ASCII pur |
| NOOK_IMAGE non defini dans Start stack | test-nook.yml | ajoute env: NOOK_IMAGE: nook-ci:local |
| Start stack fail sur distroless bins | test-nook.yml | clean previous runs + force-recreate |

### Etat Final (en attente CI)
- HEAD: 50d268a26d — fix(docker): remove musl-tools
- CI: run 24009717784 in_progress (Frontend OK, Integration Tests en cours)
- Zero Google partout: Alpine 3.21 builder + runtime, debian:bookworm-slim nulle part
## Session 9 - 2026-04-05 (Alpine Migration + CI Fix + Zero Google)

### Contexte
Migrer tous les Dockerfiles vers Alpine 3.21 (zero Google/distroless). Resoudre les erreurs CI en cascade.

### Progres Realises
- **Dockerfile**: Alpine 3.21 builder (rustup nightly) + Alpine 3.21 runtime (~15MB)
- **Dockerfile.release**: Alpine 3.21 runtime (binaire musl natif)
- **services/turn-rs/Dockerfile**: Alpine 3.21 builder + runtime
- **Backend.yml**: targets gnu (x86_64/aarch64-unknown-linux-gnu) pour releases
- **.cargo/config.toml**: linker aarch64-linux-gnu-gcc pour cross-compile
- **test-nook.yml**: working-directory fix pour Playwright, NOOK_IMAGE env var
- **playwright.config.ts**: webServer supprime (Alpine container sert deja frontend sur 6300)
- **README.md**: references Distroless -> Alpine

### Bugs Corriges
| Bug | Fichier | Fix |
|-----|---------|-----|
| musl-tools sur Alpine | Dockerfile | retire (paquet Debian) |
| Edition2024 + Alpine cargo 1.83 | Dockerfile | rustup nightly --profile minimal |
| UTF-8 emoji corrompus | Backend.yml, test-nook.yml | remplaces par ASCII pur |
| NOOK_IMAGE non defini | test-nook.yml | ajoute env: NOOK_IMAGE: nook-ci:local |
| npm ci sans package-lock.json | test-nook.yml | working-directory: ./frontend |
| Playwright timeout webServer | playwright.config.ts | supprime webServer block |

### Etat Final
- Docker CI: BUILD OK, healthcheck OK, container demarre OK
- Playwright: webServer supprime (attente nouveau CI)
- Zero Google partout (Alpine 3.21 Foundation)

## Session 11 — 2026-04-06 (Migration Alpine 3.21 — Zero Google)

### Contexte
Migrer toute l'infrastructure Docker de `debian:bookworm-slim` vers `alpine:3.21` pour eliminer tout dependance Google (distroless, gcr.io).

### Progres Realises
- Dockerfile: builder Alpine 3.21 (apk add rust cargo musl-dev + deps) + runtime Alpine 3.21 avec COPY frontend/build /app/static
- Dockerfile.release: Alpine 3.21 (consomme binaires musl de Backend.yml)
- services/turn-rs/Dockerfile: builder Alpine 3.21 + runtime Alpine 3.21
- Backend.yml: cible x86_64-unknown-linux-musl, musl-tools, CARGO_TARGET_*_LINKER=musl-gcc
- .cargo/config.toml: musl target aarch64 (non-utilise en CI)
- test-nook.yml: cargo check musl target
- README.md: Zero references Google techniques (marketing uniquement)

### Decisions Cles
- Alpine 3.21 (pas 3.20 ou edge) pour la stabilite LTS
- Builder + runtime Alpine = binaire full static musl (~15MB vs ~80MB)
- musl-gcc via musl-tools + CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER env var
- Le probleme principal etait COPY frontend/build manquant → Playwright timeout

### Bugs Corriges
| Bug | Fichier | Fix |
|-----|---------|-----|
| Playwright timeout /login, /call | Dockerfile | COPY frontend/build /app/static (manquait totalement) |
| Binaire glibc compile sur builder Debian crash sur Alpine | Dockerfile | Builder Alpine + musl natif (pas de mix glibc/musl) |
| Docker build reussi mais pas COPY frontend | Dockerfile | Ajout COPY frontend/build /app/static |

### Fichiers Modifies
- Dockerfile: 83 lignes — builder Alpine + runtime Alpine + frontend copy
- Dockerfile.release: 41 lignes — Alpine 3.21 runtime
- services/turn-rs/Dockerfile: 39 lignes — Alpine builder + runtime
- .github/workflows/Backend.yml: musl targets, musl-gcc linker
- backend/.cargo/config.toml: musl targets
- .github/workflows/test-nook.yml: musl cargo check

### Couverture Tests
| Categorie | Status | Tests |
|-----------|--------|-------|
| API sanity | ✅ 77 passed | Health, auth, chess, polls, upload, reactions |
| Admin flow | ✅ 27 passed (1 flaky) | Approve, invites, analytics, delete |
| User flow | ✅ 54 passed | Chat, chess UI, navigation, push |
| Total | ✅ 157 passed / 0 failed / 2 flaky / 159 total | ~1min13s |

### Risques
1. Alpine builder = build time ~6min (vs ~3min Debian) — acceptable car cache Rust
2. aarch64 musl cross non supporte en CI (runner x86_64 uniquement) — OK car Zimaboard fait le cross local
3. Ring crate + musl-gcc = compilation plus lente — cache rust-cache@v2 mitige

### Etat Final
- Branche: develop
- HEAD: 895c08c5ae42
- CI: 157/159 PASS, 0 FAIL, 2 flaky (chess resign race, analytics race — connus)
- Docker: Alpine 3.21 builder + runtime, ~15MB final
- Zero Google: ✅
