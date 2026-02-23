# Nook — La messagerie qui protège ta famille 🏠

> ⚠️ **WORK IN PROGRESS** — projet en développement actif

<div align="center">

[![CI Backend](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml)
[![CI Frontend](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml)
[![CI Tests](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml)
[![Docker Build](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml)

[![Docker Image Version](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/latest_tag?color=blue&label=version&trim=&ignore=sha-*,latest)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Docker Image Size](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/size?color=green&label=image%20size&tag=latest)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Platforms](https://img.shields.io/badge/platforms-amd64%20%7C%20arm64-lightgrey)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)

[![Rust](https://img.shields.io/badge/Backend-Rust%201.88%20%2B%20Axum%200.8-orange?logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/Frontend-SvelteKit%205%20Runes-FF3E00?logo=svelte)](https://kit.svelte.dev/)
[![SQLite](https://img.shields.io/badge/DB-SQLite-003B57?logo=sqlite)](https://sqlite.org/)
[![Docker](https://img.shields.io/badge/Runtime-Distroless%20Docker-2496ED?logo=docker)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

</div>

---

## Pourquoi Nook ?

Tu en as marre que les grandes firmes lisent tes messages et vendent tes données ? Nous aussi. 😤

Nook, c'est simple :
- **Toi** qui héberges — sur ton serveur, ton NAS, ou un Raspberry Pi
- **Tes données** qui restent chez toi — elles ne vont nulle part ailleurs
- **Chiffré** de bout en bout — même l'hébergeur ne peut pas lire tes messages

---

## Ce que Nook sait faire

### 💬 Messagerie privée
Conversations textuelles instantanées et chiffrées. Partage de fichiers, photos, documents — tout reste entre vous.

### 📞 Appels audio & vidéo
WebRTC pour des connexions directes entre appareils. Le serveur établit la connexion mais ne voit jamais le flux — zéro donnée transit via Nook.

### 📅 Calendrier familial
Organisez vos événements partagés sans passer par Google ou Apple.

### 📊 Sondages & votes
Décidez ensemble où partir en vacances, quoi manger le soir — sans quitter l'app.

### 🔐 Chiffrement de bout en bout
XChaCha20-Poly1305 pour les fichiers. Tes clés sont générées chez toi, stockées chez toi. Comme un coffre-fort dont seul toi as la clé.

### 👨‍👩‍👧 Gestion familiale
Système d'invitation par lien, approbation admin, gestion des comptes. Tu contrôles qui rejoint ton espace.

---

## Démarrer en 3 étapes

### 1. Récupérer le projet

```bash
git clone https://github.com/MX10-AC2N/Nook.git
cd Nook
```

### 2. Lancer avec Docker

```bash
docker compose up -d
```

Ou directement depuis l'image GHCR (sans compiler) :

```bash
docker run -d \
  -p 6300:3000 \
  -v ./data:/app/data \
  -e PUBLIC_SITE_URL=http://ton-serveur:6300 \
  ghcr.io/mx10-ac2n/nook:latest
```

### 3. Ouvrir ton navigateur

Va sur `http://localhost:6300` — c'est parti ! 🎉

> **Premier lancement** : un compte `admin` est créé automatiquement avec le mot de passe `changeme2026`.  
> **Change-le immédiatement** lors de la première connexion.

---

## Configuration

Variables d'environnement dans `docker-compose.yml` :

| Variable | Défaut | Description |
|----------|--------|-------------|
| `PUBLIC_SITE_URL` | `http://localhost:6300` | URL publique de l'instance |
| `DATABASE_URL` | `sqlite:/app/data/nook.db` | Chemin vers la DB SQLite |
| `UPLOADS_DIR` | `/app/data/uploads` | Stockage des fichiers |
| `RUST_LOG` | `info` | Niveau de logs (`debug`, `info`, `warn`) |
| `TZ` | `Europe/Paris` | Fuseau horaire |

---

## Ce qu'il faut

- **Docker** et **Docker Compose** — c'est tout
- Un navigateur récent (Chrome, Firefox, Safari, Edge)
- **Architectures supportées** : `linux/amd64` (PC/serveur) · `linux/arm64` (Raspberry Pi 4+, NAS ARM)

---

## Architecture technique

```
Nook/
├── backend/               # 🦀 Rust 1.88 + Axum 0.8 — API REST + WebSocket
├── frontend/              # 🎨 SvelteKit 5 Runes — UI réactive TypeScript
├── Dockerfile             # 🔧 Build depuis sources (dev + CI tests)
├── Dockerfile.release     # 🚀 Image finale distroless (production)
├── docker-compose.yml     # 📦 Stack production
└── docker-compose.ci.yml  # 🧪 Override CI (tests E2E automatisés)
```

### Stack

| Composant | Technologie |
|-----------|------------|
| Backend | Rust 1.88 · Axum 0.8 · SQLx 0.8 · SQLite |
| Auth | Argon2id (hash mdp) · Cookie HttpOnly · XChaCha20-Poly1305 (fichiers) |
| Frontend | SvelteKit 5 · Svelte Runes · TypeScript strict |
| Runtime | Docker Distroless `gcr.io/distroless/cc-debian12:nonroot` |
| Tests | Playwright E2E · cargo test · cargo clippy |

### Pourquoi Distroless ?

L'image finale ne contient **que le strict nécessaire** : le binaire Rust compilé + les libs système. Pas de shell, pas d'outils, pas de surface d'attaque inutile. Résultat : ~10 MB et une sécurité maximale.

---

## Pipeline CI/CD

```
Backend.yml  ──┐
               ├──▶ test-nook.yml ──▶ Docker.yml ──▶ GHCR
Frontend.yml ──┘   (Docker + E2E)
```

| Workflow | Rôle |
|----------|------|
| `Backend.yml` | Compile Rust pour `amd64` + `arm64` → artifacts |
| `Frontend.yml` | Build SvelteKit → artifact |
| `test-nook.yml` | Intégration Docker + tests API + Playwright E2E |
| `Docker.yml` | Assemble les artifacts → image distroless → GHCR |
| `Release.yml` | Bump version sémantique + tag git |

**Ordre de lancement** : `Backend.yml` → `Frontend.yml` → `test-nook.yml` → `Docker.yml`

---

## FAQ

**"C'est compliqué à installer ?"**  
Pas du tout. Docker s'occupe de tout. Une commande, c'est lancé.

**"Mes données sont où ?"**  
Sur ta machine, dans le dossier `./data/`. Rien ne part sur un serveur tiers.

**"Je peux l'utiliser sur mon NAS ?"**  
Oui ! NAS ARM (Synology, QNAP, TrueNAS) et Raspberry Pi 4+ sont supportés nativement grâce au build `arm64`.

**"Les appels vidéo passent par votre serveur ?"**  
Non. WebRTC établit une connexion directe entre les appareils. Le serveur ne voit jamais le flux audio/vidéo.

**"Comment ajouter des membres ?"**  
L'admin génère un lien d'invitation depuis le panneau d'administration. La personne crée son compte, l'admin l'approuve.

---

## Besoin d'aide ?

Tu bloques ? [Ouvre une issue](https://github.com/MX10-AC2N/Nook/issues) sur GitHub !

---

<div align="center">

Nook, c'est un projet construit avec l'envie de proposer une alternative aux géants du web.  
Pas de pub, pas de tracking, pas de revente de données.  
Juste un outil pour que ta famille communique en toute sérénité.

**Amuse-toi bien ! 🎈**

🤜🤛

</div>
