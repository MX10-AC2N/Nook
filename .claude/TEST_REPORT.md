# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-05 07:15 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`dcd4583`](https://github.com/MX10-AC2N/Nook/commit/dcd458392dac35e5caff181fde5c5fd5f8d421a0) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22705331046) |

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
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  113 |     test.setTimeout(30_000);\n  114 |     await page.goto('/login');\n> 115 |     await page.fill('#username', 'e2e_ci');\n      |                ^\n  116 |     await page.fill('#password', 'E2eTest123!');\n  117 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  118 |     await expect(page).toHaveURL(/\\/chat/, { timeout: 15_000 });\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:115:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  113 |     test.setTimeout(30_000);\n  114 |     await page.goto('/login');\n> 115 |     await page.fill('#username', 'e2e_ci');\n      |                ^\n  116 |     await page.fill('#password', 'E2eTest123!');\n  117 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  118 |     await expect(page).toHaveURL(/\\/chat/, { timeout: 15_000 });\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:115:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  113 |     test.setTimeout(30_000);\n  114 |     await page.goto('/login');\n> 115 |     await page.fill('#username', 'e2e_ci');\n      |                ^\n  116 |     await page.fill('#password', 'E2eTest123!');\n  117 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  118 |     await expect(page).toHaveURL(/\\/chat/, { timeout: 15_000 });\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:115:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 2,
--
                          "message": "Error: page.fill: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  123 |     test.setTimeout(20_000);\n  124 |     await page.goto('/login');\n> 125 |     await page.fill('#username', 'nope');\n      |                ^\n  126 |     await page.fill('#password', 'wrong');\n  127 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  128 |     await page.waitForTimeout(3_000);\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:125:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.fill: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  123 |     test.setTimeout(20_000);\n  124 |     await page.goto('/login');\n> 125 |     await page.fill('#username', 'nope');\n      |                ^\n  126 |     await page.fill('#password', 'wrong');\n  127 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  128 |     await page.waitForTimeout(3_000);\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:125:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.fill: Test timeout of 20000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  123 |     test.setTimeout(20_000);\n  124 |     await page.goto('/login');\n> 125 |     await page.fill('#username', 'nope');\n      |                ^\n  126 |     await page.fill('#password', 'wrong');\n  127 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  128 |     await page.waitForTimeout(3_000);\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:125:16"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-05T06:31:10.102621Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
