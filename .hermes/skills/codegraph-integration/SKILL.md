---
name: codegraph-integration
description: Intégrer CodeGraph v0.9.x comme MCP Hermes pour navigation symbolique, callees/callers et recherche cross-runtime.
---
# codegraph-integration

## Quick start
```
MCP: mcp_servers.codegraph
Index: /opt/data/Nook/
```

## Tools MCP disponibles
- `codegraph_status` — santé de l'index
- `codegraph_files` — arborescence filtrée
- `codegraph_search` — recherche symbolique par nom
- `codegraph_node` — détail d'un symbole
- `codegraph_callers` / `codegraph_callees` — dépendances entrantes/sortantes
- `codegraph_impact` — radius d'impact d'un changement
- `codegraph_context` — tout-en-un (recommandé en entrée)

## Usage patterns
1. Toute question "comment X marche" → appeler `codegraph_context` en premier
2. Pour inspecter plusieurs symboles liés → `codegraph_explore`
3. Vérifier l'avant/après d'un refactor → `codegraph_impact`

## Pitfalls
- L'index doit être régénéré après un `git pull` massif
- `codegraph_context` est coûteux en tokens: limiter `maxNodes` quand c'est possible
