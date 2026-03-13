# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-13 14:23 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`9a7b387`](https://github.com/MX10-AC2N/Nook/commit/9a7b387b37d1e2a386a7bc7b44cb382b8c9b1fd4) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23051155930) |

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
                          "message": "Error: page.goto: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - navigating to \"http://localhost:6300/login\", waiting until \"load\"\u001b[22m\n\n\n  93 | async function loginAs(page: Page, username: string, password: string) {\n  94 |   await clearSession(page);\n> 95 |   await page.goto('/login');\n     |              ^\n  96 |   // CRITICAL : le layout Svelte a loading=true jusqu'à la fin de onMount\n  97 |   // (waitForSodium + initCryptoSystem + authStore.init).\n  98 |   // Pendant ce temps {#if loading} masque {@render children()} → #username absent du DOM.\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:95:14)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:202:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.goto: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - navigating to \"http://localhost:6300/login\", waiting until \"load\"\u001b[22m\n\n\n  93 | async function loginAs(page: Page, username: string, password: string) {\n  94 |   await clearSession(page);\n> 95 |   await page.goto('/login');\n     |              ^\n  96 |   // CRITICAL : le layout Svelte a loading=true jusqu'à la fin de onMount\n  97 |   // (waitForSodium + initCryptoSystem + authStore.init).\n  98 |   // Pendant ce temps {#if loading} masque {@render children()} → #username absent du DOM.\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:95:14)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:202:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.goto: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - navigating to \"http://localhost:6300/login\", waiting until \"load\"\u001b[22m\n\n\n  93 | async function loginAs(page: Page, username: string, password: string) {\n  94 |   await clearSession(page);\n> 95 |   await page.goto('/login');\n     |              ^\n  96 |   // CRITICAL : le layout Svelte a loading=true jusqu'à la fin de onMount\n  97 |   // (waitForSodium + initCryptoSystem + authStore.init).\n  98 |   // Pendant ce temps {#if loading} masque {@render children()} → #username absent du DOM.\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:95:14)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:202:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 2,
--
                          "message": "Error: page.goto: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - navigating to \"http://localhost:6300/login\", waiting until \"load\"\u001b[22m\n\n\n  208 |     test.setTimeout(20_000);\n  209 |     await clearSession(page);\n> 210 |     await page.goto('/login');\n      |                ^\n  211 |     await page.fill('#username', 'nope');\n  212 |     await page.fill('#password', 'wrong');\n  213 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:210:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.goto: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - navigating to \"http://localhost:6300/login\", waiting until \"load\"\u001b[22m\n\n\n  208 |     test.setTimeout(20_000);\n  209 |     await clearSession(page);\n> 210 |     await page.goto('/login');\n      |                ^\n  211 |     await page.fill('#username', 'nope');\n  212 |     await page.fill('#password', 'wrong');\n  213 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:210:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.goto: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - navigating to \"http://localhost:6300/login\", waiting until \"load\"\u001b[22m\n\n\n  208 |     test.setTimeout(20_000);\n  209 |     await clearSession(page);\n> 210 |     await page.goto('/login');\n      |                ^\n  211 |     await page.fill('#username', 'nope');\n  212 |     await page.fill('#password', 'wrong');\n  213 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:210:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-13T12:41:51.255972Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
nook  | [2m2026-03-13T14:23:32.747816Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-13T14:23:32.749828Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-13T14:23:32.751890Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-13T14:23:32.754522Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-13T14:23:32.761049Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
