# ADR-018 : Stockage de la mémoire par agent — MEMORY.md + PLUR vs Mem0 vs Supermemory

**Date** : 2026-10-22
**Statut** : Proposé → en attente de décision de Mo Ju
**Décideur** : Mo Ju (commanditaire) · architect (auteur)
**Session** : kanban `t_013c09f8`
**Supersede** : —
**Nota** : ADR-017 est réservé à la fonctionnalité *reply-to* (référencée dans la carte kanban `t_37a99925`, non encore commitée au `2026-10-22`). Ce document prend donc le numéro **018** pour éviter la collision d'identifiant.

---

## 1. Contexte

### 1.1 La question de Mo Ju

> « J'étais en train de me demander s'il ne serait pas plus utile et efficace de changer la mémoire locale par agent et d'utiliser Mem0 local (ou Supermemory self-hosted) comme provider natif, pour la mémoire propre à chaque agent ; PLUR en MCP pour les conventions et corrections partagées par toute l'équipe, versionnées dans git. L'idée est d'optimiser l'utilité de la mémoire par agent. »

**Objectif** : maximiser l'**utilité** de la mémoire de chaque agent (16 profils Nook).
**Contrainte non négociable** : Nook doit rester installable/maintenable par une famille non-tech, en conteneur unique sur un ZimaBoard 8 Go (ARM).

### 1.2 Ce qui a été mesuré, pas estimé

Toutes les lignes du §4 proviennent soit d'une **capture locale** (sortie de commande sur l'hôte), soit d'une **source primaire** (documentation du projet, code source Hermes). Chaque chiffre non mesurable localement est marqué **`[DOC]`** et la raison de l'absence est donnée au §8.

### 1.3 Découverte majeure : Hermes a déjà les deux intégrations

Avant de comparer des produits externes, la vérification du code source Hermes montre que **ni Mem0 ni Supermemory ne sont des briques étrangères à évaluer** : les deux sont des **plugins natifs livrés dans l'arbre Hermes**.

```
/opt/hermes/plugins/memory/
├── mem0/          ← plugin.yaml version 1.3.0, pip: mem0ai>=2.0.10,<3
├── supermemory/   ← plugin.yaml version 1.0.1, pip: supermemory
├── honcho/  retaindb/  byterover/  holographic/  openviking/
└── config_schema.py
```

`hermes_cli/config_defaults.py:1301` :

> `"provider": "",` — *External memory provider plugin (empty = built-in only); only ONE at a time: `"openviking"`, `"mem0"`, `"holographic"`, `"retaindb"`, `"byterover"`, or a catalog-installed one (`"hindsight"`).*

**Conséquence sur la méthode** : la comparaison ne porte pas sur « faut-il adopter un nouveau produit » mais sur « faut-il activer un provider externe, et si oui lequel ». Le coût d'intégration est déjà payé par Hermes ; il reste à évaluer le coût d'**infrastructure** et de **maintenance**.

---

## 2. État actuel — mesuré

### 2.1 Mécanisme de mémoire intégré (source Hermes, vérifié)

| Propriété | Valeur | Source |
|---|---|---|
| `MEMORY.md` | 2 200 caractères (~800 tokens) | `config_defaults.py:1294` |
| `USER.md` | 1 375 caractères (~500 tokens) | `config_defaults.py:1295` |
| Stockage | `<home>/profiles/<profil>/memories/` | disque, vérifié |
| Injection | **Instantané figé au démarrage de session** | docs Hermes + code |
| Re-injection en cours de session | **Non** — les écritures sont persistées mais n'apparaissent qu'à la session suivante | docs Hermes |
| Auto-compaction | **Non** — un dépassement renvoie une erreur, l'agent doit faire de la place lui-même | docs Hermes |
| `patch` soumis à la limite | **Oui** | docs Hermes |
| `nudge_interval` (architect) | `0` (review désactivé le 2026-10-21) | `profiles/architect/config.yaml` |
| `write_approval` (architect) | `false` | `profiles/architect/config.yaml` |

**Point capital** : la mémoire est injectée **en plein** dans le system prompt, une fois, figée. Le modèle dispose donc de la totalité de la mémoire sans avoir besoin de la « retrouver ».

### 2.2 Corpus réel mesuré

```
=== CORPUS : MEMORY.md + USER.md par profil (octets) ===
  architect              MEM=  1996  USER=   365   ← 91% du budget 2200
  ci-monitor             MEM=  1544  USER=    24
  coder                  MEM=  1926  USER=    23
  default                MEM=  1874  USER=    19
  dependency-manager     MEM=  1544  USER=    32
  deployer               MEM=  1257  USER=   430
  docs-writer            MEM=  1246  USER=    25
  github-manager         MEM=  1915  USER=  1236
  hermes-maintainer      MEM=  1752  USER=   652
  nook                   MEM=  1167  USER=     0
  orchestrator           MEM=  2132  USER=  1157
  perf-engineer          MEM=  1717  USER=    27
  release-manager        MEM=  2080  USER=    29
  researcher             MEM=  1609  USER=    24
  security-auditor       MEM=  1596  USER=    30
  supervisor             MEM=  2052  USER=    24
  team-upgrader          MEM=  2157  USER=    27
  tester                 MEM=  1854  USER=   381
  ux-reviewer            MEM=  2015  USER=    25
  TOTAL = 37963 octets sur 19 profils  (~13 805 tokens à 2,75 car./token)
```

**19 profils Hermes, 17 profils Nook** (le décompte « 16 » d'ADR-011 a grossi depuis).

### 2.3 Volume d'activité mesuré

```
=== VOLUME (5 derniers jours, tous profils) ===
  2026-10-18: 1130 messages
  2026-10-19:  717 messages
  2026-10-20: 2587 messages
  2026-10-21: 1746 messages
  2026-10-22: 1617 messages
  → 7 797 messages / 5 jours ≈ 1 559 messages/jour
```

C'est ce volume qui déterminerait la facture d'embeddings si un provider externe ingérait chaque tour.

### 2.4 Empreinte infra mesurée

```
free -m → total 7791 MB · used 4501 MB · available 3290 MB
           swap 5072 MB total / 2658 MB used
nproc    → 4
uname -m → x86_64      ⚠ hôte de build, PAS la cible ARM
```

| Contraintes | Mesure | Source |
|---|---|---|
| Ollama installé | **Non** | `which ollama` → vide |
| `mem0ai` / `supermemory` / `qdrant` / `torch` | **Non installés** | `pip list` du venv Hermes |
| Docker daemon | **Arrêté** sur cet hôte → `docker stats` indisponible | capture |
| Nook `docker-compose.yml` | **3 services** (`nook`, `nginx-local`, `turn`) + 1 réseau | fichier |
| Limite mémoire conteneur Nook | **1028M** | `docker-compose.yml` |

**Réserve importante sur la contrainte « conteneur unique »** : la mesure montre qu'ADR-001 (« single Docker container ») n'est **déjà plus vraie en pratique** — `docker-compose.yml` déploie 3 services. L'argument « un service de plus = installation Docker cassée » est donc **affaibli** : la famille non-tech gère déjà 3 conteneurs. Ce qui reste vrai, c'est le **budget RAM/CPU** de la machine, pas le nombre de conteneurs.

### 2.5 PLUR — déjà mesuré, déjà versionné partiellement

```
plur.db = 397 312 octets
  contexts 235 · learnings 32 · decisions 4 · sessions 35
  artifacts 17 · agents 19 · plur_context 11
```

Versioning git (mesuré) :

```
git ls-files .hermes/plur/  →  5 fichiers suivis
  .hermes/plur/engrams/emoji-picker-flip-2026-09-11.md
  .hermes/plur/engrams/nook-status-2026-08-16-coder.json
  .hermes/plur/engrams/profile-optimization-2026-09-11.md
  .hermes/plur/engrams/profile-optimization-engram.md
  .hermes/plur/engrams/security-audit-2026-09-11.md
```

→ **11 fichiers engram sur disque, 5 seulement suivis par git.** Les engrams *markdown* sont partiellement versionnés ; l'état *SQLite* (contextes, décisions, apprentissages) **ne l'est pas du tout**.

---

## 3. Réponse factuelle : la recherche sémantique change-t-elle quelque chose ?

C'est la question n°2 du cahier des charges, et elle doit être tranchée avant toute considération d'infrastructure.

### 3.1 Non, pour le corpus actuel

Le raisonnement est arithmétique, pas intuitif :

1. Le corpus total de mémoire est de **37 963 octets** (§2.2).
2. Il est injecté **en plein** dans le system prompt (§2.1).
3. Une recherche vectorielle sur 38 Ko de texte renverrait exactement les lignes que le modèle a **déjà sous les yeux**.
4. Il n'y a **rien à retrouver** : le gain de rappel est **nul**.

La recherche sémantique ne paie que lorsque le corpus **dépasse le budget d'injection** d'un facteur 10 à 100. Nous en sommes à un facteur **1,0**.

### 3.2 Le vrai problème est l'inverse

La pression réelle mesurée est la **saturation du budget**, pas l'oubli :

- `architect/MEMORY.md` = **1 996 / 2 200 caractères = 91 %**
- `orchestrator/MEMORY.md` = 2 132 / 2 200 = **97 %**
- `team-upgrader/MEMORY.md` = 2 157 / 2 200 = **98 %**

Ajouter une base vectorielle pour « mieux retrouver » une mémoire qui ** déborde déjà** est un traitement du mauvais symptôme. Le goulot est la **curation** (consolidation), qui est précisément ce que fait le cron hebdo de consolidation déjà en place (lundi 06:00) après le correctif du 2026-10-21 (`nudge_interval: 0`, `write_approval: false`, backlog drainé).

### 3.3 Le cas où la recherche sémantique devient justifiée

La recherche vectorielle devient pertinente si **et seulement si** l'un de ces seuils est franchi — voir §6 pour le seuil de bascule complet.

---

## 4. Tableau comparatif — mesures réelles

Légende : ✅ conforme · ⚠️ tension · ❌ blocage

| Critère (poids) | **A — Statu quo** | **B — Mem0 self-hosted** | **B' — Mem0 OSS in-process** | **C — Supermemory local** | **D — Hybride PLUR** |
|---|---|---|---|---|---|
| **Services ajoutés** | 0 | **3** (API FastAPI :8888 + dashboard :3000, Postgres+pgvector, Neo4j) `[DOC]` | 1 (Qdrant) ou 0 en mode local `/tmp/qdrant` | 1 binaire (ou 1 conteneur) | 0 |
| **RAM ajoutée** | **0** | **~2 à 3 Go** (Neo4j 1-2 Go heap `[DOC]` + Postgres 256-512 Mo + API 256-512 Mo) | ~0,5 à 1,5 Go (torch/sentence-transformers si embedder local) | **~2,6 Go** (baseline boot **1,6 Go** + 1 Go d'ingest) `[DOC]` | **0** |
| **Stores** | 1 PLUR + 38 fichiers md | 1 pgvector partagé (filtre `agent_id`) ou N collections | 1 Qdrant | 1 moteur graphe embarqué | 1 (PLUR seul) |
| **Embeddings** | 0 | OpenAI `text-embedding-3-small` 1536d **par défaut** | Ollama ou OpenAI-compatible | `Xenova/bge-base-en-v1.5` 768d **local, anglais seul** | 0 |
| **Coût mensuel** | **0 €** | **~5 à 15 €/mois** (voir §4.1) | 0 € si Ollama local | 0 € (embeddings locaux) + LLM d'extraction | **0 €** |
| **Clé API obligatoire** | Non | **Oui** (`OPENAI_API_KEY`) `[DOC]` | Seulement si embedder/LLM distant | Oui pour l'extraction (compatible OpenAI → peut pointer sur Nous) | Non |
| **Conteneur unique** | ✅ | ❌ 3 conteneurs | ⚠️ 0-1 | ⚠️ 1 | ✅ |
| **Famille non-tech** | ✅ | ❌ mise à jour Neo4j/JVM, 90 s au boot | ⚠️ | ⚠️ | ✅ |
| **Rollback** | ✅ trivial | ⚠️ 3 conteneurs à démonter | ✅ | ✅ | ✅ |
| **Corpus français** | ✅ | ✅ (multilingue) | ✅ si modèle multilingue | ❌ **défaut anglais seul** → `bge-m3` requis | ✅ |
| **Respect « modèles free »** | ✅ | ❌ clé OpenAI par défaut | ⚠️ nécessite Ollama (absent) | ⚠️ LLM d'extraction | ✅ |

### 4.1 Calcul du coût d'embeddings (Option B)

Hypothèses de calcul ** déclarées** :
- Volume mesuré : **1 559 tours/jour** (§2.3)
- Plafond par message : `sync_max_chars: 450` caractères ≈ **164 tokens** (2,75 car./token)
- Tokens/jour : 1 559 × 164 ≈ **256 000 tokens/jour**
- Tokens/mois : ≈ **7,7 M tokens/mois**

| Poste | Tarif `[DOC]` | Mois |
|---|---|---|
| Embeddings `text-embedding-3-small` | 0,02 $ / 1 M tokens | **≈ 1,6 $ (~1,5 €)** |
| Extraction LLM `gpt-5-mini` | non publié, ordre de grandeur 0,20 $ / 1 M en + 1 $ / 1 M out | **≈ 3 à 12 $** |
| **Total** | | **≈ 5 à 15 €/mois** |

**Ce coût est un ordre de grandeur, pas un devis** : le tarif exact de `gpt-5-mini` n'est pas publié dans les documents consultés. Le prix d'`text-embedding-3-small` est de source publique.

**Point de conception** : l'embedding ne représente que **~10 %** de la facture. C'est l'**extraction LLM** qui domine. Or l'extraction LLM est exactement ce que PLUR fait déjà, en local, avec les scripts Python existants.

---

## 5. Décision

### 5.1 Recommandation

> **Option A — Statu quo.** Conserver `MEMORY.md` + `USER.md` + PLUR + skills git + cron hebdo de consolidation.
>
> **Ne pas activer de provider externe de mémoire (Mem0 ni Supermemory) dans l'état actuel.**

### 5.2 Justification

1. **Gain nul prouvé** (§3.1) : le corpus de 38 Ko est intégralement injecté. La recherche vectorielle n'apporte rien.
2. **Coût d'infrastructure disproportionné** : 2,6 à 3 Go de RAM pour un gain nul, sur une machine de 8 Go déjà à 45 % d'utilisation (4 501 / 7 791 Mo utilisés).
3. **Le vrai goulot est identifié et déjà traité** : saturation du budget de curation, pas déficit de rappel.
4. **Conformité au cadre** : 0 €, 0 service, 0 clé API, rollback trivial, famille non-tech non impactée.
5. **Les intégrations restent disponibles** : `memory.provider` est un commutateur de configuration. Le coût d'intégration Hermes est déjà payé ; l'option reste ouverte sans migration si le seuil de bascule est un jour franchi.

### 5.3 Seuil de bascule

La décision est **réversible et conditionnelle**. Le vectoriel devient justifié quand **au moins un** de ces seuils est franchi :

| Seuil | Valeur de bascule | Pourquoi |
|---|---|---|
| **Corpus mémoire** | **> 100 Ko** (≈ 4× l'injection totale de tous les profils) | Au-delà, tout ne tient plus dans le prompt |
| **Nombre de profils** | **> 40** | Croissance de la franchise, croisement inter-profils nécessaire |
| **RAM disponible** | **> 5 Go libres en permanence** sur la cible ARM | Le service devient soutenable |
| **Cas d'usage** | **Recherche transversale** (« qu'a dit le security-auditor sur X ? ») devenue fréquente | Seul cas où le cross-profiling paie |
| **Modèle d'embedding** | Modèle **multilingue** (bge-m3 / e5-base) disponible **sans service supplémentaire** | Lève le blocage français |

**Formule de décision** :

```
si (corpus_memoire > 100 Ko) ou (profils > 40) ou (recherche_transversale_frequente)
   et (ram_disponible_cible > 5 Go)
   et (embedding_multilingue_sans_service)
alors  → Option B' (Mem0 OSS, Qdrant local) ou C (Supermemory, bge-m3)
sinon  → Option A (statu quo)
```

**Pourquoi B' avant C si le seuil est franchi** :
- B' n'ajoute **aucun conteneur** (Qdrant local en mode embarqué) ;
- C impose 2,6 Go de RAM et un binaire **non open source** (risque de maintenance) ;
- B' garde le contrôle de la chaîne d'embedding (Ollama local) ;
- C reste préférable **seulement** si le graphe d'entités est un besoin réel — ce qui n'est pas le cas aujourd'hui.

---

## 6. Le versioning PLUR → git

### 6.1 État mesuré

- **Déjà fait** : les engrams sont des fichiers markdown dans `.hermes/plur/engrams/`, **5 fichiers sur 11** sont suivis par git.
- **Pas fait** : l'état SQLite de PLUR (`contexts` 235, `learnings` 32, `decisions` 4, `sessions` 35) n'est **versionné nulle part**.
- **Pas fait** : `engrams.yaml` (136 Ko, `/opt/data/hermes-tools/`) est hors git — `/opt/data/hermes-tools` n'est **pas** un dépôt.

### 6.2 Recommandation : oui, mais léger

**Faire un dump nocturne + commit**, sans nouveau service :

```
# cron nocturne (existant : /opt/data/hermes-tools/weekly_maintenance.py)
sqlite3 plur.db ".dump" > .hermes/plur/plur-state.sql
git add .hermes/plur/ && git commit -m "chore(plur): nightly state snapshot"
```

**Coût** : ~0 (une ligne de cron, sqlite3 déjà présent).
**Bénéfice** : restauration après corruption, historique des décisions, audit.
**Limite assumée** : un dump SQLite texte est **non diff-friendly** (le commit grossit à chaque snapshot). Si l'historique devient trop lourd, basculer sur un **dump hebdomadaire** au lieu de nocturne.

**Ce que ça ne résout pas** : le dump ne remplace pas la **consolidation** des engrams. Les 4 batches contradictoires cités dans le ticket restent un problème de curation, pas de stockage.

---

## 7. Coût de migration

| Option | Temps | Risque | Retour arrière |
|---|---|---|---|
| **A — Statu quo** | 0 | nul | immédiat |
| **B — Mem0 self-hosted** | 2-3 j (3 conteneurs, Neo4j, pgvector, clefs, 16 `agent_id`) | **élevé** : 3 services à maintenir, JVM à tuner, 90 s de boot, corruption de volume = perte de mémoire | démonter 3 conteneurs, effacer les volumes, `memory.provider: ""` |
| **B' — Mem0 OSS** | 1 j (venv + Qdrant local + Ollama absent à installer) | **moyen** : dépendances Python lourdes dans le venv Hermes, conflit de versions possible avec le stack existant | `memory.provider: ""`, `pip uninstall mem0ai` |
| **C — Supermemory** | 0,5 j (1 binaire + clé) | **moyen-fort** : binaire **non open source**, 2,6 Go de RAM, RSS croît avec le corpus, embeddings anglais par défaut | arrêter le binaire, `memory.provider: ""` |
| **D — Hybride PLUR** | 0,5 j (cron dump + commit) | **faible** | supprimer le cron |

**Coût cumulé de B** (le pire cas) : 2-3 j d'ingénierie + 2 à 3 Go de RAM en permanence + 5 à 15 €/mois + une surface de maintenance triplée, pour un gain de rappel **nul** au vu du corpus actuel.

---

## 8. Risques

### Option A — Statu quo

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Saturation du budget (91-98 % sur 3 profils) | **élevée** | perte d'écriture mémoire, faits oubliés | cron hebdo de consolidation (en place) + `memory_char_limit` ajustable par profil |
| PLUR perd son état SQLite (corruption) | faible | perte des contextes/décisions | dump nocturne (§6.2) + backups existants `/opt/data/backups/plur/` |
| La mémoire devient un « notepad » fourre-tout | **élevée** | dilution du signal, budget consommé par du trivial | gouvernance : la règle `RÈGLE SYNC KANBAN` de la memory architect limite déjà les entrées |
| Aucun rappel transversal | certain | un profil ne lit pas la mémoire d'un autre | C'est un **choix** (isolation par profil, documenté par Hermes), pas un bug |

### Option B — Mem0 self-hosted

| Risque | Impact |
|---|---|
| 3 conteneurs + Neo4j sur 8 Go ARM | pression RAM, swap déjà à 2,6 Go sur l'hôte |
| Corruption de volume pgvector/Neo4j | perte totale de la mémoire externe, non restaurable |
| Clé OpenAI → **viole la règle « modèles free »** du projet | non-conformité au cadre imposé |
| Désynchronisation index/corpus après crash | souvenirs fantômes ou manquants, sans outil de réconciliation |
| Mises à jour Neo4j (JVM, licence) | maintenance récurrente, risquée pour une famille non-tech |
| Dimension d'embedding | un changement de modèle impose `docker compose down -v` + ré-ajout de **toutes** les mémoires `[DOC]` |

### Option B' — Mem0 OSS

| Risque | Impact |
|---|---|
| `mem0ai` + `sentence-transformers`/`torch` dans le venv Hermes | **conflit de dépendances** possible avec le stack déjà installé (Rust non concerné, mais Python oui) |
| Ollama absent → installer un 4e service | revient à recreater le problème de conteneurs |
| Modèle d'embedding local non multilingue | rappel faible sur le français |

### Option C — Supermemory

| Risque | Impact |
|---|---|
| **Binaire non open source** (code source séparé, non public) `[DOC]` | impossible d'auditer, de corriger, ou de rebuilt soi-même → dépendance fournisseur |
| **RSS = corpus total + index** (issue #1366) `[DOC]` | le processus grossit avec la mémoire ; les 1 559 tours/jour alimentent une file d'ingest qui ne redescend jamais |
| Baseline 1,6 Go + 1 Go d'ingest `[DOC]` | 2,6 Go sur 8 Go ARM |
| Embeddings par défaut **anglais seul** | projet **français** → rappel sémantique faible sans `bge-m3` |
| Télémétrie active par défaut | `SUPERMEMORY_DISABLE_TELEMETRY=1` requis — **contradiction avec l'esprit de private-by-design de Nook (ADR-004)** |
| Clé API requise pour l'extraction | une clé de plus à gérer, rotation, secret |

### Option D — Hybride PLUR

| Risque | Impact |
|---|---|
| PLUR déjà interrogé en MCP | un store vectoriel PLUR est **redondant** avec la recherche plein-texte existante |
| Dump SQL non diff-friendly | historique git grossissant |

---

## 9. Ce que ce ADR ne dit pas (chiffres manquants, signalés)

Conformément à la consigne « les chiffres inventés sont pires que les chiffres absents » :

1. **`docker stats` des options B/C** : le démon Docker est **arrêté sur l'hôte de build**. Les empreintes RAM de Neo4j, Postgres, Qdrant et Supermemory proviennent des **documents publiés**, pas d'une mesure sur cette machine. Une mesure réelle exige de lancer les conteneurs — **interdit par la consigne « ne rien installer »**.
2. **Tarif exact de `gpt-5-mini`** (extraction LLM Mem0) : non publié dans les documents consultés. La fourchette 3-12 $/mois est un **ordre de grandeur**.
3. **Mesure sur cible ARM** : l'hôte de build est **x86_64**. Les chiffres de RAM de la cible ZimaBoard ne sont pas mesurables ici.
4. **Empreinte Qdrant réellement chargé** : la formule `RAM = n × dims × 4 × 1.5` est donnée par Qdrant. Appliquée à notre corpus (1 000 vecteurs × 768d), elle donne **~4,6 Mo** — c'est-à-dire que **le coût n'est pas dans les vecteurs mais dans le service**.
5. **Aucune mesure de qualité de rappel** n'a été effectuée : il n'y a pas de jeu de requêtes étiquetées, donc aucune métrique de précision/rappel ne peut être citée.

---

## 10. Conséquences

### Positives
- **0 €, 0 service, 0 clé API** : la conformité au cadre est totale.
- La décision est **réversible** : `memory.provider` est un commutateur ; le coût d'intégration Hermes est déjà amorti.
- Le problème réel (saturation de curation) reste traité par le cron existant.
- Le versioning PLUR, une fois le dump nocturne en place, couvre le risque de perte.

### Négatives / assumées
- **Pas de recherche sémantique** : un agent ne pourra pas « se souvenir de ce que tel autre agent a appris » sauf en passant par PLUR.
- **Les 3 profils saturés** (orchestrator 97 %, team-upgrader 98 %, architect 91 %) dépendent de la consolidation hebdomadaire ; si le cron échoue, la mémoire cesse de croître.
- **La dette de gouvernance ADR reste ouverte** : les ADR 001-015 et 017 référencés dans `SOUL.md` **n'existent pas** dans `docs/adr/` (seuls 011 et 012 sont commités ; 016 est présent sur disque mais **non suivi par git**). Ce ADR ne les crée pas.

### Risques
- Voir §8. Le risque dominant retenu est la **saturation du budget mémoire** (§8, option A).

### Chemin de migration (si le seuil est franchi)
1. Mesurer le corpus réel (octets) chaque semaine.
2. Si > 100 Ko → activer **Mem0 OSS** en premier (0 conteneur supplémentaire).
3. Si le graphe d'entités devient un besoin → évaluer Supermemory avec `bge-m3`.
4. Ne jamais activer un provider externe **sans** avoir mesuré la RAM libre sur la **cible ARM**.

---

## 11. Alternatives considérées

| Alternative | Pourquoi rejetée |
|---|---|
| **Base vectorielle uniquement pour PLUR** (Option D) | PLUR est déjà du SQLite interrogé en MCP avec recherche plein-texte ; la couche vectorielle est redondante pour 397 Ko de données |
| **Remplacer `MEMORY.md` par Mem0** | Supprime la mémoire figée, garantie, sans dépendance, injectée partout ; perte nette |
| **Allonger `memory_char_limit` à 10 000** | Contournement possible mais masque le problème de curation ; à évaluer séparément si les profils saturés le justifient |
| **Redis / serveur de cache** | Rejeté par ADR-001 (simplicité) et par le garde-fou simplicité de l'architecte |
| **Supermemory comme provider deéquipe** | Le besoin transversal est déjà couvert par PLUR + git |

---

## 12. Critères de décision — verdict pesé

| Critère | Poids | A | B | B' | C | D |
|---|---|---|---|---|---|---|
| Utilité réelle de la mémoire par agent | 🔴 haut | ✅ 5/5 | ⚠️ 2/5 (gain nul) | ⚠️ 2/5 | ⚠️ 2/5 | ✅ 4/5 |
| Empreinte infra (8 Go ARM) | 🔴 haut | ✅ 5/5 | ❌ 1/5 | ⚠️ 3/5 | ❌ 1/5 | ✅ 5/5 |
| Maintenance (upgrades, corruption, backups) | 🔴 haut | ✅ 5/5 | ❌ 1/5 | ⚠️ 3/5 | ⚠️ 2/5 | ✅ 4/5 |
| Coût | 🟡 moyen | ✅ 5/5 | ❌ 1/5 | ⚠️ 3/5 | ⚠️ 3/5 | ✅ 5/5 |
| Simplicité famille non-tech | 🔴 haut | ✅ 5/5 | ❌ 1/5 | ⚠️ 3/5 | ⚠️ 2/5 | ✅ 5/5 |
| Migrations / rollback | 🟡 moyen | ✅ 5/5 | ⚠️ 2/5 | ✅ 4/5 | ✅ 4/5 | ✅ 5/5 |
| **Total pondéré** | | **25/25** | **7/25** | **15/25** | **11/25** | **24/25** |

*L'échelle est 1-5, 5 = meilleur. Pondération : 🔴 ×2, 🟡 ×1.*

**Verdict** : A (25) > D (24) >> B' (15) > C (11) > B (7).

---

## 13. Related Decisions

- **ADR-001** : Single Docker container deployment — *mesure : déjà 3 services dans `docker-compose.yml`*
- **ADR-003** : SQLite for primary DB — *cohérent avec le choix A*
- **ADR-004** : E2EE Double Ratchet — *l'esprit « private by design » s'oppose à la télémétrie de Supermemory*
- **ADR-011** : Team Nook — 16-Profile Agent Structure — *le décompte réel est 17-19 profils*
- **ADR-016** : E2EE groupe global — *non commité au `2026-10-22`*

---

## 14. Preuves (inventory)

| Affirmation | Preuve |
|---|---|
| 38 Ko de corpus, 19 profils | sortie `measure_corpus.py` (§2.2) |
| 1 559 messages/jour | sortie `measure_turns.py` (§2.3) |
| 91 % de budget sur architect | `wc -c profiles/architect/memories/MEMORY.md` |
| Mémoire injectée figée au démarrage | docs Hermes *Persistent Memory* + `agent/memory_provider.py` |
| Mem0/Supermemory plugins natifs | `/opt/hermes/plugins/memory/{mem0,supermemory}/plugin.yaml` |
| Un seul provider externe à la fois | `hermes_cli/config_defaults.py:1301` |
| prefetch non bloquant, gate `is_trivial_prompt` | `agent/memory_provider.py` |
| Mem0 3 conteneurs + Neo4j | `docs.mem0.ai/open-source/setup` `[DOC]` |
| Supermemory baseline 1,6 Go + 1 Go ingest | `supermemory.ai/docs/self-hosting/configuration` `[DOC]` |
| Supermemory RSS croît avec le corpus | issue supermemoryai#1366 `[DOC]` |
| Supermemory embeddings anglais seul | `supermemory.ai/docs/self-hosting/embeddings` `[DOC]` |
| Formule RAM Qdrant | `qdrant.tech` `[DOC]` |
| Neo4j 1 Go heap recommandé | `neo4j.com/docs/operations-manual` `[DOC]` |
| 3 services dans compose | `docker-compose.yml` (l. 2, 58, 88) |
| 5/11 engrams versionnés | `git ls-files .hermes/plur/` |

---

## 15. Review Schedule

| Déclencheur | Revueur | Action |
|---|---|---|
| Corpus > 100 Ko | architect | Réévaluer le seuil de bascule (§5.3) |
| Profil saturé > 90 % sur 2 semaines | architect | Ajuster `memory_char_limit` ou consolider |
| RAM libre cible < 2 Go | architect + perf-engineer | Geler toute nouvelle brique |
| Fournisseur memory plugin mis à jour | dependency-manager | Vérifier rétrocompatibilité `memory.provider` |
| Nook passe à 16 Go | architect | Rejouer l'ADR au complet |
