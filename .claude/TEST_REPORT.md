# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-14 16:17 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ✅ SUCCÈS |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`6fab76a`](https://github.com/MX10-AC2N/Nook/commit/6fab76a129b9a4d43e2f27abee276201372bbee8) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23091549925) |

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
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32m401\u001b[39m\nReceived: \u001b[31m404\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32m401\u001b[39m\nReceived: \u001b[31m404\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:48:28",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts",
                          "column": 28,
                          "line": 48
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32m401\u001b[39m\nReceived: \u001b[31m404\u001b[39m\n\n  46 |           ? await request.delete(`${BASE}${route.path}`)\n  47 |           : await request.post(`${BASE}${route.path}`, { data: route.body });\n> 48 |       expect(res.status()).toBe(401);\n     |                            ^\n  49 |     });\n  50 |   }\n  51 |\n    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:48:28"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32m401\u001b[39m\nReceived: \u001b[31m404\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32m401\u001b[39m\nReceived: \u001b[31m404\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:48:28",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts",
                          "column": 28,
                          "line": 48
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoBe\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m) // Object.is equality\u001b[22m\n\nExpected: \u001b[32m401\u001b[39m\nReceived: \u001b[31m404\u001b[39m\n\n  46 |           ? await request.delete(`${BASE}${route.path}`)\n  47 |           : await request.post(`${BASE}${route.path}`, { data: route.body });\n> 48 |       expect(res.status()).toBe(401);\n     |                            ^\n  49 |     });\n  50 |   }\n  51 |\n    at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:48:28"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-14T16:16:31.594592Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
