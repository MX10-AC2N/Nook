# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-08 18:20 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`8aed26f`](https://github.com/MX10-AC2N/Nook/commit/8aed26fe5102bb63172f02179292e2dd8fb04dea) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22825934401) |

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
                        "message": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n",
                        "stack": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:85:35)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:180:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 35,
                          "line": 85
                        },
--
                          "message": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n\n\n  83 |   // goto('/login') se resolve au 'load' event (HTML+JS chargés) AVANT que onMount finisse.\n  84 |   // → Il faut attendre explicitement que #username soit visible avant de fill.\n> 85 |   await page.locator('#username').waitFor({ state: 'visible', timeout: 20_000 });\n     |                                   ^\n  86 |   await page.fill('#username', username);\n  87 |   await page.fill('#password', password);\n  88 |   await page.getByRole('button', { name: 'Se connecter' }).click();\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:85:35)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:180:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n",
                        "stack": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:85:35)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:180:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 35,
                          "line": 85
                        },
--
                          "message": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n\n\n  83 |   // goto('/login') se resolve au 'load' event (HTML+JS chargés) AVANT que onMount finisse.\n  84 |   // → Il faut attendre explicitement que #username soit visible avant de fill.\n> 85 |   await page.locator('#username').waitFor({ state: 'visible', timeout: 20_000 });\n     |                                   ^\n  86 |   await page.fill('#username', username);\n  87 |   await page.fill('#password', password);\n  88 |   await page.getByRole('button', { name: 'Se connecter' }).click();\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:85:35)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:180:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                        "message": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n",
                        "stack": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:85:35)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:180:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 35,
                          "line": 85
                        },
--
                          "message": "TimeoutError: locator.waitFor: Timeout 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username') to be visible\u001b[22m\n\n\n  83 |   // goto('/login') se resolve au 'load' event (HTML+JS chargés) AVANT que onMount finisse.\n  84 |   // → Il faut attendre explicitement que #username soit visible avant de fill.\n> 85 |   await page.locator('#username').waitFor({ state: 'visible', timeout: 20_000 });\n     |                                   ^\n  86 |   await page.fill('#username', username);\n  87 |   await page.fill('#password', password);\n  88 |   await page.getByRole('button', { name: 'Se connecter' }).click();\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:85:35)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:180:5"
                        }
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-08T17:24:29.578396Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
nook  | [2m2026-03-08T18:20:12.481650Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-08T18:20:12.484224Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-08T18:20:12.486287Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-08T18:20:12.488263Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-08T18:20:12.490633Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
