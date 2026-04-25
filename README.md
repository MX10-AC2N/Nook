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

**Nook, c'est une messagerie pour ta famille — hébergée chez toi, sur ta machine.**

Pas de compte Google. Pas d'abonnement. Pas de données envoyées ailleurs.
Juste un serveur qui tourne dans ton réseau, accessible depuis n'importe quel appareil.

---

## Ce que tu peux faire avec Nook

**💬 Discuter en famille**
Messages en temps réel, réactions emoji, partage de photos et fichiers (jusqu'à 50 Mo).
Un groupe global pour tout le monde, et la possibilité de créer des conversations privées.

**🔐 Sans compromis sur la vie privée**
Les messages sont chiffrés de bout en bout (X25519). Les fichiers sont chiffrés sur le disque (XChaCha20). Les mots de passe ne sont jamais stockés en clair (Argon2id). Même toi, en tant qu'admin, tu ne peux pas les lire.

**🔔 Notifications push**
Reçois une notification sur ton téléphone ou ordinateur quand quelqu'un t'écrit — même quand l'onglet est fermé. Activable depuis les Paramètres.

**📅 Calendrier partagé**
Tous les événements de la famille au même endroit, sans passer par Google ou Apple.

**♟️ Échecs en ligne**
Parties entre membres ou contre l'IA (easy / medium / hard). Les coups de l'adversaire arrivent en temps réel via WebSocket.

**📊 Sondages**
Crée un vote en quelques secondes. Chacun répond, peut changer d'avis, et tu clos quand tu veux.

**📞 Appels audio & vidéo**
Connexion directe entre appareils (WebRTC P2P) — le serveur ne voit jamais le flux. Fonctionne bien en LAN.

**🎨 Trois thèmes**
Jardin Secret 🌿 · Space Hub 🚀 · Maison Chaleureuse 🏠 — avec mode sombre en option.

---

## Démarrage en 3 commandes

**Prérequis** : Docker + Docker Compose. C'est tout.
Fonctionne sur `linux/amd64` et `linux/arm64` (Raspberry Pi 4+, Zimaboard, NAS ARM).

```bash
git clone https://github.com/MX10-AC2N/Nook.git && cd Nook
cp .env.example .env          # édite PUBLIC_SITE_URL avec l'IP de ton serveur
docker compose up -d
```

Ouvre `http://ton-serveur:6300` dans un navigateur. C'est prêt. 🎉

> **Premier lancement** — un compte `admin` est créé automatiquement avec le mot de passe `changeme2026`.
> Tu seras forcé à le changer à la première connexion.

**Mise à jour :**
```bash
docker compose pull && docker compose up -d
```

---

## Configuration

Tout se passe dans le fichier `.env` — le `.env.example` contient toutes les variables documentées.

Les essentielles :

| Variable | Ce qu'elle fait |
|----------|----------------|
| `PUBLIC_SITE_URL` | L'URL depuis laquelle tu accèdes à Nook (`http://192.168.1.x:6300` ou ton domaine) |
| `ALLOWED_ORIGINS` | Si tu accèdes depuis plusieurs URLs (LAN + domaine externe), liste-les ici séparées par des virgules |
| `DATA_DIR` | Où stocker la base de données et les fichiers uploadés (utilise un chemin absolu sur un vrai serveur) |
| `VAPID_PRIVATE_KEY` | Pour les notifications push — générer avec `npx web-push generate-vapid-keys` |
| `VAPID_PUBLIC_KEY` | Idem — les deux vont ensemble |
| `GIPHY_API_KEY` | Pour les GIFs — clé SDK gratuite sur [developers.giphy.com](https://developers.giphy.com) |

> Le fichier `.env` reste sur ton serveur. Ne le committe jamais dans git.

---

## Accès depuis internet (optionnel)

Tu veux accéder à Nook depuis l'extérieur de ton réseau ? Place-le derrière un reverse proxy.

Compatible avec **Nginx Proxy Manager**, **Caddy**, **Traefik**.

```
https://nook.ta-famille.fr  →  http://localhost:6300
```

Deux choses importantes :
- Ajoute ton domaine dans `PUBLIC_SITE_URL` (et `ALLOWED_ORIGINS` si différent)
- Active le support WebSocket dans ton proxy (`/ws` est utilisé pour le chess et les appels)

---

## Comment inviter quelqu'un

1. Connecte-toi avec le compte `admin`
2. Va dans `/admin` → onglet **Invitations**
3. Génère un lien — il expire dans 48h et ne fonctionne qu'une fois
4. Envoie ce lien à la personne par SMS, email, ou en main propre
5. Elle crée son compte → tu l'approuves dans l'onglet **Membres en attente**

---

## GIFs — Mise à jour hebdomadaire automatique

Les GIFs sont stockés dans le volume de données (`DATA_DIR/gifs/`) et servis directement par Nook — aucune requête externe n'est envoyée quand un membre envoie un GIF.

**Première installation** : le dossier est vide. Lance le script une première fois pour peupler la collection :

```bash
# Sur la Zimaboard (dans le dossier Nook)
GIPHY_API_KEY=ta_clé bash scripts/update-gifs.sh
```

**Mise à jour automatique** — ajoute un cron sur la Zimaboard :

```bash
crontab -e
# Ajouter cette ligne (lundi à 3h du matin) :
0 3 * * 1 cd /chemin/vers/Nook && bash scripts/update-gifs.sh >> logs/gifs-update.log 2>&1
```

Le script télécharge ~10 GIFs pour chacun des 12 thèmes les plus populaires Giphy (réactions, humour, animaux, fête, anniversaire…). Aucun rebuild Docker nécessaire — les GIFs sont servis directement depuis le volume.

---

## Architecture (pour les curieux)

```
Nook/
├── backend/          Rust + Axum 0.8 — API REST, WebSocket, auth, crypto
│   ├── src/          auth, chat, chess, polls, upload, push, e2ee...
│   └── migrations/   6 migrations SQLite (users, conversations, chess, push...)
│
├── frontend/         SvelteKit 5 Runes + TypeScript
│   ├── src/routes/   login, chat, admin, calendar, chess, polls, settings...
│   ├── src/lib/      stores Svelte (auth, chat, chess, crypto, push...)
│   └── tests/        115 tests Playwright E2E — tous verts ✅
│
├── Dockerfile         Build depuis les sources (utilisé par les tests CI)
├── Dockerfile.release Binaires pré-compilés → image distroless (production)
└── docker-compose.yml Stack de production
```

**Ce qui tourne réellement :**
- Un binaire Rust dans une image [distroless](https://github.com/GoogleContainerTools/distroless) — pas de shell, pas d'outils inutiles, surface d'attaque minimale
- Une base SQLite dans le dossier `DATA_DIR`
- Un dossier d'uploads chiffrés, nettoyés automatiquement toutes les 24h

---

## Sécurité & Audit

Nook passe régulièrement des audits de sécurité, Docker et dépendances. Derniers scores (2026-04-25) :

- 🔒 Sécurité : **92/100** — CSP renforcée, pas de secrets en dur, sanitisation SVG
- 🐳 Docker : **92/100** — Images distroless, healthchecks, `.dockerignore`
- 📦 Dépendances : **74/100** — `chacha20poly1305` à jour, dépendances inutilisées supprimées

Pour plus de détails, voir le dossier `.claude/` dans le dépôt.

---

## Questions fréquentes

**Mes données sont où ?**
Dans le dossier `DATA_DIR` sur ta machine. Rien ne sort de chez toi.

**Ça tourne sur Raspberry Pi ?**
Oui. L'image est compilée nativement pour `arm64` — Raspberry Pi 4+, Zimaboard, NAS Synology/QNAP/TrueNAS.

|**Le chiffrement est vraiment activé ?**
|Oui depuis la v0.5.0 : les clés X25519 sont générées à la première connexion de chaque membre et stockées chiffrées sur leur appareil. Les fichiers partagés sont chiffrés sur le disque depuis le début.

**Les appels passent par ton serveur ?**
Non. WebRTC connecte les appareils directement entre eux. Le serveur fait uniquement le handshake initial.

**Un membre a oublié son mot de passe ?**
Depuis `/admin` → **Membres** → tu peux réinitialiser son mot de passe. Il devra le changer à la prochaine connexion.

---

<div align="center">

Pas de pub. Pas de tracking. Pas de numéro de carte bancaire.
Juste ta famille, chez toi.

**🤜🤛**

</div>
