<div align="center">

# 🌿 Nook — Ta messagerie familiale ultra privée

[![Stars](https://img.shields.io/github/stars/MX10-AC2N/Nook?style=social)](https://github.com/MX10-AC2N/Nook/stargazers)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Status](https://img.shields.io/badge/Status-En%20développement%20actif-orange)]()
[![Rust](https://img.shields.io/badge/Rust-black?style=flat&logo=rust&logoColor=white)]()
[![Svelte](https://img.shields.io/badge/Svelte-orange?style=flat&logo=svelte&logoColor=white)]()

**Auto-hébergée • Chiffrée E2EE • Zéro cloud • Zéro tracking**

Une messagerie pensée pour ta famille et tes proches.  
Pas de compte, pas de pub, juste du privé et du cosy. 🚀

</div>

## 🚀 Pourquoi Nook ?
- 🔐 **Tout chiffré côté client** (libsodium – rien ne transite en clair)
- 👥 Invitation + validation manuelle : tu décides qui entre
- 💬 Chat fun : réactions, emojis, GIFs (anonymes via Tenor)
- 📎 Fichiers jusqu’à 50 Mo chiffrés (auto-supprimés après 7j) • plus gros en P2P direct
- 📞 Appels audio/vidéo 1:1 en WebRTC (zéro serveur intermédiaire)
- 🗓️ Calendrier partagé pour toute la famille
- 🎨 **3 thèmes de ouf** : Jardin Secret 🌿 (douceur nature), Space Hub 🚀 (vibes futuristes), Maison Chaleureuse 🏠 (cocooning total) + mode sombre/clair
- 📲 PWA : installe-la comme une app sur téléphone ou ordi

## 🖼️ Aperçu
Captures d’écran en cours de préparation – ça arrive bientôt !  
En attendant, imagine des interfaces magnifiques et super fluides selon le thème que tu choisis. 😏

## ⚡ Installation ultra simple (2 minutes top chrono)

### Le plus facile : CasaOS (parfait pour les débutants)
1. Apps → Custom Install
2. Image : `ghcr.io/mx10-ac2n/nook:latest`
3. Port 3000 → Volume `/appdata/nook` → `/app/data`
4. Ton token admin apparaît dans `/appdata/nook/data/admin.token` → c’est parti !

### Pour les pros : Docker Compose
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
```markdown
## 🛠 Stack technique (pour les curieux)
| Partie       | Techno                  |
|--------------|-------------------------|
| Frontend     | Svelte + TypeScript     |
| Backend      | Rust                    |
| Chiffrement  | libsodium               |
| P2P          | WebRTC                  |
| Déploiement  | Docker, CasaOS, etc.    |

## 🔮 Ce qui arrive bientôt
- Appels de groupe 🎉
- Notifications push 📲
- Encore plus de thèmes et d’options mobile
- Plein d’améliorations basées sur vos retours !

## 🤝 Tu veux contribuer ?
Le projet est ouvert à tous !  
Bug, idée, code, design… tout est bienvenu.  
Fork → bidouille → PR → on discute ensemble. C’est cool de construire ça à plusieurs. ❤️

<div align="center">

**License** : [MIT](LICENSE)  
**Si Nook te plaît, claque une ⭐ – ça motive grave !**

</div>
