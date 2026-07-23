# AGENTS.md — Nook (contexte projet)

> v1.0 — Contenu extrait de SOUL.md v2.0 pour suivre la convention Hermes Agent officielle : SOUL.md = persona courte (500-2000 tokens), AGENTS.md = règles et faits projet, chargé automatiquement depuis le répertoire de travail — place ce fichier à la racine du repo Nook. Doc source : hermes-agent.nousresearch.com/docs. Utilisé par l'Orchestrateur et tous les profils délégués (coder, tester, etc.) quand ils travaillent sur Nook.

## Agents & Toolsets (from `.hermes/roles/`)
| Agent | Toolsets | Specialization | Rules Source |
|-------|----------|-----------------|----------------|
| **github-manager** | github, terminal, file, web | PR lifecycle, releases, GHCR, dawidd6 | rules/workflows.md, rules/architecture.md |
| **ci-monitor** | github, terminal, file | GitHub Actions (22 workflows), Docker, musl | rules/workflows.md, rules/critical-pitfalls.md |
| **coder** | terminal, file, web, skills | Rust (Axum 0.8, SQLx) + Svelte 5 Runes | roles/api-specialist.md, roles/security-crypto.md, rules/architecture.md, rules/critical-pitfalls.md |
| **tester** | terminal, file, web, browser | Playwright E2E (156 tests), helpers validés | roles/e2e-testing.md, rules/critical-pitfalls.md (E2E section) |
| **researcher** | web, file, terminal | WebRTC/SFU (rustrtc), Svelte 5, E2EE Double Ratchet | roles/architect.md, roles/delegate.md, roles/security-crypto.md |
| **deployer** | terminal, file, web, github | Homeserver (192.168.1.192), Docker compose, rollback | rules/workflows.md, rules/critical-pitfalls.md (Docker) |
| **architect** | file, terminal, web, skills | System design, ADRs, tech debt DT-01..07 | roles/architect.md, rules/architecture.md |
| **security-auditor** | file, terminal, web, skills | E2EE audit, threat model, crypto, DT-04 | roles/security-crypto.md, rules/critical-pitfalls.md |
| **docs-writer** | file, web, skills | SOUL.md, ARCHITECTURE.md, CHANGELOG, API docs | roles/docs-writer.md |
| **release-manager** | github, terminal, file | 5-workflow pipeline (FE→BE→Turn→Docker→Release) ⚠️ see CI/CD note | rules/workflows.md |
| **perf-engineer** | terminal, file, web | Bundle, WebRTC latency, SQLx, **DT-01 owner** | rules/critical-pitfalls.md, roles/performance.md |
| **ux-reviewer** | browser, file, web, skills | WCAG 2.1 AA, theme system, Svelte 5 runes | roles/ux-reviewer.md |
| **dependency-manager** | terminal, file, web, github | Cargo/pnpm, Dependabot, CVEs, rand_core 0.6 pin | rules/workflows.md |
| **supervisor** | file, terminal, web, delegation | Token budget enforcement, rate limiting, throttling | roles/supervisor.md |
| **team-upgrader** | file, terminal, web, delegation | Profile lifecycle (create/update/deprecate) | roles/team-upgrader.md |

*(Chaque agent = un profil Hermes. Rappel doc officielle : `delegate_task` n'a pas de paramètre `toolsets` — un sous-agent hérite du toolset activé de son parent, ou tourne avec le toolset propre à son profil quand il est invoqué par nom via une carte Kanban. Cette table décrit donc la config attendue par profil, pas un choix fait à la volée par l'Orchestrateur.)*

## GitHub Workflow Rules (CRITICAL)
- **NEVER** auto-trigger or schedule workflows (free GitHub account — no cron jobs, including Docker.yml)
- **ONLY** trigger manually, following the CI/CD Order below
- **ALWAYS** check repo state first: `git log --oneline -5`, `gh run list --limit 5`
- **Don't repeat** actions already done — check state before re-running (stop the "repeat loop")

## Stack & Pièges (from `rules/critical-pitfalls.md`)
- **Rust**: `rand 0.9` → `rng()` pas `thread_rng()`, `distr::` pas `distributions::`, `rand_core 0.6` forcé pour argon2
- **Axum 0.8**: routes `{param}` pas `:param`, `Utf8Bytes` pas `String`
- **Svelte 5**: `$state`/`$derived`/`$effect` (runes), pas syntaxe Svelte 4, `<form onsubmit>` NE FONCTIONNE PAS
- **SQLx**: éviter macros quand `queries.json` vide, pas de `?` sans macro `query!`
- **SFU (rustrtc)**: `PeerConnection::new(config)` par valeur, `set_remote_description(SessionDescription)`, `MediaRelay::with_capacity()`, `added_sources` HashSet pour dédup tracks
- **Docker**: Alpine `sqlite-libs` pas `libsqlite3`, artefacts Backend = triplet Rust, Turn TOML format, `turn-config :rw`, UID/GID 1000

*(Duplique `rules/critical-pitfalls.md` — pratique à injecter tel quel dans le `context` d'un `delegate_task`, mais garde les deux synchronisés.)*

## Architecture (from `rules/architecture.md`)
- **Backend**: Rust/Axum :3000 → auth, db, chess_engine, webrtc, polls, upload, e2ee, SFU
- **Frontend**: SvelteKit 5 :6300 → stores `$state`, libsodium 938kB (DT-01), WebRTC P2P
- **DB**: SQLite — users, conversations, conversation_participants, messages, uploads, invites, chess, e2ee, polls
- **Auth**: Cookie HttpOnly `auth_token=<userId>:<token>`, SameSite=Lax (LAN) / None;Secure (WAN), token révocable en DB
- **LAN/WAN**: Nginx Proxy Manager injecte `X-Forwarded-Proto: https`

## CI/CD Order
```
1. Frontend.yml      (build SvelteKit, artifact 7j)
2. Backend.yml       (compile amd64+arm64, 2 jobs séparés → 2 rapports, éviter race condition)
3. test-nook.yml     (Docker + E2E 156 tests, healthcheck /api/health)
4. Docker.yml        (dawidd6/action-download-artifact cross-workflow → GHCR distroless)
5. Release.yml       (bump VERSION + tag git)
```

⚠️ **Incohérence non résolue** — 3 sources donnent un ordre différent :
- *GitHub Workflow Rules (ancienne version du SOUL.md)* : Frontend → Backend → **Turn** → Docker (ni E2E, ni Release)
- *Cette liste* : Frontend → Backend → **test-nook (E2E)** → Docker → Release (pas de Turn)
- *Spécialisation release-manager* : FE → BE → **Turn** → Docker → Release (pas d'E2E)

`Turn` (serveur TURN/coturn WebRTC) n'apparaît que dans 2 des 3 versions, jamais au même endroit. Rien dans la doc Hermes Agent ne concerne ce détail — c'est purement Nook. Soit Turn est un workflow indépendant hors de cette chaîne séquentielle, soit il manque une étape ici. À confirmer avant de déclencher quoi que ce soit dans un ordre supposé.

## Dette Technique Active (from `roles/architect.md`)
| ID | Description | Fichier | Impact | Priorité |
|----|-------------|---------|--------|----------|
| DT-01 | libsodium 938 kB — pas dynamic import | sodium.svelte.js | LCP mobile dégradé | 🔴 |
| DT-02 | Chess pas temps réel | chess.rs + chessStore | UX dégradée | 🔴 |
| DT-03 | Polls backend localStorage only | polls.rs | Données non persistées | 🟡 |
| DT-04 | Rate limiting governor non configuré | main.rs | Sécurité | 🟡 |
| DT-05 | E2EE partiellement implémenté | e2ee.rs + e2ee.ts | Feature incomplète | 🟡 |
| DT-06 | Analytics endpoint incomplet | backend | Dashboard vide | 🟢 |
| DT-07 | Bug state_invalid_export conversationStore | conversationStore.svelte.ts | Warning CI | 🟢 |

*(Synchronise avec `roles/architect.md` quand un item se ferme ou qu'une nouvelle dette est identifiée.)*

## Quality Gates (before any merge)
- All tests pass (unit, integration, E2E 156 tests)
- No new Clippy warnings
- No new compiler warnings
- Code coverage maintained or improved
- Documentation updated
- Changelog updated
- Architecture decisions documented
- Security implications considered
- Performance impact assessed
- Accessibility compliance verified
- User experience validated

## Development Workflow
1. **Issue first** — Always start with a clear issue or user story
2. **Design discussion** — Architecture decisions require team consensus (ARCHITECT agent)
3. **Small PRs** — Keep changes focused and reviewable
4. **Code review** — Thorough, constructive, and timely
5. **Testing** — Comprehensive test coverage for all changes
6. **Documentation** — Update all relevant docs
7. **Deployment** — Follow the deployment checklist (CI/CD Order above)

## Delegation — valeurs de référence (Hermes Agent)
Les valeurs réelles vivent dans `~/.hermes/config.yaml` — ne pas les recopier ailleurs comme des faits figés, elles peuvent changer sans que ce fichier soit mis à jour. Pour mémoire, défauts Hermes officiels :
- `delegation.max_concurrent_children` : 3
- `delegation.max_spawn_depth` : 1 (flat — un enfant en `role="orchestrator"` ne peut rien déléguer tant que cette valeur n'est pas montée)
- `delegation.orchestrator_enabled` : true (coupe-circuit global si mis à `false`)
- Statuts Kanban natifs : `triage → todo → ready → running → blocked/done → archived`

Source : hermes-agent.nousresearch.com/docs/user-guide/features/delegation

## Live Status & Secrets
Le statut CI/E2EE/déploiement change trop vite pour vivre dans un fichier de règles versionné — lire `memory/nook-context.md` avant tout rapport, ne pas se fier à un snapshot figé ici.
Hôte/identifiants du serveur de test : `.env` ou gestionnaire de secrets, jamais en clair dans un fichier de config/règles — même sur LAN privé.

## Operational State
Budget/throttle events et changements de modèle sont un état runtime, pas un fait projet stable — suivis dans le state du supervisor, pas recopiés ici.
