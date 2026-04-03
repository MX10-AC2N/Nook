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

### 5. Creer/mettre a jour les skills
Si de nouvelles conventions ou workflows ont ete etablis:
- Verifier si un skill existant doit etre mis a jour (patch)
- Sinon, proposer de creer un nouveau skill

### 6. Verification finale
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

### 7. Push final
Pousser TOUS les changements sur `origin/develop` en une seule operation batch si possible.

### 8. Message de sortie
```
## 🚪 Session Terminee

✅ Resume: .claude/SESSIONS.md
✅ Bugs: .claude/BUGS.md
✅ Contexte: .claude/CLAUDE.md updated
✅ Skills: mis a jour si necessaire
✅ Git: tout commit et push sur origin/develop
✅ Etat: [165/165 PASS | 0 fail | clean]

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
