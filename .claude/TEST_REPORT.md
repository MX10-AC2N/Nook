# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-07 20:54 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`a752bf5`](https://github.com/MX10-AC2N/Nook/commit/a752bf5fc4280db8f3c519ab37044b27b1fedd75) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22806889316) |

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
                        "message": "Error: Login admin API échoué : HTTP 429",
                        "stack": "Error: Login admin API échoué : HTTP 429\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:132:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:347:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 13,
                          "line": 132
                        },
--
                          "message": "Error: Login admin API échoué : HTTP 429\n\n  130 |     });\n  131 |     if (!loginRes.ok()) {\n> 132 |       throw new Error(`Login admin API échoué : HTTP ${loginRes.status()}`);\n      |             ^\n  133 |     }\n  134 |   }\n  135 |\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:132:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:347:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeVisible\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('.cell-last').first()\nExpected: visible\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeVisible\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('.cell-last').first()\u001b[22m\n",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeVisible\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('.cell-last').first()\nExpected: visible\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeVisible\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('.cell-last').first()\u001b[22m\n\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:844:54",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 54,
                          "line": 844
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeVisible\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('.cell-last').first()\nExpected: visible\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeVisible\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('.cell-last').first()\u001b[22m\n\n\n  842 |     await page.reload();\n  843 |     await expect(page.locator('.chess-board')).toBeVisible({ timeout: 10_000 });\n> 844 |     await expect(page.locator('.cell-last').first()).toBeVisible({ timeout: 8_000 });\n      |                                                      ^\n  845 |     console.log('✅ Case last-move visible après rechargement');\n  846 |   });\n  847 |\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:844:54"
                        }
                      ],
                      "stdout": [
                        {
                          "text": "✅ Partie IA créée → game_id=6b5be2be-4d27-4a5a-8f07-119a5fa397ab\n"
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeVisible\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('.cell-last').first()\nExpected: visible\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeVisible\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('.cell-last').first()\u001b[22m\n",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeVisible\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('.cell-last').first()\nExpected: visible\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeVisible\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('.cell-last').first()\u001b[22m\n\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:844:54",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 54,
                          "line": 844
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeVisible\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('.cell-last').first()\nExpected: visible\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeVisible\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('.cell-last').first()\u001b[22m\n\n\n  842 |     await page.reload();\n  843 |     await expect(page.locator('.chess-board')).toBeVisible({ timeout: 10_000 });\n> 844 |     await expect(page.locator('.cell-last').first()).toBeVisible({ timeout: 8_000 });\n      |                                                      ^\n  845 |     console.log('✅ Case last-move visible après rechargement');\n  846 |   });\n  847 |\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:844:54"
                        }
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-07T20:53:23.402899Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:53:39.785670Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:53:55.734843Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.232974Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.234334Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.235621Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.236899Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.238688Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.240126Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.241482Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.242789Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.244026Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.245324Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.246552Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.248214Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.249531Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T20:54:13.250941Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
