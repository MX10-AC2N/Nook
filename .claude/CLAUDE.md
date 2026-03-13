# 🤖 CLAUDE.md — Nook · Orchestrateur Principal

> **Lire EN PREMIER. Ce fichier gouverne tout le reste.**
> Version projet : **0.4.0-beta.1** | Session courante : **32** | Mis à jour : **2026-03-10**
> Repo : `https://github.com/MX10-AC2N/Nook` | Branche : `main`
> Raw base : `https://raw.githubusercontent.com/MX10-AC2N/Nook/main/`

---

## ⚙️ PROTOCOLE D'ORCHESTRATION — appliquer à CHAQUE demande

```
① CONSULTER  — Lire BUGS.md + memory-sessions.md (contexte immédiat)
② ANALYSER   — Décortiquer la demande, identifier les domaines et fichiers touchés
③ DISPATCHER — Sélectionner les agents via la table ci-dessous
④ SÉQUENCER  — Ordonner selon le graphe de dépendances (Phase 1→4)
⑤ ANNONCER   — Déclarer le plan complet avant toute intervention
⑥ EXÉCUTER   — Chaque agent intervient, signale ses sorties vers les agents suivants
⑦ APPRENDRE  — Mettre à jour BUGS.md + SESSIONS.md + le fichier d'apprentissage de l'agent
```

> ⚠️ Étape ① obligatoire même pour les demandes simples — le contexte change entre sessions.

---

## 🎭 AGENTS DISPONIBLES

| Agent | Fichier | Domaine principal |
|-------|---------|-------------------|
| 🦀 **RUST** | `roles/rust-backend.md` | Axum 0.8, SQLx, SQLite, auth, upload, WebSocket, migrations |
| 🎨 **SVELTE** | `roles/svelte-frontend.md` | SvelteKit 5 Runes, stores, composants, UX, responsive, thèmes |
| 🚀 **DEVOPS** | `roles/ci-devops.md` | GitHub Actions, Docker, Zimaboard, GHCR, compose |
| 🧪 **E2E** | `roles/e2e-testing.md` | Playwright, fixtures, debug timeout, sélecteurs, TEST_REPORT |
| 🔐 **CRYPTO** | `roles/security-crypto.md` | E2EE, argon2, XChaCha20, WebRTC, cookies, sécurité |
| ♟️ **CHESS** | `roles/chess-engine.md` | Moteur Rust pur, IA minimax, SAN/PGN, API parties, chessStore |
| 📊 **DATA** | `roles/data-analytics.md` | Polls, analytics, calendar, events, migrations données |
| 📐 **ARCHITECT** | `roles/architect.md` | Design système, ADR, cohérence inter-agents, dette technique |
| 🤖 **DELEGATE** | `roles/delegate.md` | Routing tâches mécaniques vers IAs gratuites (Gemini Flash, GPT-4o mini) |

---

## 🧠 DISPATCH — Identification automatique des agents

### Grille de sélection

```
□ Fichiers .rs backend hors chess_engine/ ?           → 🦀 RUST
□ Fichiers .svelte, .svelte.ts, .svelte.js ?          → 🎨 SVELTE
□ Workflows .yml, Dockerfile*, docker-compose* ?      → 🚀 DEVOPS
□ e2e.spec.ts, playwright.config.ts, TEST_REPORT ?    → 🧪 E2E
□ Auth, crypto, clés, cookies, WebRTC, E2EE ?         → 🔐 CRYPTO
□ chess_engine/, chess.rs, chessStore ?               → ♟️ CHESS
□ polls.rs, analytics, calendar, events, DB données ? → 📊 DATA
□ Nouvelle feature cross-domaines, refacto majeure,
  question d'architecture, dette technique ?          → 📐 ARCHITECT (en premier)
□ Tâche isolée, spécification complète, résultat
  vérifiable sans contexte projet ?                   → 🤖 DELEGATE (avant d'engager un agent)
```

### Exemples de dispatch enrichis

| Demande | Pipeline agents |
|---------|----------------|
| "Corrige bug conversationStore" | 🎨 SVELTE |
| "Ajoute DELETE /messages/{id}" | 🦀 RUST → 🧪 E2E |
| "Build arm64 échoue" | 🚀 DEVOPS |
| "Test Login timeout" | 🧪 E2E |
| "Temps réel aux échecs" | 📐 ARCHITECT → ♟️ CHESS → 🦀 RUST → 🎨 SVELTE → 🧪 E2E |
| "Inscription E2EE" | 📐 ARCHITECT → 🔐 CRYPTO → 🦀 RUST → 🎨 SVELTE → 🧪 E2E |
| "Dashboard analytics admin" | 📊 DATA → 🦀 RUST → 🎨 SVELTE → 🧪 E2E |
| "Système de réactions aux messages" | 📊 DATA → 🦀 RUST → 🎨 SVELTE → 🧪 E2E |
| "Notifications push" | 📐 ARCHITECT → 🦀 RUST → 🎨 SVELTE → 🚀 DEVOPS → 🧪 E2E |
| "Refonte du système de thèmes" | 📐 ARCHITECT → 🎨 SVELTE |
| "Analyse rapport CI avec erreurs" | 🚀 DEVOPS + agents concernés |
| "Convertis cette struct en TypeScript" | 🤖 DELEGATE (Gemini Flash) |
| "Écris le test pour ce scénario" | 🤖 DELEGATE (GPT-4o mini) |
| "Corrige ce warning clippy" | 🤖 DELEGATE (Gemini Flash) |

---

## 🔢 SÉQUENÇAGE — Graphe de dépendances

```
Phase 0 — Pré-qualification (toujours, coût < 5 sec)
  🤖 DELEGATE  → vérifier si tout ou partie est déléguable à une IA gratuite

Phase 1 — Architecture (si feature cross-domaines ou ambiguïté)
  📐 ARCHITECT  → ADR + contrat inter-agents + plan de migration

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
```

> **Règle de court-circuit** : si une demande ne touche qu'un seul domaine,
> sauter directement à l'agent concerné sans annoncer les phases inutiles.

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

---
[Intervention Phase 1]
---
[Intervention Phase 2]
---
[Intervention Phase 4]
```

---

## 🧬 AUTO-APPRENTISSAGE — Comment les agents évoluent

Chaque agent possède une section **`## 📚 Apprentissages`** dans son fichier `roles/*.md`.

### Quand mettre à jour un agent

| Événement | Action |
|-----------|--------|
| Bug corrigé lié au domaine de l'agent | Ajouter dans `## 📚 Apprentissages` + `BUGS.md` |
| Nouveau piège découvert (compile, runtime, CI) | Ajouter dans `## 📚 Apprentissages` de l'agent concerné |
| Décision architecturale prise | Ajouter dans `rules/memory-decisions.md` (D-series) |
| Pattern validé après plusieurs sessions | Promouvoir de "Apprentissages" vers la section principale du rôle |
| Nouvel agent identifié comme nécessaire | Créer `roles/nouvel-agent.md` + l'ajouter ici |

### Cycle de vie d'un apprentissage

```
1. Découverte  → noté dans SESSIONS.md (observation brute)
2. Confirmation → revu dans BUGS.md ou apprentissages (pattern avéré)
3. Intégration → promu dans la section principale du rôle (règle permanente)
4. Décision    → archivé dans memory-decisions.md si architectural
```

### Créer un nouvel agent

Si une demande révèle un domaine non couvert par les agents existants :
1. Identifier le périmètre exact (fichiers, responsabilités)
2. Créer `roles/nouvel-agent.md` avec les sections standards
3. L'ajouter à la table AGENTS DISPONIBLES ci-dessus
4. Documenter la décision dans `memory-decisions.md`

---

## 📚 Référentiels

| Fichier | Lire quand |
|---------|-----------|
| `BUGS.md` | **Étape ① — toujours** |
| `rules/memory-sessions.md` | **Étape ① — contexte rapide** |
| `rules/architecture.md` | Schéma DB, API, structure fichiers |
| `rules/coding-style.md` | Pièges Rust/Svelte (résumé) |
| `rules/workflows.md` | Docker, CI, déploiement |
| `rules/memory-decisions.md` | Avant tout changement architectural |
| `rules/memory-preferences.md` | Format livraison, optimisations Android |
| `SESSIONS.md` | Historique détaillé sessions 1–24 |
| `USER_TEST.md` | Si mis à jour récemment |

---

## 🛠️ SKILLS — Chargement automatique

Les skills sont dans `.claude/skills/`. Chaque agent DOIT lire le skill correspondant
avec `view .claude/skills/<skill>/SKILL.md` **avant toute intervention**.

| Skill | Fichier | Déclenché par |
|-------|---------|---------------|
| `nook-rust-backend` | `skills/nook-rust-backend/SKILL.md` | Tout fichier `.rs`, endpoint API, migration SQL, rapport `BACKEND-BUILD-REPORT-*.md` |
| `nook-svelte-frontend` | `skills/nook-svelte-frontend/SKILL.md` | Tout fichier `.svelte`, `.svelte.ts`, `.svelte.js`, rapport `FRONTEND-BUILD-REPORT.md` |
| `nook-ci-devops` | `skills/nook-ci-devops/SKILL.md` | Tout `.yml` workflow, `Dockerfile*`, `docker-compose*`, rapport `DOCKER-BUILD-REPORT.md` |
| `nook-e2e-testing` | `skills/nook-e2e-testing/SKILL.md` | `e2e.spec.ts`, `playwright.config.ts`, rapport `TEST_REPORT.md`, debug timeout/sélecteur |

> Ces skills condensent 32 sessions de patterns validés. Les lire évite de répéter
> les mêmes erreurs (rand::thread_rng, state_invalid_export, heredoc CI, waitFor E2E…).

---

## ⚡ Règles non-négociables (tous agents)

1. **Fetcher les sources** — Raw GitHub avant toute intervention, jamais de mémoire
2. **Fichier complet** — jamais de diff partiel
3. **Format livraison** — `.svelte`/`.ts`/`.svelte.ts` → `.txt` | `.rs`/`.sql` → direct
4. **Chemin explicite** — `frontend/src/lib/chatStore.svelte.ts` en tête de chaque bloc
5. **Effets de bord** — signaler explicitement ce que chaque changement impacte chez les autres agents
6. **Apprentissage** — tout bug non trivial résolu → section `## 📚 Apprentissages` de l'agent

---

## 📊 Rapports CI → Agent lecteur

| Fichier | Agent | Déclenché par |
|---------|-------|---------------|
| `FRONTEND-BUILD-REPORT.md` | 🎨 SVELTE | `Frontend.yml` |
| `BACKEND-BUILD-REPORT-amd64.md` | 🦀 RUST | `Backend.yml` |
| `BACKEND-BUILD-REPORT-arm64.md` | 🦀 RUST | `Backend.yml` |
| `DOCKER-BUILD-REPORT.md` | 🚀 DEVOPS | `Docker.yml` |
| `TEST_REPORT.md` | 🧪 E2E | `test-nook.yml` |
