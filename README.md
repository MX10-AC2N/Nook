![Logo animé du projet](frontend/static/logo-animated.svg)

<div align="center">

> **v0.5.0**

[![CI Backend](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml)
[![CI Frontend](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml)
[![CI Tests](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml)
[![Docker Build](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml)

[![Docker Image Version](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/latest_tag?color=blue&label=version&trim=&ignore=sha-*,latest)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Docker Image Size](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/size?color=green&label=image%20size&tag=latest)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Platforms](https://img.shields.io/badge/platforms-amd64%20%7C%20arm64-lightgrey)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)

[![Rust](https://img.shields.io/badge/Backend-Rust%20+%20Axum%200.8-orange?logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/Frontend-SvelteKit%205%20Runes-FF3E00?logo=svelte)](https://kit.svelte.dev/)
[![SQLite](https://img.shields.io/badge/DB-SQLite-003B57?logo=sqlite)](https://sqlite.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

</div>

---

## 👋 Bonjour !

**Nook, c'est la messagerie de votre famille.**

Pas de compte chez Google ou Meta. Pas d'abonnement à payer. Pas de données qui partent je-ne-sais-où.

Juste un petit serveur qui tourne chez vous, sur votre machine, et que votre famille peut utiliser depuis n'importe quel appareil — téléphone, tablette, ordinateur.

C'est comme avoir votre propre WhatsApp, mais en mieux, parce que c'est le vôtre.

---

## 🏠 Ce qu'on peut faire avec Nook

### 💬 Discuter en famille
Envoyez des messages en temps réel, réagissez avec des emojis, partagez des photos et des fichiers. Il y a un groupe global pour tout le monde, et vous pouvez aussi créer des conversations privées.

### 🔐 Sans compromis sur la vie privée
Les messages sont chiffrés de bout en bout (X25519). Les fichiers (jusqu'à 50 Mo) sont chiffrés sur le disque (XChaCha20). Les fichiers volumineux (>50 Mo) sont transférés directement entre appareils via WebRTC (P2P) avec chiffrement E2EE. Les mots de passe ne sont jamais stockés en clair (Argon2id). Même vous, en tant qu'admin, vous ne pouvez pas les lire.

### 🔔 Notifications sur votre téléphone
Recevez une notification quand quelqu'un vous écrit — même quand l'onglet est fermé. Ça s'active depuis les Paramètres de Nook.

### 📅 Calendrier partagé
Tous les événements de la famille au même endroit. Anniversaires, rendez-vous, sorties... Glisser-déposer pour déplacer les événements.

### ♟️ Parties d'échecs
Jouez entre membres ou contre l'IA (5 niveaux). Animations des pièces, coups spéciaux, minuteur configurable.

### 📊 Sondages
"Qu'est-ce qu'on mange ce soir ?", "Qui vient dimanche ?". Créez un vote en quelques secondes.

### 📞 Appels audio & vidéo
Connexion directe entre appareils (WebRTC). Le serveur ne voit jamais le flux. Fonctionne en LAN et via internet.

### 🎨 Trois thèmes
Jardin Secret 🌿 · Space Hub 🚀 · Maison Chaleureuse 🏠 — avec mode sombre en option.

---

## 🚀 Installation rapide

**Ce qu'il faut** : Docker + Docker Compose. C'est tout.

Ça fonctionne sur `linux/amd64` et `linux/arm64` (Raspberry Pi 4+, Zimaboard, NAS).

```bash
git clone https://github.com/MX10-AC2N/Nook.git && cd Nook
cp .env.example .env          # éditez PUBLIC_SITE_URL avec l'IP de votre serveur
docker compose up -d
```

Ouvrez `http://votre-serveur:6300` dans un navigateur. C'est prêt ! 🎉

> **Premier lancement** — un compte `admin` est créé automatiquement avec le mot de passe `changeme2026`.
> Vous serez forcé de le changer à la première connexion.

**Mise à jour :**
```bash
docker compose pull && docker compose up -d
```

---

## 📸 Captures d'écran

Voici Nook en action sur différents appareils :

### 💬 Conversation
![Chat Nook](docs/screenshots/chat.png)
*Interface de chat avec messages chiffrés E2EE*

### 📞 Appels audio/vidéo
![Appel Nook](docs/screenshots/call.png)
*Appel WebRTC P2P (chiffré de bout en bout)*

### 📅 Calendrier
![Calendrier Nook](docs/screenshots/calendar.png)
*Événements familiaux partagés*

### ♟️ Parties d'échecs
![Échecs Nook](docs/screenshots/chess.png)
*Jouez contre l'IA ou entre membres*

### 📊 Sondages
![Sondages Nook](docs/screenshots/polls.png)
*Créez des votes en quelques secondes*

### 🔧 Paramètres
![Paramètres Nook](docs/screenshots/settings.png)
*Thèmes, notifications, sécurité*

---

## 👥 Comment inviter votre famille

1. Connectez-vous avec le compte `admin`
2. Allez dans `/admin` → onglet **Invitations**
3. Générez un lien — il expire dans 48h et ne fonctionne qu'une fois
4. Envoyez ce lien à la personne par SMS, email, ou en main propre
5. Elle crée son compte → vous l'approuvez dans l'onglet **Membres en attente**

---

## 🔔 Installer le certificat CA (pour les notifications)

Pour que les notifications push fonctionnent sur votre téléphone, il faut installer un petit certificat. C'est normal et sécurisé — c'est le certificat de votre propre serveur.

**Comment faire :**

1. **Ouvrez** `http://votre-serveur:6300/ca/help` dans votre navigateur
2. **Téléchargez** le certificat via le bouton
3. **Installez-le** sur votre téléphone :
   - **Android** : Paramètres → Sécurité → Certificats → Installer depuis le stockage
   - **Samsung** : Paramètres → Biométrie et sécurité → Autres paramètres → Certificats → Installer
   - **iPhone** : Réglages → Général → VPN et gestion de l'appareil → Installer
4. **Redémarrez** votre navigateur
5. **Activez** les notifications dans Nook (Paramètres → Notifications)

> Le certificat est valide 10 ans. Vous n'aurez plus jamais à y toucher.

---

## 🌐 Accès depuis internet (optionnel)

Vous voulez accéder à Nook depuis l'extérieur de votre réseau ? Placez-le derrière un reverse proxy.

Compatible avec **Nginx Proxy Manager**, **Caddy**, **Traefik**.

```
https://nook.votre-famille.fr  →  http://localhost:6300
```

Deux choses importantes :
- Ajoutez votre domaine dans `PUBLIC_SITE_URL` (et `ALLOWED_ORIGINS` si différent)
- Activez le support WebSocket dans votre proxy (`/ws` est utilisé pour les échecs et les appels)

---

## 🔒 HTTPS local (pour les appels audio/vidéo)

L'enregistrement audio et vidéo dans le navigateur nécessite un **contexte sécurisé** (HTTPS). Sur HTTP LAN, le navigateur bloque l'accès au microphone.

Nook inclut un reverse proxy **nginx local** qui sert HTTPS sur le port 6443 :

```bash
# Démarrez Nook normalement
docker compose up -d
```

Un certificat auto-signé est généré **automatiquement** au premier lancement (valide 10 ans).

```
LAN (HTTPS)                     WAN (HTTPS)
https://192.168.1.x:6443        https://votre-domaine.com
       │                              │
   ┌───┴───┐                   ┌──────┴──────┐
   │ nginx │                   │ nginx proxy │
   │ local │                   │  manager    │
   └───┬───┘                   └──────┬──────┘
       │                              │
       └──────────┬───────────────────┘
                  │
             ┌────┴────┐
             │  Nook   │ :3000
             └─────────┘
```

Le port HTTPS local est configurable via `NGINX_HTTPS_PORT` dans `.env`.

---

## 🎁 GIFs — Mise à jour automatique

Les GIFs sont stockés dans le volume de données (`DATA_DIR/gifs/`) et servis directement par Nook — aucune requête externe n'est envoyée quand un membre envoie un GIF.

**Bonne nouvelle** : la mise à jour des GIFs est **automatique**. Le backend lance une tâche au démarrage qui vérifie toutes les 7 jours si de nouveaux GIFs sont disponibles sur Giphy.

**Ce qu'il faut :**
1. Une clé API Giphy (gratuite) dans votre `.env` : `GIPHY_API_KEY=***`
2. Redémarrez Nook : `docker compose up -d`

Le script télécharge ~10 GIFs pour chacun des 12 thèmes les plus populaires Giphy (réactions, humour, animaux, fête, anniversaire…). Aucun rebuild Docker nécessaire — les GIFs sont servis directement depuis le volume.

> **Note** : Si vous n'avez pas de clé Giphy, les GIFs par défaut (inclus dans l'image Docker) seront utilisés.

---

## ⚙️ Configuration avancée

Tout se passe dans le fichier `.env` — le `.env.example` contient toutes les variables documentées.

Les essentielles :

| Variable | Ce qu'elle fait |
|----------|----------------|
| `PUBLIC_SITE_URL` | L'URL depuis laquelle vous accédez à Nook (`http://192.168.1.x:6300` ou votre domaine) |
| `ALLOWED_ORIGINS` | Si vous accédez depuis plusieurs URLs (LAN + domaine externe), listez-les ici |
| `DATA_DIR` | Où stocker la base de données et les fichiers uploadés |
| `VAPID_PRIVATE_KEY` | Pour les notifications push — voir ci-dessous pour générer les clés |
| `VAPID_PUBLIC_KEY` | Idem — les deux vont ensemble |
| `GIPHY_API_KEY` | Pour les GIFs — clé SDK gratuite sur [developers.giphy.com](https://developers.giphy.com) |

> Le fichier `.env` reste sur votre serveur. Ne le commitez jamais dans git.

### Génération des clés VAPID

Les clés VAPID servent à authentifier les notifications push. Vous n'avez besoin de les générer qu'une seule fois.

**Option 1 — Avec OpenSSL (recommandé) :**
```bash
# Générer la clé privée
openssl ecparam -name prime256v1 -genkey -noout -out vapid_private.pem

# Extraire la clé privée en base64url (pour VAPID_PRIVATE_KEY)
openssl ec -in vapid_private.pem -outform DER | tail -c +8 | head -c 32 | base64 -w0 | tr '+/' '-_' | tr -d '='

# Extraire la clé publique en base64url (pour VAPID_PUBLIC_KEY)
openssl ec -in vapid_private.pem -pubout -outform DER | tail -c 65 | base64 -w0 | tr '+/' '-_' | tr -d '='
```

**Option 2 — Avec Node.js (si vous l'avez installé) :**
```bash
npx web-push generate-vapid-keys
```

**Option 3 — En ligne :**
Allez sur [vapidkeys.com](https://www.vapidkeys.com/) et copiez les clés générées.

> Copiez les deux clés dans votre `.env` et redémarrez Nook avec `docker compose up -d`.

---

## ❓ Questions fréquentes

**Mes données sont où ?**
Dans le dossier `DATA_DIR` sur votre machine. Rien ne sort de chez vous.

**Ça tourne sur Raspberry Pi ?**
Oui. L'image est compilée nativement pour `arm64` — Raspberry Pi 4+, Zimaboard, NAS Synology/QNAP/TrueNAS.

**Le chiffrement est vraiment activé ?**
Oui depuis la v0.5.0 : les clés X25519 sont générées à la première connexion de chaque membre et stockées chiffrées sur leur appareil. Les fichiers partagés sont chiffrés sur le disque depuis le début.

**Les appels passent par votre serveur ?**
Non pour 2 participants. WebRTC connecte les appareils directement entre eux. Le serveur fait uniquement le handshake initial. Pour les appels à 3+ participants, un mode SFU (via rustrtc) relaye le flux depuis le serveur.

**Un membre a oublié son mot de passe ?**
Depuis `/admin` → **Membres** → vous pouvez réinitialiser son mot de passe. Il devra le changer à la prochaine connexion.

**Les notifications ne fonctionnent pas ?**
Assurez-vous d'avoir installé le certificat CA sur votre téléphone (voir la section "Installer le certificat CA" plus haut).

---

## 🔒 Sécurité & Audit

Nook passe régulièrement des audits de sécurité, Docker et dépendances. Derniers scores (2026-04-25) :

- 🔒 **Sécurité** : **92/100** — CSP renforcée, pas de secrets en dur, sanitisation SVG
- 🐳 **Docker** : **92/100** — Images distroless, healthchecks, `.dockerignore`
- 📦 **Dépendances** : **74/100** — `chacha20poly1305` à jour, dépendances inutilisées supprimées

Pour plus de détails, voir le dossier `.claude/` dans le dépôt.

---

## 🏗️ Architecture (pour les curieux)

```
Nook/
├── backend/            Rust + Axum 0.8 — API REST, WebSocket, auth, crypto, SFU
│   ├── src/            18 modules : auth, chat, chess, polls, sfu, webrtc, push, e2ee, admin...
│   ├── .sqlx/          Requêtes SQL pré-compilées pour compilation hors ligne (SQLX_OFFLINE=true)
│   └── migrations/     7 migrations SQLite (users, chess, e2ee, polls, reactions, push, timer)
│
├── frontend/            SvelteKit 5 Runes + TypeScript
│   ├── src/routes/     login, chat, admin, calendar, chess, call, polls, settings...
│   ├── src/lib/        stores Svelte (auth, chat, chess, crypto, push, webrtc...)
│   └── tests/          144 tests Playwright E2E (admin, user, chess, webrtc, calls, API sanit)
│
├── services/            Services additionnels
│   └── turn-rs/        Serveur TURN/STUN pour relais WebRTC multi-appelants (edition 2024)
│
├── .github/workflows/  6 pipelines CI : Backend, Frontend, Docker, Tests, SQLx, Frontend Build
├── Dockerfile           Build depuis les sources (utilisé par les tests CI)
├── Dockerfile.release   Binaires pré-compilés → image Alpine 3.21 (production)
└── docker-compose.yml   Stack de production Alpine (nook + turn-rs)
```

**Ce qui tourne réellement :**
- Un binaire Rust dans une image Alpine — pas de shell, pas d'outils inutiles, surface d'attaque minimale
- Un serveur TURN/STUN (`turn-rs`) pour le relais WebRTC quand la connexion directe échoue — également en Alpine
- Une base SQLite dans le dossier `DATA_DIR`
- Un dossier d'uploads chiffrés, nettoyés automatiquement toutes les 24h

---

## 📚 Documentation

- [HTTPS local](docs/nginx-local.md) — Configuration nginx pour l'enregistrement audio/vidéo en LAN
- [API Reference](docs/API.md) — Tous les endpoints REST + WebSocket events
- [CHANGELOG.md](CHANGELOG.md) — Historique des versions

---

<div align="center">

Pas de pub. Pas de tracking. Pas de numéro de carte bancaire.

Juste votre famille, chez vous.

**🤜🤛**

</div>
