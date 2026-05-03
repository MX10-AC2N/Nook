# .hermes - Hermes Agent Workspace for Nook

Ce répertoire est **MON ESPACE DE TRAVAIL PERSONNEL** pour le développement de Nook.

## 🎯 Objectif

Ce répertoire me sert à :
- **Mémoriser** tout mon contexte de travail sur Nook
- **Stocker** les outils et scripts nécessaires au développement
- **Sauvegarder** l'état de mes interventions
- **Optimiser** mes futures interventions sur le projet

## 📁 Structure

```
.hermes/
├── workspace/           # MON espace de travail actif
│   ├── memory/         # Mémoire organisée (par domaine)
│   ├── scripts/        # Scripts utiles pour le dev
│   ├── tools/          # Outils installés manuellement
│   ├── state/          # État actuel (TODO, sessions)
│   └── notes/          # Notes de travail
├── skills/             # Skills pour Hermes Agent (28 skills)
├── roles/              # Rôles d'agents (32 agents)
├── reports/            # Rapports d'audit et analyses
├── tools/              # Outils et scripts (setup-env.sh, etc.)
├── rules/              # Règles de développement
└── archive/            # Anciens rapports/archives
```

## 🛠️ Outils Configurés

- `wasm-pack` → `tools/bin/wasm-pack` (lien symbolique)
- `nook-status` → `tools/bin/nook-status` (script de statut rapide)
- `setup-env.sh` → `tools/setup-env.sh` (configuration environnement)

## 📊 État Actuel (2026-05-03)

- **Audit Score**: 68/100
- **Priorité P0**: 
  - ❌ events.rs (34 erreurs compilation)
  - ❌ PWA broken (service-worker.ts)
  - ❌ WebRTC ICE config manquante
  - ❌ export_pgn() cassé (en cours de fix)
- **Priorité P1**:
  - ❌ Pas de tests frontend
  - ❌ Pas cargo test en CI
  - ❌ 106 E2E skippés

## 🚀 Utilisation

```bash
# Charger l'environnement
source .hermes/tools/setup-env.sh

# Vérifier le statut du projet
nook-status

# Voir mes tâches en cours
cat .hermes/workspace/state/current-session.json
```

## 📝 Notes pour le Futur

- Ce répertoire est versionné sur GitHub (branche develop)
- Tout ce que j'apprends sur Nook doit être sauvegardé ici
- Les scripts doivent être génériques et réutilisables
- La mémoire doit être organisée par domaine (frontend, backend, devops, etc.)

---
*Dernière mise à jour: 2026-05-03 par Hermes Agent*
