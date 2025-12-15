# 🌿 Nook — Messagerie familiale privée & sécurisée

> **Une messagerie instantanée auto-hébergée, chiffrée de bout en bout, pour ta famille et tes proches.**  
> ✅ Zéro cloud • ✅ Zéro compte • ✅ Zéro tracking • ✅ Libre et gratuit • ✅ Open-source

[![CI/CD](https://github.com/MX10-AC2N/Nook/actions/workflows/ci-new2.yml/badge.svg)](https://github.com/MX10-AC2N/Nook/actions)  
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)  
[![Docker Image Size](https://img.shields.io/docker/image-size/ghcr.io/mx10-ac2n/nook/latest?label=Image%20size)](https://github.com/MX10-AC2N/Nook/pkgs/container/nook)

![Nook Screenshot](https://raw.githubusercontent.com/MX10-AC2N/Nook/refs/heads/main/screenshots/chat-jardin.png)  
*Thème « Jardin Secret » — doux, naturel, aquarelle*

![Nook Themes](https://raw.githubusercontent.com/MX10-AC2N/Nook/refs/heads/main/screenshots/themes-switcher.png)  
*Choisis parmi 3 univers : Jardin Secret 🌿, Space Hub 🚀, Maison Chaleureuse 🏠*

---

## ✨ Fonctionnalités

- 🔐 **Chiffrement de bout en bout (E2EE)** avec **libsodium** (clés générées côté client, jamais sur le serveur)
- 👥 **Gestion des membres** : inviter → approuver → discuter (aucun accès sans validation)
- 💬 **Chat riche** : texte, emojis, réactions (👍 ❤️), GIFs (proxy anonyme Tenor)
- 📎 **Partage de fichiers** :
  - **≤ 50 Mo** : upload chiffré (stockage temporaire, auto-supprimé après 7j)
  - **> 50 Mo** : envoi **P2P direct** via WebRTC (aucun stockage serveur)
- 📞 **Appels audio/vidéo 1:1** (WebRTC, P2P, chiffrés)
- 🗓️ **Calendrier partagé** : planifiez rendez-vous, anniversaires, appels
- 🎨 **3 thèmes personnalisables** :
  - **🌿 Jardin Secret** : doux, aquarelle, feuilles animées
  - **🚀 Space Hub** : futuriste, néon, effets de particules
  - **🏠 Maison Chaleureuse** : feutre, crayon, bois
- 📲 **PWA installable** : comme une app native sur **Android, iOS, PC**
- 🌙 **Mode sombre/clair** : respecte les préférences système
- 🌐 **Multiplateforme** : CasaOS, Docker, Docker Compose, Portainer, Yunohost…

---

## 🚀 Déploiement rapide

### 🏠 Option 1 : CasaOS (recommandé pour les débutants)

1. Dans **CasaOS → Apps → Installer depuis l’image Docker**
2. Colle : `ghcr.io/mx10-ac2n/nook:latest`
3. Configure :
   - **Port** : `3000`
   - **Volume** : `/casaos/appdata/nook //app/data`
4. **Démarre** → accède à `http://votre-nas:3000`

> 🔑 Le **token admin** se trouve dans `/casaos/appdata/nook/data/admin.token`

---

### 🐳 Option 2 : Docker Compose (universel)

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
      - nook-/app/data
    restart: unless-stopped

volumes:
  nook- ```

