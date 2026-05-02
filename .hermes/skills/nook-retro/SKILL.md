---
name: nook-retro
description: Mode engineering manager — Analyser l'historique des sessions pour identifier les patterns, la vélocité, la dette technique et les priorités. Activer avec /retro ou en début de session pour orienter les efforts. Produit un rapport honnête avec les 3 choses qui bloquent et les 3 priorités suivantes. Spécifique à Nook : historique 36 sessions, BUGS.md, SESSIONS.md, backlog dette technique.
---

# 📊 Nook — Mode Engineering Manager (/retro)

## Rôle

Pas de vibes. Des faits. Qu'est-ce qui a été livré, qu'est-ce qui bloque encore, où va l'énergie.

Ton travail : lire l'historique réel (BUGS.md, SESSIONS.md, TEST_REPORT.md, BACKEND-BUILD-REPORT) et produire une analyse honnête qui oriente les prochaines sessions.

---

## Protocole /retro

### Étape 1 — Lire les sources

```
OBLIGATOIRE :
1. .claude/BUGS.md         → bugs actifs + historique résolutions
2. .claude/SESSIONS.md     → sessions récentes (5 dernières minimum)
3. .claude/TEST_REPORT.md  → état actuel des tests E2E
4. .claude/BACKEND-BUILD-REPORT-*.md → état build Rust
5. .claude/rules/memory-sessions.md  → résumé état projet
```

### Étape 2 — Métriques à calculer

```
Sessions analysées : N
Bugs résolus       : N bugs (R-series dans BUGS.md)
Bugs actifs        : N
Tests E2E          : X/Y passing
Sécurité           : N SEC-XX résolus / N restants
Dette technique    : liste DT-XX actifs
```

### Étape 3 — Identifier les patterns

**Patterns positifs** (à continuer) :
```
- Bugs résolus définitivement (jamais réintroduits)
- Agents bien calibrés (domaines clairs)
- Tests stables depuis N sessions
```

**Patterns négatifs** (à corriger) :
```
- Même type de bug qui revient (ex : timing E2E)
- Features démarrées mais jamais terminées
- Dette technique qui grossit sans être attaquée
- Sessions consacrées à débugger plutôt qu'à construire
```

### Étape 4 — Prioriser

**Backlog Nook actuel (à mettre à jour à chaque retro) :**

```
🔴 Bloquant / Sécurité
  - [SEC-03] Token session 256 bits (faible risque, mais propre à faire)
  - [SEC-06] emergency.rs : ajouter require_auth avant toute activation

🟡 Dette technique
  - [DT-01] libsodium 938 kB : chargement bloque le layout (attendre DT-02)
  - [DT-02] Chess temps réel : WS client pour coups adversaires
  - [DT-03] Polls : backend API (actuellement localStorage only ?)
  - [DT-04] Rate limit par IP → ✅ résolu S36

🟢 Features backlog
  - Notifications push (PWA Service Worker)
  - Chess : invitations par lien
  - Appels WebRTC stables en WAN (serveur TURN)
  - Mode urgence frontend connecté au backend
  - Export/backup données utilisateur UI

🔵 Infrastructure
  - CI : confirmer 75/75 tests verts après fix S33/S36
  - Monitoring Zimaboard (logs structurés, alertes)
```

---

## Format de sortie

```markdown
## 📊 Retrospective Nook — Session [N]

### Métriques
| Indicateur | Valeur |
|------------|--------|
| Sessions totales | 36 |
| Bugs résolus | N |
| Bugs actifs | 0 |
| Tests E2E | ?/75 |
| Vulnérabilités sécurité | 4/6 résolues |
| Dette technique active | N items |

### 3 choses qui ont bien marché
1. [Pattern positif avec exemple concret]
2. ...
3. ...

### 3 choses à améliorer
1. [Pattern négatif + cause + action corrective]
2. ...
3. ...

### Priorités pour les prochaines sessions

**Immédiat (cette session ou la prochaine)**
- [ ] [Action précise → agent concerné]

**Court terme (2-3 sessions)**
- [ ] [Feature ou fix → agents concernés]

**Moyen terme (backlog planifié)**
- [ ] [Feature + estimation effort]

### Santé du projet
[Évaluation honnête : est-ce que le projet est en bonne santé ?
Rythme de livraison, stabilité, dette technique sous contrôle ?]
```
