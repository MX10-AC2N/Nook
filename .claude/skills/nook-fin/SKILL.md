---
name: nook-fin
category: "devops"
description: "Session exit command — clean shutdown with full context persistence. Run at end of any session to summarize work, update .claude docs, push state, and exit cleanly."
---

# 🚪 Nook Fin (Session Exit)

## Trigger
Run this skill when the user says:
- `/fini`
- "termine la session"
- "quitte proprement"
- "exit cleanly"

## Steps

### 1. Audit git state
```bash
cd /tmp/nook-repo
git fetch origin develop
git status --short
git log --oneline -10
git diff --stat origin/develop
```
- Identifie fichiers modifies, non-commits, et etat de la branche
- Verifie si tout est commit et push

### 2. Resume exhaustif de la session
Generer un résumé markdown avec ces sections:

```markdown
## Session [N] — [Date] ([theme principal])

### Contexte
[Objectif initial, probleme a resoudre]

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
- [fichier: lignes modifiees, nature du changement]

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

### 3. Mettre a jour SESSIONS.md
```bash
# Recuperer le contenu courant
python3 -c "
import urllib.request, json
token = open('/root/.hermes/.env').read().strip().split('=')[1].strip()
req = urllib.request.Request(f'https://raw.githubusercontent.com/MX10-AC2N/Nook/develop/.claude/SESSIONS.md')
with urllib.request.urlopen(req) as resp:
    content = resp.read().decode()
print(content[-500:])  # Voir la fin actuelle
"
```
- Ajouter le resume de la session a la fin de `.claude/SESSIONS.md`
- Pousser vers `origin/develop`

### 4. Mettre a jour les fichiers .claude
Pour CHAQUE fichier pertinent, verifier et mettre a jour:

| Fichier | Action |
|---------|--------|
| `.claude/SESSIONS.md` | ✅ Ajouter resume session |
| `.claude/BUGS.md` | ✅ Marquer bugs corriges |
| `.claude/E2E-TARGETED-REPORT.md` | ✅ Mettre a jour stats tests |
| `.claude/CLAUDE.md` | ✅ Mettre a jour statut CI |
| `.claude/rules/critical-pitfalls.md` | ✅ Ajouter nouvelles regles |
| `.claude/TEST-AND-SECURITY-AUDIT-2026.md` | ✅ Mettre a jour si audit fait |
| `.claude/TEST_REPORT.md` | ✅ Mettre a jour si CI run |

### 5. Creer/mettre a jour les skills pertinents

**Analyse : identifier ce qui a ete accompli pendant la session**

Examiner les changements de la session et determiner quels skills doivent etre crees ou mis a jour :

| Si la session a touche... | Skill a verifier/creer |
|---------------------------|----------------------|
| Tests E2E, CI, Playwright | `nook-e2e-testing` |
| Backend Rust, Axum, API | `nook-rust-backend` |
| Frontend Svelte, UI, CSS | `nook-svelte-frontend` |
| DevOps, Docker, deploy | `nook-ci-devops` |
| Review code, PR, audit | `nook-review` |
| Planification, roadmap | `nook-plan-ceo` ou `nook-plan-eng` |
| Retro, retrospective | `nook-retro` |
| Ship, release, version | `nook-ship` |
| Nouveau domaine non couvert | Creer un NOUVEAU skill |

**Procedure pour chaque skill concerne :**

```python
# a) Charger le skill existant pour voir son contenu actuel
# skill_view(name='nook-e2e-testing')

# b) Identifier les sections qui doivent etre mises a jour :
#    - Nouvelles conventions decouvertes pendant la session
#    - Nouveaux patterns de code utilises
#    - Nouveaux bugs/solutions documentes
#    - Changements d'architecture

# c) Mettre a jour avec skill_manage(action='patch', name='...', ...)
#    ou skill_manage(action='edit', name='...', content='...') pour reecriture majeure

# d) Creer un NOUVEAU skill si un domaine n'est pas couvert :
#    skill_manage(action='create', name='nook-[domaine]', ...)
```

**Checklist skills :**
- [ ] Lister tous les skills existants du projet (`skills/` dans `.claude/`)
- [ ] Pour chaque skill, demander : "Est-ce que cette session a ajoute des connaissances pertinentes pour ce skill ?"
- [ ] Si OUI → patcher le skill avec les nouvelles info
- [ ] Si un domaine important n'a pas de skill → creer un nouveau skill
- [ ] Pousser les skills modifies sur `origin/develop`

### 5b. Enrichir la memoire persistante (Hermes memory)

**C'est CRITIQUE** — sans ca, la prochaine session ne saura pas ce qui a ete fait.

Utiliser le `memory` tool pour sauvegarder :

```
1. Memoriser les decisions d'architecture prises
   → memory(action='add', target='memory', content='Decision: [quoi] pour [raison]')

2. Memoriser les bugs et leurs solutions
   → memory(action='add', target='memory', content='Bug [X]: [symptome] → Fix: [solution]')

3. Memoriser les conventions etablies
   → memory(action='add', target='memory', content='Convention: [regle] — etablie [date]')

4. Memoriser l'etat actuel du projet
   → memory(action='add', target='memory', content='Etat projet: [branches, CI, coverage, etc.]')

5. Memoriser les prochaines etapes prioritaires
   → memory(action='add', target='memory', content='Next: [tache 1], [tache 2]')

6. Mettre a jour les entrees existantes si elles sont obsolete
   → memory(action='replace', target='memory', old_text='[ancien]', content='[nouveau]')
```

**Regles pour la memoire :**
- Garder compact — max 2000 chars total dans la section memory
- Prioriser ce qui EVITE que l'agent refasse les memes erreurs
- Inclure le contexte necessaire pour reprendre le travail
- Supprimer les entrees devenues obsolete (action='remove')
- TOUJOURS inclure la date dans les entrees

**Exemple de ce a sauvegarder :**
```
Decision: `npx playwright test --list` obligatoire avant push tests — evite les push avec syntax errors
Bug-CI: Test Chess UI (user.spec.ts:555) manquait `});` — verifier fermeture tests apres modif conventions
E2E: 165/165 tests PASS, coverage chess 67% (roque, en passant, mat non couverts)
Backend: nook-backend v0.5.0-beta.1, build 2m46s, SQLX_OFFLINE=true
Next: Tests roque/en passant, scripts bash CI, migration Node 24
```

### 6. Pousser les skills modifies
Pousser TOUS les skills qui ont ete crees ou modifies sur `origin/develop` via le push script.
Verifier que chaque fichier `.claude/skills/*/SKILL.md` modifie est bien pousse.

### 7. Verification finale
```bash
# Git state
cd /tmp/nook-repo
git status
git log --oneline -5

# Tests (si fichiers tests modifies)
cd frontend
npx playwright test --list

# Build (si fichiers backend modifies)
cd /tmp/nook-repo/backend
cargo check --quiet
```

### 8. Push final
Pousser TOUS les changements sur `origin/develop` en une seule operation batch si possible.

### 9. Message de sortie
```
## 🚪 Session Terminee

✅ Resume: .claude/SESSIONS.md
✅ Bugs: .claude/BUGS.md
✅ Contexte: .claude/CLAUDE.md updated
✅ Skills: crees/mis a jour selon la session
✅ Memoire: enrichie avec decisions, bugs, conventions, etat projet
✅ Git: tout commit et push sur origin/develop
✅ Etat: [165/165 PASS | 0 fail | clean]

Prochaine session: dire `/fini` pour quitter proprement a nouveau.
A la prochaine! 💪
```

## Conventions
- **TOUJOURS** valider avec `npx playwright test --list` si des fichiers test ont ete modifies
- **TOUJOURS** pousser sur `origin/develop` — jamais sur une autre branche
- **TOUJOURS** mettre a jour SESSIONS.md en PREMIER
- **JAMAIS** quitter sans avoir pousse le resume — c'est la memoire de la session
- Utiliser le token GitHub depuis `/root/.hermes/.env`

## Template de Push Script
```python
import os, json, base64, urllib.request

token = open("/root/.hermes/.env").read().strip().split("=")[1].strip()

def get_remote(path):
    req = urllib.request.Request(f"https://raw.githubusercontent.com/MX10-AC2N/Nook/develop/{path}")
    with urllib.request.urlopen(req) as resp:
        return resp.read().decode()

def push_file(path, msg, content):
    req_sha = urllib.request.Request(
        f"https://api.github.com/repos/MX10-AC2N/Nook/contents/{path}?ref=develop",
        headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github.v3+json"}
    )
    with urllib.request.urlopen(req_sha) as resp:
        sha = json.loads(resp.read().decode())['sha']
    payload = json.dumps({
        "message": msg,
        "content": base64.b64encode(content.encode()).decode(),
        "sha": sha,
        "branch": "develop"
    }).encode()
    req_put = urllib.request.Request(
        f"https://api.github.com/repos/MX10-AC2N/Nook/contents/{path}",
        data=payload,
        headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github.v3+json", "Content-Type": "application/json"},
        method="PUT"
    )
    try:
        with urllib.request.urlopen(req_put) as resp:
            result = json.loads(resp.read().decode())
            sha_short = result.get('commit',{}).get('sha','')[:12]
            print(f"  ✅ PUSH {path} sha={sha_short}")
            return True
    except urllib.error.HTTPError as e:
        print(f"  ❌ PUSH FAIL {path}: {e.code}")
        print(f"  {e.read().decode()[:200]}")
        return False
```
