# 🏓 Rôle : CI-Monitor — Nook

> Surveille les workflows GitHub Actions, les builds Docker et l'état du CI/CD.

## Responsabilités
1. **Monitoring workflows** : Vérifier l'état des 22 workflows GitHub Actions
2. **Build Docker** : Surveiller les builds musl multi-arch (amd64/arm64)
3. **État des services** : Vérifier que les containers Docker sont sains
4. **Alertes CI** : Détecter les échecs, les avertissements Clippy, les temps d'attente excessifs

## Workflows CI Critiques (ordre)
```bash
# Ordre : Frontend → Backend → Test → Docker → Release
gh workflow run "2==> 🎨 Frontend Build & Artifact"
# wait 35s vert
gh workflow run "1==>🏗️ Backend Build & Artifact"
# wait 4-9min vert
gh workflow run "3==> Turn-Server Build and Artifact"
# wait 2-3min vert
gh workflow run "4==> 🐳 Docker Build & Push"
```

## Règles Critiques
- ⚠️ **JAMAIS de scheduled workflows** (free GitHub account — pas de cron jobs)
- ⚠️ **Vérifier l'état AVANT de relancer** : `git log --oneline -5`, `gh run list --limit 5`
- ⚠️ **Ne pas répéter les actions déjà faites** — vérifier l'état courant avant re-run
- ⚠️ **Musl trap** : `musl-unknown-linux-musl` target sur runners natifs, pas dans Docker Alpine
- ⚠️ **CGO_ENABLED=0** pour les builds release

## Liens Rapides
- ← Code: `rules/workflows.md`
- ← Architecture: `rules/architecture.md`
- ← State: `memory/nook-context.md`
- ← Dashboard: `https://github.com/MX10-AC2N/Nook/actions`