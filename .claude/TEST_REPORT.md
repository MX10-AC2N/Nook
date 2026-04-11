# 🧪 Rapport E2E — Nook

> Généré par `test-nook.yml` · **2026-04-11 16:35 UTC**

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
| **Durée totale** | 0.4s |
| **Branche** | `develop` |
| **Commit** | [`564de83`](https://github.com/MX10-AC2N/Nook/commit/564de837a8ff4bb5c4ecddd1c03c99dd2f07f88d) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/24286518155) |

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

### 📋 Logs Playwright

```
SyntaxError: /home/runner/work/Nook/Nook/frontend/tests/user.spec.ts: Unexpected token, expected "," (864:47)

[0m [90m 862 |[39m   [90m// ── Chess Improvements ────────────────────────────────────────────[39m
 [90m 863 |[39m   test([32m'Chess — sélection pièce → coups légaux visibles (dots)'[39m[33m,[39m [36masync[39m ({ page }) [33m=>[39m {
[31m[1m>[22m[39m[90m 864 |[39m     [36mconst[39m createRes [33m=[39m [36mawait[39m page[33m.[39mrequest[33m.[39mpost(${[33mBASE[39m}[33m/[39mchess[33m/[39mcreate[32m`, {[39m
 [90m     |[39m                                                [31m[1m^[22m[39m
 [90m 865 |[39m [32m      data: { color: 'white', opponent: 'easy' },[39m
 [90m 866 |[39m [32m    });[39m
 [90m 867 |[39m [32m    expect(createRes.status()).toBeLessThan(500);[39m[0m

   at user.spec.ts:864

  862 |   // ── Chess Improvements ────────────────────────────────────────────
  863 |   test('Chess — sélection pièce → coups légaux visibles (dots)', async ({ page }) => {
> 864 |     const createRes = await page.request.post(${BASE}/chess/create`, {
      |                                               ^
  865 |       data: { color: 'white', opponent: 'easy' },
  866 |     });
  867 |     expect(createRes.status()).toBeLessThan(500);


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
> - **URL du run :** [https://github.com/MX10-AC2N/Nook/actions/runs/24286518155](https://github.com/MX10-AC2N/Nook/actions/runs/24286518155)
> - **Chemin local (CI) :** `frontend/playwright-report/`

Pour examiner visuellement les échecs :
1. Télécharger l'artifact `playwright-report` depuis le [run CI](https://github.com/MX10-AC2N/Nook/actions/runs/24286518155)
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

*Rapport généré par `scripts/generate-test-report.py` — 2026-04-11 16:35 UTC*
