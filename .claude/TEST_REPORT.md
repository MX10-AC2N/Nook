# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-04-03 05:23 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** |
| **Tests passés** | 137 |
| **Tests échoués** | 6 |
| **Tests ignorés** | 20 |
| **Total** | 163 |
| **Durée** | 4.0m 51s |
| **Branche** | `develop` |
| **Commit** | [`unknown`](https://github.com/MX10-AC2N/Nook/commit/unknown) |
| **Run CI** | [Voir le run complet](#) |

---

## 🗂️ Suites de tests

| Suite | Fichier | Périmètre |
|-------|---------|-----------|
| **Sanité API** | `api-sanity.spec.ts` | 401/403 sur toutes les routes protégées |
| **Admin** | `admin.spec.ts` | Login, change-pwd, membres, inscription→approbation, invitations, analytics, isolation |
| **User** | `user.spec.ts` | Auth, chat, réactions, upload, polls, chess, calendar, settings, E2EE, push, navigation |

---

## 📋 Résultats par suite

### ✅ api-sanity.spec.ts — 75/76 passés

| Statut | Test | Durée |
|--------|------|-------|
| ✅ | GET /api/health → "OK" | 0s |
| ✅ | GET /push/vapid-public-key → 200 (route publique, pas d'auth requise) | 0s |
| ✅ | GET /auth/me → 401 | 0s |
| ✅ | POST /auth/logout → 401 | 0s |
| ✅ | POST /auth/change-password → 401 | 0s |
| ✅ | POST /auth/public-key → 401 | 0s |
| ✅ | GET /auth/public-keys?conversation_id=default_global → 401 | 0s |
| ✅ | GET /conversations → 401 | 0s |
| ✅ | POST /conversations → 401 | 0s |
| ✅ | GET /conversations/default_global → 401 | 0s |
| ✅ | GET /conversations/default_global/messages → 401 | 0s |
| ✅ | POST /conversations/default_global/messages → 401 | 0s |
| ✅ | GET /conversations/default_global/participants → 401 | 0s |
| ✅ | POST /conversations/default_global/participants → 401 | 0s |
| ✅ | POST /conversations/default_global/leave → 401 | 0s |
| ✅ | PATCH /conversations/default_global/rename → 401 | 0s |
| ✅ | GET /download/fake-id-000 → 401 | 0s |
| ✅ | GET /events → 401 | 0s |
| ✅ | POST /events → 401 | 0s |
| ✅ | DELETE /events/fake-id → 401 | 0s |
| ✅ | GET /polls → 401 | 0s |
| ✅ | POST /polls → 401 | 0s |
| ✅ | GET /polls/fake-id → 401 | 0s |
| ✅ | POST /polls/fake-id/vote → 401 | 0s |
| ✅ | POST /polls/fake-id/close → 401 | 0s |
| ✅ | DELETE /polls/fake-id → 401 | 0s |
| ✅ | GET /chess/list → 401 | 0s |
| ✅ | POST /chess/create → 401 | 0s |
| ✅ | GET /chess/invitations → 401 | 0s |
| ✅ | GET /chess/fake-id → 401 | 0s |
| ✅ | POST /chess/fake-id/move → 401 | 0s |
| ✅ | GET /chess/fake-id/moves?from=e2 → 401 | 0s |
| ✅ | POST /chess/fake-id/ai-move → 401 | 0s |
| ✅ | POST /chess/fake-id/resign → 401 | 0s |
| ✅ | POST /conversations/default_global/messages/x/reactions → 401 | 0s |
| ✅ | DELETE /conversations/default_global/messages/x/reactions → 401 | 0s |
| ✅ | GET /conversations/default_global/messages/x/reactions → 401 | 0s |
| ✅ | POST /user/update → 401 | 0s |
| ✅ | GET /users/available → 401 | 0s |
| ✅ | GET /push/preferences → 401 | 0s |
| ✅ | POST /api/upload/chat sans auth → 401 | 0s |
| ✅ | GET /users/pending → 401 | 0s |
| ✅ | GET /users → 401 | 0s |
| ✅ | POST /users/approve → 401 | 0s |
| ✅ | GET /invites → 401 | 0s |
| ✅ | POST /invites → 401 | 0s |
| ✅ | POST /invites/delete → 401 | 0s |
| ✅ | GET /analytics → 401 | 0s |
| ✅ | Mot de passe 1 char → 400 | 0s |
| ✅ | Mot de passe 5 chars → 400 | 0s |
| ✅ | Mot de passe 7 chars → 400 | 0s |
| ✅ | Mot de passe 8 chars → accepte | 0s |
| ✅ | User normal change pwd autre user → 403 (secuite) | 0s |
| ✅ | Upload fichier vide → 400 | 0s |
| ✅ | Upload fichier texte → file_id, puis download OK | 0s |
| ✅ | Download fichier inexistant → 404 | 0s |
| ✅ | Envoyer message → 200, récupérer → contient message | 0s |
| ✅ | Rename conversation → 200 (integration) | 0s |
| ✅ | /call/fake-id sans auth → redirige vers /login | 0s |
| ✅ | /call/fake-id avec auth → page charge | 1s |
| ✅ | Créer partie → jouer e2→e4 → IA répond | 0s |
| ⏭️ | Chess coup illégal → 400 | 0s |
| ✅ | Chess resign → status finished | 0s |
| ✅ | 1 char → 400 | 0s |
| ✅ | 5 chars → 400 | 0s |
| ✅ | 8 chars → accepte | 0s |
| ✅ | User normal cannot change another user pwd → 403 | 0s |
| ✅ | Upload fichier vide avec auth → 400 (taille=0) | 0s |
| ✅ | Upload fichier texte → 200 | 0s |
| ✅ | Upload → Download end-to-end | 0s |
| ✅ | Download inexistant → 404 | 0s |
| ✅ | Envoyer message → 200 | 0s |
| ✅ | Modifier message → 200 | 0s |
| ✅ | Lister messages → contient le message modifié | 0s |
| ✅ | Supprimer message → 200/204 | 0s |
| ✅ | Rename conversation → 200 | 0s |

### ❌ admin.spec.ts — 21/25 passés

| Statut | Test | Durée |
|--------|------|-------|
| ✅ | Admin — page /admin chargée avec header | 0s |
| ✅ | Admin — 3 onglets visibles | 0s |
| ✅ | GET /auth/me avec session admin → role=admin | 0s |
| ✅ | GET /users → liste complète (admin) | 0s |
| ✅ | GET /users/pending → 200 | 0s |
| ✅ | Onglet "Membres" → users visibles dans UI | 0s |
| ✅ | Flux inscription : register → pending → approve → connecté | 0s |
| ✅ | POST /invites → génère un invite_link valide | 0s |
| ✅ | GET /invites → liste non vide | 0s |
| ✅ | POST /invites/delete → supprime une invitation | 0s |
| ✅ | GET /invite/validate?token=xxx → valide le token | 0s |
| ✅ | Admin UI — invitation générée visible dans l'interface | 0s |
| ✅ | GET /analytics → tous les champs requis | 0s |
| ✅ | Page /admin/analytics → stat-cards + 2 charts | 0s |
| ✅ | Admin — DELETE /polls/{id} → 200 | 0s |
| ✅ | GET /analytics avec user normal → 403 | 0s |
| ✅ | GET /users/pending avec user normal → 403 | 0s |
| ✅ | Page /admin → non accessible pour user normal | 2s |
| ⏭️ | Admin — DELETE /users/{id} → supprime un utilisateur | 1m 00s |
| ❌ | Admin — approve + login after approve → accès complet | 25s |
| ✅ | Admin — analytics contient toutes les sections | 0s |
| ✅ | DELETE /users/{id} → supprime un utilisateur | 1s |
| ❌ | GET /analytics → contient user_count, message_count | 1m 01s |
| ✅ | GET /analytics sans auth → 401 | 0s |
| ❌ | Register + Approve + Login → accès complet | 1m 01s |

### ❌ user.spec.ts — 41/64 passés

| Statut | Test | Durée |
|--------|------|-------|
| ✅ | Login e2e_ci → redirigé vers /chat | 0s |
| ✅ | GET /auth/me → username=e2e_ci | 0s |
| ✅ | Login invalide → reste sur /login | 6s |
| ✅ | GET /conversations → default_global présente | 0s |
| ✅ | GET /conversations/default_global → détail de la conv | 0s |
| ✅ | GET /conversations/default_global/participants → e2e_ci présent | 0s |
| ✅ | Chat UI — sidebar et envoi message | 0s |
| ✅ | GET /conversations/default_global/messages → messages récupérés | 0s |
| ✅ | POST /conversations → créer un groupe de test | 0s |
| ✅ | GET /users/available → liste des membres disponibles | 0s |
| ✅ | Réactions — POST emoji valide 👍 → counts mis à jour | 0s |
| ✅ | Réactions — POST emoji non autorisé 🦄 → 400 | 0s |
| ✅ | Réactions — UPSERT : 👍 → ❤️ remplace sans doublon | 0s |
| ✅ | Réactions — DELETE → my_emoji null | 0s |
| ✅ | Réactions — GET → structure {message_id, counts, my_emoji} | 0s |
| ✅ | Réactions — message inexistant → 404 | 0s |
| ✅ | Réactions UI — hover → picker → pill visible | 2s |
| ✅ | Upload — fichier texte → file_id, url=/api/download/, download OK | 0s |
| ✅ | Download — id inexistant → 404 | 0s |
| ✅ | GET /polls → tableau de sondages | 0s |
| ✅ | Polls — cycle complet : créer → voter → changer → double vote → fermer → vote fermé | 0s |
| ✅ | Polls UI — créer sondage via formulaire → visible dans liste | 1s |
| ✅ | GET /chess/list → 200 | 0s |
| ✅ | Chess — créer vs IA, coups légaux, coup légal e2→e4, coup illégal → 400 | 0s |
| ✅ | Chess — POST /chess/{id}/ai-move → 200 | 4s |
| ✅ | Chess — POST /chess/{id}/resign → 200 | 0s |
| ✅ | Chess — invitations : créer, inviter, lister, décliner | 0s |
| ❌ | Chess UI — plateau 64 cases + sélection case + coup via UI | 30s |
| ⏭️ | Calendar — GET /events → 200 | N/A |
| ⏭️ | Calendar — POST /events → crée et DELETE /events/{id} → supprime | N/A |
| ⏭️ | Calendar UI — page, grille et bouton ajouter visibles | N/A |
| ⏭️ | Settings UI — 3 onglets navigables | N/A |
| ⏭️ | Settings — changement de thème (clic → sélectionné) | N/A |
| ⏭️ | POST /user/update → mise à jour du nom | N/A |
| ⏭️ | Navigation /chat → accessible sans erreur | N/A |
| ⏭️ | Navigation /calendar → accessible sans erreur | N/A |
| ⏭️ | Navigation /chess → accessible sans erreur | N/A |
| ⏭️ | Navigation /polls → accessible sans erreur | N/A |
| ⏭️ | Navigation /settings → accessible sans erreur | N/A |
| ⏭️ | Navigation /help → accessible sans erreur | N/A |
| ⏭️ | Navigation /events → accessible sans erreur | N/A |
| ⏭️ | E2EE — POST /auth/public-key → enregistre la clé | N/A |
| ⏭️ | E2EE — GET /auth/public-keys → objet avec clés des membres | N/A |
| ⏭️ | Push — GET /push/vapid-public-key → 200 | N/A |
| ⏭️ | Push — GET /push/preferences → prefs par défaut | N/A |
| ⏭️ | Push — POST /push/preferences → mise à jour | N/A |
| ⏭️ | Push — POST /push/subscribe → 200 | N/A |
| ⏭️ | Logout UI → redirigé vers /login | N/A |
| ✅ | Flood /auth/login × 20 depuis même IP → au moins un 429 | 0s |
| ✅ | /call/default_global → page charge avec titre "Appel" | 1s |
| ❌ | /call/default_global → bouton "Appel audio" visible | 2s |
| ✅ | /call/[id] sans auth → redirige vers /login | 0s |
| ✅ | Créer partie vs IA (facile) | 0s |
| ✅ | Chess — UI plateau 8x8 (64 cases) avec sélection | 0s |
| ✅ | Chess — coup illégal → message erreur | 0s |
| ✅ | /call/default_global → page charge avec titres | 1s |
| ❌ | /call/default_global → boutons "Appel audio" et "Appel vidéo" visibles | 2s |
| ✅ | /call/[id] session → page appel chargee (sans auth first) | 0s |
| ✅ | Créer partie vs IA (facile) → game_id | 0s |
| ✅ | Chess — UI plateau 8x8 (64 cases) | 0s |
| ✅ | Chess — coup légal e2→e4 | 0s |
| ✅ | Chess — coup illégal → 400 | 0s |
| ✅ | Chess — coups légaux depuis e2 → contient e3 et e4 | 0s |
| ✅ | Chess — resign → status finished | 0s |

---

## ❌ Échecs détaillés

> 6 test(s) en échec

### Échec 1 — `Admin — approve + login after approve → accès complet`

**Suite :** `admin.spec.ts > Admin — Complément`

**Message :**
```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /chat|admin/
Received string:  "http://localhost:6300/login"
Timeout: 10000ms

Call log:
  - Expect "toHaveURL" with timeout 10000ms
    14 × unexpected value "http://localhost:6300/login"


  388 |     await newPage.getByRole('button', { name: 'Se connecter' }).click();
  389 |
> 390 |     await expect(newPage).toHaveURL(/chat|admin/, { timeout: 10000 });
      |                           ^
  391 |
  392 |     // Vérifier /auth/me
  393 |     const meRes = await newPage.request.get(`${BASE}/auth/me`);
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:390:27
```

**Message :**
```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /chat|admin/
Received string:  "http://localhost:6300/login"
Timeout: 10000ms

Call log:
  - Expect "toHaveURL" with timeout 10000ms
    14 × unexpected value "http://localhost:6300/login"


  388 |     await newPage.getByRole('button', { name: 'Se connecter' }).click();
  389 |
> 390 |     await expect(newPage).toHaveURL(/chat|admin/, { timeout: 10000 });
      |                           ^
  391 |
  392 |     // Vérifier /auth/me
  393 |     const meRes = await newPage.request.get(`${BASE}/auth/me`);
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:390:27
```

### Échec 2 — `GET /analytics → contient user_count, message_count`

**Suite :** `admin.spec.ts > Admin — Analytics`

**Message :**
```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /\/(chat|admin|change-password)/
Received string:  "http://localhost:6300/login"
Timeout: 15000ms

Call log:
  - Expect "toHaveURL" with timeout 15000ms
    19 × unexpected value "http://localhost:6300/login"


   at helpers.ts:49

  47 |     await page.fill('#password', password);
  48 |     await page.getByRole('button', { name: 'Se connecter' }).click();
> 49 |     await expect(page).toHaveURL(/\/(chat|admin|change-password)/, { timeout: 15_000 });
     |                        ^
  50 |   }
  51 | }
  52 |
```

**Message :**
```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /\/(chat|admin|change-password)/
Received string:  "http://localhost:6300/login"
Timeout: 15000ms

Call log:
  - Expect "toHaveURL" with timeout 15000ms
    19 × unexpected value "http://localhost:6300/login"


   at helpers.ts:49

  47 |     await page.fill('#password', password);
  48 |     await page.getByRole('button', { name: 'Se connecter' }).click();
> 49 |     await expect(page).toHaveURL(/\/(chat|admin|change-password)/, { timeout: 15_000 });
     |                        ^
  50 |   }
  51 | }
  52 |
```

### Échec 3 — `Register + Approve + Login → accès complet`

**Suite :** `admin.spec.ts > Admin — Approve user + login flow`

**Message :**
```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /\/(chat|admin|change-password)/
Received string:  "http://localhost:6300/login"
Timeout: 15000ms

Call log:
  - Expect "toHaveURL" with timeout 15000ms
    19 × unexpected value "http://localhost:6300/login"


   at helpers.ts:49

  47 |     await page.fill('#password', password);
  48 |     await page.getByRole('button', { name: 'Se connecter' }).click();
> 49 |     await expect(page).toHaveURL(/\/(chat|admin|change-password)/, { timeout: 15_000 });
     |                        ^
  50 |   }
  51 | }
  52 |
```

**Message :**
```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /\/(chat|admin|change-password)/
Received string:  "http://localhost:6300/login"
Timeout: 15000ms

Call log:
  - Expect "toHaveURL" with timeout 15000ms
    19 × unexpected value "http://localhost:6300/login"


   at helpers.ts:49

  47 |     await page.fill('#password', password);
  48 |     await page.getByRole('button', { name: 'Se connecter' }).click();
> 49 |     await expect(page).toHaveURL(/\/(chat|admin|change-password)/, { timeout: 15_000 });
     |                        ^
  50 |   }
  51 | }
  52 |
```

### Échec 4 — `Chess UI — plateau 64 cases + sélection case + coup via UI`

**Suite :** `user.spec.ts > User — Flux complet`

**Message :**
```
Error: expect(locator).toBeVisible() failed

Locator: locator('.chess-board')
Expected: visible
Timeout: 15000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 15000ms
  - waiting for locator('.chess-board')


  560 |     await page.goto(`/chess/${game_id}`);
  561 |     await waitForAppReady(page);
> 562 |     await expect(page.locator('.chess-board')).toBeVisible({ timeout: 15_000 });
      |                                                ^
  563 |     expect(await page.locator('.chess-board .cell').count()).toBe(64);
  564 |     console.log('✅ Échiquier 8×8 rendu');
  565 |
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:562:48
```

**Message :**
```
Error: expect(locator).toBeVisible() failed

Locator: locator('.chess-board')
Expected: visible
Timeout: 15000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 15000ms
  - waiting for locator('.chess-board')


  560 |     await page.goto(`/chess/${game_id}`);
  561 |     await waitForAppReady(page);
> 562 |     await expect(page.locator('.chess-board')).toBeVisible({ timeout: 15_000 });
      |                                                ^
  563 |     expect(await page.locator('.chess-board .cell').count()).toBe(64);
  564 |     console.log('✅ Échiquier 8×8 rendu');
  565 |
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:562:48
```

### Échec 5 — `/call/default_global → bouton "Appel audio" visible`

**Suite :** `user.spec.ts > Call page`

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false

  820 |     const hasAudioBtn = await page.getByText('Appel audio').isVisible().catch(() => false);
  821 |     const hasVideoBtn = await page.getByText('Appel vidéo').isVisible().catch(() => false);
> 822 |     expect(hasAudioBtn || hasVideoBtn).toBe(true);
      |                                        ^
  823 |   });
  824 |
  825 |   test('/call/[id] sans auth → redirige vers /login', async ({ browser }) => {
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:822:40
```

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false

  820 |     const hasAudioBtn = await page.getByText('Appel audio').isVisible().catch(() => false);
  821 |     const hasVideoBtn = await page.getByText('Appel vidéo').isVisible().catch(() => false);
> 822 |     expect(hasAudioBtn || hasVideoBtn).toBe(true);
      |                                        ^
  823 |   });
  824 |
  825 |   test('/call/[id] sans auth → redirige vers /login', async ({ browser }) => {
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:822:40
```

### Échec 6 — `/call/default_global → boutons "Appel audio" et "Appel vidéo" visibles`

**Suite :** `user.spec.ts > Call page`

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false

  922 |     const hasAudioBtn = await page.getByText('Appel audio').isVisible().catch(() => false);
  923 |     const hasVideoBtn = await page.getByText('Appel vidéo').isVisible().catch(() => false);
> 924 |     expect(hasAudioBtn || hasVideoBtn).toBe(true);
      |                                        ^
  925 |   });
  926 |
  927 |   test('/call/[id] session → page appel chargee (sans auth first)', async ({ browser }) => {
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:924:40
```

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false

  922 |     const hasAudioBtn = await page.getByText('Appel audio').isVisible().catch(() => false);
  923 |     const hasVideoBtn = await page.getByText('Appel vidéo').isVisible().catch(() => false);
> 924 |     expect(hasAudioBtn || hasVideoBtn).toBe(true);
      |                                        ^
  925 |   });
  926 |
  927 |   test('/call/[id] session → page appel chargee (sans auth first)', async ({ browser }) => {
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:924:40
```

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=40ca7b89-76c4-4955-a8a6-5d2da6ef9f80 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=40ca7b89-76c4-4955-a8a6-5d2da6ef9f80 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=40ca7b89-76c4-4955-a8a6-5d2da6ef9f80 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=40ca7b89-76c4-4955-a8a6-5d2da6ef9f80 username=e2e_ci
```

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

*Rapport généré par `scripts/generate-test-report.py` — 2026-04-03 05:23 UTC*
