# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-04-26 17:57 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** |
| **Tests passés** | 74 |
| **Tests échoués** | 2 |
| **Tests flaky** | 0 |
| **Tests ignorés** | 136 |
| **Total** | 212 |
| **Durée totale** | 56.5s |
| **Branche** | `develop` |
| **Commit** | [`de4d5a1`](https://github.com/MX10-AC2N/Nook/commit/de4d5a18fe1633f8da6593d92986b24b0abd3ac0) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/24962901484) |

---

## 🗂️ Suites de tests

| Suite | Fichier | Périmètre |
|-------|---------|-----------|
| **Sanité API** | `api-sanity.spec.ts` | 401/403 sur toutes les routes protégées |
| **Admin** | `admin.spec.ts` | Login, change-pwd, membres, inscription→approbation, invitations, analytics, isolation |
| **User** | `user.spec.ts` | Auth, chat, réactions, upload, polls, chess, calendar, settings, E2EE, push, navigation |

---

## ⏱️ Top 10 Tests les plus lents (Performance Hotspots)

| Rang | Test | Durée | Fichier |
|------|------|-------|---------|
| 1 | /call/fake-id avec auth → page charge | 30.2s | `?` |
| 2 | /call/fake-id sans auth → redirige vers /login | 20.2s | `?` |
| 3 | Créer partie → jouer e2→e4 → IA répond | 0.1s | `?` |
| 4 | User change pwd autre user → 403 (integration) | 0.0s | `?` |
| 5 | Mot de passe 8 chars → accepte | 0.0s | `?` |
| 6 | 8 chars → accepte | 0.0s | `?` |
| 7 | User normal change pwd autre user → 403 | 0.0s | `?` |
| 8 | Upload fichier texte → file_id, puis download OK | 0.0s | `?` |
| 9 | Download fichier inexistant → 404 | 0.0s | `?` |
| 10 | Rename conversation → 200 | 0.0s | `?` |

---

## 📋 Résultats par catégorie

### ❌ **api-sanity.spec.ts** — 74/76 passés · ❌ 2 failed

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ❌ | /call/fake-id avec auth → page charge | 30.2s  +1 | |
| ❌ | /call/fake-id sans auth → redirige vers /login | 20.2s  +1 | |
| ✅ | Créer partie → jouer e2→e4 → IA répond | 0.1s | |
| ✅ | User change pwd autre user → 403 (integration) | 0.0s | |
| ✅ | Mot de passe 8 chars → accepte | 0.0s | |
| ✅ | 8 chars → accepte | 0.0s | |
| ✅ | User normal change pwd autre user → 403 | 0.0s | |
| ✅ | Upload fichier texte → file_id, puis download OK | 0.0s | |
| ✅ | Download fichier inexistant → 404 | 0.0s | |
| ✅ | Rename conversation → 200 | 0.0s | |
| ✅ | Upload sec -- fichier vide refuse → 400 (second block) | 0.0s | |
| ✅ | Download inexistant → 404 | 0.0s | |
| ✅ | Envoyer message → 200 | 0.0s | |
| ✅ | Upload fichier vide → 400 | 0.0s | |
| ✅ | Envoyer message → 200, récupérer → contient message | 0.0s | |
| ✅ | Upload fichier texte → 200 | 0.0s | |
| ✅ | Upload → Download end-to-end | 0.0s | |
| ✅ | Rename conversation → 200 (second block) | 0.0s | |
| ✅ | GET /api/health → "OK" | 0.0s | |
| ✅ | POST /api/upload/chat sans auth → 401 | 0.0s | |
| ✅ | GET /auth/me → 401 | 0.0s | |
| ✅ | GET /push/vapid-public-key → 200 (route publique, pas d'auth requise) | 0.0s | |
| ✅ | POST /auth/logout → 401 | 0.0s | |
| ✅ | 1 char → 400 | 0.0s | |
| ✅ | POST /polls/fake-id/vote → 401 | 0.0s | |
| ✅ | POST /auth/change-password → 401 | 0.0s | |
| ✅ | POST /auth/public-key → 401 | 0.0s | |
| ✅ | GET /conversations/default_global → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/messages → 401 | 0.0s | |
| ✅ | 5 chars → 400 | 0.0s | |
| ✅ | GET /auth/public-keys?conversation_id=default_global → 401 | 0.0s | |
| ✅ | GET /conversations → 401 | 0.0s | |
| ✅ | POST /conversations → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/messages → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/participants → 401 | 0.0s | |
| ✅ | GET /polls/fake-id → 401 | 0.0s | |
| ✅ | POST /polls/fake-id/close → 401 | 0.0s | |
| ✅ | DELETE /polls/fake-id → 401 | 0.0s | |
| ✅ | GET /users → 401 | 0.0s | |
| ✅ | POST /invites → 401 | 0.0s | |
| ✅ | Chess coup illégal → 400 | 0.0s | |
| ✅ | GET /conversations/default_global/participants → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/leave → 401 | 0.0s | |
| ✅ | PATCH /conversations/default_global/rename → 401 | 0.0s | |
| ✅ | GET /download/fake-id-000 → 401 | 0.0s | |
| ✅ | GET /events → 401 | 0.0s | |
| ✅ | POST /events → 401 | 0.0s | |
| ✅ | DELETE /events/fake-id → 401 | 0.0s | |
| ✅ | GET /polls → 401 | 0.0s | |
| ✅ | POST /polls → 401 | 0.0s | |
| ✅ | GET /chess/list → 401 | 0.0s | |
| ✅ | POST /chess/create → 401 | 0.0s | |
| ✅ | GET /chess/fake-id → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/move → 401 | 0.0s | |
| ✅ | GET /chess/fake-id/moves?from=e2 → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/resign → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | DELETE /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | POST /user/update → 401 | 0.0s | |
| ✅ | GET /users/available → 401 | 0.0s | |
| ✅ | GET /push/preferences → 401 | 0.0s | |
| ✅ | GET /users/pending → 401 | 0.0s | |
| ✅ | POST /users/approve → 401 | 0.0s | |
| ✅ | GET /invites → 401 | 0.0s | |
| ✅ | POST /invites/delete → 401 | 0.0s | |
| ✅ | GET /analytics → 401 | 0.0s | |
| ✅ | Mot de passe 1 char → 400 | 0.0s | |
| ✅ | Mot de passe 5 chars → 400 | 0.0s | |
| ✅ | Mot de passe 7 chars → 400 | 0.0s | |
| ✅ | Chess resign → status finished | 0.0s | |
| ✅ | GET /chess/invitations → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/ai-move → 401 | 0.0s | |
| ✅ | Modifier message → 200 | 0.0s | |
| ✅ | Supprimer message → 200/204 | 0.0s | |
| ✅ | Lister messages → contient le message modifié | 0.0s | |

### ✅ **admin-ui.spec.ts** — 0/6 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Admin panel accessible | N/A | |
| ⏭️ | Pending users list (if any) | N/A | |
| ⏭️ | Invite management | N/A | |
| ⏭️ | User list and search | N/A | |
| ⏭️ | API: Get pending users | N/A | |
| ⏭️ | API: Create and delete invite | N/A | |

### ✅ **admin.spec.ts** — 0/25 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Admin — page /admin chargée avec header | N/A | |
| ⏭️ | Admin — 3 onglets visibles | N/A | |
| ⏭️ | GET /auth/me avec session admin → role=admin | N/A | |
| ⏭️ | GET /users → liste complète (admin) | N/A | |
| ⏭️ | GET /users/pending → 200 | N/A | |
| ⏭️ | Onglet "Membres" → users visibles dans UI | N/A | |
| ⏭️ | Flux inscription : register → pending → approve → connecté | N/A | |
| ⏭️ | POST /invites → génère un invite_link valide | N/A | |
| ⏭️ | GET /invites → liste non vide | N/A | |
| ⏭️ | POST /invites/delete → supprime une invitation | N/A | |
| ⏭️ | GET /invite/validate?token=xxx → valide le token | N/A | |
| ⏭️ | Admin UI — invitation générée visible dans l'interface | N/A | |
| ⏭️ | GET /analytics → tous les champs requis | N/A | |
| ⏭️ | Page /admin/analytics → stat-cards + 2 charts | N/A | |
| ⏭️ | Admin — DELETE /polls/{id} → 200 | N/A | |
| ⏭️ | GET /analytics avec user normal → 403 | N/A | |
| ⏭️ | GET /users/pending avec user normal → 403 | N/A | |
| ⏭️ | Page /admin → non accessible pour user normal | N/A | |
| ⏭️ | Admin — DELETE /users/{id} → supprime un utilisateur | N/A | |
| ⏭️ | Admin — approve + login after approve → accès complet | N/A | |
| ⏭️ | Admin — analytics contient toutes les sections | N/A | |
| ⏭️ | DELETE /users/{id} → supprime un utilisateur | N/A | |
| ⏭️ | GET /analytics → contient user_count, message_count | N/A | |
| ⏭️ | GET /analytics sans auth → 401 | N/A | |
| ⏭️ | Register + Approve + Login → accès complet | N/A | |

### ✅ **call-ui.spec.ts** — 0/5 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Navigate to call page (audio) | N/A | |
| ⏭️ | Navigate to call page (video) | N/A | |
| ⏭️ | Call controls visible (mic/cam toggles) | N/A | |
| ⏭️ | Call banner appears on chat page when call active | N/A | |
| ⏭️ | WebSocket connection for signaling | N/A | |

### ✅ **chat-ui-advanced.spec.ts** — 0/6 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | @Mentions — dropdown appears on @ | N/A | |
| ⏭️ | Send message + Edit + Delete | N/A | |
| ⏭️ | Reactions — hover + click emoji | N/A | |
| ⏭️ | File upload button opens dialog | N/A | |
| ⏭️ | Scroll to load more messages | N/A | |
| ⏭️ | Typing indicator appears | N/A | |

### ✅ **chat-ui.spec.ts** — 0/5 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Login + navigate to chat | N/A | |
| ⏭️ | Header is compact (< 60px) | N/A | |
| ⏭️ | Send emoji-only message (should be large) | N/A | |
| ⏭️ | Reaction picker opens on hover + click | N/A | |
| ⏭️ | Input stays enabled while sending | N/A | |

### ✅ **e2ee-chat.spec.ts** — 0/5 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | E2EE: crypto store initializes | N/A | |
| ⏭️ | E2EE: Send encrypted message | N/A | |
| ⏭️ | E2EE: Refresh preserves decrypted messages | N/A | |
| ⏭️ | E2EE: Key exchange UI (if available) | N/A | |
| ⏭️ | API: Check encryption status | N/A | |

### ✅ **events-ui.spec.ts** — 0/5 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Events UI: Navigate to calendar | N/A | |
| ⏭️ | Events UI: Create event button | N/A | |
| ⏭️ | Events UI: List events | N/A | |
| ⏭️ | API: List events | N/A | |
| ⏭️ | API: Create and delete event | N/A | |

### ✅ **p2p-file-transfer.spec.ts** — 0/5 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Upload button visible in chat | N/A | |
| ⏭️ | File size > 50MB shows P2P required message | N/A | |
| ⏭️ | Group conversation > 50MB file shows restriction message | N/A | |
| ⏭️ | P2P transfer UI elements exist | N/A | |
| ⏭️ | File type validation (SVG XSS prevention) | N/A | |

### ✅ **polls-ui.spec.ts** — 0/4 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Polls UI: Create poll button | N/A | |
| ⏭️ | Polls UI: Display polls in chat | N/A | |
| ⏭️ | API: List polls | N/A | |
| ⏭️ | API: Create and vote on poll | N/A | |

### ✅ **push-notifications.spec.ts** — 0/4 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | GET /api/push/vapid-public-key → 200 (public route) | N/A | |
| ⏭️ | GET /api/push/preferences sans auth → 401 | N/A | |
| ⏭️ | Push subscription flow (mocked) | N/A | |
| ⏭️ | Push preferences API | N/A | |

### ✅ **push-test.spec.ts** — 0/1 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Test Service Worker et Push Notifications | N/A | |

### ✅ **user.spec.ts** — 0/65 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⏭️ | Login e2e_ci → redirigé vers /chat | N/A | |
| ⏭️ | GET /auth/me → username=e2e_ci | N/A | |
| ⏭️ | Login invalide → reste sur /login | N/A | |
| ⏭️ | GET /conversations → default_global présente | N/A | |
| ⏭️ | GET /conversations/default_global → détail de la conv | N/A | |
| ⏭️ | GET /conversations/default_global/participants → e2e_ci présent | N/A | |
| ⏭️ | Chat UI — sidebar et envoi message | N/A | |
| ⏭️ | GET /conversations/default_global/messages → messages récupérés | N/A | |
| ⏭️ | POST /conversations → créer un groupe de test | N/A | |
| ⏭️ | GET /users/available → liste des membres disponibles | N/A | |
| ⏭️ | Réactions — POST emoji valide 👍 → counts mis à jour | N/A | |
| ⏭️ | Réactions — POST emoji non autorisé 🦄 → 400 | N/A | |
| ⏭️ | Réactions — UPSERT : 👍 → ❤️ remplace sans doublon | N/A | |
| ⏭️ | Réactions — DELETE → my_emoji null | N/A | |
| ⏭️ | Réactions — GET → structure {message_id, counts, my_emoji} | N/A | |
| ⏭️ | Réactions — message inexistant → 404 | N/A | |
| ⏭️ | Réactions — cycle complet via API | N/A | |
| ⏭️ | Upload — fichier texte → file_id, url=/api/download/, download OK | N/A | |
| ⏭️ | Download — id inexistant → 404 | N/A | |
| ⏭️ | GET /polls → tableau de sondages | N/A | |
| ⏭️ | Polls — cycle complet : créer → voter → changer → double vote → fermer → vote fermé | N/A | |
| ⏭️ | Polls UI — créer sondage via formulaire → visible dans liste | N/A | |
| ⏭️ | GET /chess/list → 200 | N/A | |
| ⏭️ | Chess — créer vs IA, coups légaux, coup légal e2→e4, coup illégal → 400 | N/A | |
| ⏭️ | Chess — POST /chess/{id}/ai-move → 200 | N/A | |
| ⏭️ | Chess — POST /chess/{id}/resign → 200 | N/A | |
| ⏭️ | Chess — invitations : créer, inviter, lister, décliner | N/A | |
| ⏭️ | Chess UI — plateau 64 cases + sélection case + coup via UI | N/A | |
| ⏭️ | Calendar — GET /events → 200 | N/A | |
| ⏭️ | Calendar — POST /events → crée et DELETE /events/{id} → supprime | N/A | |
| ⏭️ | Calendar UI — page, grille et bouton ajouter visibles | N/A | |
| ⏭️ | Settings UI — 3 onglets navigables | N/A | |
| ⏭️ | Settings — changement de thème (clic → sélectionné) | N/A | |
| ⏭️ | POST /user/update → mise à jour du nom | N/A | |
| ⏭️ | Navigation /chat → accessible sans erreur | N/A | |
| ⏭️ | Navigation /calendar → accessible sans erreur | N/A | |
| ⏭️ | Navigation /chess → accessible sans erreur | N/A | |
| ⏭️ | Navigation /polls → accessible sans erreur | N/A | |
| ⏭️ | Navigation /settings → accessible sans erreur | N/A | |
| ⏭️ | Navigation /help → accessible sans erreur | N/A | |
| ⏭️ | Navigation /events → accessible sans erreur | N/A | |
| ⏭️ | E2EE — POST /auth/public-key → enregistre la clé | N/A | |
| ⏭️ | E2EE — GET /auth/public-keys → objet avec clés des membres | N/A | |
| ⏭️ | Push — GET /push/vapid-public-key → 200 | N/A | |
| ⏭️ | Push — GET /push/preferences → prefs par défaut | N/A | |
| ⏭️ | Push — POST /push/preferences → mise à jour | N/A | |
| ⏭️ | Push — POST /push/subscribe → 200 | N/A | |
| ⏭️ | Avatar — composant visible avec initiales dans le chat | N/A | |
| ⏭️ | Settings — section avatar visible avec grille d'options | N/A | |
| ⏭️ | Calendar — switcher vue Mois/Semaine/Jour visible | N/A | |
| ⏭️ | Chess — sélection pièce → coups légaux visibles (dots) | N/A | |
| ⏭️ | Logout UI → redirigé vers /login | N/A | |
| ⏭️ | Calendar — drag-drop: PUT /events/{id} change date | N/A | |
| ⏭️ | Chess — PGN notation via move_history | N/A | |
| ⏭️ | Analytics — chart.js lazy loaded | N/A | |
| ⏭️ | Flood /auth/login × 20 depuis même IP → au moins un 429 | N/A | |
| ⏭️ | /call/default_global → page charge avec titres | N/A | |
| ⏭️ | /call/default_global → page contient contenu call | N/A | |
| ⏭️ | /call/[id] session → page appel chargee (sans auth first) | N/A | |
| ⏭️ | Créer partie vs IA (facile) → game_id | N/A | |
| ⏭️ | Chess — UI plateau 8x8 (64 cases) | N/A | |
| ⏭️ | Chess — coup légal e2→e4 | N/A | |
| ⏭️ | Chess — coup illégal → 400 | N/A | |
| ⏭️ | Chess — coups légaux depuis e2 → contient e3 et e4 | N/A | |
| ⏭️ | Chess — resign → status finished | N/A | |

---

## 📁 Résultats par fichier de test

| Fichier | ✅ Passés | ❌ Échoués | ⚠️ Flaky | Total |
|---------|-----------|-------------|-----------|-------|
| ❌ `unknown` | 74 | 2 | 0 | 212 |

---

## ❌ Échecs détaillés

> 2 test(s) en échec

### Échec 1 — `/call/fake-id sans auth → redirige vers /login`

**Suite :** `api-sanity.spec.ts > Sécurité — Call page access`
**Durée :** 20.2s

**Message :**
```
TimeoutError: page.waitForURL: Timeout 10000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================

  281 |     const page = await browser.newPage();
  282 |     await page.goto('http://localhost:6300/call/fake-id');
> 283 |     await page.waitForURL(/login/, { timeout: 10000 });
      |                ^
  284 |     expect(page.url()).toContain('login');
  285 |   });
  286 |
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:283:16
```

**Message :**
```
TimeoutError: page.waitForURL: Timeout 10000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================

  281 |     const page = await browser.newPage();
  282 |     await page.goto('http://localhost:6300/call/fake-id');
> 283 |     await page.waitForURL(/login/, { timeout: 10000 });
      |                ^
  284 |     expect(page.url()).toContain('login');
  285 |   });
  286 |
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:283:16
```

### Échec 2 — `/call/fake-id avec auth → page charge`

**Suite :** `api-sanity.spec.ts > Sécurité — Call page access`
**Durée :** 30.2s

**Message :**
```
TimeoutError: page.waitForSelector: Timeout 15000ms exceeded.
Call log:
  - waiting for locator('input[name="username"], input[type="text"]') to be visible


  288 |     const page = await browser.newPage();
  289 |     await page.goto('http://localhost:6300/login');
> 290 |     await page.waitForSelector('input[name="username"], input[type="text"]', { state: 'visible', timeout: 15000 });
      |                ^
  291 |     await page.fill('input[name="username"], input[type="text"]', 'e2e_ci');
  292 |     await page.fill('input[name="password"], input[type="password"]', 'E2eTest123!');
  293 |     await page.click('button[type="submit"]');
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:290:16
```

**Message :**
```
TimeoutError: page.waitForSelector: Timeout 15000ms exceeded.
Call log:
  - waiting for locator('input[name="username"], input[type="text"]') to be visible


  288 |     const page = await browser.newPage();
  289 |     await page.goto('http://localhost:6300/login');
> 290 |     await page.waitForSelector('input[name="username"], input[type="text"]', { state: 'visible', timeout: 15000 });
      |                ^
  291 |     await page.fill('input[name="username"], input[type="text"]', 'e2e_ci');
  292 |     await page.fill('input[name="password"], input[type="password"]', 'E2eTest123!');
  293 |     await page.click('button[type="submit"]');
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:290:16
```

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

## 🖼️ Rapport HTML Playwright

> Le rapport HTML complet est disponible en artifact GitHub Actions.
>
> - **Nom de l'artifact :** `playwright-report`
> - **URL du run :** [https://github.com/MX10-AC2N/Nook/actions/runs/24962901484](https://github.com/MX10-AC2N/Nook/actions/runs/24962901484)
> - **Chemin local (CI) :** `frontend/playwright-report/`

Pour examiner visuellement les échecs :
1. Télécharger l'artifact `playwright-report` depuis le [run CI](https://github.com/MX10-AC2N/Nook/actions/runs/24962901484)
2. Ouvrir `index.html` dans un navigateur
3. Utiliser l'interface pour explorer les traces et screenshots

---

## 🔍 Couverture fonctionnelle

| Domaine | Endpoints / Fonctionnalités | Couverture |
|---------|----------------------------|-----------|
| **Auth** | login, logout, /me, change-pwd, register→approve | ✅ Complet |
| **Conversations** | GET/POST conv, messages, participants, rename | ✅ Complet |
| **Réactions** | POST/DELETE/GET, UPSERT, UI picker→pill | ✅ Complet |
| **Upload/Download** | upload chat, download, 401/404 | ✅ Complet |
| **Polls** | CRUD, vote, UPSERT, double vote, fermeture, vote fermé | ✅ Complet |
| **Chess** | créer, coups légaux/illégaux, IA, resign, invitations, UI plateau | ✅ Complet |
| **Calendrier** | GET/POST/DELETE événements, UI grille | ✅ Complet |
| **Settings** | profil, sécurité, apparence, update nom | ✅ Complet |
| **Admin** | users, pending, approve, invites, delete, analytics | ✅ Complet |
| **E2EE** | register/get public keys | ✅ Complet |
| **Push** | subscribe, preferences, vapid-key | ✅ Complet |
| **Sécurité** | ~47 routes 401, 403 admin, rate limit flood | ✅ Complet |
| **Navigation** | 7 routes accessibles sans erreur | ✅ Complet |

---

*Rapport généré par `scripts/generate-test-report.py` — 2026-04-26 17:57 UTC*
