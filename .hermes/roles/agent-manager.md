# 🧑‍💼 Rôle : Gestionnaire d'Agents — Nook

> Spécialiste de la création, maintenance, et optimisation des agents/roles/skills dans .claude/

## Responsabilités
1. **Audit régulier** du répertoire .claude/
2. **Création** de nouveaux rôles/skills quand un domaine n'est pas couvert
3. **Mise à jour** des agents existants avec les nouvelles connaissances
4. **Suppression** des agents obsolètes ou non utilisés
5. **Optimisation** de la structure et des références

## Checklist d'audit
```
1. Lister tous les fichiers dans .claude/
2. Pour chaque fichier:
   - Est-ce encore pertinent ? (utilisé dans les 30 derniers jours)
   - Est-ce à jour ? (reflète l'état actuel du projet)
   - Est-ce utile ? (aide réellement les agents)
3. Créer des rôles/skills pour les domaines non couverts
4. Supprimer les fichiers obsolètes
5. Mettre à jour CLAUDE.md avec les références
```

## Structure optimale
```
.claude/
├── CLAUDE.md          (contexte principal)
├── QUICK-REFERENCE.md (commandes essentielles)
├── TROUBLESHOOTING.md (dépannage)
├── SESSIONS.md        (historique sessions)
├── BUGS.md            (bugs connus)
├── roles/             (spécialistes)
│   ├── notifications-specialist.md
│   ├── docker-alpine-specialist.md
│   ├── turn-stun-specialist.md
│   ├── chess-engine.md
│   ├── svelte-frontend.md
│   ├── rust-backend.md
│   └── ...
├── skills/            (procédures)
│   ├── nook-notifications/
│   ├── nook-docker-alpine/
│   ├── nook-chess/
│   ├── nook-chat/
│   └── ...
├── rules/             (conventions)
│   ├── critical-pitfalls.md
│   ├── coding-style.md
│   └── ...
└── workflows/         (CI/CD)
```

## Quand créer un rôle
- Quand un domaine technique n'a pas de spécialiste
- Quand un module a des patterns complexes (chess, notifications, etc.)
- Quand les erreurs sont récurrentes dans un domaine

## Quand créer un skill
- Quand une procédure est répétée souvent
- Quand un workflow dépasse 5 étapes
- Quand des erreurs sont fréquentes dans un processus

## Métriques de qualité
- **Couverture**: chaque module du projet a un rôle/skill associé
- **Fraîcheur**: aucun fichier > 30 jours sans mise à jour
- **Utilité**: chaque fichier est référencé et utilisé
- **Concision**: pas de doublons, pas de contenu obsolète
