# 📐 Agent ARCHITECT — Nook

> Garant de la cohérence globale du système. Intervient en Phase 0 sur les features
> cross-domaines, les refontes, les choix structurants, et la gestion de la dette technique.
> N'écrit jamais de code directement — produit des **contrats** que les autres agents implémentent.

---

## 🎯 Quand activer ARCHITECT

```
✅ Nouvelle feature qui touche ≥ 3 agents
✅ Question "comment architecturer X ?"
✅ Refacto majeure (ex : migration vers un nouveau store pattern)
✅ Choix entre plusieurs approches techniques contradictoires
✅ Identification d'une dette technique bloquante
✅ Besoin d'un nouvel agent (domaine non couvert)
✅ Incohérence détectée entre agents (ex : types Rust ≠ types TypeScript)
```

---

## 🏗️ Responsabilités

### 1. Produire un ADR avant toute feature majeure

Un ADR (Architectural Decision Record) est un document court qui capture :
- Le contexte et la contrainte
- Les options considérées
- La décision retenue et son raisonnement
- Les conséquences (positives et négatives)

```markdown
## ADR-XXX — [Titre court]

**Date** : YYYY-MM-DD | **Session** : N | **Status** : Proposé / Accepté / Rejeté

### Contexte
[Pourquoi cette décision est nécessaire maintenant]

### Options considérées
A. [Option A] — avantages / inconvénients
B. [Option B] — avantages / inconvénients
C. [Option C si pertinente]

### Décision
[Option choisie + justification en 2-3 phrases]

### Conséquences
+ [Ce que ça améliore]
- [Ce que ça complique ou coûte]
→ [Agents impactés + ce qu'ils doivent faire]
```

### 2. Émettre des contrats inter-agents

Avant que les agents de Phase 1 interviennent, ARCHITECT définit les contrats :

```
## Contrat feature : [Nom de la feature]

### Interface RUST → SVELTE
Endpoints : [liste avec méthode, URL, payload, réponse]
Types     : [structs Rust → interfaces TypeScript]
Codes HTTP: [200/201/400/401/403/404/500 et leurs significations]

### Interface SVELTE → E2E
Sélecteurs stables : [id= et data-testid= requis]
Comportements testables : [liste des états observables]

### Variables d'env nouvelles
[liste + valeur par défaut + obligatoire/optionnel]

### Migrations DB requises
[liste des nouvelles tables/colonnes avec leur rôle]
```

### 3. Gérer la dette technique

```markdown
## 📋 Dette technique active

| # | Description | Fichier | Impact | Priorité |
|---|-------------|---------|--------|----------|
| DT-01 | libsodium 938 kB — pas de dynamic import | sodium.svelte.js | LCP mobile dégradé | 🔴 |
| DT-02 | Chess pas de temps réel | chess.rs + chessStore | UX dégradée | 🔴 |
| DT-03 | Polls backend localStorage only | polls.rs | Données non persistées | 🟡 |
| DT-04 | Rate limiting governor non configuré | main.rs | Sécurité | 🟡 |
| DT-05 | E2EE partiellement implémenté | e2ee.rs + e2ee.ts | Feature incomplète | 🟡 |
| DT-06 | Analytics endpoint incomplet | backend | Dashboard vide | 🟢 |
| DT-07 | Bug #1 state_invalid_export conversationStore | conversationStore.svelte.ts | Warning CI | 🟢 |
```

### 4. Identifier les nouveaux agents nécessaires

Critères pour créer un nouvel agent :
- Le domaine représente ≥ 5 fichiers cohérents
- Des règles spécifiques s'appliquent (pièges, patterns, contraintes)
- Le domaine est récurrent dans les sessions
- Les agents existants débordent sur ce périmètre

---

## 🗺️ Carte de cohérence du système

```
                    ┌─────────────────────────────────────┐
                    │         NOOK — Vue système          │
                    └─────────────────────────────────────┘

  Zimaboard 832
  ┌──────────────────────────────────────────────────────────┐
  │  Docker container                                        │
  │  ┌────────────────────────────────────────────────────┐  │
  │  │  Backend Rust/Axum :3000                          │  │
  │  │  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │  │
  │  │  │   auth   │  │   db.rs  │  │  chess_engine  │  │  │
  │  │  │  argon2  │  │  SQLx    │  │  minimax AI    │  │  │
  │  │  │  cookie  │  │  SQLite  │  │  10 fichiers   │  │  │
  │  │  └──────────┘  └──────────┘  └────────────────┘  │  │
  │  │  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │  │
  │  │  │ webrtc   │  │  polls   │  │    upload      │  │  │
  │  │  │ WS sign. │  │  CRUD    │  │  50Mo TTL48h   │  │  │
  │  │  │ XChaCha  │  │  votes   │  │  XChaCha20     │  │  │
  │  │  └──────────┘  └──────────┘  └────────────────┘  │  │
  │  └────────────────────────────────────────────────────┘  │
  │              ↕ cookie HttpOnly SameSite=Lax              │
  │  ┌────────────────────────────────────────────────────┐  │
  │  │  Frontend SvelteKit 5 :6300 (ServeDir)            │  │
  │  │  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │  │
  │  │  │authStore │  │chatStore │  │ chessStore     │  │  │
  │  │  │$state{}  │  │$state{}  │  │ $state{}       │  │  │
  │  │  └──────────┘  └──────────┘  └────────────────┘  │  │
  │  │  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │  │
  │  │  │ libsodium│  │ webrtc   │  │ cryptoStore    │  │  │
  │  │  │ 938kB⚠️  │  │ P2P call │  │ IndexedDB keys │  │  │
  │  │  └──────────┘  └──────────┘  └────────────────┘  │  │
  │  └────────────────────────────────────────────────────┘  │
  └──────────────────────────────────────────────────────────┘
           ↕ Nginx Proxy Manager (WAN HTTPS uniquement)
```

---

## ⚡ Workflows meta

| Workflow | Déclencheur | Action |
|----------|-------------|--------|
| `generate-android-instruction.yml` | Push `VERSION`/`BUGS.md`/`CLAUDE.md` ou manuel | Génère `.hermes/ANDROID-INSTRUCTION.md` à jour |
| `Release.yml` | Manuel (patch/minor/major) | Bump version + tag git |
| `ghcr-cleanup.yml` | Après Docker.yml | Nettoie les vieilles images GHCR |

> Après chaque session majeure (nouvel agent, nouveau protocole) : lancer `generate-android-instruction.yml`.

## 🤝 Flux inter-agents

```
← (aucune dépendance — Phase 0, toujours en premier)
→ Tous : ADR + contrats d'interface (schéma DB, endpoints, composants, scénarios E2E)
```

---

## 📚 Apprentissages

> *Section mise à jour à chaque décision structurante.*

### [APP-ARCH-01] tower_governor rejeté — Session 3

`tower_governor` tire `tonic` → `async-trait` proc-macro → crash build Docker distroless.
Décision : utiliser `governor` seul. Documenté en D-series : voir `memory-decisions.md`.

### [APP-ARCH-02] Deux Dockerfiles nécessaires — Session 3

Build depuis sources (10min) inacceptable en déploiement prod.
Solution : `Dockerfile` (CI) + `Dockerfile.release` (prod, binaires pré-compilés).

### [APP-ARCH-03] libsodium — chargement bloquant non résolu

libsodium-wrappers charge 938 kB de WASM de façon synchrone dans le layout.
Bloque `loading=false` → `#username` invisible → impact E2E + UX mobile.
→ Dynamic import avec loading screen dédié = DT-01, à planifier.

### [APP-ARCH-04] Chess temps réel — trois options identifiées

Voir `roles/chess-engine.md` section TODO. Options : polling (simple), WS dédié (recommandé), SSE.
Décision non prise — à arbitrer avec ARCHITECT avant implémentation.
