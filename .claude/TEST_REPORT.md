# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-04-05 08:01 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ✅ **SUCCÈS** · ⚠️ **2 flaky** |
| **Tests passés** | 157 |
| **Tests échoués** | 0 |
| **Tests flaky** | 2 |
| **Tests ignorés** | 0 |
| **Total** | 159 |
| **Durée totale** | 1m 9.1s |
| **Branche** | `develop` |
| **Commit** | [`27d4be6`](https://github.com/MX10-AC2N/Nook/commit/27d4be66b5d7d080cc4e93613e18a9fc7d316b63) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23997246932) |

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
| 1 | Chess UI — plateau 64 cases + sélection case + coup via UI | 25.0s | `?` |
| 2 | Register + Approve + Login → accès complet | 4.2s | `?` |
| 3 | /call/default_global → page contient contenu call | 4.1s | `?` |
| 4 | Login invalide → reste sur /login | 3.3s | `?` |
| 5 | Chess — POST /chess/{id}/ai-move → 200 | 2.4s | `?` |
| 6 | Page /admin → non accessible pour user normal | 2.1s | `?` |
| 7 | Admin — approve + login after approve → accès complet | 2.1s | `?` |
| 8 | Admin — DELETE /users/{id} → supprime un utilisateur | 2.0s | `?` |
| 9 | /call/default_global → page charge avec titres | 1.6s | `?` |
| 10 | Créer partie vs IA (facile) → game_id | 1.1s | `?` |

---

## 📋 Résultats par catégorie

### ❌ **admin.spec.ts** — 24/25 passés · ⚠️ 1 flaky

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⚠️ | GET /analytics → contient user_count, message_count | 0.0s  +1 | |
| ✅ | Register + Approve + Login → accès complet | 4.2s | |
| ✅ | Page /admin → non accessible pour user normal | 2.1s | |
| ✅ | Admin — approve + login after approve → accès complet | 2.1s | |
| ✅ | Admin — DELETE /users/{id} → supprime un utilisateur | 2.0s | |
| ✅ | DELETE /users/{id} → supprime un utilisateur | 1.1s | |
| ✅ | Admin — analytics contient toutes les sections | 0.9s | |
| ✅ | Onglet "Membres" → users visibles dans UI | 0.1s | |
| ✅ | Page /admin/analytics → stat-cards + 2 charts | 0.1s | |
| ✅ | Admin UI — invitation générée visible dans l'interface | 0.1s | |
| ✅ | Flux inscription : register → pending → approve → connecté | 0.1s | |
| ✅ | GET /analytics avec user normal → 403 | 0.1s | |
| ✅ | GET /users/pending avec user normal → 403 | 0.1s | |
| ✅ | Admin — page /admin chargée avec header | 0.0s | |
| ✅ | Admin — 3 onglets visibles | 0.0s | |
| ✅ | Admin — DELETE /polls/{id} → 200 | 0.0s | |
| ✅ | GET /analytics → tous les champs requis | 0.0s | |
| ✅ | GET /auth/me avec session admin → role=admin | 0.0s | |
| ✅ | GET /invite/validate?token=xxx → valide le token | 0.0s | |
| ✅ | GET /users → liste complète (admin) | 0.0s | |
| ✅ | GET /users/pending → 200 | 0.0s | |
| ✅ | POST /invites → génère un invite_link valide | 0.0s | |
| ✅ | POST /invites/delete → supprime une invitation | 0.0s | |
| ✅ | GET /analytics sans auth → 401 | 0.0s | |
| ✅ | GET /invites → liste non vide | 0.0s | |

### ❌ **api-sanity.spec.ts** — 75/76 passés · ⚠️ 1 flaky

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ⚠️ | Chess resign → status finished | 0.0s  +1 | |
| ✅ | /call/fake-id avec auth → page charge | 1.1s | |
| ✅ | Créer partie → jouer e2→e4 → IA répond | 0.5s | |
| ✅ | /call/fake-id sans auth → redirige vers /login | 0.2s | |
| ✅ | GET /api/health → "OK" | 0.0s | |
| ✅ | 8 chars → accepte | 0.0s | |
| ✅ | 1 char → 400 | 0.0s | |
| ✅ | User change pwd autre user → 403 (integration) | 0.0s | |
| ✅ | User normal change pwd autre user → 403 | 0.0s | |
| ✅ | Upload sec -- fichier vide refuse → 400 (second block) | 0.0s | |
| ✅ | Mot de passe 8 chars → accepte | 0.0s | |
| ✅ | Upload fichier texte → file_id, puis download OK | 0.0s | |
| ✅ | Download fichier inexistant → 404 | 0.0s | |
| ✅ | Upload → Download end-to-end | 0.0s | |
| ✅ | Envoyer message → 200 | 0.0s | |
| ✅ | Rename conversation → 200 (second block) | 0.0s | |
| ✅ | Upload fichier vide → 400 | 0.0s | |
| ✅ | Envoyer message → 200, récupérer → contient message | 0.0s | |
| ✅ | Rename conversation → 200 | 0.0s | |
| ✅ | Upload fichier texte → 200 | 0.0s | |
| ✅ | Download inexistant → 404 | 0.0s | |
| ✅ | POST /api/upload/chat sans auth → 401 | 0.0s | |
| ✅ | GET /auth/me → 401 | 0.0s | |
| ✅ | 5 chars → 400 | 0.0s | |
| ✅ | GET /push/vapid-public-key → 200 (route publique, pas d'auth requise) | 0.0s | |
| ✅ | POST /auth/logout → 401 | 0.0s | |
| ✅ | POST /auth/change-password → 401 | 0.0s | |
| ✅ | POST /auth/public-key → 401 | 0.0s | |
| ✅ | GET /auth/public-keys?conversation_id=default_global → 401 | 0.0s | |
| ✅ | POST /conversations → 401 | 0.0s | |
| ✅ | GET /conversations/default_global → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/messages → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/participants → 401 | 0.0s | |
| ✅ | POST /polls/fake-id/vote → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | POST /invites → 401 | 0.0s | |
| ✅ | GET /conversations → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/messages → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/participants → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/leave → 401 | 0.0s | |
| ✅ | PATCH /conversations/default_global/rename → 401 | 0.0s | |
| ✅ | GET /events → 401 | 0.0s | |
| ✅ | POST /events → 401 | 0.0s | |
| ✅ | DELETE /events/fake-id → 401 | 0.0s | |
| ✅ | POST /polls → 401 | 0.0s | |
| ✅ | GET /polls/fake-id → 401 | 0.0s | |
| ✅ | POST /polls/fake-id/close → 401 | 0.0s | |
| ✅ | DELETE /polls/fake-id → 401 | 0.0s | |
| ✅ | GET /chess/list → 401 | 0.0s | |
| ✅ | POST /chess/create → 401 | 0.0s | |
| ✅ | GET /chess/fake-id → 401 | 0.0s | |
| ✅ | GET /chess/fake-id/moves?from=e2 → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/resign → 401 | 0.0s | |
| ✅ | DELETE /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | POST /user/update → 401 | 0.0s | |
| ✅ | GET /push/preferences → 401 | 0.0s | |
| ✅ | GET /users/pending → 401 | 0.0s | |
| ✅ | GET /users → 401 | 0.0s | |
| ✅ | POST /users/approve → 401 | 0.0s | |
| ✅ | POST /invites/delete → 401 | 0.0s | |
| ✅ | Mot de passe 5 chars → 400 | 0.0s | |
| ✅ | Chess coup illégal → 400 | 0.0s | |
| ✅ | GET /download/fake-id-000 → 401 | 0.0s | |
| ✅ | GET /polls → 401 | 0.0s | |
| ✅ | GET /chess/invitations → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/move → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/ai-move → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | GET /users/available → 401 | 0.0s | |
| ✅ | GET /invites → 401 | 0.0s | |
| ✅ | GET /analytics → 401 | 0.0s | |
| ✅ | Mot de passe 1 char → 400 | 0.0s | |
| ✅ | Mot de passe 7 chars → 400 | 0.0s | |
| ✅ | Modifier message → 200 | 0.0s | |
| ✅ | Lister messages → contient le message modifié | 0.0s | |
| ✅ | Supprimer message → 200/204 | 0.0s | |

### ✅ **user.spec.ts** — 58/58 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ✅ | Chess UI — plateau 64 cases + sélection case + coup via UI | 25.0s | |
| ✅ | /call/default_global → page contient contenu call | 4.1s | |
| ✅ | Login invalide → reste sur /login | 3.3s | |
| ✅ | Chess — POST /chess/{id}/ai-move → 200 | 2.4s | |
| ✅ | /call/default_global → page charge avec titres | 1.6s | |
| ✅ | Créer partie vs IA (facile) → game_id | 1.1s | |
| ✅ | Réactions UI — hover → picker → pill visible | 1.1s | |
| ✅ | Navigation /polls → accessible sans erreur | 0.6s | |
| ✅ | Navigation /chat → accessible sans erreur | 0.6s | |
| ✅ | Navigation /chess → accessible sans erreur | 0.5s | |
| ✅ | Navigation /events → accessible sans erreur | 0.5s | |
| ✅ | Navigation /calendar → accessible sans erreur | 0.5s | |
| ✅ | Navigation /help → accessible sans erreur | 0.5s | |
| ✅ | Navigation /settings → accessible sans erreur | 0.5s | |
| ✅ | Polls UI — créer sondage via formulaire → visible dans liste | 0.4s | |
| ✅ | /call/[id] session → page appel chargee (sans auth first) | 0.2s | |
| ✅ | Logout UI → redirigé vers /login | 0.2s | |
| ✅ | Settings UI — 3 onglets navigables | 0.2s | |
| ✅ | Settings — changement de thème (clic → sélectionné) | 0.1s | |
| ✅ | Chat UI — sidebar et envoi message | 0.1s | |
| ✅ | Calendar UI — page, grille et bouton ajouter visibles | 0.1s | |
| ✅ | Chess — invitations : créer, inviter, lister, décliner | 0.1s | |
| ✅ | Flood /auth/login × 20 depuis même IP → au moins un 429 | 0.0s | |
| ✅ | Polls — cycle complet : créer → voter → changer → double vote → fermer → vote fermé | 0.0s | |
| ✅ | Chess — coups légaux depuis e2 → contient e3 et e4 | 0.0s | |
| ✅ | Chess — resign → status finished | 0.0s | |
| ✅ | Chess — UI plateau 8x8 (64 cases) | 0.0s | |
| ✅ | Chess — coup illégal → 400 | 0.0s | |
| ✅ | Chess — coup légal e2→e4 | 0.0s | |
| ✅ | Login e2e_ci → redirigé vers /chat | 0.0s | |
| ✅ | Upload — fichier texte → file_id, url=/api/download/, download OK | 0.0s | |
| ✅ | Chess — créer vs IA, coups légaux, coup légal e2→e4, coup illégal → 400 | 0.0s | |
| ✅ | Réactions — UPSERT : 👍 → ❤️ remplace sans doublon | 0.0s | |
| ✅ | Réactions — DELETE → my_emoji null | 0.0s | |
| ✅ | Réactions — GET → structure {message_id, counts, my_emoji} | 0.0s | |
| ✅ | Chess — POST /chess/{id}/resign → 200 | 0.0s | |
| ✅ | Réactions — POST emoji valide 👍 → counts mis à jour | 0.0s | |
| ✅ | GET /auth/me → username=e2e_ci | 0.0s | |
| ✅ | Réactions — POST emoji non autorisé 🦄 → 400 | 0.0s | |
| ✅ | POST /user/update → mise à jour du nom | 0.0s | |
| ✅ | Calendar — POST /events → crée et DELETE /events/{id} → supprime | 0.0s | |
| ✅ | E2EE — GET /auth/public-keys → objet avec clés des membres | 0.0s | |
| ✅ | GET /conversations/default_global/messages → messages récupérés | 0.0s | |
| ✅ | GET /chess/list → 200 | 0.0s | |
| ✅ | GET /conversations → default_global présente | 0.0s | |
| ✅ | GET /conversations/default_global/participants → e2e_ci présent | 0.0s | |
| ✅ | POST /conversations → créer un groupe de test | 0.0s | |
| ✅ | GET /users/available → liste des membres disponibles | 0.0s | |
| ✅ | Push — GET /push/preferences → prefs par défaut | 0.0s | |
| ✅ | GET /conversations/default_global → détail de la conv | 0.0s | |
| ✅ | Réactions — message inexistant → 404 | 0.0s | |
| ✅ | GET /polls → tableau de sondages | 0.0s | |
| ✅ | Calendar — GET /events → 200 | 0.0s | |
| ✅ | E2EE — POST /auth/public-key → enregistre la clé | 0.0s | |
| ✅ | Push — POST /push/preferences → mise à jour | 0.0s | |
| ✅ | Download — id inexistant → 404 | 0.0s | |
| ✅ | Push — GET /push/vapid-public-key → 200 | 0.0s | |
| ✅ | Push — POST /push/subscribe → 200 | 0.0s | |

---

## 📁 Résultats par fichier de test

| Fichier | ✅ Passés | ❌ Échoués | ⚠️ Flaky | Total |
|---------|-----------|-------------|-----------|-------|
| ❌ `unknown` | 157 | 0 | 2 | 159 |

---

## ✅ Aucun échec

Tous les tests ont passé.

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=fd68ea06-1d47-467b-8856-14a099770731 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=fd68ea06-1d47-467b-8856-14a099770731 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=fd68ea06-1d47-467b-8856-14a099770731 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=fd68ea06-1d47-467b-8856-14a099770731 username=e2e_ci
```

---

## 🖼️ Rapport HTML Playwright

> Le rapport HTML complet est disponible en artifact GitHub Actions.
>
> - **Nom de l'artifact :** `playwright-report`
> - **URL du run :** [https://github.com/MX10-AC2N/Nook/actions/runs/23997246932](https://github.com/MX10-AC2N/Nook/actions/runs/23997246932)
> - **Chemin local (CI) :** `frontend/playwright-report/`

Pour examiner visuellement les échecs :
1. Télécharger l'artifact `playwright-report` depuis le [run CI](https://github.com/MX10-AC2N/Nook/actions/runs/23997246932)
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-04-05 08:01 UTC*
