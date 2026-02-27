# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-02-27 09:36 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `MX10-AC2N-patch-svelte5-runes` |
| **Commit** | [`63ee435`](https://github.com/MX10-AC2N/Nook/commit/63ee4354840ba2f42ba7398653cecba49cc860a2) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22480497663) |

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
                          "message": "Error: locator.click: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for getByRole('button', { name: /d.connect/i })\u001b[22m\n\n\n  63 |     await expect(menuToggle).toBeVisible({ timeout: 8_000 });\n  64 |     await menuToggle.click();\n> 65 |     await page.getByRole('button', { name: /d.connect/i }).click();\n     |                                                            ^\n  66 |     await expect(page).toHaveURL(/\\/login/, { timeout: 10_000 });\n  67 |     console.log('✅ Logout → /login');\n  68 |   });\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:65:60"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: locator.click: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for getByRole('button', { name: /d.connect/i })\u001b[22m\n\n\n  63 |     await expect(menuToggle).toBeVisible({ timeout: 8_000 });\n  64 |     await menuToggle.click();\n> 65 |     await page.getByRole('button', { name: /d.connect/i }).click();\n     |                                                            ^\n  66 |     await expect(page).toHaveURL(/\\/login/, { timeout: 10_000 });\n  67 |     console.log('✅ Logout → /login');\n  68 |   });\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:65:60"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: locator.click: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for getByRole('button', { name: /d.connect/i })\u001b[22m\n\n\n  63 |     await expect(menuToggle).toBeVisible({ timeout: 8_000 });\n  64 |     await menuToggle.click();\n> 65 |     await page.getByRole('button', { name: /d.connect/i }).click();\n     |                                                            ^\n  66 |     await expect(page).toHaveURL(/\\/login/, { timeout: 10_000 });\n  67 |     console.log('✅ Logout → /login');\n  68 |   });\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:65:60"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 2,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBeGreaterThan\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected: > \u001b[32m0\u001b[39m\nReceived:   \u001b[31m0\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBeGreaterThan\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected: > \u001b[32m0\u001b[39m\nReceived:   \u001b[31m0\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:127:26",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 26,
                          "line": 127
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBeGreaterThan\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected: > \u001b[32m0\u001b[39m\nReceived:   \u001b[31m0\u001b[39m\n\n  125 |     const body = await res.json();\n  126 |     const convs = Array.isArray(body) ? body : body.conversations ?? [];\n> 127 |     expect(convs.length).toBeGreaterThan(0);\n      |                          ^\n  128 |     const global = convs.find((c: { id: string }) => c.id === 'default_global');\n  129 |     expect(global).toBeDefined();\n  130 |     console.log(`✅ GET /api/conversations → ${convs.length} conversation(s), default_global présente`);\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:127:26"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBeGreaterThan\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected: > \u001b[32m0\u001b[39m\nReceived:   \u001b[31m0\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBeGreaterThan\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected: > \u001b[32m0\u001b[39m\nReceived:   \u001b[31m0\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:127:26",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
```

---

## Logs backend (warnings/erreurs)

```

```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
