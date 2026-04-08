# 🐛 BUGS.md — Nook

> Mis à jour : **2026-04-08** (session 47 — déploiement Zimaboard)

---

## 🔴 BUGS ACTIFS

### BUG-01 : Emoji — impossible d'envoyer plusieurs emojis dans un message

**Fichier** : `frontend/src/routes/chat/+page.svelte` — `handleSelectEmoji()`
**Symptôme** : Quand l'input est vide, cliquer sur un emoji l'envoie immédiatement comme message standalone. Impossible de les empiler.
**Impact** : L'utilisateur ne peut envoyer qu'un seul emoji par message.
**Fix (session 47)** : `handleSelectEmoji()` ajoute toujours l'emoji au champ de saisie au lieu d'envoyer immédiatement. Le picker reste ouvert pour sélectionner plusieurs emojis. L'utilisateur clique "Envoyer" quand prêt.
**Statut** : ✅ Fixé (commit f2a146ce0ce8)

### BUG-02 : GIF trop petit à l'écran

**Fichier** : `frontend/src/routes/chat/+page.svelte` — CSS `:global(.chat-gif)`
**Symptôme** : Les GIFs envoyés dans le chat s'affichent en 200×200px, trop petit.
**Impact** : Mauvaise expérience utilisateur pour les GIFs.
**Fix (session 47)** : CSS `max-width` et `max-height` augmentés de 200px à 300px.
**Statut** : ✅ Fixé (commit f2a146ce0ce8)

### BUG-03 : Échecs — bloqué sur la page de chargement

**Fichier** : `frontend/src/routes/chess/[game_id]/+page.svelte` — `onMount()`
**Symptôme** : Après création d'une partie, la page montre un spinner "Chargement de la partie…" indéfiniment.
**Impact** : Les parties d'échecs ne se lancent pas.
**Fix (session 47)** : `onMount()` entouré de try/catch/finally pour garantir que `pageLoading = false` même si `loadGame` lance une exception inattendue.
**Statut** : ✅ Fixé (commit 026706216881) — à tester après build frontend

---

## ✅ BUGS RÉSOLUS (session 47 — déploiement Docker)

### Docker — Permissions denied sur volumes

**Cause** : L'utilisateur `nook` dans le conteneur avait UID 100 (auto-assigné) alors que l'hôte `casaos` est UID 1000.
**Fix** : `Dockerfile.release` (nook + turn) utilise `addgroup -S -g 1000 nook && adduser -S -u 1000 -G nook nook`.

### Docker — .env mal configuré

**Cause** : Les chemins conteneur (`DATABASE_URL`, `UPLOADS_DIR`, etc.) étaient dans le `.env` avec des chemins hôtes.
**Fix** : `docker-compose.yml` hardcode les chemins conteneur. `.env.example` simplifié — seuls `DATA_DIR`, `LOGS_DIR`, `TURN_CONFIG_DIR` restent configurables.

### Turn-server — template écrasé par volume mount

**Cause** : Le template `turnserver.conf.template` était dans `/etc/turn-server/` qui est écrasé par le volume mount.
**Fix** : Template déplacé vers `/opt/turn-server/`. Entrypoint copie de `/opt` vers le volume mounté.

### Turn-server — `--config` manquant

**Cause** : Le binaire `turn-server` attend `--config <CONFIG>` mais l'entrypoint ne le passait pas.
**Fix** : Entrypoint lance `turn-server --config /etc/turn-server/turnserver.conf`.

### Docker.yml — fichiers turn-rs manquants dans le contexte Docker

**Cause** : Le contexte Docker est `docker-context/turn/` (binaires uniquement). Les fichiers de conf n'étaient pas copiés.
**Fix** : Ajout d'un step "Copy turn-rs config files to docker context" dans le workflow.

### Docker.yml — tag `latest` sur develop

**Cause** : Toutes les branches poussaient le tag `:latest`.
**Fix** : `develop` pousse `:dev`, `main` pousse `:latest`.

---

## 📋 Pièges critiques

- **Alpine 3.21 uniquement** — pas de Debian/glibc dans les Dockerfiles
- **UID/GID 1000** — tous les conteneurs doivent utiliser UID 1000 pour matcher `casaos` sur l'hôte
- **Contexte Docker** : `docker-context/turn/` ne contient que les binaires. Tout nouveau fichier dans `services/turn-rs/` doit être copié dans le contexte pendant le workflow `Docker.yml`
- **Volume turn-config** : `:rw` (pas `:ro`) pour permettre l'init automatique du config
