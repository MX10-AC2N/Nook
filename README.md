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
[![Last Commit](https://img.shields.io/github/last-commit/MX10-AC2N/Nook/main?label=dernier%20commit&color=informational)](https://github.com/MX10-AC2N/Nook/commits/main)

[![Rust](https://img.shields.io/badge/Backend-Rust%20+%20Axum%200.8-orange?logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/Frontend-SvelteKit%205%20Runes-FF3E00?logo=svelte)](https://kit.svelte.dev/)
[![SQLite](https://img.shields.io/badge/DB-SQLite-003B57?logo=sqlite)](https://sqlite.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

</div>

---

## Bonjour ! 👋

**Nook, c'est notre messagerie de famille.**

Pas de compte à créer chez Google ou Meta. Pas d'abonnement à payer. Pas de données qui partent je-ne-sais-où.

Juste un petit serveur qui tourne chez toi, sur ta machine, et que ta famille peut utiliser depuis n'importe quel appareil — téléphone, tablette, ordinateur.

C'est comme avoir son propre WhatsApp, mais en mieux, parce que c'est le vôtre.

---

## Ce qu'on peut faire avec Nook

**💬 Discuter en famille**
Envoyez des messages en temps réel, réagissez avec des emojis, partagez des photos et des fichiers. Il y a un groupe global pour tout le monde, et vous pouvez aussi créer des conversations privées.

**🔐 Sans compromis sur la vie privée**
Les messages sont chiffrés de bout en bout (X25519). Les fichiers (jusqu'à 50 Mo) sont chiffrés sur le disque (XChaCha20). Les fichiers volumineux (>50 Mo) sont transférés directement entre appareils via WebRTC (P2P) avec chiffrement E2EE. Les mots de passe ne sont jamais stockés en clair (Argon2id). Même toi, en tant qu'admin, tu ne peux pas les lire.

**🔔 Notifications sur ton téléphone**
Reçois une notification quand quelqu'un t'écrit — même quand l'onglet est fermé. Ça s'active depuis les Paramètres de Nook.

**📅 Calendrier partagé**
Tous les événements de la famille au même endroit. Anniversaires, rendez-vous, sorties... Glisser-déposer pour déplacer les événements.

**♟️ Parties d'échecs**
Jouez entre membres ou contre l'IA (5 niveaux). Animations des pièces, coups spéciaux, minuteur configurable.

**📊 Sondages**
"Qu'est-ce qu'on mange ce soir ?", "Qui vient dimanche ?". Créez un vote en quelques secondes.

**📞 Appels audio & vidéo**
Connexion directe entre appareils (WebRTC). Le serveur ne voit jamais le flux. Fonctionne bien en LAN.

**🎨 Trois thèmes**
Jardin Secret 🌿 · Space Hub 🚀 · Maison Chaleureuse 🏠 — avec mode sombre en option.

---

## Installation rapide

**Ce qu'il faut** : Docker + Docker Compose. C'est tout.
Ça fonctionne sur `linux/amd64` et `linux/arm64` (Raspberry Pi 4+, Zimaboard, NAS).

```bash
git clone https://github.com/MX10-AC2N/Nook.git && cd Nook
cp .env.example .env          # édite PUBLIC_SITE_URL avec l'IP de ton serveur
docker compose up -d
```

Ouvre `http://ton-serveur:6300` dans un navigateur. C'est prêt ! 🎉

> **Premier lancement** — un compte `admin` est créé automatiquement avec le mot de passe `changeme2026`.
> Tu seras forcé à le changer à la première connexion.

**Mise à jour :**
```bash
docker compose pull && docker compose up -d
```

---

## ⚠️ Installation du certificat CA (pour les notifications)

Pour que les notifications push fonctionnent sur ton téléphone, il faut installer un petit certificat. C'est normal et sécurisé — c'est le certificat de ton propre serveur.

**Comment faire :**

1. **Ouvre** `http://ton-serveur:6300/ca/help` dans ton navigateur
2. **Télécharge** le certificat via le bouton
3. **Installe-le** sur ton téléphone :
   - **Android** : Paramètres → Sécurité → Certificats → Installer depuis le stockage
   - **Samsung** : Paramètres → Biométrie et sécurité → Autres paramètres → Certificats → Installer
   - **iPhone** : Réglages → Général → VPN et gestion de l'appareil → Installer
4. **Redémarre** ton navigateur
5. **Active** les notifications dans Nook

> Le certificat est valide 10 ans. Tu n'auras plus jamais à y toucher.

---

## Configuration

Tout se passe dans le fichier `.env` — le `.env.example` contient toutes les variables documentées.

Les essentielles :

| Variable | Ce qu'elle fait |
|----------|----------------|
| `PUBLIC_SITE_URL` | L'URL depuis laquelle tu accèdes à Nook (`http://192.168.1.x:6300` ou ton domaine) |
| `ALLOWED_ORIGINS` | Si tu accèdes depuis plusieurs URLs (LAN + domaine externe), liste-les ici |
| `DATA_DIR` | Où stocker la base de données et les fichiers uploadés |
| `VAPID_PRIVATE_KEY` | Pour les notifications push — voir ci-dessous pour générer les clés |
| `VAPID_PUBLIC_KEY` | Idem — les deux vont ensemble |
| `GIPHY_API_KEY` | Pour les GIFs — clé SDK gratuite sur [developers.giphy.com](https://developers.giphy.com) |

> Le fichier `.env` reste sur ton serveur. Ne le committe jamais dans git.

---

## Génération des clés VAPID

Les clés VAPID servent à authentifier les notifications push. Tu n'as besoin de les générer qu'une seule fois.

**Option 1 — Avec OpenSSL (recommandé) :**
```bash
# Générer la clé privée
openssl ecparam -name prime256v1 -genkey -noout -out vapid_private.pem

# Extraire la clé privée en base64url (pour VAPID_PRIVATE_KEY)
openssl ec -in vapid_private.pem -outform DER | tail -c +8 | head -c 32 | base64 -w0 | tr '+/' '-_' | tr -d '='

# Extraire la clé publique en base64url (pour VAPID_PUBLIC_KEY)
openssl ec -in vapid_private.pem -pubout -outform DER | tail -c 65 | base64 -w0 | tr '+/' '-_' | tr -d '='
```

**Option 2 — Avec Node.js (si tu l'as installé) :**
```bash
npx web-push generate-vapid-keys
```

**Option 3 — En ligne :**
Va sur [vapidkeys.com](https://www.vapidkeys.com/) et copie les clés générées.

> Copie les deux clés dans ton `.env` et redémarre Nook avec `docker compose up -d`.

---

## Comment inviter quelqu'un

1. Connecte-toi avec le compte `admin`
2. Va dans `/admin` → onglet **Invitations**
3. Génère un lien — il expire dans 48h et ne fonctionne qu'une fois
4. Envoie ce lien à la personne par SMS, email, ou en main propre
5. Elle crée son compte → tu l'approuves dans l'onglet **Membres en attente**

---

## Accès depuis internet (optionnel)

Tu veux accéder à Nook depuis l'extérieur de ton réseau ? Place-le derrière un reverse proxy.

Compatible avec **Nginx Proxy Manager**, **Caddy**, **Traefik**.

```
https://nook.ta-famille.fr  →  http://localhost:6300
```

Deux choses importantes :
- Ajoute ton domaine dans `PUBLIC_SITE_URL` (et `ALLOWED_ORIGINS` si différent)
- Active le support WebSocket dans ton proxy (`/ws` est utilisé pour les échecs et les appels)

---

## HTTPS local (pour les appels audio/vidéo)

L'enregistrement audio et vidéo dans le navigateur nécessite un **contexte sécurisé** (HTTPS). Sur HTTP LAN, le navigateur bloque l'accès au microphone.

Nook inclut un reverse proxy **nginx local** qui sert HTTPS sur le port 6443 :

```bash
# 1. Créer le dossier des certificats
mkdir -p nginx-ssl

# 2. Démarrer
docker compose up -d
```

Un certificat auto-signé est généré **automatiquement** au premier lancement (valide 10 ans).

```
LAN (HTTPS)                     WAN (HTTPS)
https://192.168.1.x:6443        https://ton-domaine.com
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
Le dossier `nginx-ssl/` est persistant (volume Docker) et ignoré par git.

---

## GIFs — Mise à jour automatique

Les GIFs sont stockés dans le volume de données (`DATA_DIR/gifs/`) et servis directement par Nook — aucune requête externe n'est envoyée quand un membre envoie un GIF.

**Bonne nouvelle** : la mise à jour des GIFs est **automatique**. Le backend lance une tâche au démarrage qui vérifie toutes les 7 jours si de nouveaux GIFs sont disponibles sur Giphy.

**Ce qu'il faut :**
1. Une clé API Giphy (gratuite) dans ton `.env` : `GIPHY_API_KEY=ta-cle-ici`
2. Redémarrer Nook : `docker compose up -d`

Le script télécharge ~10 GIFs pour chacun des 12 thèmes les plus populaires Giphy (réactions, humour, animaux, fête, anniversaire…). Aucun rebuild Docker nécessaire — les GIFs sont servis directement depuis le volume.

> **Note** : Si tu n'as pas de clé Giphy, les GIFs par défaut (inclus dans l'image Docker) seront utilisés.

---

## 📚 Documentation

- [HTTPS local](docs/nginx-local.md) — Configuration nginx pour l'enregistrement audio/vidéo en LAN
- [API Reference](docs/API.md) — Tous les endpoints REST + WebSocket events
- [CHANGELOG.md](CHANGELOG.md) — Historique des versions

---

## Architecture (pour les curieux)

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
│   └── tests/          144 tests Playwright E2E (admin, user, chess, webrtc, calls, API sanité)
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

## Questions fréquentes

**Mes données sont où ?**
Dans le dossier `DATA_DIR` sur ta machine. Rien ne sort de chez toi.

**Ça tourne sur Raspberry Pi ?**
Oui. L'image est compilée nativement pour `arm64` — Raspberry Pi 4+, Zimaboard, NAS Synology/QNAP/TrueNAS.

**Le chiffrement est vraiment activé ?**
Oui depuis la v0.4.0-beta.2 : les clés X25519 sont générées à la première connexion de chaque membre et stockées chiffrées sur leur appareil. Les fichiers partagés sont chiffrés sur le disque depuis le début.

**Les appels passent par ton serveur ?**
Non pour 2 participants. WebRTC connecte les appareils directement entre eux. Le serveur fait uniquement le handshake initial. Pour les appels à 3+ participants, un mode SFU (via rustrtc) relaye le flux depuis le serveur.

**Un membre a oublié son mot de passe ?**
Depuis `/admin` → **Membres** → tu peux réinitialiser son mot de passe. Il devra le changer à la prochaine connexion.

**Les notifications ne fonctionnent pas ?**
Assure-toi d'avoir installé le certificat CA sur ton téléphone (voir la section "Installation du certificat CA" plus haut).

---

<div align="center">

Pas de pub. Pas de tracking. Pas de numéro de carte bancaire.
Juste ta famille, chez toi.

**🤜🤛**

</div>
