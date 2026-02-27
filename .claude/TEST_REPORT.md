# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-02-27 10:33 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `MX10-AC2N-patch-svelte5-runes` |
| **Commit** | [`850d9f0`](https://github.com/MX10-AC2N/Nook/commit/850d9f04f616b4e7dee03989f3ef9f5b03643a71) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22482476191) |

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
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:171:36",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 36,
                          "line": 171
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m\n\n  169 |     const notAuth = await page.locator('.not-authorized').isVisible().catch(() => false);\n  170 |     // L'une des deux vues doit être affichée\n> 171 |     expect(adminHeader || notAuth).toBe(true);\n      |                                    ^\n  172 |     console.log(`✅ /admin admin OK (header=${adminHeader}, not-auth=${notAuth})`);\n  173 |   });\n  174 |\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:171:36"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:171:36",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 36,
                          "line": 171
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m\n\n  169 |     const notAuth = await page.locator('.not-authorized').isVisible().catch(() => false);\n  170 |     // L'une des deux vues doit être affichée\n> 171 |     expect(adminHeader || notAuth).toBe(true);\n      |                                    ^\n  172 |     console.log(`✅ /admin admin OK (header=${adminHeader}, not-auth=${notAuth})`);\n  173 |   });\n  174 |\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:171:36"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:171:36",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 36,
                          "line": 171
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32mtrue\u001b[39m\nReceived: \u001b[31mfalse\u001b[39m\n\n  169 |     const notAuth = await page.locator('.not-authorized').isVisible().catch(() => false);\n  170 |     // L'une des deux vues doit être affichée\n> 171 |     expect(adminHeader || notAuth).toBe(true);\n      |                                    ^\n  172 |     console.log(`✅ /admin admin OK (header=${adminHeader}, not-auth=${notAuth})`);\n  173 |   });\n  174 |\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:171:36"
                        }
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-02-27T10:30:16.434465Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
