# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-03-19 06:18 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** |
| **Tests passés** | 55 |
| **Tests échoués** | 1 |
| **Tests ignorés** | 59 |
| **Total** | 115 |
| **Durée** | 6.0s |
| **Branche** | `develop` |
| **Commit** | [`d461c0d`](https://github.com/MX10-AC2N/Nook/commit/d461c0d8c77cc361d196838c8b55a1158d3ee1e3) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23282294906) |

---

## 🗂️ Suites de tests

| Suite | Fichier | Périmètre |
|-------|---------|-----------|
| **Sanité API** | `api-sanity.spec.ts` | 401/403 sur toutes les routes protégées |
| **Admin** | `admin.spec.ts` | Login, change-pwd, membres, inscription→approbation, invitations, analytics, isolation |
| **User** | `user.spec.ts` | Auth, chat, réactions, upload, polls, chess, calendar, settings, E2EE, push, navigation |

---

## 📋 Résultats par suite

### ✅ api-sanity.spec.ts — 48/48 passés

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

### ❌ admin.spec.ts — 7/18 passés

| Statut | Test | Durée |
|--------|------|-------|
| ✅ | Admin — page /admin chargée avec header | 0s |
| ✅ | Admin — 3 onglets visibles | 0s |
| ✅ | GET /auth/me avec session admin → role=admin | 0s |
| ✅ | GET /users → liste complète (admin) | 0s |
| ✅ | GET /users/pending → 200 | 0s |
| ✅ | Onglet "Membres" → users visibles dans UI | 0s |
| ✅ | Flux inscription : register → pending → approve → connecté | 0s |
| ❌ | POST /invites → génère un token valide | 0s |
| ⏭️ | GET /invites → liste non vide | N/A |
| ⏭️ | POST /invites/delete → supprime une invitation | N/A |
| ⏭️ | GET /invite/validate?token=xxx → valide le token | N/A |
| ⏭️ | Admin UI — invitation générée visible dans l'interface | N/A |
| ⏭️ | GET /analytics → tous les champs requis | N/A |
| ⏭️ | Page /admin/analytics → stat-cards + 2 charts | N/A |
| ⏭️ | Admin — DELETE /polls/{id} → 200 | N/A |
| ⏭️ | GET /analytics avec user normal → 403 | N/A |
| ⏭️ | GET /users/pending avec user normal → 403 | N/A |
| ⏭️ | Page /admin → non accessible pour user normal | N/A |

### ✅ user.spec.ts — 0/49 passés

| Statut | Test | Durée |
|--------|------|-------|
| ⏭️ | Login e2e_ci → redirigé vers /chat | N/A |
| ⏭️ | GET /auth/me → username=e2e_ci | N/A |
| ⏭️ | Login invalide → reste sur /login | N/A |
| ⏭️ | GET /conversations → default_global présente | N/A |
| ⏭️ | GET /conversations/default_global → détail de la conv | N/A |
| ⏭️ | GET /conversations/default_global/participants → e2e_ci présent | N/A |
| ⏭️ | Chat UI — sidebar et envoi message | N/A |
| ⏭️ | GET /conversations/default_global/messages → messages récupérés | N/A |
| ⏭️ | POST /conversations → créer un groupe de test | N/A |
| ⏭️ | GET /users/available → liste des membres disponibles | N/A |
| ⏭️ | Réactions — POST emoji valide 👍 → counts mis à jour | N/A |
| ⏭️ | Réactions — POST emoji non autorisé 🦄 → 400 | N/A |
| ⏭️ | Réactions — UPSERT : 👍 → ❤️ remplace sans doublon | N/A |
| ⏭️ | Réactions — DELETE → my_emoji null | N/A |
| ⏭️ | Réactions — GET → structure {message_id, counts, my_emoji} | N/A |
| ⏭️ | Réactions — message inexistant → 404 | N/A |
| ⏭️ | Réactions UI — hover → picker → pill visible | N/A |
| ⏭️ | Upload — fichier texte → file_id, url=/api/download/, download OK | N/A |
| ⏭️ | Download — id inexistant → 404 | N/A |
| ⏭️ | GET /polls → tableau de sondages | N/A |
| ⏭️ | Polls — cycle complet : créer → voter → changer → double vote → fermer → vote fermé | N/A |
| ⏭️ | Polls UI — créer sondage via formulaire → visible dans liste | N/A |
| ⏭️ | GET /chess/list → 200 | N/A |
| ⏭️ | Chess — créer vs IA, coups légaux, coup légal e2→e4, coup illégal → 400 | N/A |
| ⏭️ | Chess — POST /chess/{id}/ai-move → 200 | N/A |
| ⏭️ | Chess — POST /chess/{id}/resign → 200 | N/A |
| ⏭️ | Chess — invitations : créer, inviter, lister, décliner | N/A |
| ⏭️ | Chess UI — plateau 64 cases + sélection case + coup via UI | N/A |
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
| ⏭️ | Flood /auth/login × 20 depuis même IP → au moins un 429 | N/A |

---

## ❌ Échecs détaillés

> 1 test(s) en échec

### Échec 1 — `POST /invites → génère un token valide`

**Suite :** `admin.spec.ts > Admin — Flux complet`

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: 200
Received: 403

  150 |   test('POST /invites → génère un token valide', async () => {
  151 |     const res = await adminPage.request.post(`${BASE}/invites`);
> 152 |     expect(res.status()).toBe(200);
      |                          ^
  153 |     const body = await res.json();
  154 |     expect(body.token).toBeTruthy();
  155 |     expect(body.expires_at).toBeTruthy();
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:152:26
```

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: 200
Received: 403

  150 |   test('POST /invites → génère un token valide', async () => {
  151 |     const res = await adminPage.request.post(`${BASE}/invites`);
> 152 |     expect(res.status()).toBe(200);
      |                          ^
  153 |     const body = await res.json();
  154 |     expect(body.token).toBeTruthy();
  155 |     expect(body.expires_at).toBeTruthy();
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:152:26
```

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=792fcfd0-3bb4-440f-849f-29716af19882 username=testuser_1773901096730
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=d52159ef-61a9-46c4-b4fd-cfc3b169107f username=testuser_1773901098615
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-03-19 06:18 UTC*
