# 🐛 BUGS.md — Nook

> Mis à jour : **2026-05-16** (session 2026-05-16 — validation E2EE + .hermes update)

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

### BUG-04 : Messages chiffrés 🔒 (clé indisponible)

**Fichier** : `frontend/src/lib/stores/cryptoStore.ts` — gestion des clés E2EE
**Symptôme** : Les messages sont marqués comme chiffrés mais la clé n'est pas disponible, rendant le message illisible.
**Impact** : Perte de messages privés, expérience utilisateur dégradée.
**Fix** : BUG-002 non déployé (à porter sur develop)
**Statut** : 🔴 Actif (fix commit en attente de déploiement)

### BUG-05 : Navigation chats privés défaillante

**Fichier** : `frontend/src/routes/chat/+page.svelte` — navigation entre groupes/privés
**Symptôme** : Cliquer sur un chat privé reste sur le groupe actuel, la navigation ne se fait pas.
**Impact** : Impossible d'accéder aux chats privés.
**Statut** : 🔴 Actif (à investiguer)

### BUG-06 : 401 Unauthorized sur ressources + Service Workers non supportés

**Fichier** : `frontend/src/service-worker.ts` + backend auth middleware
**Symptôme** : 
- Requêtes vers ressources statiques renvoient 401 Unauthorized
- Service Workers ne sont pas supportés car le site est servi en HTTP (Service Workers requièrent HTTPS)
**Impact** : Pas de cache offline, requêtes authentifiées échouent
**Statut** : 🔴 Actif (HTTPS fixé dans entrypoint, à redéployer)

### BUG-07 : Réaction emoji étendue défaillante

**Fichier** : `frontend/src/routes/chat/+page.svelte` — composant de réaction emoji
**Symptôme** :
- Clic sur le bouton "+" (emoji étendus) ne fait rien
- Les emojis étendus ne s'affichent plus du tout
- Seuls les 6 emojis principaux fonctionnent pour les réactions
**Impact** : Expérience utilisateur limitée pour les réactions
**Statut** : 🟡 En cours de correction (Hermes Bot)

### BUG-08 : Message perdu après rafraîchissement chat

**Fichier** : `frontend/src/lib/stores/chatStore.svelte.ts` — gestion de l'état du chat
**Symptôme** :
- Poster un nouveau message dans un chat (ex: avec Géraldine)
- Rafraîchir la page
- Le dernier message posté avant le refresh n'est plus visible
**Impact** : Perte de messages récents après navigation/refresh
**Statut** : 🟡 En cours de correction (Hermes Bot)

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

---

## ✅ BUGS E2EE RÉSOLUS (sessions 2026-05-15/16)

### E2EE-BUG-01 : Clé publique désynchronisée — `incorrect key pair`

**Fichiers** : `cryptoStore.svelte.ts` — `unlockCrypto()` + `registerPublicKeyOnServer()`
**Symptôme** : `sodium.crypto_box_open_easy` échoue avec "incorrect key pair" au déchiffrement.
**Cause** : `unlockCrypto` assignait `ready=true` avant `await registerPublicKeyOnServer()` (fire-and-forget) → `users.public_key` pas synchronisé.
**Fix** (commit `36eefe5c`) : `await registerPublicKeyOnServer(kp.publicKey)` avant `_keyPair=kp` et `ready=true`.

### E2EE-BUG-02 : `encryptForRecipients` casse tout si un destinataire invalide

**Fichiers** : `crypto.ts` — `encryptForRecipients()`
**Symptôme** : `encrypted_keys: {}` vide sur serveur — messages sans moyen de déchiffrement.
**Cause** : Boucle sans try/catch — une clé invalide casse toute la boucle.
**Fix** (commit `f0a8c8d1`) : try/catch par destinataire + `console.warn` par échec individué.
**Note** : `db.rs ligne 456` ne stocke pas `encrypted_keys` si vide.

### E2EE-BUG-03 : `_FAILED_DECRYPT_IDS` mutilait les champs E2EE

**Fichier** : `chatStore.svelte.ts` — `_decryptAllIfReady()`
**Symptôme** : `delete message.nonce, message.encrypted` en cas d'échec → message irrémédiablement perdu.
**Fix** (commit `36eefe5c`) : Conservation des champs E2EE, seul l'ID ajouté au Set → re-déchiffrement futur possible.

---

## ⚠️ LIMITES E2EE CONNUES (non-bugs)

### Anciens messages indéchiffrables après rotation de clé X25519
- **Nature** : Structurale, pas un bug de code.
- **Cause** : `users.public_key` change entre sessions → ancienne clé de session n'a pas de paire privée.
- **Conséquence** : Échec systématique "incorrect key pair" sur anciens messages.
- **Pas de fix** : Rotation de clé = opération destructive par conception en chiffrement asymétrique.
- **À documenter** : guide utilisateur → changement compte = perte lisibilité anciens messages.

