# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-07 18:43 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`a486d39`](https://github.com/MX10-AC2N/Nook/commit/a486d392dd9146a30df7413e40c2c6ca1d0eee82) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22804772507) |

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
                        "stack": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:206:16",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 16,
                          "line": 206
                        },
--
                          "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n\n  204 |     await waitForAppReady(page);\n  205 |\n> 206 |     await page.waitForResponse(\n      |                ^\n  207 |       (res) => res.url().includes('/api/conversations') && res.request().method() === 'GET',\n  208 |       { timeout: 10_000 }\n  209 |     );\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:206:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"",
                        "stack": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:206:16",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 16,
                          "line": 206
                        },
--
                          "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n\n  204 |     await waitForAppReady(page);\n  205 |\n> 206 |     await page.waitForResponse(\n      |                ^\n  207 |       (res) => res.url().includes('/api/conversations') && res.request().method() === 'GET',\n  208 |       { timeout: 10_000 }\n  209 |     );\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:206:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                        "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"",
                        "stack": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:206:16",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 16,
                          "line": 206
                        },
--
                          "message": "TimeoutError: page.waitForResponse: Timeout 10000ms exceeded while waiting for event \"response\"\n\n  204 |     await waitForAppReady(page);\n  205 |\n> 206 |     await page.waitForResponse(\n      |                ^\n  207 |       (res) => res.url().includes('/api/conversations') && res.request().method() === 'GET',\n  208 |       { timeout: 10_000 }\n  209 |     );\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:206:16"
                        }
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-07T18:42:12.686265Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:42:30.985300Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:42:49.218506Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.667171Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.668931Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.670967Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.673116Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.674935Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.676577Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.678169Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.679656Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.681355Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.682930Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.685254Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.686818Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T18:43:08.688220Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
