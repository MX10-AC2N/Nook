# Nook — La messagerie qui protège ta famille 🏠

> ⚠️ **WORK IN PROGRESS** — projet en développement actif

<div align="center">

<!-- Statut général -->
[![CI Backend](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml)
[![CI Frontend](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml)
[![CI Tests](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/test-nook.yml)
[![Docker Build](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml)

<!-- Image Docker -->
[![Docker Image Version](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/latest_tag?color=blue&label=version&trim=)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Docker Image Size](https://ghcr-badge.egpl.dev/mx10-ac2n/nook/size?color=green&label=image%20size)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)
[![Platforms](https://img.shields.io/badge/platforms-amd64%20%7C%20arm64-lightgrey)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)

<!-- Stack -->
[![Rust](https://img.shields.io/badge/Backend-Rust%20%2B%20Axum-orange?logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/Frontend-SvelteKit%205-FF3E00?logo=svelte)](https://kit.svelte.dev/)
[![SQLite](https://img.shields.io/badge/DB-SQLite-003B57?logo=sqlite)](https://sqlite.org/)
[![Docker](https://img.shields.io/badge/Runtime-Distroless%20Docker-2496ED?logo=docker)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)

<!-- Licence -->
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)
[![Built with love](https://img.shields.io/badge/build%20with%20🫶-for%20family-8A2BE2)](https://github.com/MX10-AC2N/Nook)

</div>

---

## Pourquoi Nook ?

Tu en as marre que les grandes firmes vendent tes données ? Que tes messages servent à alimenter leurs pubs ciblées ? Nous aussi. 😤

Nook, c'est simple :
- C'est **toi** qui héberges l'application (sur ton serveur, ton NAS, ou même un Raspberry Pi)
- C'est **toi** qui contrôles tes données (elles ne vont nulle part ailleurs)
- C'est **chiffré** de bout en bout (même nous on ne peut pas lire tes messages)

---

## Ce que Nook sait faire

### 💬 Discuter en toute liberté
Des conversations textuelles simples, rapides, et surtout privées.

### 📞 S'entendre et se voir
Appels audio et vidéo pour garder le contact avec ceux qu'on aime, même quand ils sont loin.

### 📁 Partager sans complexe
Photos de famille, vidéos des kids, documents importants... Tout ce que tu partages reste entre vous.

### 🔐 Sécurité de ninja
On utilise libsodium + XChaCha20-Poly1305 pour chiffrer tes fichiers. Tes clés sont générées chez toi, stockées chez toi. C'est un peu comme un coffre-fort dont seul toi as la clé.

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

Ou avec l'image depuis GHCR (pas besoin de compiler) :

```bash
docker pull ghcr.io/mx10-ac2n/nook:latest
docker run -d \
  -p 3000:3000 \
  -v nook-data:/app/data \
  ghcr.io/mx10-ac2n/nook:latest
```

### 3. Ouvrir ton navigateur

Va sur `http://localhost:3000` et c'est parti ! 🎉

> **Premier lancement** : un compte `admin` est créé automatiquement avec le mot de passe `changeme2026`. Change-le immédiatement après la première connexion.

---

## Ce qu'il faut

- Docker et Docker Compose (c'est tout !)
- Un navigateur récent (Chrome, Firefox, Safari, Edge... ça marche partout)
- **Architecture supportée** : `linux/amd64` (PC/serveur) · `linux/arm64` (Raspberry Pi 4+, NAS ARM)

---

## Architecture technique

```
Nook/
├── backend/          # 🦀 Rust + Axum  — API REST + WebSocket + crypto
├── frontend/         # 🎨 SvelteKit 5  — UI réactive avec runes
├── Dockerfile        # 🔧 Build depuis sources (dev / tests CI)
├── Dockerfile.release# 🚀 Image finale distroless (production)
└── docker-compose.yml# 📦 Stack complète pour auto-hébergement
```

### Pourquoi Distroless ?

L'image Docker finale ne contient **que le strict nécessaire** : le binaire Rust + les libs système. Pas de shell, pas d'outils, pas de surface d'attaque inutile. Résultat : une image de quelques MB seulement (voir le badge en haut 👆).

---

## Pipeline CI/CD

```
Backend.yml  ──┐
               ├──▶ test-nook.yml ──▶ Docker.yml ──▶ GHCR
Frontend.yml ──┘         │
                    (tests API +
                     Playwright)
```

| Workflow | Rôle |
|----------|------|
| `Backend.yml` | Compile + teste Rust pour `amd64` et `arm64` |
| `Frontend.yml` | Build SvelteKit |
| `test-nook.yml` | Tests d'intégration (API + E2E) |
| `Docker.yml` | Assemble les artifacts → image distroless → GHCR |

---

## FAQ

**"C'est compliqué à installer ?"**
Pas du tout ! Docker s'occupe de tout. Une fois que c'est lancé, tu n'as plus rien à faire.

**"Mes données sont où ?"**
Sur ta machine ! Aucune donnée ne part sur un serveur tiers. C'est le principe de l'auto-hébergement.

**"Je peux l'utiliser sur mon NAS ?"**
Absolument ! NAS ARM (Synology, QNAP, TrueNAS...) et Raspberry Pi 4+ sont supportés nativement grâce au build `arm64`.

**"Les appels vidéo ça consomme beaucoup ?"**
On utilise WebRTC pour des connexions directes entre les appareils. Le serveur ne voit jamais le flux vidéo — il sert juste à établir la connexion.

**"L'image Docker est vraiment petite ?"**
Oui ! Grâce à l'image Distroless de Google, le binaire Rust est lié statiquement et l'image finale ne contient que l'essentiel. Le badge en haut du README affiche la taille réelle.

---

## Besoin d'aide ?

Tu bloques sur quelque chose ? [Ouvre une issue](https://github.com/MX10-AC2N/Nook/issues) sur GitHub, on sera ravis de t'aider !

---

<div align="center">

Nook, c'est un projet construit avec l'envie de proposer une alternative aux géants du web.
Pas de pub, pas de tracking, pas de revente de données.
Juste un outil pour que ta famille puisse communiquer en toute sérénité.

**Amuse-toi bien ! 🎈**

🤜🤛

</div>
