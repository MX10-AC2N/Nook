# 🛠️ État des Outils - Nook Development

> Dernière mise à jour: 2026-05-03
> Vérifié automatiquement au démarrage

## ✅ Outils Installés et Fonctionnels

| Outil | Version | Chemin | Usage |
|-------|---------|--------|-------|
| **git** | (system) | `/usr/bin/git` | Version control |
| **node** | (system) | `/usr/bin/node` | Frontend runtime |
| **npm** | (system) | `/usr/bin/npm` | Frontend packages |
| **jq** | (system) | `/usr/bin/jq` | JSON processing |
| **curl** | (system) | `/usr/bin/curl` | HTTP requests |
| **make** | (system) | `/usr/bin/make` | Build automation |
| **gcc** | (system) | `/usr/bin/gcc` | C compiler |
| **g++** | (system) | `/usr/bin/g++` | C++ compiler |
| **pkg-config** | (system) | `/usr/bin/pkg-config` | Library config |
| **rustc** | 1.85.0 | `/usr/bin/rustc` | Rust compiler |
| **cargo** | 1.85.0 | `/usr/bin/cargo` | Rust package manager |
| **gh** | 2.46.0 | `/usr/bin/gh` | GitHub CLI |

## ⚠️ Outils Partiellement Configurés

### GitHub CLI (gh)
- **État** : Installé mais nécessite auth manuelle
- **Action** : `gh auth login` avec token (voir memory/core.md)
- **Alternative** : Git configuré avec token dans remote URL

## ❌ Outils Manquants (à installer si besoin)

| Outil | Raison | Commande Install |
|-------|--------|-----------------|
| **rustup** | Gestionnaire toolchains Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **rustfmt** | Formatage Rust (inclus avec rustc) | Déjà disponible |
| **clippy** | Linter Rust (inclus avec rustc) | Déjà disponible |
| **wasm-pack** | Build WASM (si frontend WASM) | `cargo install wasm-pack` |
| **docker** | Build local (si besoin tester) | `apt-get install docker.io` |
| **docker-compose** | Orchestration locale | `apt-get install docker-compose` |

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

## 🩺 Vérification Rapide

```bash
# Script de vérification (à créer)
bash /opt/data/home/.hermes/Nook/.hermes/tools/scripts/check-tools.sh
```

## 📝 Historique Installations

### 2026-05-03
- ✅ Installé `rustc`, `cargo` via `apt-get install rustc cargo`
- ✅ Installé `gh` via `apt-get install gh`
- ✅ Versions : Rust 1.85.0, Cargo 1.85.0, GH 2.46.0

---
*Mettre à jour après chaque modification d'outils*
