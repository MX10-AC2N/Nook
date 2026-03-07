# 🧪 Rapport de test E2E — Nook

> Généré automatiquement par le workflow CI `test-nook.yml`
> **Dernière mise à jour : 2026-03-07 16:46 UTC**

---

## Résumé du run

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Tests passés** | ? |
| **Tests échoués** | ? |
| **Tests ignorés** | 0 |
| **Branche** | `develop` |
| **Commit** | [`a8f1e5a`](https://github.com/MX10-AC2N/Nook/commit/a8f1e5a912ca176dcb6497b6d62814a378c8e553) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22802854957) |

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
nook  | [2m2026-03-07T16:45:29.499468Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:45:47.880543Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:06.246354Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.153439Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.155150Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.156629Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.158264Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.159962Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.162587Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.164340Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.165856Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.167479Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.168929Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.170367Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.171942Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.173476Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.174920Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:26.176485Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:27.363035Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
nook  | [2m2026-03-07T16:46:27.370385Z[0m [33m WARN[0m [2mnook_backend[0m[2m:[0m Rate limit dépassé sur route d'authentification (429) [3mpath[0m[2m=[0m/auth/login
```

---

*Rapport généré par `.github/workflows/test-nook.yml` — session 14*
