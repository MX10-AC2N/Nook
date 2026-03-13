![Logo animé du projet](frontend/static/logo-animated.svg)

> **v0.4.0-beta.1 — Work in progress**

<div align="center">

[![CI Backend](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml)
[![CI Frontend](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml)
[![CI Tests](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml)
[![Docker Build](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml)

[![Docker Image Version](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/latest_tag?color=blue&label=version&trim=&ignore=sha-*,latest)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Docker Image Size](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/size?color=green&label=image%20size&tag=v0.4.0-beta.1)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Last Commit](https://img.shields.io/github/last-commit/MX10-AC2N/Nook/main?label=dernier%20commit&color=informational)](https://github.com/MX10-AC2N/Nook/commits/main)
[![Platforms](https://img.shields.io/badge/platforms-amd64%20%7C%20arm64-lightgrey)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)

[![Rust](https://img.shields.io/badge/Backend-Rust%201.88%20%2B%20Axum%200.8-orange?logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/Frontend-SvelteKit%202.49%20%2B%20Svelte%205%20Runes-FF3E00?logo=svelte)](https://kit.svelte.dev/)
[![SQLite](https://img.shields.io/badge/DB-SQLite-003B57?logo=sqlite)](https://sqlite.org/)
[![Docker](https://img.shields.io/badge/Runtime-Distroless-2496ED?logo=docker)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

</div>

---

Messagerie familiale **self-hosted** — hébergée chez toi, sur ton matériel, sous ton contrôle. Tes données ne quittent jamais ton serveur.

---

## Ce que Nook sait faire aujourd'hui

### ✅ Fonctionnel

**💬 Chat familial**
Messages en temps réel dans un groupe global, partage de fichiers et photos (max 50 Mo, TTL 48h), chiffrement XChaCha20-Poly1305 pour les fichiers transférés.

**👨‍👩‍👧 Contrôle total**
Invitations par lien avec expiration, approbation admin obligatoire, gestion complète des comptes. Tu décides qui rejoint ton espace.

**📅 Calendrier partagé**
Création et consultation des événements de la famille, sans passer par Google ou Apple.

**♟️ Échecs en ligne**
Parties entre membres ou contre l'IA (minimax easy/medium/hard) via WebSocket — plateau synchronisé depuis le serveur, coups validés côté backend.

**📊 Sondages**
Création, vote et fermeture directement dans l'app via l'API backend — 1 vote par utilisateur par sondage, modifiable jusqu'à fermeture.

**📞 Appels audio & vidéo**
Interface opérationnelle, signaling WebSocket en place. WebRTC établit une connexion directe entre appareils — le serveur ne voit jamais le flux.
> ⚠️ Stable sur réseau LAN. Pas de serveur TURN configuré — les appels WAN (internet) peuvent échouer selon la topologie réseau.

**🎨 Thèmes personnalisables**
Trois ambiances dans les Paramètres : **🌿 Jardin Secret**, **🚀 Space Hub**, **🏠 Maison Chaleureuse**.

**🔐 Infrastructure E2EE en place**
Génération de clés X25519 à l'inscription, stockage chiffré Argon2id en IndexedDB, distribution des clés publiques via l'API. Les messages texte sont envoyés en clair en attendant l'activation finale.

### 🚧 En cours de développement

| Fonctionnalité | État actuel | Ce qui manque |
|---|---|---|
| **Chiffrement E2E des messages** | Infrastructure complète (clés, API, crypto) | Activation dans l'UI — connecter `e2ee.ts` aux composants chat |
| **Chess temps réel adversaire** | Coups propres joueur validés | Abonnement WS aux coups adverses côté client (DT-02) |
| **Notifications push** | Service Worker présent | Intégration push backend |
| **libsodium perf** | Charge 938 kB WASM synchrone | Dynamic import() pour réduire le délai layout (DT-01) |

---

## Démarrage rapide

**Prérequis** : Docker + Docker Compose — c'est tout.
Architectures supportées : `linux/amd64` · `linux/arm64` (Raspberry Pi 4+, NAS ARM, Zimaboard).

```bash
# 1. Cloner
git clone https://github.com/MX10-AC2N/Nook.git && cd Nook

# 2. Configurer
cp .env.example .env
# Éditer .env : adapter PUBLIC_SITE_URL à l'IP/domaine de ton serveur

# 3. Lancer
docker compose up -d
```

Ouvre `http://ton-serveur:6300` — c'est parti. 🎉

> **Premier lancement** : compte `admin` créé automatiquement avec le mot de passe `changeme2026`.
> Tu seras forcé à le changer à la première connexion.

### Mise à jour

```bash
docker compose pull && docker compose up -d
```

---

## Configuration

Toute la configuration passe par le fichier `.env` à la racine du projet.
Le fichier `.env.example` contient toutes les variables disponibles avec leurs valeurs par défaut et leur documentation.

| Variable | Défaut | Description |
|----------|--------|-------------|
| `PUBLIC_SITE_URL` | `http://localhost:6300` | URL principale — automatiquement autorisée en CORS |
| `ALLOWED_ORIGINS` | *(vide)* | Origines CORS supplémentaires, séparées par des virgules |
| `HOST_PORT` | `6300` | Port exposé sur l'hôte |
| `DATA_DIR` | `./data` | Chemin vers le dossier de données persistantes (base SQLite, uploads) |
| `LOGS_DIR` | `./logs` | Chemin vers le dossier de logs |
| `TZ` | `Europe/Paris` | Fuseau horaire |
| `RUST_LOG` | `info` | Niveau de logs (`debug`, `info`, `warn`, `error`) |

> **Homeserver avec chemins absolus** (Zimaboard, NAS, etc.) : définir `DATA_DIR` et `LOGS_DIR` avec les chemins complets vers tes volumes persistants.
> Le fichier `.env` doit rester sur ton serveur uniquement — ne jamais le committer.

---

## Accès depuis l'extérieur (reverse proxy)

Pour un accès HTTPS depuis internet, place Nook derrière un reverse proxy.
Compatible avec **Nginx Proxy Manager**, **Caddy**, **Traefik**.

```
https://nook.ta-famille.fr  →  http://localhost:6300
```

Ajouter ton domaine dans `PUBLIC_SITE_URL` (et dans `ALLOWED_ORIGINS` si différent).

> **WebSocket obligatoire** : activer "Websockets Support" dans Nginx Proxy Manager (nécessaire pour `/ws` — chess + appels).

---

## Architecture

```
Nook/
├── backend/            # 🦀 Rust 1.88 + Axum 0.8 — API REST + WebSocket
│   ├── src/
│   │   ├── main.rs     # Router, middleware, initialisation DB
│   │   ├── auth.rs     # Inscription / Connexion / Déconnexion / Changement mdp
│   │   ├── db.rs       # Conversations, messages, événements calendrier
│   │   ├── admin.rs    # Gestion des membres, approbation
│   │   ├── webrtc.rs   # Signaling WebSocket (chess + appels)
│   │   ├── upload.rs   # Upload fichiers (50 Mo max, TTL 48h, chiffré)
│   │   ├── prune.rs    # Nettoyage automatique DB (toutes les 24h)
│   │   └── config.rs   # Configuration depuis variables d'environnement
│   └── migrations/     # SQLite via SQLx (mode offline pour CI/Docker)
│
├── frontend/           # 🎨 SvelteKit 2.49 + Svelte 5 Runes + TypeScript strict
│   ├── src/routes/     # login, chat, admin, calendar, chess, polls, call, settings…
│   ├── src/lib/        # Stores Svelte 5 Runes (auth, chat, chess, webrtc…)
│   └── tests/          # Suite Playwright E2E (38 tests)
│
├── Dockerfile          # Build depuis les sources (CI test-nook.yml)
├── Dockerfile.release  # Binaires pré-compilés → image distroless (prod)
├── docker-compose.yml  # Stack production
└── .env.example        # Template de configuration
```

### Stack technique

| Composant | Technologie |
|-----------|-------------|
| Backend | Rust 1.88 · Axum 0.8 · SQLx 0.8.6 · SQLite |
| Auth | Argon2id · Cookie HttpOnly · token révocable en base |
| Chiffrement fichiers | XChaCha20-Poly1305 |
| Chiffrement messages | X25519 + XSalsa20 (infrastructure prête, activation en cours) |
| Frontend | SvelteKit 2.49 · Svelte 5.46 Runes · TypeScript strict |
| Temps réel | WebSocket (signaling chess + appels WebRTC + chat) |
| Image runtime | `gcr.io/distroless/cc-debian12` — binaire Rust + libs système uniquement |
| Tests E2E | Playwright · 38 tests · cargo clippy `-D warnings` |

L'image finale ne contient ni shell ni outils système — surface d'attaque minimale.

---

## Pipeline CI/CD

Tous les workflows se déclenchent manuellement depuis GitHub Actions.

```
1. Backend.yml  ──┐
                  ├──▶  2. test-nook.yml  ──▶  3. Docker.yml  ──▶  GHCR
   Frontend.yml ──┘        (stack + E2E)          (multi-arch)
```

| Workflow | Rôle |
|----------|------|
| `Backend.yml` | Compile Rust `amd64` + `arm64` — check, clippy, build release → artifacts 7j |
| `Frontend.yml` | Build SvelteKit → artifact 7j |
| `test-nook.yml` | Stack Docker complète + 38 tests Playwright E2E |
| `Docker.yml` | Assemble les artifacts → image multi-arch → GHCR |
| `Release.yml` | Bump version (`VERSION`, `Cargo.toml`, `package.json`) + tag git |

---

## FAQ

**Mes données sont où ?**
Dans le dossier défini par `DATA_DIR` dans ton `.env`, sur ta machine. Rien ne quitte ton serveur.

**Je peux l'utiliser sur mon NAS ou Raspberry Pi ?**
Oui. L'image supporte nativement `linux/arm64` — Raspberry Pi 4+, NAS Synology/QNAP/TrueNAS, Zimaboard.

**Comment ajouter un membre de la famille ?**
Depuis `/admin`, générer un lien d'invitation. La personne s'inscrit, l'admin approuve le compte.

**Les appels passent par ton serveur ?**
Non. WebRTC connecte les appareils directement entre eux (P2P). Le serveur assure uniquement le handshake initial.

**Le chiffrement est activé ?**
Les fichiers partagés sont chiffrés (XChaCha20-Poly1305). L'infrastructure de chiffrement E2E des messages texte est complète (clés X25519 générées à l'inscription, stockées chiffrées) — l'activation dans l'UI est la dernière étape.

---

## Besoin d'aide ?

[Ouvre une issue](https://github.com/MX10-AC2N/Nook/issues) sur GitHub.

---

<div align="center">

Pas de pub. Pas de tracking. Pas de revente de données.
Juste ta famille, chez toi.

**🤜🤛**

</div>
