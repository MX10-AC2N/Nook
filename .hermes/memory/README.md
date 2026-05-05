# 🧠 Mémoire Organisée - Nook Hermes

> Ce répertoire contient la mémoire organisée de Hermes Agent pour le projet Nook.

## Structure

```
memory/
├── README.md              # Ce fichier
├── context/               # Contexte persistant par domaine
│   ├── backend.md         # Rust, Axum, SQLx, migrations
│   ├── frontend.md        # Svelte 5, stores, composants
│   ├── devops.md          # Docker, CI/CD, GitHub Actions
│   ├── security.md        # E2EE, auth, WebRTC
│   └── architecture.md    # Design système, ADR
├── sessions/              # Résumés de sessions par date
│   └── YYYY-MM-DD.md
├── decisions/             # Decisions architechturales (ADR)
│   └── ADR-XXX-title.md
└── patterns/              # Patterns récurrents et solutions
    ├── bugs-solutions.md  # Bugs fréquents et leurs solutions
    └── optimizations.md   # Optimisations découvertes
```

## Utilisation

- **Avant chaque session** : Lire les fichiers pertinents dans `context/`
- **Pendant la session** : Noter les nouvelles découvertes
- **Fin de session** : Mettre à jour le contexte et créer un résumé dans `sessions/`

## Synchronisation

Ce répertoire est versionné dans le repo Nook (branche develop).
Les changements sont poussés automatiquement à la fin des sessions importantes.
