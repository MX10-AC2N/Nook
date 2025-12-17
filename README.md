<div align="center">

# 🌿 Nook

[![Stars](https://img.shields.io/github/stars/MX10-AC2N/Nook?style=social)](https://github.com/MX10-AC2N/Nook/stargazers)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![WIP](https://img.shields.io/badge/Status-En%20développement-orange)]()
[![Rust](https://img.shields.io/badge/Rust-black?style=flat&logo=rust)]()
[![Svelte](https://img.shields.io/badge/Svelte-orange?style=flat&logo=svelte)]()

**Messagerie familiale privée • Auto-hébergée • Chiffrée E2EE**

**Zéro cloud • Zéro compte • Zéro tracking • 100% open-source**

🚀 **Déploie en 2 minutes** avec Docker ou CasaOS  
🔐 **Tout chiffré côté client** • Appels WebRTC P2P • Calendrier partagé • GIFs & réactions

### 🎨 Choisis ton univers

| Jardin Secret 🌿 | Space Hub 🚀 | Maison Chaleureuse 🏠 |
|--------------------|--------------------|--------------------|
| ![Jardin Secret – ambiance douce et naturelle](https://thumbs.dreamstime.com/b/colorful-flowers-bloom-brightly-along-winding-path-leading-to-charming-gate-lush-botanical-garden-inviting-visitors-394781787.jpg) | ![Space Hub – ambiance futuriste](https://thumbs.dreamstime.com/b/awe-inspiring-d-render-colossal-futuristic-space-station-orbiting-majestic-ringed-planet-intricate-multi-layered-398780776.jpg) | ![Maison Chaleureuse – ambiance cosy](https://thumbs.dreamstime.com/b/cozy-living-room-night-warm-light-fireplace-candles-christmas-decor-comfortable-sofa-coffee-table-winter-scene-outside-351073807.jpg) |
| Thème doux, aquarelle, nature apaisante | Thème sci-fi, néons, espace infini | Thème chaleureux, foyer, tons cocooning |

*(Illustrations évocatrices – screenshots réels à venir !)*

## 🚀 Quick Start (Docker Compose)

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