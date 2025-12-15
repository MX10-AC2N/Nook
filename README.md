# 🌿 Nook — Messagerie familiale privée & sécurisée

> **Une messagerie instantanée auto-hébergée, chiffrée de bout en bout, pour ta famille et tes proches.**  
> ✅ Zéro cloud • ✅ Zéro compte • ✅ Zéro tracking • ✅ Open-source

[![CI/CD](https://github.com/MX10-AC2N/Nook/actions/workflows/build.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions)  
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

![Nook Screenshot](https://raw.githubusercontent.com/MX10-AC2N/Nook/main/screenshots/chat-jardin.png)
![Nook Themes](https://raw.githubusercontent.com/MX10-AC2N/Nook/main/screenshots/themes-switcher.png)

---

## ✨ Fonctionnalités

- 🔐 **Chiffrement E2EE** avec libsodium (clés générées côté client)
- 👥 **Invitations sécurisées + validation par admin**
- 💬 **Chat texte, emojis, réactions et GIFs (via Giphy)**
- 📎 **Partage de fichiers** (≤ 50 Mo direct, > 50 Mo via WebRTC P2P)
- 📞 **Appels audio/vidéo 1:1** (WebRTC)
- 🗓️ **Calendrier partagé & rendez-vous familiaux**
- 🎨 **Thèmes personnalisés** (Jardin Secret 🌿, Space Hub 🚀, Maison Chaleureuse 🏠) avec **transitions animées fluides**
- 📲 **PWA installable** sur mobile & desktop
- 🌙 **Thème persistant** (dark/light/auto selon système)
- 🌐 **Multiplateforme** : Docker, Docker Compose, CasaOS, Yunohost, Portainer...

---

## 🚀 Déploiement rapide

### Option 1 : Docker Compose (recommandé – universel)

```yaml
# docker-compose.yml
version: '3.8'
services:
  nook:
    image: ghcr.io/mx10-ac2n/nook:latest
    container_name: nook
    ports:
      - "3000:3000"
    volumes:
      - nook-data:/app/data
    restart: unless-stopped

volumes:
  nook-data: