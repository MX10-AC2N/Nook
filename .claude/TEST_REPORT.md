# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-04-03 09:14 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** |
| **Tests passés** | 161 |
| **Tests échoués** | 3 |
| **Tests ignorés** | 0 |
| **Total** | 164 |
| **Durée** | 1.0m 14s |
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
| ✅ | User normal change pwd autre user → 403 | 0s |
| ✅ | Upload fichier vide → 400 | 0s |
| ✅ | Upload fichier texte → file_id, puis download OK | 0s |
| ✅ | Download fichier inexistant → 404 | 0s |
| ✅ | Envoyer message → 200, récupérer → contient message | 0s |
| ✅ | Rename conversation → 200 | 0s |
| ✅ | /call/fake-id sans auth → redirige vers /login | 0s |
| ✅ | /call/fake-id avec auth → page charge | 1s |
| ✅ | Créer partie → jouer e2→e4 → IA répond | 0s |
| ✅ | Chess coup illégal → 400 | 0s |
| ⏭️ | Chess resign → status finished | 0s |
| ✅ | 1 char → 400 | 0s |
| ✅ | 5 chars → 400 | 0s |
| ✅ | 8 chars → accepte | 0s |
| ✅ | User change pwd autre user → 403 (integration) | 0s |
| ✅ | Upload sec -- fichier vide refuse → 400 (second block) | 0s |
| ✅ | Upload fichier texte → 200 | 0s |
| ✅ | Upload → Download end-to-end | 0s |
| ✅ | Download inexistant → 404 | 0s |
| ✅ | Envoyer message → 200 | 0s |
| ✅ | Modifier message → 200 | 0s |
| ✅ | Lister messages → contient le message modifié | 0s |
| ✅ | Supprimer message → 200/204 | 0s |
| ✅ | Rename conversation → 200 (second block) | 0s |

### ❌ admin.spec.ts — 22/25 passés

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
| ❌ | Admin — DELETE /users/{id} → supprime un utilisateur | 0s |
| ❌ | Admin — approve + login after approve → accès complet | 0s |
| ✅ | Admin — analytics contient toutes les sections | 0s |
| ✅ | DELETE /users/{id} → supprime un utilisateur | 1s |
| ❌ | GET /analytics → contient user_count, message_count | 0s |
| ✅ | GET /analytics sans auth → 401 | 0s |
| ✅ | Register + Approve + Login → accès complet | 4s |

### ✅ user.spec.ts — 64/64 passés

| Statut | Test | Durée |
|--------|------|-------|
| ✅ | Login e2e_ci → redirigé vers /chat | 0s |
| ✅ | GET /auth/me → username=e2e_ci | 0s |
| ✅ | Login invalide → reste sur /login | 3s |
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
| ✅ | Réactions UI — hover → picker → pill visible | 1s |
| ✅ | Upload — fichier texte → file_id, url=/api/download/, download OK | 0s |
| ✅ | Download — id inexistant → 404 | 0s |
| ✅ | GET /polls → tableau de sondages | 0s |
| ✅ | Polls — cycle complet : créer → voter → changer → double vote → fermer → vote fermé | 0s |
| ✅ | Polls UI — créer sondage via formulaire → visible dans liste | 0s |
| ✅ | GET /chess/list → 200 | 0s |
| ✅ | Chess — créer vs IA, coups légaux, coup légal e2→e4, coup illégal → 400 | 0s |
| ✅ | Chess — POST /chess/{id}/ai-move → 200 | 1s |
| ✅ | Chess — POST /chess/{id}/resign → 200 | 0s |
| ✅ | Chess — invitations : créer, inviter, lister, décliner | 0s |
| ✅ | Chess UI — plateau 64 cases + sélection case + coup via UI | 25s |
| ✅ | Calendar — GET /events → 200 | 0s |
| ✅ | Calendar — POST /events → crée et DELETE /events/{id} → supprime | 0s |
| ✅ | Calendar UI — page, grille et bouton ajouter visibles | 0s |
| ✅ | Settings UI — 3 onglets navigables | 0s |
| ✅ | Settings — changement de thème (clic → sélectionné) | 0s |
| ✅ | POST /user/update → mise à jour du nom | 0s |
| ✅ | Navigation /chat → accessible sans erreur | 0s |
| ✅ | Navigation /calendar → accessible sans erreur | 0s |
| ✅ | Navigation /chess → accessible sans erreur | 0s |
| ✅ | Navigation /polls → accessible sans erreur | 0s |
| ✅ | Navigation /settings → accessible sans erreur | 0s |
| ✅ | Navigation /help → accessible sans erreur | 0s |
| ✅ | Navigation /events → accessible sans erreur | 0s |
| ✅ | E2EE — POST /auth/public-key → enregistre la clé | 0s |
| ✅ | E2EE — GET /auth/public-keys → objet avec clés des membres | 0s |
| ✅ | Push — GET /push/vapid-public-key → 200 | 0s |
| ✅ | Push — GET /push/preferences → prefs par défaut | 0s |
| ✅ | Push — POST /push/preferences → mise à jour | 0s |
| ✅ | Push — POST /push/subscribe → 200 | 0s |
| ✅ | Logout UI → redirigé vers /login | 0s |
| ✅ | Flood /auth/login × 20 depuis même IP → au moins un 429 | 0s |
| ✅ | /call/default_global → page charge avec titre "Appel" | 1s |
| ✅ | /call/default_global → page call charge correctement | 4s |
| ✅ | /call/[id] sans auth → redirige vers /login | 0s |
| ✅ | Créer partie vs IA (facile) | 0s |
| ✅ | Chess — UI plateau 8x8 (64 cases) avec sélection | 0s |
| ✅ | Chess — coup illégal → message erreur | 0s |
| ✅ | /call/default_global → page charge avec titres | 1s |
| ✅ | /call/default_global → page contient contenu call | 4s |
| ✅ | /call/[id] session → page appel chargee (sans auth first) | 0s |
| ✅ | Créer partie vs IA (facile) → game_id | 1s |
| ✅ | Chess — UI plateau 8x8 (64 cases) | 0s |
| ✅ | Chess — coup légal e2→e4 | 0s |
| ✅ | Chess — coup illégal → 400 | 0s |
| ✅ | Chess — coups légaux depuis e2 → contient e3 et e4 | 0s |
| ✅ | Chess — resign → status finished | 0s |

---

## ❌ Échecs détaillés

> 3 test(s) en échec

### Échec 1 — `Admin — DELETE /users/{id} → supprime un utilisateur`

**Suite :** `admin.spec.ts > Admin — Complément`

**Message :**
```
ReferenceError: adminPage is not defined

  330 |   test('Admin — DELETE /users/{id} → supprime un utilisateur', async () => {
  331 |     // Register via admin's authenticated context
> 332 |     const regRes = await adminPage.request.post(`${BASE}/auth/register`, {
      |                    ^
  333 |       data: { username: 'del_t3', password: 'DelTest123!', email: 'dt3@nook.local', name: 'Dt3' },
  334 |     });
  335 |     expect([200, 201, 409]).toContain(regRes.status());
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:332:20
```

**Message :**
```
ReferenceError: adminPage is not defined

  330 |   test('Admin — DELETE /users/{id} → supprime un utilisateur', async () => {
  331 |     // Register via admin's authenticated context
> 332 |     const regRes = await adminPage.request.post(`${BASE}/auth/register`, {
      |                    ^
  333 |       data: { username: 'del_t3', password: 'DelTest123!', email: 'dt3@nook.local', name: 'Dt3' },
  334 |     });
  335 |     expect([200, 201, 409]).toContain(regRes.status());
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:332:20
```

### Échec 2 — `Admin — approve + login after approve → accès complet`

**Suite :** `admin.spec.ts > Admin — Complément`

**Message :**
```
ReferenceError: adminPage is not defined

  360 |   test('Admin — approve + login after approve → accès complet', async ({ browser }) => {
  361 |     // Create user via admin's authenticated request
> 362 |     const regRes = await adminPage.request.post(`${BASE}/auth/register`, {
      |                    ^
  363 |       data: { username: 'approve_t3', password: 'Approve123!', email: 'apt3@nook.local', name: 'Apt3' },
  364 |     });
  365 |     expect([200, 201, 409]).toContain(regRes.status());
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:362:20
```

**Message :**
```
ReferenceError: adminPage is not defined

  360 |   test('Admin — approve + login after approve → accès complet', async ({ browser }) => {
  361 |     // Create user via admin's authenticated request
> 362 |     const regRes = await adminPage.request.post(`${BASE}/auth/register`, {
      |                    ^
  363 |       data: { username: 'approve_t3', password: 'Approve123!', email: 'apt3@nook.local', name: 'Apt3' },
  364 |     });
  365 |     expect([200, 201, 409]).toContain(regRes.status());
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:362:20
```

### Échec 3 — `GET /analytics → contient user_count, message_count`

**Suite :** `admin.spec.ts > Admin — Analytics`

**Message :**
```
ReferenceError: adminPage is not defined

  465 |   test('GET /analytics → contient user_count, message_count', async () => {
  466 |     // adminPage already has auth from loginAsAdmin in beforeAll
> 467 |     const res = await adminPage.request.get(`${BASE}/analytics`);
      |                 ^
  468 |     expect(res.status()).toBe(200);
  469 |     const body = await res.json();
  470 |     expect(body).toHaveProperty('user_count');
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:467:17
```

**Message :**
```
ReferenceError: adminPage is not defined

  465 |   test('GET /analytics → contient user_count, message_count', async () => {
  466 |     // adminPage already has auth from loginAsAdmin in beforeAll
> 467 |     const res = await adminPage.request.get(`${BASE}/analytics`);
      |                 ^
  468 |     expect(res.status()).toBe(200);
  469 |     const body = await res.json();
  470 |     expect(body).toHaveProperty('user_count');
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:467:17
```

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=9e90e0e2-36fb-4119-af26-29fa2410d690 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=9e90e0e2-36fb-4119-af26-29fa2410d690 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=9e90e0e2-36fb-4119-af26-29fa2410d690 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=9e90e0e2-36fb-4119-af26-29fa2410d690 username=e2e_ci
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-04-03 09:14 UTC*
