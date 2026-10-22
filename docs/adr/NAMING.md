# Convention de nommage des ADR — décision

**Date** : 2026-10-22
**Carte** : kanban `t_db6f9481`
**Auteur** : `@docs-writer`
**Validateur** : `@architect` (Gate gouvernance ADR)
**Statut** : Décision documentée · en attente de validation architecte

---

## 1. Problème constaté

`docs/adr/` mélangeait **trois formes de nommage** pour désigner la même chose :

```
docs/adr/011-team-structure-decisions.md   ← numérique, 3 chiffres
docs/adr/012-sfu-relay-capacity.md        ← numérique, 3 chiffres
docs/adr/ADR-016-e2ee-group-hybrid.md     ← préfixe "ADR-", 3 chiffres
docs/adr/ADR-018-agent-memory-storage.md  ← préfixe "ADR-", 3 chiffres
docs/adr/0013-dual-message-store-architecture.md              ← 4 chiffres avec zéro initial
docs/adr/0014-rand_core-argon2-migration.md                   ← underscore dans le slug
```

Effets concrets :
- Les références `ADR-011` vs `ADR-11` vs `011` ne pointent pas toutes au même endroit.
- Un lien `ADR-016` est ambigu : il peut désigner l'ADR E2EE **ou** la migration de dépendance (collision historique, voir `docs/adr/README.md` §4.4).
- Les slugs avec underscores (`rand_core`) ou sans kebab-case ne sont pas triables proprelement.

## 2. Décision

**Norme unique** pour tout nouveau ADR :

```
docs/adr/NNNN-title-kebab-case.md
```

| Règle | Détail |
|---|---|
| Numéro | 4 chiffres, zéro-padding (`019`, `020`) |
| Slug | `kebab-case` ASCII, sans accents, sans underscores, sans espaces |
| Préfixe `ADR-` | **facultatif** dans le nom de fichier |
| Préfixe `ADR-` | **obligatoire** dans le titre (`# ADR-019 : …`) et dans toutes les références croisées |
| Numéro | Unique, jamais réutilisé, même pour un ADR orphelin |
| Tri | Lexicographique = chronologique |

**Exemples valides** : `019-single-container-budget.md`, `020-postgres-migration.md`, `021-e2ee-rotation-automatique.md`

## 3. Table de correspondance (migration)

| Fichier d'origine | Fichier normalisé | Statut |
|---|---|---|
| `011-team-structure-decisions.md` | `011-team-structure-decisions.md` | ✅ conforme, inchangé |
| `012-sfu-relay-capacity.md` | `012-sfu-relay-capacity.md` | ✅ conforme, inchangé |
| `ADR-016-e2ee-group-hybrid.md` | `016-e2ee-group-hybrid.md` | ✅ renommé |
| `ADR-018-agent-memory-storage.md` | `018-agent-memory-storage.md` | ✅ renommé |
| `ADR-017-reply-to-feature.md` | `017-reply-to-feature.md` | ⏳ à faire (fichier non encore commité) |
| `0013-dual-message-store-architecture.md` | `013-dual-message-store-architecture.md` | ⏳ orphelin, non commité |
| `0014-rand_core-argon2-migration.md` | `014-rand-core-argon2-migration.md` | ⏳ orphelin, non commité |
| `0015-sqlx-08-to-09-migration.md` | `015-sqlx-08-to-09-migration.md` | ⏳ orphelin, non commité |
| `0016-chacha20poly1305-010-to-011-migration.md` | — | ⛔ doublon de collision, ne pas recréer (numéro 016 attribué à l'ADR E2EE) |

## 4. Pourquoi pas l'inverse (préfixe `ADR-` partout) ?

Le préfixe `ADR-` dans le **nom de fichier** présente deux inconvénients :
1. Il casse le tri lexicographique attendu par les outils (`ls docs/adr/` mélange `011` et `ADR-016`).
2. Il oblige à taper `ADR-011-…` dans les liens alors que le numéro seul suffit.

Il reste **obligatoire dans le titre et les références**, parce que c'est la forme lisible par un humain dans un paragraphe de revue.

## 5. Voir aussi

- `docs/adr/README.md` — inventaire et statuts
- `docs/adr/HISTORY.md` — historique chronologique
- `.hermes/roles/architect.md:25` — processus ADR (template)

---

*Décision ouverte à la validation `@architect` — les renommages de fichiers sont appliqués ; le point de découpage (préfixe facultatif vs obligatoire dans le nom de fichier) peut être inversé sans impact sur les contenus.*
