# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-04-11 18:55 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ❌ **ÉCHEC** · ⚠️ **1 flaky** |
| **Tests passés** | 160 |
| **Tests échoués** | 1 |
| **Tests flaky** | 1 |
| **Tests ignorés** | 1 |
| **Total** | 163 |
| **Durée totale** | 1m 40.2s |
| **Branche** | `develop` |
| **Commit** | [`a2c4412`](https://github.com/MX10-AC2N/Nook/commit/a2c4412f3e88f42f79ba602ebdccdef028814897) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/24288952502) |

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
| 1 | Réactions UI — hover → picker → pill visible | 29.4s | `?` |
| 2 | Login invalide → reste sur /login | 7.1s | `?` |
| 3 | Chess UI — plateau 64 cases + sélection case + coup via UI | 5.1s | `?` |
| 4 | Register + Approve + Login → accès complet | 4.1s | `?` |
| 5 | /call/default_global → page contient contenu call | 4.0s | `?` |
| 6 | Créer partie → jouer e2→e4 → IA répond | 3.7s | `?` |
| 7 | Avatar — composant visible avec initiales dans le chat | 2.2s | `?` |
| 8 | Chess — POST /chess/{id}/ai-move → 200 | 2.2s | `?` |
| 9 | Admin — approve + login after approve → accès complet | 2.2s | `?` |
| 10 | Page /admin → non accessible pour user normal | 2.1s | `?` |

---

## 📋 Résultats par catégorie

### ❌ **user.spec.ts** — 59/62 passés · ⚠️ 1 flaky · ❌ 1 failed

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ❌ | Chess — sélection pièce → coups légaux visibles (dots) | 0.1s  +1 | |
| ⚠️ | Réactions UI — hover → picker → pill visible | 29.4s  +1 | |
| ✅ | Login invalide → reste sur /login | 7.1s  +1 | |
| ✅ | Chess UI — plateau 64 cases + sélection case + coup via UI | 5.1s  +1 | |
| ✅ | /call/default_global → page contient contenu call | 4.0s | |
| ✅ | Avatar — composant visible avec initiales dans le chat | 2.2s  +1 | |
| ✅ | Chess — POST /chess/{id}/ai-move → 200 | 2.2s  +1 | |
| ✅ | /call/default_global → page charge avec titres | 1.5s | |
| ✅ | Settings — section avatar visible avec grille d'options | 1.3s  +1 | |
| ✅ | Chat UI — sidebar et envoi message | 1.2s  +1 | |
| ✅ | Calendar — switcher vue Mois/Semaine/Jour visible | 1.2s  +1 | |
| ✅ | Créer partie vs IA (facile) → game_id | 1.0s | |
| ✅ | Navigation /polls → accessible sans erreur | 0.7s  +1 | |
| ✅ | Navigation /chat → accessible sans erreur | 0.6s  +1 | |
| ✅ | Navigation /help → accessible sans erreur | 0.6s  +1 | |
| ✅ | Navigation /chess → accessible sans erreur | 0.6s  +1 | |
| ✅ | Navigation /events → accessible sans erreur | 0.6s  +1 | |
| ✅ | Navigation /settings → accessible sans erreur | 0.6s  +1 | |
| ✅ | Navigation /calendar → accessible sans erreur | 0.6s  +1 | |
| ✅ | Polls UI — créer sondage via formulaire → visible dans liste | 0.6s  +1 | |
| ✅ | /call/[id] session → page appel chargee (sans auth first) | 0.3s | |
| ✅ | Settings UI — 3 onglets navigables | 0.3s  +1 | |
| ✅ | Settings — changement de thème (clic → sélectionné) | 0.2s  +1 | |
| ✅ | Calendar UI — page, grille et bouton ajouter visibles | 0.2s  +1 | |
| ✅ | Chess — invitations : créer, inviter, lister, décliner | 0.1s  +1 | |
| ✅ | Flood /auth/login × 20 depuis même IP → au moins un 429 | 0.1s | |
| ✅ | Polls — cycle complet : créer → voter → changer → double vote → fermer → vote fermé | 0.1s  +1 | |
| ✅ | Login e2e_ci → redirigé vers /chat | 0.1s  +1 | |
| ✅ | Upload — fichier texte → file_id, url=/api/download/, download OK | 0.0s  +1 | |
| ✅ | Chess — UI plateau 8x8 (64 cases) | 0.0s | |
| ✅ | Chess — resign → status finished | 0.0s | |
| ✅ | Chess — créer vs IA, coups légaux, coup légal e2→e4, coup illégal → 400 | 0.0s  +1 | |
| ✅ | Chess — coup légal e2→e4 | 0.0s | |
| ✅ | Réactions — UPSERT : 👍 → ❤️ remplace sans doublon | 0.0s  +1 | |
| ✅ | Réactions — DELETE → my_emoji null | 0.0s  +1 | |
| ✅ | Chess — coup illégal → 400 | 0.0s | |
| ✅ | Réactions — GET → structure {message_id, counts, my_emoji} | 0.0s  +1 | |
| ✅ | Chess — coups légaux depuis e2 → contient e3 et e4 | 0.0s | |
| ✅ | Réactions — POST emoji valide 👍 → counts mis à jour | 0.0s  +1 | |
| ✅ | GET /auth/me → username=e2e_ci | 0.0s  +1 | |
| ✅ | Réactions — POST emoji non autorisé 🦄 → 400 | 0.0s  +1 | |
| ✅ | Calendar — POST /events → crée et DELETE /events/{id} → supprime | 0.0s  +1 | |
| ✅ | GET /conversations/default_global → détail de la conv | 0.0s  +1 | |
| ✅ | Chess — POST /chess/{id}/resign → 200 | 0.0s  +1 | |
| ✅ | GET /conversations → default_global présente | 0.0s  +1 | |
| ✅ | GET /conversations/default_global/participants → e2e_ci présent | 0.0s  +1 | |
| ✅ | GET /conversations/default_global/messages → messages récupérés | 0.0s  +1 | |
| ✅ | E2EE — GET /auth/public-keys → objet avec clés des membres | 0.0s  +1 | |
| ✅ | POST /conversations → créer un groupe de test | 0.0s  +1 | |
| ✅ | POST /user/update → mise à jour du nom | 0.0s  +1 | |
| ✅ | GET /users/available → liste des membres disponibles | 0.0s  +1 | |
| ✅ | Push — GET /push/preferences → prefs par défaut | 0.0s  +1 | |
| ✅ | Réactions — message inexistant → 404 | 0.0s  +1 | |
| ✅ | GET /polls → tableau de sondages | 0.0s  +1 | |
| ✅ | GET /chess/list → 200 | 0.0s  +1 | |
| ✅ | E2EE — POST /auth/public-key → enregistre la clé | 0.0s  +1 | |
| ✅ | Push — POST /push/subscribe → 200 | 0.0s  +1 | |
| ✅ | Download — id inexistant → 404 | 0.0s  +1 | |
| ✅ | Calendar — GET /events → 200 | 0.0s  +1 | |
| ✅ | Push — GET /push/vapid-public-key → 200 | 0.0s  +1 | |
| ✅ | Push — POST /push/preferences → mise à jour | 0.0s  +1 | |
| ⏭️ | Logout UI → redirigé vers /login | N/A  +1 | |

### ✅ **admin.spec.ts** — 25/25 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ✅ | Register + Approve + Login → accès complet | 4.1s | |
| ✅ | Admin — approve + login after approve → accès complet | 2.2s | |
| ✅ | Page /admin → non accessible pour user normal | 2.1s | |
| ✅ | Admin — DELETE /users/{id} → supprime un utilisateur | 2.1s | |
| ✅ | Admin — analytics contient toutes les sections | 1.1s | |
| ✅ | DELETE /users/{id} → supprime un utilisateur | 1.1s | |
| ✅ | Onglet "Membres" → users visibles dans UI | 0.2s | |
| ✅ | Admin UI — invitation générée visible dans l'interface | 0.2s | |
| ✅ | Flux inscription : register → pending → approve → connecté | 0.1s | |
| ✅ | GET /analytics → contient user_count, message_count | 0.1s | |
| ✅ | Page /admin/analytics → stat-cards + 2 charts | 0.1s | |
| ✅ | GET /analytics avec user normal → 403 | 0.1s | |
| ✅ | GET /users/pending avec user normal → 403 | 0.1s | |
| ✅ | Admin — DELETE /polls/{id} → 200 | 0.0s | |
| ✅ | Admin — 3 onglets visibles | 0.0s | |
| ✅ | Admin — page /admin chargée avec header | 0.0s | |
| ✅ | GET /invite/validate?token=xxx → valide le token | 0.0s | |
| ✅ | GET /auth/me avec session admin → role=admin | 0.0s | |
| ✅ | GET /analytics → tous les champs requis | 0.0s | |
| ✅ | POST /invites → génère un invite_link valide | 0.0s | |
| ✅ | POST /invites/delete → supprime une invitation | 0.0s | |
| ✅ | GET /users → liste complète (admin) | 0.0s | |
| ✅ | GET /users/pending → 200 | 0.0s | |
| ✅ | GET /analytics sans auth → 401 | 0.0s | |
| ✅ | GET /invites → liste non vide | 0.0s | |

### ✅ **api-sanity.spec.ts** — 76/76 passés

| Statut | Test | Durée | Retries |
|--------|------|-------|---------|
| ✅ | Créer partie → jouer e2→e4 → IA répond | 3.7s | |
| ✅ | /call/fake-id avec auth → page charge | 1.2s | |
| ✅ | /call/fake-id sans auth → redirige vers /login | 0.9s | |
| ✅ | User change pwd autre user → 403 (integration) | 0.1s | |
| ✅ | Upload → Download end-to-end | 0.1s | |
| ✅ | Chess resign → status finished | 0.1s | |
| ✅ | Upload fichier texte → 200 | 0.1s | |
| ✅ | Upload sec -- fichier vide refuse → 400 (second block) | 0.1s | |
| ✅ | Envoyer message → 200 | 0.1s | |
| ✅ | 8 chars → accepte | 0.0s | |
| ✅ | Download inexistant → 404 | 0.0s | |
| ✅ | Rename conversation → 200 (second block) | 0.0s | |
| ✅ | User normal change pwd autre user → 403 | 0.0s | |
| ✅ | Mot de passe 8 chars → accepte | 0.0s | |
| ✅ | Download fichier inexistant → 404 | 0.0s | |
| ✅ | Rename conversation → 200 | 0.0s | |
| ✅ | Upload fichier vide → 400 | 0.0s | |
| ✅ | Upload fichier texte → file_id, puis download OK | 0.0s | |
| ✅ | Envoyer message → 200, récupérer → contient message | 0.0s | |
| ✅ | GET /api/health → "OK" | 0.0s | |
| ✅ | POST /api/upload/chat sans auth → 401 | 0.0s | |
| ✅ | GET /auth/me → 401 | 0.0s | |
| ✅ | GET /push/vapid-public-key → 200 (route publique, pas d'auth requise) | 0.0s | |
| ✅ | POST /auth/logout → 401 | 0.0s | |
| ✅ | Chess coup illégal → 400 | 0.0s | |
| ✅ | POST /auth/change-password → 401 | 0.0s | |
| ✅ | POST /polls/fake-id/vote → 401 | 0.0s | |
| ✅ | 5 chars → 400 | 0.0s | |
| ✅ | POST /auth/public-key → 401 | 0.0s | |
| ✅ | GET /conversations/default_global → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/messages → 401 | 0.0s | |
| ✅ | 1 char → 400 | 0.0s | |
| ✅ | GET /auth/public-keys?conversation_id=default_global → 401 | 0.0s | |
| ✅ | GET /conversations → 401 | 0.0s | |
| ✅ | POST /conversations → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/messages → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/participants → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/participants → 401 | 0.0s | |
| ✅ | GET /events → 401 | 0.0s | |
| ✅ | POST /events → 401 | 0.0s | |
| ✅ | GET /polls/fake-id → 401 | 0.0s | |
| ✅ | POST /user/update → 401 | 0.0s | |
| ✅ | GET /users → 401 | 0.0s | |
| ✅ | POST /users/approve → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/leave → 401 | 0.0s | |
| ✅ | PATCH /conversations/default_global/rename → 401 | 0.0s | |
| ✅ | GET /download/fake-id-000 → 401 | 0.0s | |
| ✅ | DELETE /events/fake-id → 401 | 0.0s | |
| ✅ | GET /polls → 401 | 0.0s | |
| ✅ | POST /polls → 401 | 0.0s | |
| ✅ | POST /polls/fake-id/close → 401 | 0.0s | |
| ✅ | DELETE /polls/fake-id → 401 | 0.0s | |
| ✅ | GET /chess/list → 401 | 0.0s | |
| ✅ | POST /chess/create → 401 | 0.0s | |
| ✅ | GET /chess/fake-id → 401 | 0.0s | |
| ✅ | GET /chess/fake-id/moves?from=e2 → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/resign → 401 | 0.0s | |
| ✅ | POST /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | DELETE /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | GET /conversations/default_global/messages/x/reactions → 401 | 0.0s | |
| ✅ | GET /users/available → 401 | 0.0s | |
| ✅ | GET /push/preferences → 401 | 0.0s | |
| ✅ | GET /users/pending → 401 | 0.0s | |
| ✅ | POST /invites → 401 | 0.0s | |
| ✅ | POST /invites/delete → 401 | 0.0s | |
| ✅ | GET /analytics → 401 | 0.0s | |
| ✅ | Mot de passe 5 chars → 400 | 0.0s | |
| ✅ | Mot de passe 7 chars → 400 | 0.0s | |
| ✅ | Modifier message → 200 | 0.0s | |
| ✅ | GET /chess/invitations → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/move → 401 | 0.0s | |
| ✅ | POST /chess/fake-id/ai-move → 401 | 0.0s | |
| ✅ | GET /invites → 401 | 0.0s | |
| ✅ | Mot de passe 1 char → 400 | 0.0s | |
| ✅ | Supprimer message → 200/204 | 0.0s | |
| ✅ | Lister messages → contient le message modifié | 0.0s | |

---

## 📁 Résultats par fichier de test

| Fichier | ✅ Passés | ❌ Échoués | ⚠️ Flaky | Total |
|---------|-----------|-------------|-----------|-------|
| ❌ `unknown` | 160 | 1 | 1 | 163 |

---

## ❌ Échecs détaillés

> 1 test(s) en échec

### Échec 1 — `Chess — sélection pièce → coups légaux visibles (dots)`

**Suite :** `user.spec.ts > User — Flux complet`
**Durée :** 0.1s

**Message :**
```
Error: expect(received).toBeTruthy()

Received: undefined

  861 |     const createData = await createRes.json();
  862 |     const game_id = createData.game_id || createData.id;
> 863 |     expect(game_id).toBeTruthy();
      |                     ^
  864 |     
  865 |     // Naviguer vers la partie
  866 |     await page.goto(`/chess/${game_id}`);
    at /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts:863:21
```

---

## 🐳 Logs backend (warnings/erreurs)

```
WARN nook_backend: ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=5890518a-9ecc-4ff0-8c20-ca5f4a7c6878 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=5890518a-9ecc-4ff0-8c20-ca5f4a7c6878 username=e2e_ci
WARN nook_backend::auth: Tentative d'accès admin refusée (non-admin) user_id=5890518a-9ecc-4ff0-8c20-ca5f4a7c6878 username=e2e_ci
```

---

## 🖼️ Rapport HTML Playwright

> Le rapport HTML complet est disponible en artifact GitHub Actions.
>
> - **Nom de l'artifact :** `playwright-report`
> - **URL du run :** [https://github.com/MX10-AC2N/Nook/actions/runs/24288952502](https://github.com/MX10-AC2N/Nook/actions/runs/24288952502)
> - **Chemin local (CI) :** `frontend/playwright-report/`

Pour examiner visuellement les échecs :
1. Télécharger l'artifact `playwright-report` depuis le [run CI](https://github.com/MX10-AC2N/Nook/actions/runs/24288952502)
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-04-11 18:55 UTC*
