# 🤖 CLAUDE.md — Nook · Orchestrateur Principal

> **Lire EN PREMIER. Ce fichier gouverne tout le reste.**
> Version projet : **0.5.0** | Session courante : **52** | Mis à jour : **2026-04-25**
> Repo : `https://github.com/MX10-AC2N/Nook` | Branche : `develop`
> Raw base : `https://raw.githubusercontent.com/MX10-AC2N/Nook/develop/`
> Déploiement : Docker multi-arch (Alpine 3.21), Zimaboard via docker-compose
> HTTPS local : nginx-alpine sur port 6443 (cert auto-signé) pour enregistrement audio/vidéo
> ## 📋 RÉSUMÉ DES SESSIONS RÉCENTES
>
> ### Session 50 (2026-04-21)
> - ✅ **Audit global** : 82/100 (+3 depuis 2026-04-09)
> - ✅ **PR #28** : `refactor/remove-simple-peer` → supprime dépendance obsolète
> - ✅ **PR #29** : `feat/healthchecks` → healthchecks pour tous les services
> - ✅ **PR #30** : `fix/hardcoded-secrets` → corrige 4 problèmes critiques
> - ✅ **0 secret en dur** : TURN_SECRET, admin password logging, chmod 0777
> - ✅ **Documentation** : `.env.example`, `.claude/` mis à jour
>
> ### Sessions précédentes (rappel)
> - Session 38-49 : MCP Servers, Svelte 5 migration, WebRTC calls, Chess, Polls, Calendar, Events
> - Session 37 : E2EE tests, Playwright setup, CI/CD fixes


---

## ⚙️ PROTOCOLE D'ORCHESTRATION — appliquer à CHAQUE demande

```
① CONSULTER  — Lire BUGS.md + memory-sessions.md (contexte immédiat)
② ANALYSER   — Décortiquer la demande, identifier les domaines et fichiers touchés
③ DISPATCHER — Sélectionner les agents via la table ci-dessous
④ SÉQUENCER  — Ordonner selon le graphe de dépendances (Phase -1→4)
⑤ ANNONCER   — Déclarer le plan complet avant toute intervention
⑥ EXÉCUTER   — Chaque agent intervient, signale ses sorties vers les agents suivants
⑦ APPRENDRE  — Mettre à jour BUGS.md + SESSIONS.md + le fichier d'apprentissage de l'agent
```

> ⚠️ Étape ① obligatoire même pour les demandes simples — le contexte change entre sessions.

---

## 🔌 MCP SERVERS — Disponibles pour les agents

> Depuis la session 38. Détails complets dans `rules/mcp-servers.md`.

| MCP | URL / Commande | Agent | Quand |
|-----|---------------|-------|-------|
| **Svelte** | `https://mcp.svelte.dev/mcp` | 🎨 SVELTE | Tout fichier `.svelte`, `.svelte.ts`, `.svelte.js` |
| **rust-mcp-server** | `cargo install rust-mcp-server` (local) | 🦀 RUST | Validation build, clippy, tests |
| **mcp-language-server** | `mcp-language-server --lsp rust-analyzer` (local) | 🦀 RUST | Navigation sémantique codebase |
| **Lightpanda** | Docker `lightpanda/browser:nightly` | 🧪 E2E | ⚠️ Beta — surveiller maturité |

### Workflow Svelte MCP (obligatoire depuis S38)
```
1. list-sections            → identifier la doc Svelte 5 pertinente
2. get-documentation(...)   → charger la doc exacte
3. [coder]
4. svelte-autofixer(code)   → valider, 0 issues avant livraison
```

---

## 🎭 AGENTS DISPONIBLES

### Agents de développement

| Agent | Fichier | Domaine principal |
|-------|---------|-------------------|
| 🦀 **RUST** | `roles/rust-backend.md` | Axum 0.8, SQLx, SQLite, auth, upload, WebSocket, migrations |
| 🎨 **SVELTE** | `roles/svelte-frontend.md` | SvelteKit 5 Runes, stores, composants, UX, responsive, thèmes + **MCP Svelte** |
| 🚀 **DEVOPS** | `roles/ci-devops.md` | GitHub Actions, Docker, Zimaboard, GHCR, compose |
| 🧪 **E2E** | `roles/e2e-testing.md` | Playwright, fixtures, debug timeout, sélecteurs, TEST_REPORT |
| 🔐 **CRYPTO** | `roles/security-crypto.md` | E2EE, argon2, XChaCha20, WebRTC, cookies, sécurité |
| ♟️ **CHESS** | `roles/chess-engine.md` | Moteur Rust pur, IA minimax, SAN/PGN, API parties, chessStore |
| 📊 **DATA** | `roles/data-analytics.md` | Polls, analytics, calendar, events, migrations données |
| 📐 **ARCHITECT** | `roles/architect.md` | Design système, ADR, cohérence inter-agents, dette technique |
| 🤖 **DELEGATE** | `roles/delegate.md` | Routing tâches mécaniques vers IAs gratuites (Gemini Flash, GPT-4o mini) |

### Agents de mode cognitif (inspirés de gstack)

> Ces agents répondent à des **commandes slash** ou s'activent automatiquement selon le type de demande.
> Chacun a un skill dédié dans `.claude/skills/` à lire avant d'intervenir.

| Agent | Fichier | Commande | Quand l'activer |
|-------|---------|----------|-----------------| 
| 🏠 **FOUNDER** | `roles/founder.md` | `/plan-ceo` | Nouvelle feature → valider qu'on construit la bonne chose |
| 🔎 **REVIEWER** | `roles/reviewer.md` | `/review` | Avant tout merge → trouver ce qui casse en prod |

> Les modes `/plan-eng`, `/ship` et `/retro` s'exécutent via ARCHITECT et DEVOPS
> avec leurs skills dédiés — pas d'agent séparé nécessaire.

---

## 🧠 DISPATCH — Identification automatique des agents

### Grille de sélection — développement

```
□ Fichiers .rs backend hors chess_engine/ ?              → 🦀 RUST
□ Fichiers .svelte, .svelte.ts, .svelte.js ?             → 🎨 SVELTE  (+ MCP Svelte)
□ Workflows .yml, Dockerfile*, docker-compose* ?         → 🚀 DEVOPS
□ `chess-extended.spec.ts`, `webrtc.spec.ts`, `TEST_REPORT` ?  → 🧪 E2E
□ Auth, crypto, clés, cookies, WebRTC, E2EE ?            → 🔐 CRYPTO
□ chess_engine/, chess.rs, chessStore ?                  → ♟️ CHESS
□ polls.rs, analytics, calendar, events, DB données ?    → 📊 DATA
□ Feature cross-domaines, refacto majeure, architecture? → 📐 ARCHITECT (en premier)
□ Tâche isolée, spec complète, vérifiable sans contexte  → 🤖 DELEGATE
```

### Grille de sélection — modes cognitifs

```
□ "Est-ce qu'on devrait faire X ?" / feature incertaine → /plan-ceo → 🏠 FOUNDER
□ Direction validée, besoin d'un plan technique béton   → /plan-eng → 📐 ARCHITECT
□ Avant merge sur main / code prêt                      → /review   → 🔎 REVIEWER
□ Review ✅, tests ✅, prêt à shipper                    → /ship     → 🚀 DEVOPS
□ Bilan de sessions / orientation backlog               → /retro    → 📐 ARCHITECT
```

### Exemples de dispatch enrichis

| Demande | Pipeline agents |
|---------|----------------|
| `"Corrige bug conversationStore"` | 🎨 SVELTE (+ MCP svelte-autofixer) |
| `"Ajoute DELETE /messages/{id}"` | 🦀 RUST → 🧪 E2E |
| `"Build arm64 échoue"` | 🚀 DEVOPS |
| `"Test Login timeout"` | 🧪 E2E |
| `"Temps réel aux échecs"` | 📐 ARCHITECT → ♟️ CHESS → 🦀 RUST → 🎨 SVELTE → 🧪 E2E |
| `"Inscription E2EE"` | 📐 ARCHITECT → 🔐 CRYPTO → 🦀 RUST → 🎨 SVELTE → 🧪 E2E |
| `"Dashboard analytics admin"` | 📊 DATA → 🦀 RUST → 🎨 SVELTE → 🧪 E2E |
| `"Notifications push"` | 🏠 FOUNDER → 📐 ARCHITECT → 🦀 RUST → 🎨 SVELTE → 🚀 DEVOPS → 🧪 E2E |
| `"Analyse rapport CI"` | 🚀 DEVOPS + agents concernés |
| `"Convertis cette struct en TypeScript"` | 🤖 DELEGATE (Gemini Flash) |
| `"/plan-ceo : ajouter X"` | 🏠 FOUNDER → spec → pipeline selon verdict |
| `"/review"` | 🔎 REVIEWER → rapport → fixes ciblés |
| `"/ship"` | 🚀 DEVOPS → pipeline complet → Zimaboard |
| `"/retro"` | 📐 ARCHITECT → bilan + priorités |

---

## 🔢 SÉQUENÇAGE — Graphe de dépendances

```
Phase -1 — Vision produit (si feature nouvelle ou incertaine)
  🏠 FOUNDER    → valider qu'on construit la bonne chose avant tout

Phase 0 — Pré-qualification (toujours, coût < 5 sec)
  🤖 DELEGATE   → vérifier si tout ou partie est déléguable
  📐 ARCHITECT  → si feature cross-domaines ou ambiguïté

Phase 1 — Fondations et contrats
  🔐 CRYPTO  → protocoles de sécurité, formats de clés
  🦀 RUST    → endpoints, types Rust, schéma DB, migrations
  ♟️ CHESS   → API chess, logique moteur, types Move/GameState
  📊 DATA    → modèles de données, agrégations, migrations data

Phase 2 — Consommateurs
  🎨 SVELTE  → consomme endpoints + types de Phase 1

Phase 3 — Infrastructure
  🚀 DEVOPS  → pipeline, variables d'env, compose, secrets

Phase 4 — Validation
  🧪 E2E     → tests sur tout ce que les phases précédentes ont produit
  🔎 REVIEWER → audit avant merge si code touche auth/upload/WS/E2EE
```

> **Règle de court-circuit** : demande mono-domaine → aller directement à l'agent,
> sans annoncer les phases inutiles.

---

## 📣 FORMAT D'ANNONCE (multi-agents obligatoire)

```markdown
## 🎯 Analyse
[Ce qui est demandé + fichiers identifiés]

## 🤝 Pipeline d'agents
Phase 1 → 🦀 RUST    : [action précise]
Phase 2 → 🎨 SVELTE  : [action précise, dépend de RUST pour : X]
Phase 4 → 🧪 E2E     : [tests à couvrir]

## ⚠️ Points de vigilance inter-agents
[Effets de bord, contrats à respecter, régressions possibles]
```

---

## 🛠️ SKILLS — Chargement automatique

Les skills sont dans `.claude/skills/`. Chaque agent DOIT lire le skill correspondant
avec `view .claude/skills/<skill>/SKILL.md` **avant toute intervention**.

### Skills de développement

| Skill | Fichier | Déclenché par |
|-------|---------|---------------|
| `nook-rust-backend` | `skills/nook-rust-backend/SKILL.md` | Tout `.rs`, endpoint API, migration SQL, `BACKEND-BUILD-REPORT-*.md` |
| `nook-svelte-frontend` | `skills/nook-svelte-frontend/SKILL.md` | Tout `.svelte/.svelte.ts/.svelte.js`, `FRONTEND-BUILD-REPORT.md` |
| `nook-ci-devops` | `skills/nook-ci-devops/SKILL.md` | Tout `.yml` workflow, `Dockerfile*`, `docker-compose*`, `DOCKER-BUILD-REPORT.md` |
| `nook-e2e-testing` | `skills/nook-e2e-testing/SKILL.md` | `e2e.spec.ts`, `playwright.config.ts`, `TEST_REPORT.md`, debug timeout/sélecteur |

### Skills de mode cognitif

| Skill | Fichier | Commande | Ce qu'il contient |
|-------|---------|----------|-------------------|
| `nook-plan-ceo` | `skills/nook-plan-ceo/SKILL.md` | `/plan-ceo` | Protocole vision produit, filtre "bonne feature", format spec |
| `nook-plan-eng` | `skills/nook-plan-eng/SKILL.md` | `/plan-eng` | Plan technique, contrats inter-agents, checklist risques Nook |
| `nook-review` | `skills/nook-review/SKILL.md` | `/review` | Checklist 60+ points, pièges historiques, format rapport audit |
| `nook-ship` | `skills/nook-ship/SKILL.md` | `/ship` | Pipeline CI/CD ordonné, bump version, déploiement Zimaboard |
| `nook-retro` | `skills/nook-retro/SKILL.md` | `/retro` | Métriques, patterns récurrents, backlog priorisé |

> Ces skills condensent **37 sessions** de patterns validés. Les lire évite de répéter
> les mêmes erreurs (rand::thread_rng, state_invalid_export, heredoc CI, waitFor E2E…).

---

## 🧬 AUTO-APPRENTISSAGE — Comment les agents évoluent

Chaque agent possède une section **`## 📚 Apprentissages`** dans son fichier `roles/*.md`.

### Quand mettre à jour un agent

| Événement | Action |
|-----------|--------|
| Bug corrigé lié au domaine de l'agent | Ajouter dans `## 📚 Apprentissages` + `BUGS.md` |
| Nouveau piège découvert | Ajouter dans `## 📚 Apprentissages` de l'agent concerné |
| Décision architecturale prise | Ajouter dans `rules/memory-decisions.md` (D-series) |
| Pattern validé après plusieurs sessions | Promouvoir vers la section principale du rôle |
| Nouvel agent identifié | Créer `roles/nouvel-agent.md` + l'ajouter ici |

### Cycle de vie d'un apprentissage

```
1. Découverte  → noté dans SESSIONS.md (observation brute)
2. Confirmation → revu dans BUGS.md ou apprentissages (pattern avéré)
3. Intégration → promu dans la section principale du rôle (règle permanente)
4. Décision    → archivé dans memory-decisions.md si architectural
```

---

## 📚 Référentiels

| Fichier | Lire quand |
|---------|-----------|
| `BUGS.md` | **Étape ① — toujours** |
| `rules/memory-sessions.md` | **Étape ① — contexte rapide** |
| `rules/mcp-servers.md` | **Étape ① pour 🎨 SVELTE et 🦀 RUST** — MCP disponibles |
| `rules/architecture.md` | Schéma DB, API, structure fichiers |
| `rules/coding-style.md` | Pièges Rust/Svelte (index vers les rôles) |
| `rules/workflows.md` | Docker, CI, déploiement |
| `rules/memory-decisions.md` | Avant tout changement architectural |
| `rules/memory-preferences.md` | Format livraison, optimisations Android |
| `SESSIONS.md` | Historique détaillé sessions 1–37 |

---

## ⚡ Règles non-négociables (tous agents)

1. **Fetcher les sources** — Raw GitHub avant toute intervention, jamais de mémoire
2. **Fichier complet** — jamais de diff partiel
3. **Format livraison** — `.svelte`/`.ts`/`.svelte.ts` → `.txt` | `.rs`/`.sql` → direct
4. **Chemin explicite** — `frontend/src/lib/chatStore.svelte.ts` en tête de chaque bloc
5. **Effets de bord** — signaler ce que chaque changement impacte chez les autres agents
6. **Apprentissage** — tout bug non trivial résolu → section `## 📚 Apprentissages` de l'agent
7. **MCP Svelte** — `svelte-autofixer` obligatoire avant toute livraison de code Svelte (depuis S38)
8. **Always speak to the user in French.** Code, commit messages, comments, and technical identifiers stay in English, but all conversation and explanations must be in French.

---

## 📊 Rapports CI → Agent lecteur

| Fichier | Agent | Déclenché par |
|---------|-------|---------------|
| `FRONTEND-BUILD-REPORT.md` | 🎨 SVELTE | `Frontend.yml` |
| `BACKEND-BUILD-REPORT-amd64.md` | 🦀 RUST | `Backend.yml` |
| `BACKEND-BUILD-REPORT-arm64.md` | 🦀 RUST | `Backend.yml` |
| `DOCKER-BUILD-REPORT.md` | 🚀 DEVOPS | `Docker.yml` |
| `TEST-AND-SECURITY-AUDIT-2026.md` | 🧪 E2E + 🔐 CRYPTO | Tests étendus + findings ouvertes |
| `E2E-TARGETED-REPORT.md` | 🧪 E2E | `e2e-targeted.yml` |

---

## 🛠️ WORKFLOWS DISPONIBLES — Catalogue rapide

> 20 workflows au total. Voir détails dans `rules/workflows.md`.

| Catégorie | Workflows | Déclencheur |
|-----------|-----------|-------------|
| CI principale | `test-nook.yml` | push/PR sur develop/main |
| Build artifacts | `Backend.yml`, `Frontend.yml`, `Docker.yml`, `ci-new2.yml` | `workflow_dispatch` |
| E2E debug | `e2e-targeted.yml` | `workflow_dispatch` (input: suite) |
| Maintenance auto | `update-cargo-lock.yml`, `update-frontend-lock.yml`, `sqlx-prepare.yml` | push sur paths |
| Bundle/audit | `bundle-analysis.yml`, `npm-audit-report.yml` | push/cron |
| Nettoyage | `clear-cache.yml`, `ghcr-cleanup.yml` | cron/workflow_run |
| Assets | `fetch-gifs.yml`, `generate-pwa-icons.yml`, `generate-android-instruction.yml` | push/dispatch |
| Migration | `auto-svelte5-migration.yml`, `fix-svelte5-runes.yml` | `workflow_dispatch` |
| Release | `Release.yml` | `workflow_dispatch` |

> ⚠️ **Nettoyage recommandé** (détails dans [WORKFLOW-CATALOG.md](#workflow-cleansing) ci-dessous):
> - `auto-svelte5-migration.yml` — migration S5 terminée depuis S37
> - `fix-svelte5-runes.yml` — idem, plus nécessaire
> - `ci-new2.yml` — doublon avec `Backend.yml`+`Docker.yml`
> - `generate-android-instruction.yml` — usage ponctuel, pas besoin de workflow dédié
> - `update-cargo-lock.yml` + `update-frontend-lock.yml` — redondants si `sqlx-prepare.yml` gère lock

---

## 🚦 Statut CI (2026-04-12 — Session 49)
- **Tests E2E**: 163/163 PASS | 0 fail | 0 skip | ~1.5min
- **Fichiers**: admin.spec.ts (25 tests), user.spec.ts (75 tests), api-sanity.spec.ts (76 tests)
- **Backend**: build OK `nook-backend v0.5.0`
- **Docker**: image `nook:dev` OK
- **Regles**: `npx playwright test --list` obligatoire avant push
- **Bugs CI**: 0 connu bloquant
- **Coverage**: Auth, Chat, Reactions, Upload, Polls, Chess, Calendar, Settings, Admin, E2EE, Push, Navigation

### Commandes Chat Disponibles
- `/fini` — Termine la session proprement: resume, update .claude docs, push state, exit.
  *Voir `.claude/skills/nook-fin/SKILL.md` pour les details*
## 📖 Référence rapide des commandes slash

| Commande | Agent | Skill lu | Résultat |
|----------|-------|----------|----------|
| `/plan-ceo` | 🏠 FOUNDER | `nook-plan-ceo` | Spec produit validée ou reformulée |
| `/plan-eng` | 📐 ARCHITECT | `nook-plan-eng` | Plan technique + contrats inter-agents |
| `/review` | 🔎 REVIEWER | `nook-review` | Rapport audit + verdict merge |
| `/ship` | 🚀 DEVOPS | `nook-ship` | Pipeline CI/CD complet + Zimaboard |
| `/retro` | 📐 ARCHITECT | `nook-retro` | Bilan sessions + priorités backlog |
