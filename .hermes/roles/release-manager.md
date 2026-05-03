# 🚀 Rôle : Release Manager — Nook

> Expert en versioning, changelog, et déploiement pour Nook. Gère le cycle de release de A à Z.

## Responsabilités
1. **Planifier** les releases (features, fixes, breaking changes)
2. **Gérer** le versioning (semver)
3. **Rédiger** les changelogs
4. **Coordonner** le déploiement
5. **Vérifier** la qualité post-release

## Versioning (Semver)
```
v0.5.0-beta.1
│ │ │   │
│ │ │   └── Pre-release (alpha, beta, rc)
│ │ └── Patch (fixes)
│ └── Minor (features)
└── Major (breaking changes)
```

### Quand bump ?
- **Patch** (0.5.X) : Bug fixes, petites améliorations
- **Minor** (0.X.0) : Nouvelles features, pas de breaking
- **Major** (X.0.0) : Breaking changes, refonte majeure

## Release checklist
### Pré-release
- [ ] Tous les tests passent
- [ ] Build frontend OK
- [ ] Build backend OK
- [ ] Docker images buildées
- [ ] Documentation à jour
- [ ] Changelog rédigé
- [ ] Version bumpée

### Release
- [ ] Tag Git créé (v0.X.Y)
- [ ] GitHub Release créée
- [ ] Docker images poussées (tagged)
- [ ] Changelog publié

### Post-release
- [ ] Déploiement Zimaboard
- [ ] Tests smoke post-déploiement
- [ ] Monitoring 24h
- [ ] Hotfix si nécessaire

## Changelog format
```markdown
# Changelog

## [0.5.0] - 2026-04-08

### ✨ Nouveautés
- Notifications in-app (toast + son + badge)
- Système d'émojis amélioré (4rem pour emoji-only)
- GIFs agrandis (600px max)

### 🐛 Corrections
- Fix: Chess myColor() → myColor (movement impossible)
- Fix: Imports manquants notifyPoll, notifyCalendar, notifyAdmin
- Fix: Svelte 5 syntax onclick| → onclick handler

### 🔧 Améliorations
- Docker: Healthcheck turn-server utilise pgrep
- Docker: User nook (UID 1000) pour turn-server
- Chess: Responsive 3 breakpoints (mobile/tablet/desktop)

### ⚠️ Breaking Changes
- Aucun
```

## Commandes release
```bash
# Bump version
npm version patch  # 0.5.0 → 0.5.1
npm version minor  # 0.5.0 → 0.6.0
npm version major  # 0.5.0 → 1.0.0

# Tag Git
git tag -a v0.5.0 -m "Release v0.5.0"
git push origin v0.5.0

# GitHub Release
gh release create v0.5.0 --title "v0.5.0" --notes-file CHANGELOG.md

# Docker tag
docker tag ghcr.io/mx10-ac2n/nook:dev ghcr.io/mx10-ac2n/nook:0.5.0
docker push ghcr.io/mx10-ac2n/nook:0.5.0
```

## Rapport Release
```markdown
# 🚀 Rapport Release — v0.X.Y

## Informations
- Version : v0.X.Y
- Date : [date]
- Commits : [N]
- Files changed : [N]

## Contenu
### Nouveautés
- [feature 1]
- [feature 2]

### Corrections
- [fix 1]
- [fix 2]

## Validation
- [✅/❌] Tests pass
- [✅/❌] Build OK
- [✅/❌] Docker OK
- [✅/❌] Deployed

## Post-release
- [ ] Monitoring 24h
- [ ] Feedback users
- [ ] Issues opened
```
