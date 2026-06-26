---
name: codegraph-integration
description: Intégrer CodeGraph v0.9.x comme MCP server dans Hermes Agent et initialiser les projets indexés.
---

# CodeGraph Integration — Hermes Agent

## Prérequis
- `codegraph` installé : `/usr/local/bin/codegraph` (v0.9.4+)
- Config Hermes : `/opt/data/home/.hermes/config.yaml` (ou `~/.hermes/config.yaml`)

### ⚠️ Deux fichiers config.yaml sur cette machine
`codegraph install --target=hermes` écrit dans DEUX fichiers. Vérifie les deux :

```bash
grep -A8 mcp_servers /opt/data/home/.hermes/config.yaml
grep -A8 mcp_servers /root/.hermes/config.yaml
```

Règle : toujours vérifier le fichier chargé par le processus Hermes en cours, pas seulement l'un des deux.

## Procédure

### 1. Vérifier installation
```bash
which codegraph && codegraph --version
# → /usr/local/bin/codegraph, v0.9.4
```

### 2. Ajouter le MCP server dans Hermes
```bash
codegraph install --target=hermes --yes
```
Écrit automatiquement `mcp_servers.codegraph` dans `~/.hermes/config.yaml` :

```yaml
mcp_servers:
  codegraph:
    command: codegraph
    args:
      - serve
      - --mcp
    timeout: 120
    connect_timeout: 60
    enabled: true
```

### 3. Redémarrer Hermes pour charger le serveur MCP
Toute modification de `mcp_servers` nécessite un redémarrage de la session Hermes.

### 4. Initialiser les projets indexés
```bash
cd /chemin/vers/mon-projet
codegraph init -i   # initialise + indexe
codegraph status    # vérifier stats
```

**Note** : Si `codegraph init -i` échoue avec `No such built-in module: node:sqlite` :
- Une version précédente de l'index (ex: 0.9.3 via npm npx) existe déjà et fonctionne
- La version standalone 0.9.4 peut lire l'index existant (query OK) mais pas le réinitialiser
- Solution : conserver l'index existant ou utiliser npx vieille version

### 5. Index existant (cas fréquent)
C'est normal qu'un `.codegraph/` existe déjà avec une ancienne version de codegraph.
`codegraph status` indique si l'index est à jour :

```
Project: /opt/data/Nook
Files:     153
Nodes:     2,414
Edges:     5,862
Pending Changes: Modified files (run sync to update)
```

### 6. Typage MCP outils exposés à Hermes
Une fois chargé, Hermes dispose implicitement des outils CodeGraph pour toute session dans un projet indexé :
- `codegraph_search` — recherche de symboles
- `codegraph_context` — contexte pour une task
- `codegraph_callers` / `codegraph_callees` — flot d'appels
- `codegraph_impact` — rayon d'impact
- `codegraph_node` — détail d'un symbole
- `codegraph_explore` — exploration groupée
- `codegraph_trace` — chemin d'appel entre deux symboles

## Pièges
- **Config Hermes par défaut** : `/opt/data/home/.hermes/config.yaml` (≠ `~/.hermes/config.yaml`)
- **FTS5 warning** : expérimental sur Node < 22.5, mais fonctionne sur des index existants
- **Indexe en profondeur** : `codegraph sync` après changements majeurs, sinon atomicité non garantie
- **Multi-versions** : ne pas mixer npx (v0.9.3) et standalone (v0.9.4) sur le même projet sans vérifier la compatibilité DB

## Vérification rapide
```bash
# Status (toujours depuis le répertoire du projet)
cd /opt/data/Nook && codegraph status

# Test query
codegraph query sendMessage --json

# MCP serve (finger-touch, kill immédiat)
timeout 2 codegraph serve --mcp

# Vérifier les deux fichiers config Hermes
grep -A8 mcp_servers /opt/data/home/.hermes/config.yaml
grep -A8 mcp_servers /root/.hermes/config.yaml
```

## Chaîne d'invocation validée (2026-05-24)
```bash
# 1. Installer le serveur MCP dans Hermes (écrit les deux configs)
codegraph install --target=hermes --yes

# 2. Vérifier que le bloc mcp_servers a été écrit
grep -A8 mcp_servers /opt/data/home/.hermes/config.yaml

# 3. Test query (depuis le répertoire projet)
cd /opt/data/Nook && codegraph query sendMessage --json

# 4. Test MCP serve (détecte si port déjà utilisé)
cd /opt/data/Nook && timeout 2 codegraph serve --mcp && echo MCP_OK || echo MCP_FAIL
```
