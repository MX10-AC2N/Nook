# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-05 18:14 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`91f4779`](https://github.com/MX10-AC2N/Nook/commit/91f4779da0b57f057cd2c4ef7a18166274c33e90) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22728050971) |

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
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  62 |   await clearSession(page);\n  63 |   await page.goto('/login');\n> 64 |   await page.fill('#username', username);\n     |              ^\n  65 |   await page.fill('#password', password);\n  66 |   await page.getByRole('button', { name: 'Se connecter' }).click();\n  67 |   await expect(page).toHaveURL(/\\/(chat|admin|change-password)/, { timeout: 15_000 });\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:64:14)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:144:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  62 |   await clearSession(page);\n  63 |   await page.goto('/login');\n> 64 |   await page.fill('#username', username);\n     |              ^\n  65 |   await page.fill('#password', password);\n  66 |   await page.getByRole('button', { name: 'Se connecter' }).click();\n  67 |   await expect(page).toHaveURL(/\\/(chat|admin|change-password)/, { timeout: 15_000 });\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:64:14)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:144:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  62 |   await clearSession(page);\n  63 |   await page.goto('/login');\n> 64 |   await page.fill('#username', username);\n     |              ^\n  65 |   await page.fill('#password', password);\n  66 |   await page.getByRole('button', { name: 'Se connecter' }).click();\n  67 |   await expect(page).toHaveURL(/\\/(chat|admin|change-password)/, { timeout: 15_000 });\n    at loginAs (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:64:14)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:144:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 2,
--
                          "message": "Error: page.fill: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  151 |     await clearSession(page);\n  152 |     await page.goto('/login');\n> 153 |     await page.fill('#username', 'nope');\n      |                ^\n  154 |     await page.fill('#password', 'wrong');\n  155 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  156 |     await page.waitForTimeout(3_000);\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:153:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.fill: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  151 |     await clearSession(page);\n  152 |     await page.goto('/login');\n> 153 |     await page.fill('#username', 'nope');\n      |                ^\n  154 |     await page.fill('#password', 'wrong');\n  155 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  156 |     await page.waitForTimeout(3_000);\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:153:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.fill: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  151 |     await clearSession(page);\n  152 |     await page.goto('/login');\n> 153 |     await page.fill('#username', 'nope');\n      |                ^\n  154 |     await page.fill('#password', 'wrong');\n  155 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  156 |     await page.waitForTimeout(3_000);\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:153:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-05T17:29:06.637414Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
