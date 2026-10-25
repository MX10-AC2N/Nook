---
type: note
scope: global
domain: nook.docs.review
tags: [workspace-protocol, pr48, review, reflog, live-vs-snapshot]
statement: Les cellules « live » d'un doc de mesures ne doivent contenir que des valeurs reproductibles par la commande affichée dans la même cellule — un reflog réécrit par git gc rend toute entrée historique non reproductible.
---

# Revue round 6 — `docs/workspace-protocol.md` (PR#48, carte t_35ea07c7)

Rejeté (REQUEST_CHANGES) le 2026-10-25, run 995.

## Le point bloquant

Le doc (v6, `ab417a4d`, 25968 o) affiche en **live** :

    git reflog --date=short | grep -i checkout | head -2
    -> checkout vers origin/fix/e2ee-admin-gate-t_83868105 le 2026-10-24

Mesure indépendante (2026-10-25) :

    21b0d93a HEAD@{2026-10-25}: checkout: moving from develop to develop
    21b0d93a HEAD@{2026-10-25}: checkout: moving from develop to develop

L'entrée historique a disparu du reflog — réécriture par `git gc` (le doc le documente lui-même :
171 → 152 entrées). La seconde moitié de la cellule (« moving from 8050ed53 to develop ») reste
reproductible.

## Leçon

Tout `git gc` réécrit les reflogs et purge les objets non référencés. Une valeur dérivée du reflog
ou de `fsck --dangling` **n'est jamais stable** : la marquer « live » garantit qu'elle deviendra
fausse. Règle de classement, appliquée par ce doc (lignes 44-50) mais pas partout :

* **live** — la commande affichée dans la cellule reproduit la valeur affichée *au moment de la lecture*.
* **snapshot ⏱️** — datée, commande conservée, cause de non-reproductibilité explicite.

Un compteur ou une entrée historique dont la commande ne reproduit plus la sortie doit être déplacé
en snapshot (ou supprimé) au même amendement — pas laissé en live.

## Contrôle reproductible

    git reflog --date=short | grep -i checkout | head -2
    git reflog --date=short | grep -i checkout | grep -c 'moving from 8050ed53 to develop'
