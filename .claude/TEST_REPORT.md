# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-07 08:35 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ✅ SUCCÈS |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`c31ba0a`](https://github.com/MX10-AC2N/Nook/commit/c31ba0a7c23aeb4716f685c34cea2e8ea7087393) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22795625452) |

---

## Résultats détaillés

```

```

---

## Suites couvertes

| Suite | Description |
|-------|-------------|
| **Auth** | Login valide/invalide, /auth/me non-auth → 401, Logout |
| **Chat** | Envoi message, affichage DOM, GET conversations, GET messages |
| **Admin** | Login admin, onglets, liste users, génération invitation |
| **Settings** | Navigation onglets (profil/sécurité/apparence), changement thème |
| **Calendar** | Chargement page, GET/POST /api/events, bouton ajout |
| **Chess** | Chargement page, GET /api/chess/list, POST /api/chess/create, formulaire UI |
| **Polls** | Chargement page, création sondage (localStorage) |
| **Navigation** | 7 routes testées (chat, calendar, chess, polls, settings, help, events) |
| **API Sanity** | /health, /conversations, /events, /chess/list, /invites (non-auth → 401) |

---

## Erreurs détectées

```
                        "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"",
                        "stack": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:479:16",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 16,
                          "line": 479
                        },
--
                          "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n\n  477 |     await page.goto('/polls');\n  478 |     await waitForAppReady(page);\n> 479 |     await page.waitForResponse(\n      |                ^\n  480 |       (res) => res.url().includes('/api/polls') && res.request().method() === 'GET',\n  481 |       { timeout: 10_000 }\n  482 |     );\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:479:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"",
                        "stack": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:479:16",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 16,
                          "line": 479
                        },
--
                          "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n\n  477 |     await page.goto('/polls');\n  478 |     await waitForAppReady(page);\n> 479 |     await page.waitForResponse(\n      |                ^\n  480 |       (res) => res.url().includes('/api/polls') && res.request().method() === 'GET',\n  481 |       { timeout: 10_000 }\n  482 |     );\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:479:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-07T08:33:36.948115Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
