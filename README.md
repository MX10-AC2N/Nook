# 🌿 Nook — Messagerie familiale privée & sécurisée

> **Une messagerie instantanée auto-hébergée, chiffrée de bout en bout, pour ta famille et tes proches.**  
> ✅ Zéro cloud • ✅ Zéro compte • ✅ Zéro tracking • ✅ Open-source

![CI](https://github.com/MX10-AC2N/Nook/actions/workflows/ci.yml/badge.svg)

---

## ✨ Fonctionnalités

- 🔐 **Chiffrement E2EE** (libsodium)
- 👥 **Invitations + validation admin**
- 💬 **Chat texte + emoji + GIFs**
- 📎 **Partage de fichiers ≤ 50 Mo**
- 📡 **WebRTC P2P** (fichiers > 50 Mo)
- 📞 **Appels audio/vidéo 1:1**
- 🗓️ **Calendrier & rendez-vous**
- 🎨 **3 thèmes personnalisables**
- 📲 **PWA installable (mobile/PC)**
- 🌐 **Multiplateforme** (CasaOS, Docker, Nginx Proxy Manager)

---

## 🚀 Déploiement (CasaOS + Nginx Proxy Manager)

1. **Dans CasaOS** → Apps → Installer depuis dossier
2. **Dans Nginx Proxy Manager** :
   - Host: `nook.votre-domaine.com`
   - Scheme: `http`
   - Forward: `nook:3000`
   - Active **Let’s Encrypt** ✅
3. Accède à `https://nook.votre-domaine.com`

> 🔑 Le **token admin** est dans `/casaos/appdata/nook/data/admin.token`

---

## 📖 Guide utilisateur

Voir [`GUIDE_UTILISATEUR.md`](GUIDE_UTILISATEUR.md)

---

## 📜 Licence

MIT License — voir [`LICENSE`](LICENSE)

---

## ❤️ Merci

Nook est construit avec ❤️ pour les familles qui veulent **reprendre le contrôle de leur communication**.

**Libre. Sécurisé. Familial.**