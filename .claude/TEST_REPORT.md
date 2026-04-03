# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-04-03 07:31 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** |
| **Tests passés** | 75 |
| **Tests échoués** | 1 |
| **Tests ignorés** | 69 |
| **Total** | 145 |
| **Durée** | 5s |
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

### ❌ api-sanity.spec.ts — 75/76 passés

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
| ❌ | Créer partie → jouer e2→e4 → IA répond | 0s |
| ✅ | Chess coup illégal → 400 | 0s |
| ✅ | Chess resign → status finished | 0s |
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

### ✅ admin.spec.ts — 0/25 passés

| Statut | Test | Durée |
|--------|------|-------|
| ⏭️ | Admin — page /admin chargée avec header | N/A |
| ⏭️ | Admin — 3 onglets visibles | N/A |
| ⏭️ | GET /auth/me avec session admin → role=admin | N/A |
| ⏭️ | GET /users → liste complète (admin) | N/A |
| ⏭️ | GET /users/pending → 200 | N/A |
| ⏭️ | Onglet "Membres" → users visibles dans UI | N/A |
| ⏭️ | Flux inscription : register → pending → approve → connecté | N/A |
| ⏭️ | POST /invites → génère un invite_link valide | N/A |
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
| ⏭️ | Admin — DELETE /users/{id} → supprime un utilisateur | N/A |
| ⏭️ | Admin — approve + login after approve → accès complet | N/A |
| ⏭️ | Admin — analytics contient toutes les sections | N/A |
| ⏭️ | DELETE /users/{id} → supprime un utilisateur | N/A |
| ⏭️ | GET /analytics → contient user_count, message_count | N/A |
| ⏭️ | GET /analytics sans auth → 401 | N/A |
| ⏭️ | Register + Approve + Login → accès complet | N/A |

### ✅ user.spec.ts — 0/44 passés

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
| ⏭️ | Flood /auth/login × 20 depuis même IP → au moins un 429 | N/A |
| ⏭️ | /call/default_global → page charge avec titre "Appel" | N/A |
| ⏭️ | /call/default_global → page call charge correctement | N/A |
| ⏭️ | /call/[id] sans auth → redirige vers /login | N/A |
| ⏭️ | Créer partie vs IA (facile) | N/A |
| ⏭️ | Chess — UI plateau 8x8 (64 cases) avec sélection | N/A |
| ⏭️ | Chess — coup illégal → message erreur | N/A |
| ⏭️ | /call/default_global → page charge avec titres | N/A |
| ⏭️ | /call/default_global → page contient contenu call | N/A |
| ⏭️ | /call/[id] session → page appel chargee (sans auth first) | N/A |
| ⏭️ | Créer partie vs IA (facile) → game_id | N/A |
| ⏭️ | Chess — UI plateau 8x8 (64 cases) | N/A |
| ⏭️ | Chess — coup légal e2→e4 | N/A |
| ⏭️ | Chess — coup illégal → 400 | N/A |
| ⏭️ | Chess — coups légaux depuis e2 → contient e3 et e4 | N/A |
| ⏭️ | Chess — resign → status finished | N/A |

---

## ❌ Échecs détaillés

> 1 test(s) en échec

### Échec 1 — `Créer partie → jouer e2→e4 → IA répond`

**Suite :** `api-sanity.spec.ts > Sécurité — Chess spécial`

**Message :**
```
Error: expect(received).toContain(expected) // indexOf

Expected value: 201
Received array: [200, 409]

  312 |         data: { opponent: 'easy', color: 'white', time_limit_secs: 0 },
  313 |       });
> 314 |       expect([200, 409]).toContain(create.status());
      |                          ^
  315 |       const body = await create.json();
  316 |       chessGameId = body.game_id;
  317 |       expect(chessGameId).toBeTruthy();
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:314:26
```

**Message :**
```
Error: expect(received).toContain(expected) // indexOf

Expected value: 201
Received array: [200, 409]

  312 |         data: { opponent: 'easy', color: 'white', time_limit_secs: 0 },
  313 |       });
> 314 |       expect([200, 409]).toContain(create.status());
      |                          ^
  315 |       const body = await create.json();
  316 |       chessGameId = body.game_id;
  317 |       expect(chessGameId).toBeTruthy();
    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:314:26
```

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-04-03 07:31 UTC*
