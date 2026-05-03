# 🚀 Mémoire DevOps - CI/CD & Déploiement

> Dernière mise à jour: 2026-05-03
> Consulté lors de tout travail CI/CD, Docker, déploiement

## 🔧 Outils & Versions

- **Docker** : Multi-arch (Alpine 3.21 base)
- **Docker Compose** : v2+
- **GitHub Actions** : Workflows dans `.github/workflows/`
- **Zimaboard** : Cible de déploiement
- **NGINX** : Alpine, port 6443 HTTPS (cert auto-signé)

## 🚀 Workflows CI/CD

### IDs et Fichiers
| Workflow | ID | Fichier | Déclenchement |
|----------|-----|---------|----------------|
| Backend | 220018362 | `Backend.yml` | Manuel (`gh workflow run`) |
| Frontend | 220018364 | `Frontend.yml` | Manuel (`gh workflow run`) |
| Turn Server | 257238341 | `Turn.yml` | Manuel (`gh workflow run`) |
| Docker | 220018363 | `Docker.yml` | Manuel, après F/E/T success |

### Ordre d'exécution critique
```
1. Lancer simultanément : Backend + Frontend + Turn
2. Attendre TOUS les succès
3. Lancer Docker (build & push multi-arch)
```

### Commandes de déclenchement
```bash
# Depuis /opt/data/home/.hermes/Nook
gh workflow run 220018362 --ref develop  # Backend
gh workflow run 220018364 --ref develop  # Frontend
gh workflow run 257238341 --ref develop  # Turn
gh workflow run 220018363 --ref develop  # Docker
```

### Surveiller les runs
```bash
# Voir les runs récents
gh run list

# Voir un run spécifique
gh run view <RUN_ID>

# Suivre un run en direct
gh run watch <RUN_ID>
```

## 🐳 Docker & Déploiement

### Images
- **Registry** : GHCR (GitHub Container Registry)
- **Image** : `ghcr.io/MX10-AC2N/nook`
- **Tags** : `latest`, `develop`, `sha-XXXXXXX`
- **Architectures** : `amd64`, `arm64`

### Docker Compose (Zimaboard)
```yaml
# Structure typique
services:
  nook-backend:
    image: ghcr.io/MX10-AC2N/nook:latest
    ports:
      - "3000:3000"
  nook-frontend:
    image: ghcr.io/MX10-AC2N/nook-frontend:latest
    ports:
      - "6443:6443"
```

### HTTPS Local (nginx)
- **Port** : 6443
- **Cert** : Auto-signé (pour enregistrement audio/vidéo)
- **URL** : `https://192.168.1.192:6443`

## 🔴 Règles Critiques

### 1. Jamais de secrets en dur
- ❌ `TURN_SECRET=mysecret` dans le code
- ✅ Utiliser GitHub Secrets
- ✅ `.env.example` comme template

### 2. Séquence CI correcte
- ❌ Lancer Docker avant les autres
- ✅ Attendre Backend + Frontend + Turn OK → Docker

### 3. Commit puis rebase pour lock updates
- ✅ `cargo update` → commit → push
- ✅ Si conflit : rebase develop

## 📝 Learnings Sessions

### Session 50-53
- ✅ Backend.yml utilise Rust **nightly** (ligne 34)
- ✅ 0 secret en dur (TURN_SECRET, admin password)
- ✅ Healthchecks ajoutés (PR #29)

### Erreurs fréquentes
1. **Lancer Docker trop tôt** → échec build
2. **Oublier nightly pour Rust** → échec compilation
3. **Secrets en dur** → fail security audit

## 🔗 Ressources

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Docker Multi-arch](https://docs.docker.com/build/building/multi-platform/)
- [GHCR Docs](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)

---
*Ajouter nouveaux apprentissages au fur et à mesure*
