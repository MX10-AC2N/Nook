# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-03-28 07:04 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ✅ **SUCCÈS** |
| **Tests passés** | 115 |
| **Tests échoués** | 0 |
| **Tests ignorés** | 0 |
| **Total** | 115 |
| **Durée** | 22.0s |
| **Branche** | `develop` |
| **Commit** | [`b131f6d`](https://github.com/MX10-AC2N/Nook/commit/b131f6d7096ba9a54fb43f2f8abfd070ecffdf77) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23679846018) |

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

### ✅ admin.spec.ts — 18/18 passés

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

### ✅ user.spec.ts — 49/49 passés

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
| ✅ | Chess — POST /chess/{id}/ai-move → 200 | 2s |
| ✅ | Chess — POST /chess/{id}/resign → 200 | 0s |
| ✅ | Chess — invitations : créer, inviter, lister, décliner | 0s |
| ✅ | Chess UI — plateau 64 cases + sélection case + coup via UI | 0s |
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

---

## ✅ Aucun échec

Tous les tests ont passé.

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=95515b9a-3857-4c3e-97b8-42b4065d6ed5 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=95515b9a-3857-4c3e-97b8-42b4065d6ed5 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=95515b9a-3857-4c3e-97b8-42b4065d6ed5 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=95515b9a-3857-4c3e-97b8-42b4065d6ed5 username=e2e_ci
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-03-28 07:04 UTC*
