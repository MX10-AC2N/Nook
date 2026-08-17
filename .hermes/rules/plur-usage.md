# Convention d'utilisation PLUR

> standardisée pour que tous les agents Nook y écrivent/lisent de façon cohérente, et que la disponibilité du MCP PLUR soit surveillée.

## Quand écrire un engram

| Situation | Moment | Exemple |
|-----------|--------|---------|
| **Fin de tâche Kanban** | 1 engram minimum à la fin de chaque tâche | `type=terminological, scope=global, domain=nook.docs-writer.usage, tags=[nook, docs-writer, 2026-08-17, plur-convention]` |
| **Découverte importante** | Immédiatement, dès la découverte | `type=architectural, scope=global, domain=nook.architecture.sfu, tags=[nook, architect, 2026-08-17, sfu-pitfall]` |
| **Erreur ou correction** | Engram de type `procedural` | `type=procedural, scope=global, domain=nook.coder.debug, tags=[nook, coder, 2026-08-17, rate-limit-fix]` |

### Format d'engram

```yaml
type: behavioral | terminological | procedural | architectural
scope: global
domain: nook.<domaine>.<sujet>
tags: [nook, <profil>, YYYY-MM-DD, <sujet>]
statement: déclaration concise de la conclusion/décision/erreur (max 1 sentence)
```

**Exemples concrets :**

```yaml
# Conclusion de tâche
type: terminological
scope: global
domain: nook.docs-writer.plur-usage
tags: [nook, docs-writer, 2026-08-17, plur-convention]
statement: "Règle PLUR standardisée : 1 engram minimum à la fin de chaque tâche Kanban"

# Découverte importante
type: architectural
scope: global
domain: nook.architecture.webrtc
tags: [nook, architect, 2026-08-17, webRTC-sfu]
statement: "L'API SFU rustrtc utilise PeerConnection::new(config) à la valeur, pas par référence"

# Erreur de correction
type: procedural
scope: global
domain: nook.coder.debug
tags: [nook, coder, 2026-08-17, rate-limit-fix]
statement: "Vérifier plur_status.engram_count > 0 avant écriture ; si MCP down, relancer watchdog"
```

## Quand relire un engram

| Situation | Action |
|-----------|--------|
| **Début de tâche Kanban** | Avant de commencer, appeler `plur_recall(query=tache, scope=global)` pour injecter le contexte des tâches précédentes |
| **Recherche d'information** | Chercher dans PLUR avec la query de la tâche pour éviter de réinventer ce qui a déjà été découvert |
| **Validation de décision** | Vérifier qu'un décision similaire n'a pas déjà été prise (search similarity before écriture) |

## Disponibilité du MCP PLUR

Avant d'écrire un engram :

1. Vérifier `plur_status.engram_count > 0` — si 0, le MCP est peut-être down
2. Si MCP down : relancer le watchdog (`kill` du processus + laisser le gateway le relancer) ou alerter l'orchestrateur
3. Ne jamais stocker de secrets/tokens dans un engram
4. Effectuer une recherche de similarité avant écriture pour éviter la duplication

## Interdictions

- **Ne pas stocker** de secrets, tokens, ou données sensibles dans les engrams
- **Ne pas dupliquer** un engram existant sans vérifier la similarity search d'abord
- **Ne pas écrire** si `plur_status.engram_count` n'est pas vérifiable et que le MCP est down

## Workflow PLUR standard

```text
1. FIN DE CHAQUE TÂCHE KANBAN :
   - Écrire 1 engram minimum (conclusion, décision importante, ou engram procedural pour une correction)
   - Format: type + scope + domain + tags + statement concise
   - Vérifier plur_status.engram_count > 0 avant écriture

2. DÉBUT DE CHAQUE NOUVELLE TÂCHE KANBAN :
   - Appeler plur_recall(query=tache, scope=global) pour injecter le contexte
   - Relire les engrams pertinents pour éviter les répétitions

3. SURVEILLANCE :
   - Vérifier régulièrement la disponibilité du MCP PLUR
   - Alerter l'orchestrateur si MCP down et que des engrams étaient attendus
```