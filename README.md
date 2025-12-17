<div align="center">

<img src="https://images.unsplash.com/photo-1748280523714-e5204d138964?auto=format&fit=crop&w=1600&q=80&ixlib=rb-4.0.3" alt="Nook – Ton jardin secret privé et sécurisé 🌿" style="border-radius: 20px; max-width: 100%; height: auto;" />

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
| <img src="https://images.unsplash.com/photo-1748280523714-e5204d138964?auto=format&fit=crop&w=800&q=80&ixlib=rb-4.0.3" width="400" /> | <img src="https://thumbs.dreamstime.com/b/colossal-ring-shaped-space-station-hangs-silent-orbit-above-vibrant-blue-planet-testament-to-humanitys-technological-prowess-406427650.jpg" width="400" /> | <img src="https://images.unsplash.com/photo-1755633128337-69788f9e8ed7?auto=format&fit=crop&w=800&q=80&ixlib=rb-4.0.3" width="400" /> |
| Thème doux, aquarelle, nature apaisante | Thème sci-fi, néons, espace infini | Thème chaleureux, foyer, tons cocooning |

*(Images évocatrices libres ou stock – screenshots réels à venir ! Crédits : Tanya Barrow & David Todd McCarty sur Unsplash)*

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