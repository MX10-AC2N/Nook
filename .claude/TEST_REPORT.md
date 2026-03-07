# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-07 16:28 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`a9ee592`](https://github.com/MX10-AC2N/Nook/commit/a9ee5928eb7ac340de33068f1cc442eb56f55dde) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22802577743) |

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
                        "stack": "Error: Login admin API échoué : HTTP 429\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:113:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:331:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 13,
                          "line": 113
                        },
--
                          "message": "Error: Login admin API échoué : HTTP 429\n\n  111 |     });\n  112 |     if (!loginRes.ok()) {\n> 113 |       throw new Error(`Login admin API échoué : HTTP ${loginRes.status()}`);\n      |             ^\n  114 |     }\n  115 |   }\n  116 |\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:113:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:331:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 0,
--
                        "message": "Error: Login admin API échoué : HTTP 429",
                        "stack": "Error: Login admin API échoué : HTTP 429\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:113:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:331:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 13,
                          "line": 113
                        },
--
                          "message": "Error: Login admin API échoué : HTTP 429\n\n  111 |     });\n  112 |     if (!loginRes.ok()) {\n> 113 |       throw new Error(`Login admin API échoué : HTTP ${loginRes.status()}`);\n      |             ^\n  114 |     }\n  115 |   }\n  116 |\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:113:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:331:5"
                        }
                      ],
                      "stdout": [],
                      "stderr": [],
                      "retry": 1,
--
                        "message": "Error: Login admin API échoué : HTTP 429",
                        "stack": "Error: Login admin API échoué : HTTP 429\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:113:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:331:5",
                        "location": {
                          "file": "/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts",
                          "column": 13,
                          "line": 113
                        },
--
                          "message": "Error: Login admin API échoué : HTTP 429\n\n  111 |     });\n  112 |     if (!loginRes.ok()) {\n> 113 |       throw new Error(`Login admin API échoué : HTTP ${loginRes.status()}`);\n      |             ^\n  114 |     }\n  115 |   }\n  116 |\n    at loginAsAdmin (/home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:113:13)\n    at /home/runner/work/Nook/Nook/frontend/tests/e2e.spec.ts:331:5"
                        }
```

---

## Logs backend (warnings/erreurs)

```
nook  | [2m2026-03-07T16:26:30.380248Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:26:30.386533Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:26:34.822150Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:26:59.017181Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:27:22.016167Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:27:40.363447Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:27:58.746982Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.656760Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.658434Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.659938Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.661753Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.663667Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.665435Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.667209Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.669950Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.671678Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.673108Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.674565Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.676132Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:28:18.677701Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
