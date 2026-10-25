---
type: procedural
scope: global
domain: nook.architect.workspace-protocol-review
tags: [nook, architect, 2026-10-25, workspace-protocol, pr48, review, git-fsck, plur-convention]
statement: "Revue de doc Nook : une cellule de tableau qui cite une commande de mesure doit être reproductible par CETTE commande exacte — un lecteur qui l'exécute doit obtenir le nombre affiché. Mesuré 2026-10-25 (PR#48, docs/workspace-protocol.md l.30) : la cellule citait `git fsck --dangling | awk '$2==\"commit\"' | wc -l` en affichant « 22 (avec reflogs) ; 33 (--no-reflogs) », mais la commande citée renvoie 22, `git fsck --no-reflogs --dangling` renvoie 34, et le 33 n'est obtenable que par `git fsck --unreachable` (non cité) — la valeur 33 était donc invérifiable par la commande du doc, ce qui a motivé un REQUEST_CHANGES. Réflexe de revue : exécuter chaque commande imprimée dans le doc (pas seulement les blocs de snippet) et comparer sa sortie au nombre affiché dans la même cellule."
---
