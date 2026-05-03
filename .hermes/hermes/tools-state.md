# 🛠️ État des Outils - Nook Development

> Dernière mise à jour: 2026-05-03
> Vérifié automatiquement au démarrage

## ✅ Outils Installés et Fonctionnels

| Outil | Version | Chemin | Usage |
|-------|---------|--------|-------|
| **git** | 2.47.3 | `/usr/bin/git` | Version control |
| **node** | v20.19.2 | `/usr/bin/node` | Frontend runtime |
| **npm** | 9.2.0 | `/usr/bin/npm` | Frontend packages |
| **jq** | jq-1.7 | `/usr/bin/jq` | JSON processing |
| **curl** | 8.14.1 | `/usr/bin/curl` | HTTP requests |
| **make** | 4.4.1 | `/usr/bin/make` | Build automation |
| **gcc** | 14.2.0 | `/usr/bin/gcc` | C compiler |
| **g++** | 14.2.0 | `/usr/bin/g++` | C++ compiler |
| **pkg-config** | 1.8.1 | `/usr/bin/pkg-config` | Library config |
| **rustc** | 1.85.0 | `/usr/bin/rustc` | Rust compiler |
| **cargo** | 1.85.0 | `/usr/bin/cargo` | Rust package manager |
| **gh** | 2.46.0 | `/usr/bin/gh` | GitHub CLI |

## ⚠️ Outils Partiellement Configurés

### GitHub CLI (gh)
- **État** : Installé mais nécessite auth manuelle
- **Action** : `gh auth login` avec token (voir memory/core.md)
- **Alternative** : Git configuré avec token dans remote URL

## ❌ Outils Manquants ou Version Insuffisante

| Outil | Raison | Statut | Solution |
|-------|--------|--------|-----------|
| **rustup** | Gestionnaire toolchains Rust | Non installé | Pas nécessaire (rustc via apt) |
| **wasm-pack** | Build WASM frontend | ❌ Échec | Nécessite Rust > 1.86 (actuel: 1.85.0) |
| **docker** | Build local/tests | ❌ Non disponible | Utiliser GitHub Actions CI |
| **docker-compose** | Orchestration locale | ❌ Non disponible | Utiliser GitHub Actions CI |

## 🔧 Variables d'Environnement

### Configurées
- `GITHUB_TOKEN` : (dans git remote URL, pas en clair)

### À configurer si besoin
```bash
# Rust
export RUSTUP_HOME=/opt/rust
export CARGO_HOME=/opt/cargo

# Nook
export NOOK_ENV=development
export NOOK_DB_PATH=/opt/data/nook.db
```

## 📝 Historique Installations

### 2026-05-03
- ✅ `rustc`, `cargo` via `apt-get install rustc cargo`
- ✅ `gh` via `apt-get install gh`
- ✅ Versions : Rust 1.85.0, Cargo 1.85.0, GH 2.46.0
- ❌ `wasm-pack` : échec compilation (nécessite Rust > 1.86)
- ℹ️ `docker`/`docker-compose` : non nécessaires (utiliser GitHub Actions)

## 🩺 Vérification Rapide

```bash
# Script de vérification
bash /opt/data/home/.hermes/Nook/.hermes/tools/scripts/check-tools.sh
```

---

*Mettre à jour après chaque modification d'outils*
