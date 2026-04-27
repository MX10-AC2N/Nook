# 🚀 Deployment Checklist — Nook

## Pré-déploiement
- [ ] `cd frontend && npm run build` → ✅
- [ ] `cd backend && cargo check --quiet` → ✅
- [ ] `npx playwright test --list` → ✅
- [ ] Tous les imports notificationStore présents
- [ ] Svelte 5 syntax (onclick pas onclick|)
- [ ] Dockerfiles Alpine 3.21, UID/GID 1000
- [ ] healthchecks fonctionnels

## GitHub Actions
- [ ] `Frontend.yml` → ✅
- [ ] `Backend.yml` → ✅
- [ ] `Docker.yml` → ✅
- [ ] `test-nook.yml` → ✅

## Déploiement Zimaboard
```bash
# 1. Pull nouvelles images
docker compose pull

# 2. Redémarrer services
docker compose up -d

# 3. Vérifier état
docker compose ps
docker compose logs --tail=50 nook
docker compose logs --tail=50 turn

# 4. Tester
# - Chat: envoyer message, emoji, GIF
# - Chess: faire un mouvement
# - Notifications: vérifier toast + son
# - TURN: tester avec icetest.simplewebrtc.com
```

## Post-déploiement
- [ ] Site accessible sur http://zimaboard:6300
- [ ] Chat fonctionne (messages, emojis, GIFs)
- [ ] Chess fonctionne (mouvements)
- [ ] Notifications fonctionnent
- [ ] TURN serveur accessible
- [ ] Logs sans erreurs critiques

## Rollback
```bash
# Si problème, revenir à l'image précédente
docker compose pull --policy=missing
docker compose up -d
```
