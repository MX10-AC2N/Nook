# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-02-27 19:06 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `MX10-AC2N-patch-svelte5-runes` |
| **Commit** | [`8f518d9`](https://github.com/MX10-AC2N/Nook/commit/8f518d91d4ab585a5d4c9ee12170ff8a754c4247) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22499604224) |

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
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    - locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m    - fill(\"admin\")\u001b[22m\n\u001b[2m  - attempting fill action\u001b[22m\n\u001b[2m    2 × waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m      - element is not enabled\u001b[22m\n\u001b[2m    - retrying fill action\u001b[22m\n\u001b[2m    - waiting 20ms\u001b[22m\n\u001b[2m    - waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m  - element was detached from the DOM, retrying\u001b[22m\n\n\n  52 |   // Si on est encore sur /login → le nouveau mdp n'est pas encore actif → utiliser l'ancien\n  53 |   if (page.url().includes('/login')) {\n> 54 |     await page.fill('#username', 'admin');\n     |                ^\n  55 |     await page.fill('#password', 'changeme2026');\n  56 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  57 |     await page.waitForURL(/\\/(change-password|admin|chat)/, { timeout: 12_000 });\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:54:16)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:218:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    - locator resolved to <input type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m    - fill(\"admin\")\u001b[22m\n\u001b[2m  - attempting fill action\u001b[22m\n\u001b[2m    - waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m  - element was detached from the DOM, retrying\u001b[22m\n\n\n  52 |   // Si on est encore sur /login → le nouveau mdp n'est pas encore actif → utiliser l'ancien\n  53 |   if (page.url().includes('/login')) {\n> 54 |     await page.fill('#username', 'admin');\n     |                ^\n  55 |     await page.fill('#password', 'changeme2026');\n  56 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  57 |     await page.waitForURL(/\\/(change-password|admin|chat)/, { timeout: 12_000 });\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:54:16)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:218:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    - locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m    - fill(\"admin\")\u001b[22m\n\u001b[2m  - attempting fill action\u001b[22m\n\u001b[2m    2 × waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m      - element is not enabled\u001b[22m\n\u001b[2m    - retrying fill action\u001b[22m\n\u001b[2m    - waiting 20ms\u001b[22m\n\u001b[2m    - waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m  - element was detached from the DOM, retrying\u001b[22m\n\n\n  52 |   // Si on est encore sur /login → le nouveau mdp n'est pas encore actif → utiliser l'ancien\n  53 |   if (page.url().includes('/login')) {\n> 54 |     await page.fill('#username', 'admin');\n     |                ^\n  55 |     await page.fill('#password', 'changeme2026');\n  56 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  57 |     await page.waitForURL(/\\/(change-password|admin|chat)/, { timeout: 12_000 });\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:54:16)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:218:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 2,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    - locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m    - fill(\"admin\")\u001b[22m\n\u001b[2m  - attempting fill action\u001b[22m\n\u001b[2m    2 × waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m      - element is not enabled\u001b[22m\n\u001b[2m    - retrying fill action\u001b[22m\n\u001b[2m    - waiting 20ms\u001b[22m\n\u001b[2m    - waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m  - element was detached from the DOM, retrying\u001b[22m\n\n\n  52 |   // Si on est encore sur /login → le nouveau mdp n'est pas encore actif → utiliser l'ancien\n  53 |   if (page.url().includes('/login')) {\n> 54 |     await page.fill('#username', 'admin');\n     |                ^\n  55 |     await page.fill('#password', 'changeme2026');\n  56 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  57 |     await page.waitForURL(/\\/(change-password|admin|chat)/, { timeout: 12_000 });\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:54:16)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:228:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    - locator resolved to <input type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m    - fill(\"admin\")\u001b[22m\n\u001b[2m  - attempting fill action\u001b[22m\n\u001b[2m    - waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m  - element was detached from the DOM, retrying\u001b[22m\n\n\n  52 |   // Si on est encore sur /login → le nouveau mdp n'est pas encore actif → utiliser l'ancien\n  53 |   if (page.url().includes('/login')) {\n> 54 |     await page.fill('#username', 'admin');\n     |                ^\n  55 |     await page.fill('#password', 'changeme2026');\n  56 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  57 |     await page.waitForURL(/\\/(change-password|admin|chat)/, { timeout: 12_000 });\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:54:16)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:228:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                          "message": "Error: page.fill: Test timeout of 30000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#username')\u001b[22m\n\u001b[2m    - locator resolved to <input disabled type=\"text\" required=\"\" id=\"username\" class=\"svelte-1x05zx6\" autocomplete=\"username\" placeholder=\"Votre identifiant unique\"/>\u001b[22m\n\u001b[2m    - fill(\"admin\")\u001b[22m\n\u001b[2m  - attempting fill action\u001b[22m\n\u001b[2m    2 × waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m      - element is not enabled\u001b[22m\n\u001b[2m    - retrying fill action\u001b[22m\n\u001b[2m    - waiting 20ms\u001b[22m\n\u001b[2m    - waiting for element to be visible, enabled and editable\u001b[22m\n\u001b[2m  - element was detached from the DOM, retrying\u001b[22m\n\n\n  52 |   // Si on est encore sur /login → le nouveau mdp n'est pas encore actif → utiliser l'ancien\n  53 |   if (page.url().includes('/login')) {\n> 54 |     await page.fill('#username', 'admin');\n     |                ^\n  55 |     await page.fill('#password', 'changeme2026');\n  56 |     await page.getByRole('button', { name: 'Se connecter' }).click();\n  57 |     await page.waitForURL(/\\/(change-password|admin|chat)/, { timeout: 12_000 });\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:54:16)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:228:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-02-27T18:59:19.582955Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m ⚠️  Aucun utilisateur trouvé - création de l'administrateur initial
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
