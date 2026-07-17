# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-07-17 05:46 UTC**

---

## 📊 Résumé

| Indicateur | Valeur |
|-----------|--------|
| **Statut** | ⚠️ **AUCUN TEST** |
| **Tests passés** | 0 |
| **Tests échoués** | 0 |
| **Tests flaky** | 0 |
| **Tests ignorés** | 0 |
| **Total** | 0 |
| **Durée totale** | N/A |
| **Branche** | `develop` |
| **Commit** | [`3e33737`](https://github.com/MX10-AC2N/Nook/commit/3e33737f448c31bf3e22028ae671154bce1599f2) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/29558180334) |

---

## 🗂️ Suites de tests

| Suite | Fichier | Périmètre |
|-------|---------|-----------|
| **Sanité API** | `api-sanity.spec.ts` | 401/403 sur toutes les routes protégées |
| **Admin** | `admin.spec.ts` | Login, change-pwd, membres, inscription→approbation, invitations, analytics, isolation |
| **User** | `user.spec.ts` | Auth, chat, réactions, upload, polls, chess, calendar, settings, E2EE, push, navigation |

---

## ⚠️ Aucun test exécuté

Playwright n'a trouvé ou exécuté aucun test.

**Causes possibles :**
- Serveur non accessible (http://localhost:6300)
- Fichiers .spec.ts non trouvés dans tests/
- Configuration Playwright incorrecte
- Erreur de compilation TypeScript

---

## 🐳 Logs backend (warnings/erreurs)

Aucun warning ou erreur dans les logs backend. ✅

---

## 🖼️ Rapport HTML Playwright

> Le rapport HTML complet est disponible en artifact GitHub Actions.
>
> - **Nom de l'artifact :** `playwright-report`
> - **URL du run :** [https://github.com/MX10-AC2N/Nook/actions/runs/29558180334](https://github.com/MX10-AC2N/Nook/actions/runs/29558180334)
> - **Chemin local (CI) :** `frontend/playwright-report/`

Pour examiner visuellement les échecs :
1. Télécharger l'artifact `playwright-report` depuis le [run CI](https://github.com/MX10-AC2N/Nook/actions/runs/29558180334)
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-07-17 05:46 UTC*
