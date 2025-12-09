# 🌿 Nook — Espace familial privé

Messagerie sécurisée, auto-hébergée, pour votre cercle proche.

## 🚀 Déploiement CasaOS

1. Dans CasaOS → Apps → Installer depuis dossier
2. Dans Nginx Proxy Manager :
   - Host: `nook.votredomaine.com`
   - Forward: `nook:3000`
   - Activer HTTPS (Let’s Encrypt)

> 🔐 Le premier lancement crée `data/admin.token`. Seul qui y a accès peut configurer l’admin.

## 🛠️ Développement
```bash
# Frontend
cd frontend && npm run dev

# Backend
cd backend && cargo run