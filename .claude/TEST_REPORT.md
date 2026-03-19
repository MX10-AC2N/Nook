# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-03-19 11:12 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** |
| **Tests passés** | 75 |
| **Tests échoués** | 2 |
| **Tests ignorés** | 37 |
| **Total** | 114 |
| **Durée** | 27.0s |
| **Branche** | `develop` |
| **Commit** | [`ba58046`](https://github.com/MX10-AC2N/Nook/commit/ba5804686802589baafee33b7922d34d9978625c) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23291967615) |

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

### ❌ admin.spec.ts — 10/18 passés

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
| ❌ | GET /invite/validate?token=xxx → valide le token | 0s |
| ⏭️ | Admin UI — invitation générée visible dans l'interface | N/A |
| ⏭️ | GET /analytics → tous les champs requis | N/A |
| ⏭️ | Page /admin/analytics → stat-cards + 2 charts | N/A |
| ⏭️ | Admin — DELETE /polls/{id} → 200 | N/A |
| ⏭️ | GET /analytics avec user normal → 403 | N/A |
| ⏭️ | GET /users/pending avec user normal → 403 | N/A |
| ⏭️ | Page /admin → non accessible pour user normal | N/A |

### ❌ user.spec.ts — 17/49 passés

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
| ⏭️ | Réactions UI — hover → picker → pill visible | 6s |
| ❌ | Upload — fichier texte → file_id, url=/api/download/, download OK | 0s |
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
| ✅ | Flood /auth/login × 20 depuis même IP → au moins un 429 | 0s |

---

## ❌ Échecs détaillés

> 2 test(s) en échec

### Échec 1 — `GET /invite/validate?token=xxx → valide le token`

**Suite :** `admin.spec.ts > Admin — Flux complet`

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: 200
Received: 400

  203 |     const { token } = await createRes.json();
  204 |     const res = await adminPage.request.get(`${BASE}/invite/validate?token=${token}`);
> 205 |     expect(res.status()).toBe(200);
      |                          ^
  206 |     const body = await res.json();
  207 |     expect(body.valid).toBe(true);
  208 |     console.log(`✅ Token validé → valid=${body.valid}`);
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:205:26
```

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: 200
Received: 400

  203 |     const { token } = await createRes.json();
  204 |     const res = await adminPage.request.get(`${BASE}/invite/validate?token=${token}`);
> 205 |     expect(res.status()).toBe(200);
      |                          ^
  206 |     const body = await res.json();
  207 |     expect(body.valid).toBe(true);
  208 |     console.log(`✅ Token validé → valid=${body.valid}`);
    at /home/runner/work/Nook/Nook/frontend/tests/admin.spec.ts:205:26
```

### Échec 2 — `Upload — fichier texte → file_id, url=/api/download/, download OK`

**Suite :** `user.spec.ts > User — Flux complet`

**Message :**
```
Error: expect(received).toBe(expected) // Object.is equality

Expected: 200
Received: 500

  281 |
  282 |     const dlRes = await page.request.get(`/api/download/${body.file_id}`);
> 283 |     expect(dlRes.status()).toBe(200);
      |                            ^
  284 |     const cd = dlRes.headers()['content-disposition'] ?? '';
  285 |     expect(cd).toContain('attachment');
  286 |     expect(cd).toContain('test-e2e.txt');
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:283:28
```

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
ERROR nook_backend::upload: Déchiffrement échoué file_id=d929534d-09a6-460a-8022-90846db29d6b err=Échec déchiffrement
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-03-19 11:12 UTC*
