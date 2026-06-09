# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-06-09 05:37 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** |
| **Tests passés** | 7 |
| **Tests échoués** | 69 |
| **Tests flaky** | 0 |
| **Tests ignorés** | 106 |
| **Total** | 182 |
| **Durée totale** | 1m 42.4s |
| **Branche** | `develop` |
| **Commit** | [`c74c3d4`](https://github.com/MX10-AC2N/Nook/commit/c74c3d4d12b397c63fa1b3e8d3accd3f94cf007e) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/27185591768) |

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
| 1 | /call/fake-id avec auth → page charge | 21.2s | `?` |
| 2 | /call/fake-id sans auth → redirige vers /login | 0.3s | `?` |
| 3 | POST /conversations/default_global/messages/x/reactions → 401 | 0.0s | `?` |
| 4 | GET /conversations → 401 | 0.0s | `?` |
| 5 | GET /conversations/default_global → 401 | 0.0s | `?` |
| 6 | POST /chess/fake-id/move → 401 | 0.0s | `?` |
| 7 | GET /api/health → "OK" | 0.0s | `?` |
| 8 | GET /push/vapid-public-key → 200 (route publique, pas d'auth requise) | 0.0s | `?` |
| 9 | GET /auth/me → 401 | 0.0s | `?` |
| 10 | POST /auth/logout → 401 | 0.0s | `?` |

---

## 📋 Résultats par catégorie

### ❌ **api-sanity.spec.ts** — 7/76 passés · ❌ 69 failed

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ❌ | /call/fake-id avec auth → page charge | 21.2s  +1 | |
| ❌ | POST /conversations/default_global/messages/x/reactions → 401 | 0.0s  +1 | |
| ❌ | GET /conversations → 401 | 0.0s  +1 | |
| ❌ | GET /conversations/default_global → 401 | 0.0s  +1 | |
| ❌ | POST /chess/fake-id/move → 401 | 0.0s  +1 | |
| ❌ | GET /api/health → "OK" | 0.0s  +1 | |
| ❌ | GET /push/vapid-public-key → 200 (route publique, pas d'auth requise) | 0.0s  +1 | |
| ❌ | GET /auth/me → 401 | 0.0s  +1 | |
| ❌ | POST /auth/logout → 401 | 0.0s  +1 | |
| ❌ | POST /auth/change-password → 401 | 0.0s  +1 | |
| ❌ | POST /auth/public-key → 401 | 0.0s  +1 | |
| ❌ | GET /auth/public-keys?conversation_id=default_global → 401 | 0.0s  +1 | |
| ❌ | POST /conversations → 401 | 0.0s  +1 | |
| ❌ | GET /conversations/default_global/messages → 401 | 0.0s  +1 | |
| ❌ | GET /conversations/default_global/participants → 401 | 0.0s  +1 | |
| ❌ | POST /conversations/default_global/participants → 401 | 0.0s  +1 | |
| ❌ | POST /conversations/default_global/leave → 401 | 0.0s  +1 | |
| ❌ | PATCH /conversations/default_global/rename → 401 | 0.0s  +1 | |
| ❌ | GET /download/fake-id-000 → 401 | 0.0s  +1 | |
| ❌ | GET /events → 401 | 0.0s  +1 | |
| ❌ | POST /events → 401 | 0.0s  +1 | |
| ❌ | DELETE /events/fake-id → 401 | 0.0s  +1 | |
| ❌ | GET /polls → 401 | 0.0s  +1 | |
| ❌ | POST /polls → 401 | 0.0s  +1 | |
| ❌ | GET /polls/fake-id → 401 | 0.0s  +1 | |
| ❌ | POST /polls/fake-id/vote → 401 | 0.0s  +1 | |
| ❌ | DELETE /polls/fake-id → 401 | 0.0s  +1 | |
| ❌ | GET /chess/list → 401 | 0.0s  +1 | |
| ❌ | GET /chess/invitations → 401 | 0.0s  +1 | |
| ❌ | GET /chess/fake-id → 401 | 0.0s  +1 | |
| ❌ | GET /chess/fake-id/moves?from=e2 → 401 | 0.0s  +1 | |
| ❌ | POST /chess/fake-id/ai-move → 401 | 0.0s  +1 | |
| ❌ | POST /chess/fake-id/resign → 401 | 0.0s  +1 | |
| ❌ | DELETE /conversations/default_global/messages/x/reactions → 401 | 0.0s  +1 | |
| ❌ | GET /conversations/default_global/messages/x/reactions → 401 | 0.0s  +1 | |
| ❌ | POST /user/update → 401 | 0.0s  +1 | |
| ❌ | GET /users/available → 401 | 0.0s  +1 | |
| ❌ | GET /push/preferences → 401 | 0.0s  +1 | |
| ❌ | GET /users → 401 | 0.0s  +1 | |
| ❌ | POST /users/approve → 401 | 0.0s  +1 | |
| ❌ | GET /invites → 401 | 0.0s  +1 | |
| ❌ | POST /invites → 401 | 0.0s  +1 | |
| ❌ | POST /invites/delete → 401 | 0.0s  +1 | |
| ❌ | GET /analytics → 401 | 0.0s  +1 | |
| ❌ | Mot de passe 1 char → 400 | 0.0s  +1 | |
| ❌ | Mot de passe 5 chars → 400 | 0.0s  +1 | |
| ❌ | Mot de passe 7 chars → 400 | 0.0s  +1 | |
| ❌ | Mot de passe 8 chars → accepte | 0.0s  +1 | |
| ❌ | User normal change pwd autre user → 403 | 0.0s  +1 | |
| ❌ | Upload fichier vide → 400 | 0.0s  +1 | |
| ❌ | Upload fichier texte → file_id, puis download OK | 0.0s  +1 | |
| ❌ | Download fichier inexistant → 404 | 0.0s  +1 | |
| ❌ | Envoyer message → 200, récupérer → contient message | 0.0s  +1 | |
| ❌ | Rename conversation → 200 | 0.0s  +1 | |
| ❌ | Créer partie → jouer e2→e4 → IA répond | 0.0s  +1 | |
| ❌ | 5 chars → 400 | 0.0s  +1 | |
| ❌ | 8 chars → accepte | 0.0s  +1 | |
| ❌ | User change pwd autre user → 403 (integration) | 0.0s  +1 | |
| ❌ | Upload sec -- fichier vide refuse → 400 (second block) | 0.0s  +1 | |
| ❌ | Download inexistant → 404 | 0.0s  +1 | |
| ❌ | Envoyer message → 200 | 0.0s  +1 | |
| ❌ | Rename conversation → 200 (second block) | 0.0s  +1 | |
| ❌ | POST /conversations/default_global/messages → 401 | 0.0s  +1 | |
| ❌ | POST /polls/fake-id/close → 401 | 0.0s  +1 | |
| ❌ | POST /chess/create → 401 | 0.0s  +1 | |
| ❌ | Upload fichier texte → 200 | 0.0s  +1 | |
| ❌ | Upload → Download end-to-end | 0.0s  +1 | |
| ❌ | GET /users/pending → 401 | 0.0s  +1 | |
| ❌ | 1 char → 400 | 0.0s  +1 | |
| ✅ | /call/fake-id sans auth → redirige vers /login | 0.3s | |
| ✅ | POST /api/upload/chat sans auth → 401 | 0.0s | |
| ✅ | Modifier message → 200 | 0.0s | |
| ✅ | Chess coup illégal → 400 | 0.0s | |
| ✅ | Supprimer message → 200/204 | 0.0s | |
| ✅ | Chess resign → status finished | 0.0s | |
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
| ❌ `unknown` | 7 | 69 | 0 | 182 |

---

## ❌ Échecs détaillés

> 69 test(s) en échec

### Échec 1 — `GET /api/health → "OK"`

**Suite :** `api-sanity.spec.ts > Sanité — Serveur`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  10 |
  11 |   test('GET /api/health → "OK"', async ({ request }) => {
> 12 |     const res = await request.get(`${BASE}/health`);
     |                                      ^
  13 |     expect(res.status()).toBe(200);
  14 |     expect((await res.text()).trim()).toBe('OK');
  15 |   });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:12:38
```

**Message :**
```
ReferenceError: BASE is not defined

  10 |
  11 |   test('GET /api/health → "OK"', async ({ request }) => {
> 12 |     const res = await request.get(`${BASE}/health`);
     |                                      ^
  13 |     expect(res.status()).toBe(200);
  14 |     expect((await res.text()).trim()).toBe('OK');
  15 |   });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:12:38
```

### Échec 2 — `GET /push/vapid-public-key → 200 (route publique, pas d'auth requise)`

**Suite :** `api-sanity.spec.ts > Sanité — Serveur`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  18 |     // La clé VAPID publique doit être accessible sans cookie :
  19 |     // le browser en a besoin pour créer un PushSubscription avant même le login.
> 20 |     const res = await request.get(`${BASE}/push/vapid-public-key`);
     |                                      ^
  21 |     expect(res.status()).toBe(200);
  22 |     const body = await res.json();
  23 |     expect(typeof body.public_key).toBe('string');
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:20:38
```

**Message :**
```
ReferenceError: BASE is not defined

  18 |     // La clé VAPID publique doit être accessible sans cookie :
  19 |     // le browser en a besoin pour créer un PushSubscription avant même le login.
> 20 |     const res = await request.get(`${BASE}/push/vapid-public-key`);
     |                                      ^
  21 |     expect(res.status()).toBe(200);
  22 |     const body = await res.json();
  23 |     expect(typeof body.public_key).toBe('string');
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:20:38
```

### Échec 3 — `GET /auth/me → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 4 — `POST /auth/logout → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 5 — `POST /auth/change-password → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 6 — `POST /auth/public-key → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 7 — `GET /auth/public-keys?conversation_id=default_global → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 8 — `GET /conversations → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 9 — `POST /conversations → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 10 — `GET /conversations/default_global → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 11 — `GET /conversations/default_global/messages → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 12 — `POST /conversations/default_global/messages → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 13 — `GET /conversations/default_global/participants → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 14 — `POST /conversations/default_global/participants → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 15 — `POST /conversations/default_global/leave → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 16 — `PATCH /conversations/default_global/rename → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
> 85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
     |                                                                        ^
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
  88 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:85:72
```

**Message :**
```
ReferenceError: BASE is not defined

  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
> 85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
     |                                                                        ^
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
  88 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:85:72
```

### Échec 17 — `GET /download/fake-id-000 → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 18 — `GET /events → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 19 — `POST /events → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 20 — `DELETE /events/fake-id → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  82 |       let res;
  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
> 84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
     |                                                                         ^
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:84:73
```

**Message :**
```
ReferenceError: BASE is not defined

  82 |       let res;
  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
> 84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
     |                                                                         ^
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:84:73
```

### Échec 21 — `GET /polls → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 22 — `POST /polls → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 23 — `GET /polls/fake-id → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 24 — `POST /polls/fake-id/vote → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 25 — `POST /polls/fake-id/close → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 26 — `DELETE /polls/fake-id → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  82 |       let res;
  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
> 84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
     |                                                                         ^
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:84:73
```

**Message :**
```
ReferenceError: BASE is not defined

  82 |       let res;
  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
> 84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
     |                                                                         ^
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:84:73
```

### Échec 27 — `GET /chess/list → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 28 — `POST /chess/create → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 29 — `GET /chess/invitations → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 30 — `GET /chess/fake-id → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 31 — `POST /chess/fake-id/move → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 32 — `GET /chess/fake-id/moves?from=e2 → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 33 — `POST /chess/fake-id/ai-move → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 34 — `POST /chess/fake-id/resign → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 35 — `POST /conversations/default_global/messages/x/reactions → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 36 — `DELETE /conversations/default_global/messages/x/reactions → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  82 |       let res;
  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
> 84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
     |                                                                         ^
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:84:73
```

**Message :**
```
ReferenceError: BASE is not defined

  82 |       let res;
  83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
> 84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
     |                                                                         ^
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87 |       expect(res.status()).toBe(401);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:84:73
```

### Échec 37 — `GET /conversations/default_global/messages/x/reactions → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 38 — `POST /user/update → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

**Message :**
```
ReferenceError: BASE is not defined

  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
     |                                        ^
  87 |       expect(res.status()).toBe(401);
  88 |     });
  89 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:86:40
```

### Échec 39 — `GET /users/available → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 40 — `GET /push/preferences → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes non-auth → 401`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

**Message :**
```
ReferenceError: BASE is not defined

  81 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82 |       let res;
> 83 |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
     |                                                                 ^
  84 |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85 |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
  86 |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:83:65
```

### Échec 41 — `GET /users/pending → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes admin → 401 sans auth`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

### Échec 42 — `GET /users → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes admin → 401 sans auth`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

### Échec 43 — `POST /users/approve → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes admin → 401 sans auth`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  117 |       const res = route.method === 'GET'
  118 |         ? await request.get(`${BASE}${route.path}`)
> 119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
      |                                 ^
  120 |       expect(res.status()).toBe(401);
  121 |     });
  122 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:119:33
```

**Message :**
```
ReferenceError: BASE is not defined

  117 |       const res = route.method === 'GET'
  118 |         ? await request.get(`${BASE}${route.path}`)
> 119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
      |                                 ^
  120 |       expect(res.status()).toBe(401);
  121 |     });
  122 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:119:33
```

### Échec 44 — `GET /invites → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes admin → 401 sans auth`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

### Échec 45 — `POST /invites → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes admin → 401 sans auth`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  117 |       const res = route.method === 'GET'
  118 |         ? await request.get(`${BASE}${route.path}`)
> 119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
      |                                 ^
  120 |       expect(res.status()).toBe(401);
  121 |     });
  122 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:119:33
```

**Message :**
```
ReferenceError: BASE is not defined

  117 |       const res = route.method === 'GET'
  118 |         ? await request.get(`${BASE}${route.path}`)
> 119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
      |                                 ^
  120 |       expect(res.status()).toBe(401);
  121 |     });
  122 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:119:33
```

### Échec 46 — `POST /invites/delete → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes admin → 401 sans auth`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  117 |       const res = route.method === 'GET'
  118 |         ? await request.get(`${BASE}${route.path}`)
> 119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
      |                                 ^
  120 |       expect(res.status()).toBe(401);
  121 |     });
  122 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:119:33
```

**Message :**
```
ReferenceError: BASE is not defined

  117 |       const res = route.method === 'GET'
  118 |         ? await request.get(`${BASE}${route.path}`)
> 119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
      |                                 ^
  120 |       expect(res.status()).toBe(401);
  121 |     });
  122 |   }
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:119:33
```

### Échec 47 — `GET /analytics → 401`

**Suite :** `api-sanity.spec.ts > Sécurité — Routes admin → 401 sans auth`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

**Message :**
```
ReferenceError: BASE is not defined

  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
> 118 |         ? await request.get(`${BASE}${route.path}`)
      |                                ^
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:118:32
```

### Échec 48 — `Mot de passe 1 char → 400`

**Suite :** `api-sanity.spec.ts > Sécurité — Mot de passe faible → rejeté`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  129 | test.describe('Sécurité — Mot de passe faible → rejeté', () => {
  130 |   test('Mot de passe 1 char → 400', async ({ request }) => {
> 131 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  132 |       data: { username: 'weakpwd1', password: 'a', email: 'w1@nook.local', name: 'W1' },
  133 |     });
  134 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:131:39
```

**Message :**
```
ReferenceError: BASE is not defined

  129 | test.describe('Sécurité — Mot de passe faible → rejeté', () => {
  130 |   test('Mot de passe 1 char → 400', async ({ request }) => {
> 131 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  132 |       data: { username: 'weakpwd1', password: 'a', email: 'w1@nook.local', name: 'W1' },
  133 |     });
  134 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:131:39
```

### Échec 49 — `Mot de passe 5 chars → 400`

**Suite :** `api-sanity.spec.ts > Sécurité — Mot de passe faible → rejeté`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  136 |
  137 |   test('Mot de passe 5 chars → 400', async ({ request }) => {
> 138 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  139 |       data: { username: 'weakpwd2', password: 'abcde', email: 'w2@nook.local', name: 'W2' },
  140 |     });
  141 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:138:39
```

**Message :**
```
ReferenceError: BASE is not defined

  136 |
  137 |   test('Mot de passe 5 chars → 400', async ({ request }) => {
> 138 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  139 |       data: { username: 'weakpwd2', password: 'abcde', email: 'w2@nook.local', name: 'W2' },
  140 |     });
  141 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:138:39
```

### Échec 50 — `Mot de passe 7 chars → 400`

**Suite :** `api-sanity.spec.ts > Sécurité — Mot de passe faible → rejeté`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  143 |
  144 |   test('Mot de passe 7 chars → 400', async ({ request }) => {
> 145 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  146 |       data: { username: 'weakpwd3', password: 'abcdefg', email: 'w3@nook.local', name: 'W3' },
  147 |     });
  148 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:145:39
```

**Message :**
```
ReferenceError: BASE is not defined

  143 |
  144 |   test('Mot de passe 7 chars → 400', async ({ request }) => {
> 145 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  146 |       data: { username: 'weakpwd3', password: 'abcdefg', email: 'w3@nook.local', name: 'W3' },
  147 |     });
  148 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:145:39
```

### Échec 51 — `Mot de passe 8 chars → accepte`

**Suite :** `api-sanity.spec.ts > Sécurité — Mot de passe faible → rejeté`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  150 |
  151 |   test('Mot de passe 8 chars → accepte', async ({ request }) => {
> 152 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  153 |       data: { username: 'okpwd1', password: 'Test1234', email: 'ok1@nook.local', name: 'OK1' },
  154 |     });
  155 |     // 200 = créé, 409 = déjà existe — les deux sont OK
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:152:39
```

**Message :**
```
ReferenceError: BASE is not defined

  150 |
  151 |   test('Mot de passe 8 chars → accepte', async ({ request }) => {
> 152 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  153 |       data: { username: 'okpwd1', password: 'Test1234', email: 'ok1@nook.local', name: 'OK1' },
  154 |     });
  155 |     // 200 = créé, 409 = déjà existe — les deux sont OK
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:152:39
```

### Échec 52 — `User normal change pwd autre user → 403`

**Suite :** `api-sanity.spec.ts > Sécurité — Change password autre user → 403`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  161 |   test('User normal change pwd autre user → 403', async ({ request }) => {
  162 |     // Login e2e_ci
> 163 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  164 |       data: { username: 'e2e_ci', password: 'E2eTest123!' },
  165 |     });
  166 |     expect([200, 401]).toContain(login.status()); // 401 if not approved yet
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:163:41
```

**Message :**
```
ReferenceError: BASE is not defined

  161 |   test('User normal change pwd autre user → 403', async ({ request }) => {
  162 |     // Login e2e_ci
> 163 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  164 |       data: { username: 'e2e_ci', password: 'E2eTest123!' },
  165 |     });
  166 |     expect([200, 401]).toContain(login.status()); // 401 if not approved yet
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:163:41
```

### Échec 53 — `Upload fichier vide → 400`

**Suite :** `api-sanity.spec.ts > Sécurité — Upload validation`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  176 | test.describe('Sécurité — Upload validation', () => {
  177 |   test('Upload fichier vide → 400', async ({ request }) => {
> 178 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  179 |       data: { username: 'admin', password: 'changeme2026' },
  180 |     });
  181 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:178:41
```

**Message :**
```
ReferenceError: BASE is not defined

  176 | test.describe('Sécurité — Upload validation', () => {
  177 |   test('Upload fichier vide → 400', async ({ request }) => {
> 178 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  179 |       data: { username: 'admin', password: 'changeme2026' },
  180 |     });
  181 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:178:41
```

### Échec 54 — `Upload fichier texte → file_id, puis download OK`

**Suite :** `api-sanity.spec.ts > Sécurité — Upload/Download end-to-end`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  194 | test.describe('Sécurité — Upload/Download end-to-end', () => {
  195 |   test('Upload fichier texte → file_id, puis download OK', async ({ request }) => {
> 196 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  197 |       data: { username: 'admin', password: 'changeme2026' },
  198 |     });
  199 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:196:41
```

**Message :**
```
ReferenceError: BASE is not defined

  194 | test.describe('Sécurité — Upload/Download end-to-end', () => {
  195 |   test('Upload fichier texte → file_id, puis download OK', async ({ request }) => {
> 196 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  197 |       data: { username: 'admin', password: 'changeme2026' },
  198 |     });
  199 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:196:41
```

### Échec 55 — `Download fichier inexistant → 404`

**Suite :** `api-sanity.spec.ts > Sécurité — Upload/Download end-to-end`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  217 |
  218 |   test('Download fichier inexistant → 404', async ({ request }) => {
> 219 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  220 |       data: { username: 'admin', password: 'changeme2026' },
  221 |     });
  222 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:219:41
```

**Message :**
```
ReferenceError: BASE is not defined

  217 |
  218 |   test('Download fichier inexistant → 404', async ({ request }) => {
> 219 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  220 |       data: { username: 'admin', password: 'changeme2026' },
  221 |     });
  222 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:219:41
```

### Échec 56 — `Envoyer message → 200, récupérer → contient message`

**Suite :** `api-sanity.spec.ts > Sécurité — Message conversation CRUD`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  229 | test.describe('Sécurité — Message conversation CRUD', () => {
  230 |   test('Envoyer message → 200, récupérer → contient message', async ({ request }) => {
> 231 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  232 |       data: { username: 'admin', password: 'changeme2026' },
  233 |     });
  234 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:231:41
```

**Message :**
```
ReferenceError: BASE is not defined

  229 | test.describe('Sécurité — Message conversation CRUD', () => {
  230 |   test('Envoyer message → 200, récupérer → contient message', async ({ request }) => {
> 231 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  232 |       data: { username: 'admin', password: 'changeme2026' },
  233 |     });
  234 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:231:41
```

### Échec 57 — `Rename conversation → 200`

**Suite :** `api-sanity.spec.ts > Sécurité — Message conversation CRUD`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  263 |
  264 |   test('Rename conversation → 200', async ({ request }) => {
> 265 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  266 |       data: { username: 'admin', password: 'changeme2026' },
  267 |     });
  268 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:265:41
```

**Message :**
```
ReferenceError: BASE is not defined

  263 |
  264 |   test('Rename conversation → 200', async ({ request }) => {
> 265 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  266 |       data: { username: 'admin', password: 'changeme2026' },
  267 |     });
  268 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:265:41
```

### Échec 58 — `/call/fake-id avec auth → page charge`

**Suite :** `api-sanity.spec.ts > Sécurité — Call page access`
**Durée :** 21.2s

**Message :**
```
TimeoutError: page.waitForURL: Timeout 10000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================

  290 |     await page.fill('input[name="password"], input[type="password"]', 'E2eTest123!');
  291 |     await page.click('button[type="submit"]');
> 292 |     await page.waitForURL(/chat|change-password/, { timeout: 10000 });
      |                ^
  293 |
  294 |     await page.goto('http://localhost:6300/call/default_global');
  295 |     await page.waitForLoadState('networkidle', { timeout: 10000 });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:292:16
```

**Message :**
```
TimeoutError: page.waitForURL: Timeout 10000ms exceeded.
=========================== logs ===========================
waiting for navigation until "load"
============================================================

  290 |     await page.fill('input[name="password"], input[type="password"]', 'E2eTest123!');
  291 |     await page.click('button[type="submit"]');
> 292 |     await page.waitForURL(/chat|change-password/, { timeout: 10000 });
      |                ^
  293 |
  294 |     await page.goto('http://localhost:6300/call/default_global');
  295 |     await page.waitForLoadState('networkidle', { timeout: 10000 });
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:292:16
```

### Échec 59 — `Créer partie → jouer e2→e4 → IA répond`

**Suite :** `api-sanity.spec.ts > Sécurité — Chess spécial`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  304 |   test('Créer partie → jouer e2→e4 → IA répond', async ({ request }) => {
  305 |     // Create game
> 306 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  307 |       data: { username: 'e2e_ci', password: 'E2eTest123!' },
  308 |     });
  309 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:306:41
```

**Message :**
```
ReferenceError: BASE is not defined

  304 |   test('Créer partie → jouer e2→e4 → IA répond', async ({ request }) => {
  305 |     // Create game
> 306 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  307 |       data: { username: 'e2e_ci', password: 'E2eTest123!' },
  308 |     });
  309 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:306:41
```

### Échec 60 — `1 char → 400`

**Suite :** `api-sanity.spec.ts > Sécurité renforcée — Mot de passe faible`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  361 | test.describe('Sécurité renforcée — Mot de passe faible', () => {
  362 |   test('1 char → 400', async ({ request }) => {
> 363 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  364 |       data: { username: 'weak1', password: 'a', email: 'w1@nook.local', name: 'W1' },
  365 |     });
  366 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:363:39
```

**Message :**
```
ReferenceError: BASE is not defined

  361 | test.describe('Sécurité renforcée — Mot de passe faible', () => {
  362 |   test('1 char → 400', async ({ request }) => {
> 363 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  364 |       data: { username: 'weak1', password: 'a', email: 'w1@nook.local', name: 'W1' },
  365 |     });
  366 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:363:39
```

### Échec 61 — `5 chars → 400`

**Suite :** `api-sanity.spec.ts > Sécurité renforcée — Mot de passe faible`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  368 |
  369 |   test('5 chars → 400', async ({ request }) => {
> 370 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  371 |       data: { username: 'weak2', password: 'abcde', email: 'w2@nook.local', name: 'W2' },
  372 |     });
  373 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:370:39
```

**Message :**
```
ReferenceError: BASE is not defined

  368 |
  369 |   test('5 chars → 400', async ({ request }) => {
> 370 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  371 |       data: { username: 'weak2', password: 'abcde', email: 'w2@nook.local', name: 'W2' },
  372 |     });
  373 |     expect(res.status()).toBe(400);
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:370:39
```

### Échec 62 — `8 chars → accepte`

**Suite :** `api-sanity.spec.ts > Sécurité renforcée — Mot de passe faible`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  375 |
  376 |   test('8 chars → accepte', async ({ request }) => {
> 377 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  378 |       data: { username: 'okpwd', password: 'Test1234', email: 'ok@nook.local', name: 'OK' },
  379 |     });
  380 |     expect([200, 409]).toContain(res.status());
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:377:39
```

**Message :**
```
ReferenceError: BASE is not defined

  375 |
  376 |   test('8 chars → accepte', async ({ request }) => {
> 377 |     const res = await request.post(`${BASE}/auth/register`, {
      |                                       ^
  378 |       data: { username: 'okpwd', password: 'Test1234', email: 'ok@nook.local', name: 'OK' },
  379 |     });
  380 |     expect([200, 409]).toContain(res.status());
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:377:39
```

### Échec 63 — `User change pwd autre user → 403 (integration)`

**Suite :** `api-sanity.spec.ts > Sécurité — Change password autre user → 403 (fix C1)`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  384 | test.describe('Sécurité — Change password autre user → 403 (fix C1)', () => {
  385 |   test('User change pwd autre user → 403 (integration)', async ({ request }) => {
> 386 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  387 |       data: { username: 'e2e_ci', password: 'E2eTest123!' },
  388 |     });
  389 |     if (login.status() === 200) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:386:41
```

**Message :**
```
ReferenceError: BASE is not defined

  384 | test.describe('Sécurité — Change password autre user → 403 (fix C1)', () => {
  385 |   test('User change pwd autre user → 403 (integration)', async ({ request }) => {
> 386 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  387 |       data: { username: 'e2e_ci', password: 'E2eTest123!' },
  388 |     });
  389 |     if (login.status() === 200) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:386:41
```

### Échec 64 — `Upload sec -- fichier vide refuse → 400 (second block)`

**Suite :** `api-sanity.spec.ts > Sécurité — Upload validation`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  398 | test.describe('Sécurité — Upload validation', () => {
  399 |   test('Upload sec -- fichier vide refuse → 400 (second block)', async ({ request }) => {
> 400 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  401 |       data: { username: 'admin', password: 'changeme2026' },
  402 |     });
  403 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:400:41
```

**Message :**
```
ReferenceError: BASE is not defined

  398 | test.describe('Sécurité — Upload validation', () => {
  399 |   test('Upload sec -- fichier vide refuse → 400 (second block)', async ({ request }) => {
> 400 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  401 |       data: { username: 'admin', password: 'changeme2026' },
  402 |     });
  403 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:400:41
```

### Échec 65 — `Upload fichier texte → 200`

**Suite :** `api-sanity.spec.ts > Sécurité — Upload validation`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  414 |
  415 |   test('Upload fichier texte → 200', async ({ request }) => {
> 416 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  417 |       data: { username: 'admin', password: 'changeme2026' },
  418 |     });
  419 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:416:41
```

**Message :**
```
ReferenceError: BASE is not defined

  414 |
  415 |   test('Upload fichier texte → 200', async ({ request }) => {
> 416 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  417 |       data: { username: 'admin', password: 'changeme2026' },
  418 |     });
  419 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:416:41
```

### Échec 66 — `Upload → Download end-to-end`

**Suite :** `api-sanity.spec.ts > Sécurité — Upload validation`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  432 |
  433 |   test('Upload → Download end-to-end', async ({ request }) => {
> 434 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  435 |       data: { username: 'admin', password: 'changeme2026' },
  436 |     });
  437 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:434:41
```

**Message :**
```
ReferenceError: BASE is not defined

  432 |
  433 |   test('Upload → Download end-to-end', async ({ request }) => {
> 434 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  435 |       data: { username: 'admin', password: 'changeme2026' },
  436 |     });
  437 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:434:41
```

### Échec 67 — `Download inexistant → 404`

**Suite :** `api-sanity.spec.ts > Sécurité — Upload validation`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  453 |
  454 |   test('Download inexistant → 404', async ({ request }) => {
> 455 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  456 |       data: { username: 'admin', password: 'changeme2026' },
  457 |     });
  458 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:455:41
```

**Message :**
```
ReferenceError: BASE is not defined

  453 |
  454 |   test('Download inexistant → 404', async ({ request }) => {
> 455 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  456 |       data: { username: 'admin', password: 'changeme2026' },
  457 |     });
  458 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:455:41
```

### Échec 68 — `Envoyer message → 200`

**Suite :** `api-sanity.spec.ts > Sécurité — Message CRUD conversation`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  467 |
  468 |   test('Envoyer message → 200', async ({ request }) => {
> 469 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  470 |       data: { username: 'admin', password: 'changeme2026' },
  471 |     });
  472 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:469:41
```

**Message :**
```
ReferenceError: BASE is not defined

  467 |
  468 |   test('Envoyer message → 200', async ({ request }) => {
> 469 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  470 |       data: { username: 'admin', password: 'changeme2026' },
  471 |     });
  472 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:469:41
```

### Échec 69 — `Rename conversation → 200 (second block)`

**Suite :** `api-sanity.spec.ts > Sécurité — Message CRUD conversation`
**Durée :** 0.0s

**Message :**
```
ReferenceError: BASE is not defined

  521 |
  522 |   test('Rename conversation → 200 (second block)', async ({ request }) => {
> 523 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  524 |       data: { username: 'admin', password: 'changeme2026' },
  525 |     });
  526 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:523:41
```

**Message :**
```
ReferenceError: BASE is not defined

  521 |
  522 |   test('Rename conversation → 200 (second block)', async ({ request }) => {
> 523 |     const login = await request.post(`${BASE}/auth/login`, {
      |                                         ^
  524 |       data: { username: 'admin', password: 'changeme2026' },
  525 |     });
  526 |     if (login.ok()) {
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:523:41
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
> - **URL du run :** [https://github.com/MX10-AC2N/Nook/actions/runs/27185591768](https://github.com/MX10-AC2N/Nook/actions/runs/27185591768)
> - **Chemin local (CI) :** `frontend/playwright-report/`

Pour examiner visuellement les échecs :
1. Télécharger l'artifact `playwright-report` depuis le [run CI](https://github.com/MX10-AC2N/Nook/actions/runs/27185591768)
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-06-09 05:37 UTC*
