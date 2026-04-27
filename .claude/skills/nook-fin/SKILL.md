---
name: nook-fin
category: "devops"
description: "Session exit command — clean shutdown with full context persistence using new .claude/ structure. Run at end of any session to summarize work, update hermes/ docs, push state, and exit cleanly."
---

# 🚪 Nook Fin (Session Exit) — NEW STRUCTURE

## Trigger
Run this skill when the user says:
- `/fini`
- `/nook-fin`
- "termine la session"
- "quitte proprement"
- "exit cleanly"

## Steps

### 1. Audit git state
```bash
cd /tmp/Nook
git fetch origin develop
git status --short
git log --oneline -10
git diff --stat origin/develop
```
- Identifie fichiers modifies, non-commits, et état de la branche
- Vérifie si tout est commit et push

### 2. Resume exhaustif de la session
Generer un résumé markdown avec ces sections:

```markdown
## Session [N] — [Date] ([theme principal])

### Contexte
[Objectif initial, problème à résoudre]

### Progres Realises
- [tache 1: detail]
- [tache 2: detail]
- [...]

### Decisions Cles
- [decision 1: rationale]
- [decision 2: rationale]

### Bugs Corriges
| Bug | Fichier | Fix |
|-----|---------|-----|
| [...] | [...] | [...] |

### Fichiers Modifies
- [fichier: lignes modifiées, nature du changement]

### Conventions Etablies
1. [regle 1]
2. [regle 2]

### Couverture Tests (si applicable)
| Categorie | Status | Tests |
|-----------|--------|-------|
| [...] | ✅/❌ | N |

### Prochaines Etapes
- [ ] [tache prioritaire]
- [ ] [tache secondaire]

### Risques
1. [risque 1: impact, mitigation]
2. [risque 2: impact, mitigation]

### Etat Final
- Branche: [branche]
- CI: [status]
- Backend: [build status]
- Docker: [image status]
- Git: [clean/dirty, commits en attente]
```

### 3. Mettre à jour .claude/hermes/ (MON ESPACE PERSO)

**C'EST CRITIQUE** — Utiliser les nouveaux fichiers dans `hermes/`:

| Fichier | Action | Contenu |
|---------|--------|---------|
| `.claude/hermes/active-session.md` | ✅ Mettre à jour | Session en cours, ce qui vient d'être fait |
| `.claude/hermes/known-issues.md` | ✅ Ajouter/Mettre à jour | Nouveaux bugs, pièges évités, leçons |
| `.claude/hermes/hermes-memory.md` | ✅ Mettre à jour | Infos critiques (tokens, comptes, règles) |
| `.claude/hermes/preferences.md` | ✅ Vérifier | Préférences utilisateur inchangées |

**Procedure :**
```python
# a) Lire l'état actuel
# read_file(path='.claude/hermes/active-session.md')

# b) Mettre à jour avec le résumé de session
# write_file(path='.claude/hermes/active-session.md', content='...')

# c) Ajouter les nouveaux bugs/pièges dans known-issues.md
# read_file(path='.claude/hermes/known-issues.md')
# puis ajouter les nouveaux éléments

# d) S'assurer que hermes-memory.md contient les infos critiques
# read_file(path='.claude/hermes/hermes-memory.md')
```

### 4. Mettre à jour .claude/project/ (REFERENCE PROJET)

| Fichier | Action | Contenu |
|---------|--------|---------|
| `.claude/project/project-state.md` | ✅ Mettre à jour | Version, branche, CI status, derniers commits |
| `.claude/project/BUGS.md` | ✅ Marquer bugs corrigés | Bugs actifs, résolus |

### 5. Archiver si nécessaire
```bash
# Si SESSIONS.md existe encore quelque part, s'assurer qu'il est dans archive/sessions/
if [ -f .claude/SESSIONS.md ]; then
  mv .claude/SESSIONS.md .claude/archive/sessions/
fi
```

### 6. Créer/mettre à jour les skills pertinents

**Analyse :** identifier ce qui a été accompli pendant la session

Examiner les changements de la session et déterminer quels skills doivent être créés ou mis à jour :

| Si la session a touché... | Skill à vérifier/mettre à jour |
|---------------------------|--------------------------------|
| Tests E2E, CI, Playwright | `nook-e2e-testing` |
| Backend Rust, Axum, API | `nook-rust-backend` |
| Frontend Svelte, UI, CSS | `nook-svelte-frontend` |
| DevOps, Docker, deploy | `nook-ci-devops` |
| Review code, PR, audit | `nook-review` |
| Planification, roadmap | `nook-plan-ceo` ou `nook-plan-eng` |
| Retro, retrospective | `nook-retro` |
| Ship, release, version | `nook-ship` |
| Nouveau domaine non couvert | Créer un NOUVEAU skill |

**Procedure pour chaque skill concerné :**

```python
# a) Charger le skill existant pour voir son contenu
# skill_view(name='nook-e2e-testing')

# b) Identifier les sections qui doivent être mises à jour :
#    - Nouvelles conventions découvertes pendant la session
#    - Nouveaux patterns de code utilisés
#    - Nouveaux bugs/solutions documentés
#    - Changements d'architecture

# c) Mettre à jour avec skill_manage(action='patch', name='...', ...)
#    ou skill_manage(action='edit', name='...', content='...') pour réécriture majeure

# d) Créer un NOUVEAU skill si un domaine n'est pas couvert :
#    skill_manage(action='create', name='nook-[domaine]', ...)
```

**Checklist skills :**
- [ ] Lister tous les skills existants du projet (`skills/` dans `.claude/`)
- [ ] Pour chaque skill, demander : "Est-ce que cette session a ajouté des connaissances pertinentes pour ce skill ?"
- [ ] Si OUI → patcher le skill avec les nouvelles info
- [ ] Si un domaine important n'a pas de skill → créer un nouveau skill
- [ ] Pousser les skills modifiés sur `origin/develop`

### 7. Enrichir la memoire persistante (Hermes memory)

**C'est CRITIQUE** — sans ça, la prochaine session ne saura pas ce qui a été fait.

Utiliser le `memory` tool pour sauvegarder :

```python
1. Memoriser les decisions d'architecture prises
   → memory(action='add', target='memory', content='Decision: [quoi] pour [raison]')

2. Memoriser les bugs et leurs solutions
   → memory(action='add', target='memory', content='Bug [X]: [symptome] → Fix: [solution]')

3. Memoriser les conventions établies
   → memory(action='add', target='memory', content='Convention: [regle] — établie [date]')

4. Memoriser l'état actuel du projet
   → memory(action='add', target='memory', content='Etat projet: [branches, CI, coverage, etc.]')

5. Memoriser les prochaines étapes prioritaires
   → memory(action='add', target='memory', content='Next: [tache 1], [tache 2]')

6. Mettre à jour les entrees existantes si elles sont obsolètes
   → memory(action='replace', target='memory', old_text='[ancien]', content='[nouveau]')
```

**Regles pour la memoire :**
- Garder compact — max 2200 chars total dans la section memory
- Prioriser ce qui EVITE que l'agent refasse les mêmes erreurs
- Inclure le contexte nécessaire pour reprendre le travail
- Supprimer les entrees devenues obsolètes (action='remove')
- TOUJOURS inclure la date dans les entrees

**Exemple de ce à sauvegarder :**
```python
Decision: `npx playwright test --list` obligatoire avant push tests — évite les push avec syntax errors
Bug-CI: Test Chess UI (user.spec.ts:555) manquait `});` — verifier fermeture tests apres modif conventions
E2E: 165/165 tests PASS, coverage chess 67% (roque, en passant, mat non couverts)
Backend: nook-backend v0.5.0, build 2m46s, SQLX_OFFLINE=true
Next: Tests roque/en passant, scripts bash CI, migration Node 24
```

### 8. Pousser les skills modifies
Pousser TOUS les skills qui ont été créés ou modifiés sur `origin/develop` via le push script.
Vérifier que chaque fichier `.claude/skills/*/SKILL.md` modifié est bien poussé.

### 9. Verification finale
```bash
# Git state
cd /tmp/Nook
git status
git log --oneline -5

# Tests (si fichiers tests modifies)
cd frontend
npx playwright test --list

# Build (si fichiers backend modifies)
cd /tmp/Nook/backend
cargo check --quiet
```

### 10. Push final
Pousser TOUS les changements sur `origin/develop` en une seule operation batch si possible.

**N'oublie pas :**
- Config git : `git config user.email "hermes-bot@users.noreply.github.com"` et `user.name "hermes-bot"`
- Token : Dans l'outil mémoire (memory tool), pas en clair dans les fichiers !
- Use : `git remote set-url origin https://hermes-bot:<token>@github.com/MX10-AC2N/Nook.git`

### 11. Message de sortie

```markdown
## 🚪 Session Terminée

✅ Résumé: .claude/hermes/active-session.md
✅ Bugs: .claude/hermes/known-issues.md
✅ Contexte: .claude/hermes/hermes-memory.md
✅ Projet: .claude/project/project-state.md
✅ Skills: créés/mis à jour selon la session
✅ Mémoire: enrichie avec decisions, bugs, conventions, état projet
✅ Git: tout commit et push sur origin/develop
✅ État: [165/165 PASS | 0 fail | clean]

Prochaine session: Je lirai .claude/hermes/ AU DEMARRAGE (hook session-start-tools.md)
Le système est prêt — à la prochaine! 🚪
```

## Conventions
- **TOUJOURS** valider avec `npx playwright test --list` si des fichiers test ont été modifies
- **TOUJOURS** pousser sur `origin/develop` — jamais sur une autre branche
- **TOUJOURS** mettre à jour `hermes/active-session.md` en PREMIER
- **JAMAIS** quitter sans avoir poussé le résumé — c'est la mémoire de la session
- **TOUJOURS** utiliser la nouvelle structure `.claude/hermes/` pour l'espace perso
- Utiliser le token GitHub depuis l'outil mémoire (pas en clair !)

## Template de Push Script (si nécessaire)
```python
import os, json, base64, urllib.request

# Token depuis la mémoire (à récupérer via memory tool)
# token = memory_content['GITHUB_TOKEN']

def push_file(path, msg, content):
    # Implementation pour pousser via GitHub API
    pass
```

## Notes importantes sur la nouvelle structure
- `hermes/` est MON espace perso — toujours lire au démarrage (hook)
- `project/` contient l'état du projet et les bugs actifs
- `reference/` contient les patterns rapides (Rust, Svelte)
- `archive/` preserve TOUT l'historique (rapports, sessions)
- `CLAUDE.md` reste à la racine (orchestrateur principal)
