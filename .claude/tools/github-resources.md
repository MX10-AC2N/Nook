# Base de Connaissances GitHub

Ce fichier recense des projets GitHub utiles pour le développement de Nook.

## Projets de Référence

### Visioconférence & Streaming (Rust)

**[videocall.rs](https://github.com/security-union/videocall-rs)** - Framework open-source de streaming média et système de téléconférence écrit en Rust

| Aspect | Détails |
|--------|---------|
| **Latence** | Ultra-basse latence sub-100ms |
| **Transport** | WebTransport (QUIC/HTTP3) + WebSocket fallback |
| **Backend** | Rust + Actix Web + PostgreSQL + NATS |
| **Frontend** | Rust + Dioxus + WebAssembly + Tailwind CSS |
| **Mobile** | Support via Tauri |
| **Cibles** | Robotique, IoT, systèmes embarqués (Raspberry Pi, Jetson Nano) |
| **Licence** | MIT |

**Cas d'usage :**
- Applications vidéo personnalisées avec API Rust type-safe
- Streaming vidéo depuis drones/robots avec latence minimale
- Auto-hébergement de visioconférence avec authentification JWT + SSO

**[rustrtc](https://github.com/restsend/rustrtc)** - Implémentation haute performance de WebRTC écrite en Rust

| Aspect | Détails |
|--------|---------|
| **Performance** | ~2.8x plus rapide que pion (Go) et webrtc-rs |
| **Débits** | 713 MB/s (vs 254 pour webrtc, 309 pour pion) |
| **Latence** | 0.22ms (vs 1.36ms pour webrtc) |
| **Mémoire** | 15 MB (vs 29 MB pour webrtc) |
| **Support** | RTP/SRTP audio/vidéo, ICE/STUN, Data Channels |
| **SFU** | Serveur de visioconférence multi-utilisateurs inclus |
| **Licence** | MIT |

**Cas d'usage :**
- Applications WebRTC haute performance
- Serveur SFU pour conférences vidéo multi-utilisateurs
- Communication peer-to-peer avec data channels

**[turn-rs](https://github.com/mycrl/turn-rs)** - Serveur TURN/STUN pur Rust pour la traverse NAT

| Aspect | Détails |
|--------|---------|
| **Performance** | 40M messages/seconde, 600K allocations/seconde |
| **Latence** | < 35 microsecondes |
| **Transport** | TCP + UDP, multi-interface réseau |
| **API** | gRPC pour contrôle externe et notifications |
| **RFC** | Support RFC 3489, 5389, 5766, 6062, 6156 |
| **IoT** | Fonctionne sur Raspberry Pi 4 |
| **Licence** | MIT |

**Cas d'usage :**
- Traverse NAT pour WebRTC
- Serveur de relai haute performance pour VoIP
- Passerelle pour trafic média temps réel

### Messagerie & Collaboration (Rust)

**[rustchat](https://github.com/rustchatio/rustchat)** - Plateforme de collaboration d'équipe auto-hébergée. Alternative à Slack, Mattermost et Zulip

| Aspect | Détails |
|--------|---------|
| **Backend** | Rust + Axum + Tokio + SQLx |
| **Frontend** | Vue 3 + TypeScript + Pinia |
| **Base de données** | PostgreSQL + Redis |
| **Stockage** | S3-compatible (MinIO) |
| **API** | Native `/api/v1` + Compatibilité Mattermost `/api/v4` |
| **Temps réel** | WebSocket + Appels intégré |
| **Sécurité** | Authentification JWT, Argon2id hashing, Rate limiting |
| **Licence** | MIT |

**Cas d'usage :**
- Système de chat d'équipe auto-hébergé avec compatibilité Mattermost
- Plateforme de collaboration temps réel avec WebSocket
- Backend Rust haute performance pour messagerie

### Échecs (Rust)

**[chess-tui](https://github.com/thomas-mauran/chess-tui)** - Jeu d'échecs en terminal, multiplateforme, écrit en Rust

| Aspect | Détails |
|--------|---------|
| **Modes** | Local 2 joueurs, moteur UCI, Lichess en ligne |
| **UI** | TUI via ratatui |
| **Moteurs** | Compatible tout moteur UCI (Stockfish, etc.) |
| **Multijoueur** | Partie en réseau local ou internet |
| **Personnalisation** | Skins, configuration moteur, sons |
| **Stats** | 989 étoiles, 60 forks |
| **Licence** | MIT |

**[Walleye](https://github.com/MitchelPaulin/Walleye)** - Moteur d'échecs UCI écrit en Rust

| Aspect | Détails |
|--------|---------|
| **Protocole** | UCI compatible |
| **Algorithme** | Alpha-Beta pruning, Iterative Deepening |
| **Optimisations** | Killer Moves, MVV-LVA, PV Search |
| **Plateau** | Square Centric 12x12 avec sentinelles |
| **Tests** | Suite complète de tests unitaires |
| **Déploiement** | AWS + Lichess (@Walleye_Bot) |
| **Licence** | MIT |

**Cas d'usage :**
- Intégration de jeu d'échecs dans vos applications
- Recherche en intelligence artificielle pour jeux de plateau
- Backend de plateforme d'échecs en ligne

## Framework & Libraries

### Frontend
- [React](https://github.com/facebook/react) - Bibliothèque JavaScript pour construire des interfaces utilisateurs
- [Vue.js](https://github.com/vuejs/vue) - Framework JavaScript progressif
- [Tailwind CSS](https://github.com/tailwindlabs/tailwindcss) - Framework CSS utilitaire

### Backend
- [Express](https://github.com/expressjs/express) - Framework web minimal pour Node.js
- [FastAPI](https://github.com/tiangolo/fastapi) - Framework Python moderne et rapide
- [Django](https://github.com/django/django) - Framework web Python de haut niveau

### Base de données
- [PostgreSQL](https://github.com/postgres/postgres) - Système de base de données relationnelle objet
- [Prisma](https://github.com/prisma/prisma) - Next-generation ORM pour Node.js et TypeScript
- [Redis](https://github.com/redis/redis) - Base de données en mémoire

## Outils de Développement

### CLI & Automation
- [yabc](https://github.com/) - Outil de build
- [webpack](https://github.com/webpack/webpack) - Bundler de modules
- [vite](https://github.com/vitejs/vite) - Outil de build nouvelle génération

### Testing
- [Jest](https://github.com/jestjs/jest) - Framework de test JavaScript délicieux
- [Playwright](https://github.com/microsoft/playwright) - Framework de test end-to-end
- [pytest](https://github.com/pytest-dev/pytest) - Framework de test Python mature

### Documentation
- [docsify](https://github.com/docsifyjs/docsify) - Générateur de documentation léger
- [Docusaurus](https://github.com/facebook/docusaurus) - Framework de documentation optimisé

## Ressources d'Apprentissage

### Awesome Lists
- [awesome-python](https://github.com/vinta/awesome-python)
- [awesome-javascript](https://github.com/sorrycc/awesome-javascript)
- [awesome-nodejs](https://github.com/sindresorhus/awesome-nodejs)
- [awesome-rust](https://github.com/rust-unofficial/awesome-rust)

## Notes

- Last updated: 2026-03-29
- Projets ajoutés: videocall-rs, rustchat, rustrtc, turn-rs, chess-tui, Walleye
- À reviser régulièrement pour ajouter de nouvelles ressources pertinentes
