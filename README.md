# 🌿 Nook — Ta messagerie familiale privée & sécurisée

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Work in Progress](https://img.shields.io/badge/Status-En%20développement-orange)]()
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)]()
[![Svelte](https://img.shields.io/badge/Svelte-4A4A55?style=for-the-badge&logo=svelte&logoColor=FF3E00)]()

> Une messagerie instantanée **auto-hébergée**, chiffrée de bout en bout, pensée pour ta famille et tes proches.  
> ✅ Zéro cloud • ✅ Zéro compte • ✅ Zéro tracking • ✅ Open-source & gratuit

*Choisis ton univers : **Jardin Secret** 🌿 (doux, aquarelle), **Space Hub** 🚀 (futuriste), ou **Maison Chaleureuse** 🏠 (cosy)*

## 🚀 Aperçu (screenshots à venir)
![Accueil Jardin Secret](screenshots/accueil-jardin.png)  
![Chat avec réactions](screenshots/chat-reactions.png)  
*(Captures d’écran en cours d’ajout – le projet est en développement actif !)*

## ✨ Fonctionnalités principales
- 🔐 Chiffrement de bout en bout (libsodium, clés client-side)
- 👥 Invitation + approbation des membres
- 💬 Chat riche : emojis, réactions, GIFs (via Tenor anonyme)
- 📎 Partage fichiers : ≤50 Mo chiffrés (auto-supprimés après 7j) • >50 Mo en P2P WebRTC
- 📞 Appels audio/vidéo 1:1 (WebRTC P2P)
- 🗓️ Calendrier familial partagé
- 🎨 3 thèmes personnalisables + mode sombre/clair
- 📲 PWA installable (Android, iOS, desktop)

## 🛠 Stack technique
- **Frontend** : Svelte + TypeScript
- **Backend** : Rust
- **Chiffrement** : libsodium
- **P2P** : WebRTC
- **Déploiement** : Docker, CasaOS, Yunohost…

## 🚀 Installation rapide
### Option recommandée : CasaOS
1. Apps → Custom Install
2. Image : `ghcr.io/mx10-ac2n/nook:latest`
3. Port 3000 → Volume `/appdata/nook` → `/app/data`
4. Le token admin est dans `/appdata/nook/data/admin.token`

### Option universelle : Docker Compose
```yaml
version: '3.8'
services:
  nook:
    image: ghcr.io/mx10-ac2n/nook:latest
    ports:
      - "3000:3000"
    volumes:
      - nook-data:/app/data
    restart: unless-stopped
volumes:
  nook-data: