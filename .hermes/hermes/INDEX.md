# 📚 INDEX - Espace de Travail Hermes

> **Mon centre de contrôle** - Navigation rapide dans mon espace .hermes
> **DERNIÈRE MISE À JOUR** : 2026-05-04
> **Status** : ✅ RESTRUCTURÉ ET OPTIMISÉ

## 🗂️ Structure Actuelle

```
.hermes/
├── hermes/                    # MON ESPACE PERSO (ce répertoire)
│   ├── INDEX.md              # Ce fichier - Navigation rapide
│   ├── memory/               # Mémoire organisée par domaines ✅
│   │   ├── core.md          # Infos critiques (GitHub, URLs, versions)
│   │   ├── rust.md          # Apprentissages Rust/Axum/SQLx
│   │   ├── svelte.md        # Apprentissages Svelte 5/Kit
│   │   ├── devops.md        # CI/CD, Docker, Zimaboard
│   │   ├── security.md      # E2EE, auth, WebRTC
│   │   └── sessions/        # Logs de session (auto-archivage)
│   ├── active-session.md    # Session en cours
│   ├── known-issues.md      # Bugs & pièges à éviter
│   ├── preferences.md        # Préférences utilisateur
│   ├── tools-state.md       # État des outils installés
│   └── hermes-memory.md    # ANCIEN FICHIER (à archiver)
│
├── skills/                    # Mes skills (ne pas modifier manuellement)
├── roles/                     # Rôles des agents (32 agents)
├── rules/                     # Règles du projet Nook
├── workflows/                 # Workflows GitHub Actions
├── tools/                     # Outils et scripts ✅
│   ├── scripts/              # Scripts d'installation/setup
│   ├── install-tools.sh      # Script installation outils manquants
│   └── *.md                  # Références outils
├── project/                   # État du projet Nook
├── reference/                 # Patterns de code
├── reports/                   # Rapports d'audit (GLOBAL, SECURITY, etc.)
├── archive/                   # Archives (ne pas supprimer)
├── workspace/                 # Workspace de développement
└── hooks/                     # Hooks de session
```

## 🚀 Démarrage Rapide (OBLIGATOIRE à chaque session)

1. **Lire en premier** : `hermes/active-session.md` (session courante)
2. **Consulter si besoin** : `hermes/memory/core.md` (infos critiques)
3. **Vérifier** : `hermes/known-issues.md` (ne pas répéter les erreurs)
4. **Outils disponibles** : `hermes/tools-state.md` (état des outils)
5. **Mettre à jour** : `hermes/active-session.md` après chaque action

## 🧠 Domaines Mémoire (NOUVEAU - 2026-05-04)

| Domaine | Fichier | Quand consulter | Dernière MAJ |
|---------|----------|-----------------|--------------|
| **Core/Critique** | `memory/core.md` | Chaque session | 2026-05-04 |
| **Rust Backend** | `memory/rust.md` | Dev backend, cargo, clippy | 2026-05-04 |
| **Svelte Frontend** | `memory/svelte.md` | Dev frontend, Svelte 5 | 2026-05-04 |
| **DevOps/CI** | `memory/devops.md` | Workflows, Docker, deploy | 2026-05-04 |
| **Security** | `memory/security.md` | E2EE, auth, WebRTC | 2026-05-04 |

## 📝 Sessions

Les logs de session sont dans `hermes/memory/sessions/` avec format :
- `YYYY-MM-DD-SSS.md` (ex: 2026-05-04-001.md)
- Auto-archivage après 30 jours dans `hermes/memory/sessions/archive/`

### Session Actuelle
- **Fichier** : `hermes/active-session.md`
- **Status** : Restructuration .hermes en cours (2026-05-04)

## 🔧 Scripts Disponibles (NOUVEAU - 2026-05-04)

| Script | Emplacement | Usage |
|--------|--------------|-------|
| **Setup Hermes** | `tools/scripts/setup-hermes.sh` | Init environnement Docker |
| **Install Tools** | `tools/scripts/install-nook-tools.sh` | Installe outils manquants |
| **Check Tools** | `tools/scripts/check-tools.sh` | Vérifie état outils |
| **Memory Backup** | `tools/scripts/backup-memory.sh` | Sauvegarde mémoire |

## ⚡ Commandes Fréquentes

```bash
# Naviguer vers le projet
cd /opt/data/home/.hermes/Nook

# Lire session active
cat .hermes/hermes/active-session.md

# Vérifier état outils
bash .hermes/tools/scripts/check-tools.sh

# Mettre à jour mémoire (éditer le fichier correspondant)
nano .hermes/hermes/memory/core.md
nano .hermes/hermes/memory/rust.md
# etc.

# Voir les rapports
ls -la .hermes/reports/
```

## 🔗 Liens Rapides

- **Repo** : https://github.com/MX10-AC2N/Nook
- **CI Backend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- **CI Frontend** : https://github.com/MX10-AC2N/Nook/actions/workflows/Frontend.yml
- **CI Docker** : https://github.com/MX10-AC2N/Nook/actions/workflows/Docker.yml
- **Deploy** : https://192.168.1.192:6443
- **Registry** : https://github.com/orgs/MX10-AC2N/packages

## 📊 Status des Outils (Résumé)

### ✅ Installés
- git, node, npm, jq, curl, make, gcc, g++, pkg-config
- rustc 1.85.0, cargo 1.85.0, gh 2.46.0

### ❌ Manquants / Problématiques
- **wasm-pack** : Échec (needs Rust > 1.86, actuel: 1.85.0)
- **docker** : Non disponible (utiliser GitHub Actions)
- **docker-compose** : Non disponible (utiliser GitHub Actions)
- **rust-mcp-server** : À installer (pour MCP Rust)
- **mcp-language-server** : À installer (pour LSP Rust)

→ Voir `tools-state.md` pour détails complets

## 🎯 Prochaines Étapes (Ma Todo List)

- [x] Créer structure memory/ organisée
- [x] Créer fichiers mémoire (core, rust, svelte, devops, security)
- [x] Mettre à jour INDEX.md
- [ ] Installer outils manquants (wasm-pack, rust-mcp-server, etc.)
- [ ] Créer scripts d'installation et vérification
- [ ] Archiver ancien hermes-memory.md
- [ ] Push vers GitHub (branche develop)
- [ ] Configurer TURN server (P0 WebRTC ICE)

## ⚠️ Notes Importantes

1. **Ce répertoire .hermes m'appartient** - Je peux y faire ce que je veux pour optimiser mon fonctionnement
2. **Persistence** - Tout est sauvegardé sur GitHub branche develop
3. **Mémoire étendue** - Au-delà de la limite 3000 chars de l'outil memory
4. **Outils** - Les outils manquants seront installés automatiquement via scripts
5. **Sécurité** - Pas de secrets en clair, utiliser GitHub Secrets ou variables d'environnement

---

*Index généré automatiquement - Éditer si structure change*
*Dernière restructuration majeure : 2026-05-04*
