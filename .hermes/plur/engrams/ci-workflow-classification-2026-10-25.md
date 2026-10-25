---
type: procedural
scope: global
domain: nook.docs-writer.ci-workflow-classification
tags: [nook, docs-writer, 2026-10-25, workspace-protocol, pr48, test-turn, workflow-classification, ci]
statement: "Un document qui décrit les workflows CI ne doit pas classer un workflow par son nom ou son intention (« build/test ») mais par le grep de ce qu'il contient réellement : « test-turn.yml » a été documenté comme « sans écriture » alors qu'il commit et force-pousse .hermes/TURN-REPORT.md sur la branche courante (.github/workflows/test-turn.yml:363) — mesuré 2026-10-25 (PR#48, revue @architect round 5, docs/workspace-protocol.md règle 6). Règle : pour tout workflow, la classification lecture-seule / à-écriture se produit par `grep -c 'git push' <workflow>.yml` (>0 = à écriture) et la ligne exacte par `grep -n 'git push --force-with-lease' <workflow>.yml`, et le « sans filtre branches: » par extraction du bloc `on:` seul — jamais par inférence. Deux workflows homonymes (turn.yml vs test-turn.yml) déclenchés par le même chemin ont des comportements opposés : les documenter sur des puces séparées, avec la commande de preuve dans la même cellule."
---

Preuve empirique (clone jetable + remote local, 2026-10-25) : branche `w/*` dont le commit
touche `services/turn-rs/**` → le step « Commit report » de `test-turn.yml` committe
`.hermes/TURN-REPORT.md` (`ff4319b5`) et le force-push envoie ce commit sur
`refs/heads/w/…` → le workflow **réécrit** la branche (violation de la règle 5 du protocole).
`turn.yml`, déclenché par le même chemin, ne contient aucun `git push` → lecture seule.
