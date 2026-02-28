# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-02-28 07:35 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `MX10-AC2N-patch-svelte5-runes` |
| **Commit** | [`170c3e4`](https://github.com/MX10-AC2N/Nook/commit/170c3e475f45ab367490c10a6bfae563fdd7f7e7) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22516198082) |

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
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:90:45)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:251:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 45,
                          "line": 90
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n\n\n  88 |   // Si /login → ADMIN_NEW_PASSWORD pas encore actif → mdp initial\n  89 |   if (page.url().includes('/login')) {\n> 90 |     await expect(page.locator('#username')).toBeEnabled({ timeout: 8_000 });\n     |                                             ^\n  91 |     await page.fill('#username', 'admin');\n  92 |     await page.fill('#password', 'changeme2026');\n  93 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:90:45)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:251:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\n\n  89 |   if (page.url().includes('/login')) {\n  90 |     await expect(page.locator('#username')).toBeEnabled({ timeout: 8_000 });\n> 91 |     await page.fill('#username', 'admin');\n     |                ^\n  92 |     await page.fill('#password', 'changeme2026');\n  93 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  94 |     await page.waitForURL(/\\/(change-password|admin|chat)/, { timeout: 12_000 });\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:91:16)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:251:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:90:45)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:251:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 45,
                          "line": 90
                        },
--
                          "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n\n\n  88 |   // Si /login → ADMIN_NEW_PASSWORD pas encore actif → mdp initial\n  89 |   if (page.url().includes('/login')) {\n> 90 |     await expect(page.locator('#username')).toBeEnabled({ timeout: 8_000 });\n     |                                             ^\n  91 |     await page.fill('#username', 'admin');\n  92 |     await page.fill('#password', 'changeme2026');\n  93 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:90:45)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:251:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 2,
--
                        "message": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n",
                        "stack": "Error: \u001b[2mexpect(\u001b[22m\u001b[31mlocator\u001b[39m\u001b[2m).\u001b[22mtoBeEnabled\u001b[2m(\u001b[22m\u001b[2m)\u001b[22m failed\n\nLocator: locator('#username')\nExpected: enabled\nTimeout: 8000ms\nError: element(s) not found\n\nCall log:\n\u001b[2m  - Expect \"toBeEnabled\" with timeout 8000ms\u001b[22m\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    2 × locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m      - unexpected value \"disabled\"\u001b[22m\n\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:90:45)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:261:5",
                        "location": {
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-02-28T07:30:48.474929Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
