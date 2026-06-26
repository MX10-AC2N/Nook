# gh CLI Prerequisite for CI Orchestration

## Problème (2026-06-13)

Le container Hermes Docker mis à jour **n'avait plus `gh` CLI installé**, rendant impossible :
- Déclenchement workflows (`gh workflow run`)
- Surveillance runs (`gh run watch`, `gh run list`)
- Vérification auth (`gh auth status`)

## Solution immédiate

```bash
# Debian/Ubuntu (container Hermes actuel)
apt update && apt install -y gh

# Alpine (si image Alpine)
apk add github-cli
```

Puis authentification :
```bash
gh auth login
# → GitHub.com → HTTPS → Paste PAT (scopes: repo + workflow)
```

## Vérification

```bash
gh --version
gh auth status  # doit montrer "Logged in to github.com account MX10-AC2N"
```

## Checklist pré-CI (À AJOUTER AU DÉBUT DE TOUTE ORCHESTRATION)

```bash
# 1. gh installé ?
command -v gh >/dev/null || { echo "gh manquant - installer et auth"; exit 1; }

# 2. gh authentifié ?
gh auth status 2>&1 | grep -q "Logged in" || { echo "gh non authentifié"; exit 1; }

# 3. Token valide pour workflow dispatch ?
gh workflow list --repo MX10-AC2N/Nook >/dev/null || { echo "token invalide/insuffisant"; exit 1; }
```

## Point d'attention Docker

L'image Docker Hermes **doit inclure `gh` CLI + config d'auth persistante** (via volume `/opt/data/home/.config/gh/` ou entrypoint `gh auth login` au démarrage).

Sans `gh` → CI bloqué → pas de build → pas de déploiement.