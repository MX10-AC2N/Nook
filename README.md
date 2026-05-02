![Logo Nook](frontend/static/logo-animated.svg)

<div align="center">

### 🏠 La messagerie de votre famille, chez vous.

[![CI Backend](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml)
[![CI Frontend](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml)
[![Docker](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml)

[![Rust](https://img.shields.io/badge/Backend-Rust%20+%20Axum-orange?logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/Frontend-SvelteKit%205%20Runes-FF3E00?logo=svelte)](https://kit.svelte.dev/)
[![MIT](https://img.shields.io/badge/License-MIT-green)](LICENSE)

</div>

---

## 👋 Qu'est-ce que Nook ?

C'est **votre** messagerie familiale. Pas celle de Google, de Meta ou d'un autre géant.

- 🔐 **Vos données** restent chez vous, sur votre serveur
- 💰 **Gratuit** : pas d'abonnement, pas de carte bancaire
- 🏠 **Simple** : un conteneur Docker et c'est prêt
- 📱 **Partout** : téléphone, tablette, ordinateur, tout le monde peut se connecter

---

## ✨ Ce que vous pouvez faire

| Fonctionnalité | Description |
|--------------|-------------|
| 💬 **Messages** | Temps réel, emojis, photos, fichiers. Groupes + privé |
| 🔐 **E2EE** | Chiffrement de bout en bout (X25519). Même l'admin ne peut pas lire |
| 📞 **Appels** | Audio/vidéo WebRTC P2P. Le serveur ne voit jamais le flux |
| 📅 **Calendrier** | Événements familiaux, anniversaires, rendez-vous |
| ♟️ **Échecs** | Jouez contre l'IA (5 niveaux) ou entre membres |
| 📊 **Sondages** | "Qu'est-ce qu'on mange ?" en 3 clics |
| 🎨 **Thèmes** | Jardin Secret 🌿, Space Hub 🚀, Maison 🏠 + mode sombre |

> 💡 **Le chiffrement E2EE** est activé par défaut depuis la v0.5.0. Vos messages sont protégés par X25519 + XChaCha20.

---

## 🚀 Installation (3 étapes)

### 1. Prérequis
Docker + Docker Compose installés sur votre machine (Linux, NAS, Raspberry Pi 4+, Zimaboard).

### 2. Lancer Nook
```bash
git clone https://github.com/MX10-AC2N/Nook.git
cd Nook
cp .env.example .env
docker compose up -d
```

### 3. Ouvrez Nook
**🔒 Recommandé (LAN) :**  
→ `https://votre-IP:6443`  
✅ Audio, vidéo, WebRTC, notifications — **tout fonctionne**

**📋 Basique (LAN) :**  
→ `http://votre-IP:6300`  
⚠️ Limite : pas d'enregistrement audio/vidéo (navigateur bloque)

> 💡 **Première connexion :** Compte `admin` créé auto avec mot de passe `changeme2026`.  
> Vous serez forcé de le changer à la première connexion.

---

## 🔒 Accès HTTPS en LAN (important !)

Nook inclut un **reverse proxy nginx local** sur le port **6443** pour :
- 🎙️ **Appels audio/vidéo** — le navigateur exige HTTPS
- 🔔 **Notifications push** — contexte sécurisé requis
- 📞 **WebRTC P2P** — connexion directe entre appareils

**Certificat auto-signé** généré automatiquement (valide 10 ans).  
Votre navigateur affichera un avertissement la première fois — c'est normal, c'est votre propre certificat.

---

## 📸 L'interface Nook

### 💬 Conversation
![Chat](docs/screenshots/chat.png)  
*Messages chiffrés E2EE, réactions, partage de fichiers*

### 📞 Appels audio/vidéo
![Appels](docs/screenshots/call.png)  
*WebRTC P2P — le serveur ne voit jamais le flux*

### 📅 Calendrier
![Calendrier](docs/screenshots/calendar.png)  
*Événements familiaux, glisser-déposer*

### ♟️ Échecs
![Échecs](docs/screenshots/chess.png)  
*Contre l'IA ou entre membres*

### 📊 Sondages
![Sondages](docs/screenshots/polls.png)  
*Votes rapides en quelques secondes*

### ⚙️ Paramètres
![Paramètres](docs/screenshots/settings.png)  
*Thèmes, notifications, avatar*

---

## 👥 Inviter votre famille (3 étapes)

1. **Connectez-vous** avec le compte `admin`
2. **Allez** dans `/admin` → onglet **Invitations**
3. **Générez** un lien (expire dans 48h, usage unique)
4. **Envoyez** le lien par SMS, email ou en main propre
5. **Approuvez** le nouveau membre dans l'onglet **Membres en attente**

---

## 🔔 Notifications sur téléphone

Pour recevoir des notifications même quand l'onglet est fermé :

### 1. Installer le certificat CA
1. Ouvrez `http://votre-IP:6300/ca/help` (ou via HTTPS)
2. Téléchargez le certificat
3. Installez-le :
   - **Android** : Paramètres → Sécurité → Certificats → Installer
   - **iPhone** : Réglages → Général → VPN → Installer
4. Redémarrez votre navigateur

### 2. Activer dans Nook
Allez dans **Paramètres → Notifications** et activez-les.

> ✅ Le certificat est valide 10 ans. Une fois installé, vous n'y touchez plus.

---

## 🌐 Accès depuis internet (optionnel)

Vous voulez accéder à Nook depuis l'extérieur ?

Placez Nook derrière un **reverse proxy** (Nginx Proxy Manager, Caddy, Traefik) :

```
https://nook.votre-famille.fr  →  http://localhost:6300
```

**Important :**
- Ajoutez votre domaine dans `PUBLIC_SITE_URL` (fichier `.env`)
- Activez le support **WebSocket** (`/ws`) pour les échecs et appels

---

## 🎁 GIFs animés (automatique)

Les GIFs sont servis **depuis votre serveur** — aucune requête vers Giphy !

- ✅ **Mise à jour auto** toutes les 7 jours
- ✅ **12 thèmes** populaires (réactions, animaux, fête...)
- ✅ **Gratuit** : une clé API Giphy dans `.env` (optionnel)

> 💡 Pas de clé ? Les GIFs par défaut sont inclus dans l'image Docker.

---

## ❓ Questions fréquentes

**🏠 Ça tourne sur Raspberry Pi ?**  
Oui ! Image compilée nativement pour `arm64` (Pi 4+, Zimaboard, NAS).

**🔐 Le chiffrement est vraiment activé ?**  
Oui depuis v0.5.0. Chaque membre a une clé X25519 générée sur son appareil.

**📞 Les appels passent par mon serveur ?**  
Non pour 2 personnes (WebRTC P2P direct). Oui pour 3+ (relais SFU).

**🔑 J'ai oublié mon mot de passe ?**  
Connectez-vous en `admin` → `/admin` → **Membres** → Réinitialiser.

**📱 Les notifications ne marchent pas ?**  
Vérifiez que le **certificat CA** est installé sur votre téléphone.

---

## 🔒 Sécurité & Audit

Derniers scores (2026-04-25) :
- 🔒 **Sécurité** : 92/100 — CSP renforcée, pas de secrets en dur
- 🐳 **Docker** : 92/100 — Images distroless, healthchecks
- 📦 **Dépendances** : 74/100 — `chacha20poly1305` à jour

> 📂 Détails disponibles dans le dossier `.hermes/` du dépôt.

---

## ⚙️ Configuration avancée

Tout se configure dans le fichier `.env` (basé sur `.env.example`) :

| Variable | Usage |
|----------|-------|
| `PUBLIC_SITE_URL` | Votre URL d'accès (ex: `https://192.168.1.50:6443`) |
| `ALLOWED_ORIGINS` | URLs multiples séparées par virgule |
| `DATA_DIR` | Où stocker la base et les fichiers |
| `VAPID_PRIVATE_KEY` | Notifications push (générer une fois, voir ci-dessous) |
| `GIPHY_API_KEY` | GIFs (gratuit sur developers.giphy.com) |

### Générer les clés VAPID (pour notifications)

**Avec OpenSSL (recommandé) :**
```bash
openssl ecparam -name prime256v1 -genkey -noout -out vapid_private.pem
# Extraire la clé privée (pour .env)
openssl ec -in vapid_private.pem -outform DER | tail -c +8 | head -c 32 | base64 -w0 | tr '+/' '-_' | tr -d '='
# Extraire la clé publique (pour .env)
openssl ec -in vapid_private.pem -pubout -outform DER | tail -c 65 | base64 -w0 | tr '+/' '-_' | tr -d '='
```

> 💡 Copiez les deux clés dans `.env` et redémarrez : `docker compose up -d`.

---

## 🏗️ Architecture (pour les curieux)

```
Nook/
├── backend/          Rust + Axum 0.8 (API REST, WebSocket, auth, crypto)
├── frontend/         SvelteKit 5 Runes + TypeScript
├── services/
│   └── turn-rs/    Serveur TURN/STUN pour WebRTC
└── docker-compose.yml
```

**Ce qui tourne :**
- Un binaire Rust dans Alpine Linux (surface d'attaque minimale)
- Un serveur TURN/STUN pour relais WebRTC
- Une base SQLite dans `DATA_DIR`
- Un dossier d'uploads chiffrés (nettoyé toutes les 24h)

> 📚 **Docs techniques :**  
> - [API](docs/API.md) — Tous les endpoints + WebSocket  
> - [HTTPS local](docs/nginx-local.md) — Config nginx pour LAN  
> - [CHANGELOG](CHANGELOG.md) — Historique des versions

---

<div align="center">

**Pas de pub. Pas de tracking. Pas de carte bancaire.**  
Juste votre famille, chez vous.

🤜🤛

</div>
