# 📚 INDEX - Espace de Travail Hermes

> **Mon centre de contrôle** - Navigation rapide dans mon espace .hermes
> Mis à jour: 2026-05-03

## 🗂️ Structure Rapide

```
.hermes/
├── hermes/                    # MON ESPACE PERSO (ce répertoire)
│   ├── INDEX.md              # Ce fichier - Navigation rapide
│   ├── memory/               # Mémoire organisée par domaines
│   │   ├── core.md          # Infos critiques (GitHub, URLs, versions)
│   │   ├── rust.md          # Apprentissages Rust/Axum/SQLx
│   │   ├── svelte.md        # Apprentissages Svelte 5/Kit
│   │   ├── devops.md        # CI/CD, Docker, Zimaboard
│   │   ├── security.md      # E2EE, auth, WebRTC
│   │   └── sessions/        # Logs de session (auto-archivage)
│   ├── active-session.md    # Session en cours
│   ├── known-issues.md      # Bugs & pièges à éviter
│   ├── preferences.md        # Préférences utilisateur
│   └── tools-state.md       # État des outils installés
│
├── skills/                    # Mes skills (ne pas modifier manuellement)
├── roles/                     # Rôles des agents
├── rules/                     # Règles du projet
├── workflows/                 # Workflows GitHub Actions
├── tools/                     # Outils et scripts
│   ├── scripts/              # Scripts d'installation/setup
│   └── *.md                  # Références outils
├── project/                   # État du projet Nook
├── reference/                 # Patterns de code
├── archive/                   # Archives (ne pas supprimer)
└── hooks/                     # Hooks de session
```

## 🚀 Démarrage Rapide

1. **Lire en premier** : `active-session.md` (session courante)
2. **Consulter si besoin** : `memory/core.md` (infos critiques)
3. **Vérifier** : `known-issues.md` (ne pas répéter les erreurs)
4. **Outils disponibles** : `tools-state.md` (état des outils)

## 🧠 Domaines Mémoire

| Domaine | Fichier | Quand consulter |
|---------|----------|-----------------|
| Core/Critique | `memory/core.md` | Chaque session |
| Rust Backend | `memory/rust.md` | Dev backend, cargo, clippy |
| Svelte Frontend | `memory/svelte.md` | Dev frontend, Svelte 5 |
| DevOps/CI | `memory/devops.md` | Workflows, Docker, deploy |
| Security | `memory/security.md` | E2EE, auth, WebRTC |

## 📝 Sessions

Les logs de session sont dans `memory/sessions/` avec format :
- `YYYY-MM-DD-SSS.md` (ex: 2026-05-03-054.md)
- Auto-archivage après 30 jours dans `memory/sessions/archive/`

## 🔧 Scripts Disponibles

| Script | Emplacement | Usage |
|--------|--------------|-------|
| Setup Hermes | `tools/scripts/setup-hermes.sh` | Init environnement Docker |
| Install Tools | `tools/scripts/install-nook-tools.sh` | Installe outils manquants |

## ⚡ Commandes Fréquentes

```bash
# Naviguer vers le projet
cd /opt/data/home/.hermes/Nook

# Vérifier état outils
cat .hermes/hermes/tools-state.md

# Lire session active
cat .hermes/hermes/active-session.md

# Mettre à jour mémoire
# (Éditer le fichier correspondant dans memory/)
```

## 🔗 Liens Rapides

- **Repo** : https://github.com/MX10-AC2N/Nook
- **CI Backend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- **CI Frontend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- **CI Docker** : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- **Deploy** : https://192.168.1.192:6443

---
*Index généré automatiquement - Éditer si structure change*
