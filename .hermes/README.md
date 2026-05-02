# 🤖 .claude — Espace de travail d'Hermes Agent

> Restructuré le : 2026-04-27
> Objectif : Ne plus perdre le fil, avoir TOUTES les infos systématiquement

## 📂 Structure actuelle

```
.claude/
├── hermes/                    # MON espace perso (Hermes Agent)
│   ├── active-session.md      # Ce que je fais MAINTENANT
│   ├── hermes-memory.md       # Infos critiques (tokens, comptes, règles)
│   ├── known-issues.md        # Bugs, pièges, leçons apprises
│   └── preferences.md         # Préférences utilisateur
│
├── project/                   # Référence projet
│   ├── project-state.md      # État actuel Nook (version, branche, CI)
│   └── BUGS.md              # Bugs actifs (anciennement à la racine)
│
├── reference/                 # Références rapides (patterns)
│   ├── rust-patterns.md     # Rust/Axum/SQLx patterns + pièges
│   └── svelte-patterns.md   # Svelte 5 Runes patterns + MCP Svelte
│
├── archive/                   # Archives (TOUT est préservé !)
│   ├── reports/             # Anciens rapports (builds, audits, tests)
│   │   ├── audits/
│   │   ├── builds/
│   │   └── tests/
│   └── sessions/            # Sessions historiques (SESSIONS.md)
│
├── skills/                    # Skills existants (gardés tels quels)
├── roles/                     # Rôles d'agents (gardés tels quels)
├── rules/                     # Règles d'orchestration (gardées)
├── workflows/                 # Workflows GitHub (gardés)
├── tools/                     # Outils MCP (gardés)
├── hooks/                     # Hooks de session (gardés)
│
└── CLAUDE.md                  # Orchestrateur principal (à la racine)
```

## 🎯 Philosophie de cette restructuration

### Avant (problèmes)
- ❌ Fichiers éparpillés à la racine (20+ .md)
- ❌ Pas de séparation "espace agent" vs "références"
- ❌ Rapports historiques mélangés avec le présent
- ❌ Perte de contexte entre sessions

### Maintenant (solutions)
- ✅ **hermes/** : Mon espace perso, mes règles, ma mémoire
- ✅ **project/** : État du projet, bugs actifs
- ✅ **reference/** : Patterns rapides pour ne pas faire d'erreurs
- ✅ **archive/** : TOUT est préservé, rien n'est supprimé
- ✅ **CLAUDE.md** reste à la racine (orchestrateur)

## 📋 Comment je travaille maintenant

### À chaque début de session
1. ✅ Lire `hermes/active-session.md` → Qu'est-ce que je faisais ?
2. ✅ Lire `hermes/known-issues.md` → Quels pièges éviter ?
3. ✅ Vérifier `project/project-state.md` → Où en est Nook ?

### En cours de travail
- ✅ Mettre à jour `hermes/active-session.md` après chaque action
- ✅ Consulter `reference/` selon le besoin (Rust ou Svelte)
- ✅ Ajouter les nouveaux bugs dans `hermes/known-issues.md`

### En cas d'erreur
- ✅ Documenter dans `known-issues.md` pour ne pas répéter
- ✅ Exemple : j'avais changé `rustrtc` version → règle apprise

## 🔗 Liens rapides

- **CI Backend :** https://github.com/MX10-AC2N/Nook/actions/workflows/Backend.yml
- **Repo :** https://github.com/MX10-AC2N/Nook
- **Local :** https://192.168.1.192:6443

## 🧠 Ce que je ne dois plus oublier (extrait de hermes-memory.md)

- **GITHUB_TOKEN :** Dans `hermes/hermes-memory.md`
- **Compte test :** hermes-bot / Hermes2026!
- **Rust nightly :** Utilisé dans Backend.yml (ligne 34)
- **rand 0.9 :** `rng()` pas `thread_rng()`
- **Svelte 5 :** `$derived.by` pas de réassignation `$state`
- **Règle d'or :** Un commit de fix ne touche QUE le bug signalé

## 📝 Notes sur l'archivage

- ✅ Aucun fichier n'est supprimé, tout est dans `archive/`
- ✅ Les rapports anciens sont classés par type (audits, builds, tests)
- ✅ `SESSIONS.md` est dans `archive/sessions/`
- ✅ Pour retrouver un vieux rapport : `find .claude/archive -name "*.md"`

---
*Cette structure est vivante. Si tu as des suggestions pour l'améliorer, dis-le-moi !*
