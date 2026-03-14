# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-14 14:06 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`1343672`](https://github.com/MX10-AC2N/Nook/commit/1343672fcfa63af102edfcd8275d4f64f723f866) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23089373757) |

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
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/384a835f-540a-481f-9a25-ee309e2c7df6\"\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/384a835f-540a-481f-9a25-ee309e2c7df6\"\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:726:22",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 22,
                          "line": 726
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/384a835f-540a-481f-9a25-ee309e2c7df6\"\u001b[39m\n\n  724 |     expect(body.file_id).toBeTruthy();\n  725 |     expect(body.file_name).toBe('test-e2e.txt');\n> 726 |     expect(body.url).toMatch(/\\/files\\//);\n      |                      ^\n  727 |     console.log(`✅ POST /api/upload/chat → file_id=${body.file_id}`);\n  728 |\n  729 |     // Vérifier que le download fonctionne avec auth\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:726:22"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/68bbe949-0eef-43ab-8e4c-fa9cfed6a9a6\"\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/68bbe949-0eef-43ab-8e4c-fa9cfed6a9a6\"\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:726:22",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 22,
                          "line": 726
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/68bbe949-0eef-43ab-8e4c-fa9cfed6a9a6\"\u001b[39m\n\n  724 |     expect(body.file_id).toBeTruthy();\n  725 |     expect(body.file_name).toBe('test-e2e.txt');\n> 726 |     expect(body.url).toMatch(/\\/files\\//);\n      |                      ^\n  727 |     console.log(`✅ POST /api/upload/chat → file_id=${body.file_id}`);\n  728 |\n  729 |     // Vérifier que le download fonctionne avec auth\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:726:22"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/cb2c2df1-ef94-4ea0-9eb2-fe14d7abe8fe\"\u001b[39m",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/cb2c2df1-ef94-4ea0-9eb2-fe14d7abe8fe\"\u001b[39m\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:726:22",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 22,
                          "line": 726
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mreceived\u001b[39m\u001b[2m).\u001b[22mtoMatch\u001b[2m(\u001b[22m\u001b[32mexpected\u001b[39m\u001b[2m)\u001b[22m\n\nExpected pattern: \u001b[32m/\\/files\\//\u001b[39m\nReceived string:  \u001b[31m\"/api/download/cb2c2df1-ef94-4ea0-9eb2-fe14d7abe8fe\"\u001b[39m\n\n  724 |     expect(body.file_id).toBeTruthy();\n  725 |     expect(body.file_name).toBe('test-e2e.txt');\n> 726 |     expect(body.url).toMatch(/\\/files\\//);\n      |                      ^\n  727 |     console.log(`✅ POST /api/upload/chat → file_id=${body.file_id}`);\n  728 |\n  729 |     // Vérifier que le download fonctionne avec auth\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:726:22"
                        }
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-14T14:05:46.457652Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé (429) — IP bloquée temporairement [3mip[0m[2m=[0m172.18.0.1 [3mpath[0m[2m=[0m/auth/login
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
