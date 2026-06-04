---
name: nook-github-workflows
description: Ordonnancer et surveiller les GitHub Actions Nook — Frontend → Backend → Turn → Docker.
---
# nook-github-workflows

## Règle d'or
Ordre strict: Frontend (220018362) → Backend (220018363) → Turn (257238341) → Docker (220018364)
Docker ne se lance JAMAIS avant les 3 autres au vert.

## Quand trigger
- `Frontend.yml` si `frontend/**` a changé
- `Backend.yml` si `backend/**` a changé
- `Turn.yml` si `services/turn-rs/**` a changé
- `Docker.yml` si Backend/Turn/Frontend sont verts **et** qu'un build d'image est nécessaire

## Séquence correcte
1. `git fetch origin`
2. Inspecter `gh run list --limit 5`
3. Trigger manuel dans l'ordre, attendre chaque fin avant le suivant
4. Ne JAMAIS utiliser de cron ou de schedule

## Commandes essentielles
- `gh run list --workflow=Frontend.yml --limit 5`
- `gh run list --workflow=Backend.yml --limit 5`
- `gh run list --workflow=Turn.yml --limit 5`
- `gh run list --workflow=Docker.yml --limit 5`

## Réutilisation d'artifacts
Frontend/Backend/Turn uploadent leurs artifacts. Docker les télécharge avec `dawidd6/action-download-artifact` selon `github.event.workflow` filtré par chemin modifié.
