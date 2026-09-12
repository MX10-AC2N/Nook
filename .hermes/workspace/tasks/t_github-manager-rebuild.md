# 📋 Tâche Kanban — t_github-manager-rebuild

> **Nom**: GITHUB-MANAGER-REBUILD
> **Assigné**: github-manager
> **Status**: 🔄 TODO
> **Créé**: 2026-09-11
> **Priorité**: 🔴 HIGH (déploiement)

---

## Description

Le fix emoji picker (commit 0afb68c8, branch develop) est pushé sur GitHub mais le homeserver (192.168.1.192:6300) n'a pas encore le nouveau build. Il faut déclencher le pipeline CI/CD pour rebuilder et déployer.

## Commits concernés

- **0afb68c8** — fix(chat): emoji picker vertical flip — open above when insufficient space below
- Pushé sur origin/develop

## Actions Requises

### 1. Déclencher les workflows manuels
```
# Frontend build
gh workflow run Frontend.yml --ref develop

# Backend build  
gh workflow run Backend.yml --ref develop

# E2E tests
gh workflow run test-nook.yml --ref develop

# Docker
gh workflow run Docker.yml --ref develop

# Release
gh workflow run Release.yml --ref develop
```

### 2. CI/CD Order (à suivre)
```
1. Frontend.yml  → build SvelteKit, artifact
2. Backend.yml   → compile amd64+arm64
3. test-nook.yml → Docker + E2E 156 tests
4. Docker.yml    → dawidd6 artifact download → GHCR
5. Release.yml   → bump VERSION + tag git
```

### 3. Déploiement sur homeserver
Après le build réussi:
- SSH sur 192.168.1.192
- Pull la nouvelle image GHCR
- Redémarrer les services Docker
- Vérifier 192.168.1.192:6300 et 192.168.1.192:3000
- Tester le picker emoji (ouvrir un message → cliquer emoji → vérifier position au-dessus/en dessous)

## Fichiers Impliqués
- .github/workflows/Frontend.yml
- .github/workflows/Backend.yml
- .github/workflows/test-nook.yml
- .github/workflows/Docker.yml
- .github/workflows/Release.yml
- backend/src/routes/chat/+page.svelte (le fix source)

## Critères d'Acceptance
- [ ] Frontend build passe
- [ ] Backend build passe
- [ ] E2E tests passent (156)
- [ ] Docker image buildée et poussée sur GHCR
- [ ] Homeserver 192.168.1.192:6300 mis à jour
- [ ] Picker emoji s'ouvre correctement (au-dessus si pas de place en dessous)
- [ ] Pas de régression sur les autres fonctionnalités

## Notes Techniques
- GitHub free account → pas de cron jobs, workflow_dispatch uniquement
- Le backend.yml se déclenche aussi sur push develop (path filter backend/**)
- Le frontend.yml est workflow_dispatch uniquement
- Les builds utilisent musl target pour amd64 + arm64

## Delegation
- Deleguer a: github-manager
- Type: CI/CD trigger + deployment
- Review: vérifier les runs GitHub Actions
- PLUR engram: après déploiement réussi
