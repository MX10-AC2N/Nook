# 🤖 Agent DELEGATE — Nook

> Routeur vers les IAs gratuites pour les tâches mécaniques bien définies.
> Activer quand une tâche est isolée, spécification complète, et ne nécessite pas
> de raisonnement sur le contexte global du projet.
> **Objectif : réserver Claude Sonnet aux tâches où il est irremplaçable.**

---

## 🎯 Principe de décision

```
La tâche peut être déléguée si :
  ✅ Le fichier d'entrée est fourni en entier (pas besoin de fetcher GitHub)
  ✅ Le résultat attendu est vérifiable sans connaissance du projet
  ✅ Une erreur dans le résultat est détectable en < 2 minutes (compile, lint, test)
  ✅ Aucun piège Nook spécifique n'est impliqué (rand_core, axum 0.8, Svelte 5 runes…)

La tâche RESTE chez Claude Sonnet si :
  ❌ Elle nécessite de lire BUGS.md (risque de régression)
  ❌ Elle touche auth, crypto, ou sécurité
  ❌ La cause racine est inconnue
  ❌ Plusieurs fichiers sont impliqués et liés
  ❌ Une décision d'architecture est requise
```

---

## 🗂️ Taxonomie des tâches — 3 niveaux

### 🟢 NIVEAU 1 — Délégable sans contexte projet
> Spécification auto-suffisante. L'IA externe n'a pas besoin de connaître Nook.

| Tâche | IA recommandée | Prompt type |
|-------|---------------|-------------|
| Struct Rust → Interface TypeScript | Gemini Flash / GPT-4o mini | `CONV-RUST-TS` |
| Migration SQL à partir d'un schéma | Gemini Flash | `GEN-SQL-MIGRATION` |
| Nouveau test Playwright (scénario fourni) | GPT-4o mini | `GEN-E2E-TEST` |
| Queries SQL analytics (COUNT, GROUP BY) | Gemini Flash | `GEN-SQL-ANALYTICS` |
| Composant Svelte simple (spec fournie) | Mistral / GPT-4o mini | `GEN-SVELTE-COMPONENT` |
| YAML docker-compose (spec fournie) | Gemini Flash | `GEN-DOCKER-YAML` |
| Commentaires/docs d'une fonction | Gemini Flash | `GEN-DOCS` |
| Générer types TS depuis JSON example | GPT-4o mini | `GEN-TYPES-FROM-JSON` |
| Version bump Cargo.toml / package.json | Manuel (30 sec) | — |

### 🟡 NIVEAU 2 — Délégable avec contexte minimal (1 fichier fourni)
> Fournir le fichier complet + la règle spécifique dans le prompt.

| Tâche | IA recommandée | Prompt type |
|-------|---------------|-------------|
| Corriger warning clippy (fichier + warning fournis) | Gemini Flash | `FIX-CLIPPY` |
| Ajouter un champ à une struct + migration | Gemini Flash | `ADD-FIELD` |
| Étendre un store Svelte $state (pattern fourni) | GPT-4o mini | `EXTEND-STORE` |
| Adapter un test E2E (nouveau sélecteur) | GPT-4o mini | `FIX-SELECTOR` |
| Endpoint CRUD basique (pattern existant fourni) | Gemini Flash | `GEN-CRUD` |

### 🔴 NIVEAU 3 — Claude Sonnet obligatoire
> Contexte projet complet, raisonnement multi-fichiers, ou pièges Nook impliqués.

- Debug cause racine inconnue
- Toute tâche impliquant auth / crypto / E2EE
- Résolution conflits de dépendances Rust
- Analyse logs CI pour identifier la vraie cause
- Feature cross-agents (≥ 2 domaines)
- Décisions architecturales
- Toute tâche où BUGS.md doit être consulté

---

## 🤖 IAs gratuites — Capacités et limites

| IA | Forces sur Nook | Limites | Accès |
|----|----------------|---------|-------|
| **Gemini Flash 2.0** | Rust, SQL, YAML, gros fichiers (1M tokens) | Svelte 5 Runes moins solide | gemini.google.com |
| **GPT-4o mini** | TypeScript, Svelte, Playwright | Fenêtre contexte plus petite | chatgpt.com |
| **Mistral Le Chat** | Code généraliste, rapide | Moins précis sur Rust avancé | chat.mistral.ai |
| **Llama (Meta AI)** | Tâches simples, JSON | Rust limité | meta.ai |

> **Règle pratique** : Gemini Flash pour tout ce qui est Rust/SQL/YAML. GPT-4o mini pour TypeScript/Svelte/Playwright.

---

## 📋 PROMPTS PRÊTS À L'EMPLOI

Les prompts ci-dessous sont **auto-suffisants** : copier-coller + remplir les `[...]`.

---

### `CONV-RUST-TS` — Struct Rust → Interface TypeScript

```
Convertis cette struct Rust en interface TypeScript.
Règles de conversion :
- snake_case Rust → camelCase TypeScript
- String → string
- i32/i64/u32/u64 → number
- f32/f64 → number
- bool → boolean
- Option<T> → T | null
- Vec<T> → T[]
- DateTime (chrono) → string (ISO 8601)
- Uuid → string

Struct Rust :
[COLLER LA STRUCT ICI]

Retourne uniquement l'interface TypeScript, sans explication.
```

---

### `GEN-SQL-MIGRATION` — Générer une migration SQLite

```
Génère une migration SQL SQLite pour créer la table suivante.
Contraintes : SQLite uniquement (pas de UUID natif → TEXT PRIMARY KEY),
timestamps en INTEGER (Unix epoch ms), clés étrangères avec REFERENCES.

Table à créer :
[DÉCRIRE LA TABLE : nom, colonnes, types, contraintes, relations]

Format attendu :
-- NNN_nom_migration.sql
CREATE TABLE IF NOT EXISTS ...;
CREATE INDEX IF NOT EXISTS ...;
```

---

### `GEN-E2E-TEST` — Nouveau test Playwright

```
Écris un test Playwright TypeScript pour le scénario suivant.

Framework : Playwright @playwright/test
Base URL : http://localhost:6300
Auth : via cookie (les helpers loginAs/clearSession sont déjà définis, tu peux les appeler)

Helpers disponibles :
- clearSession(page) : révoque le token serveur + vide les cookies
- loginAs(page, username, password) : clearSession + goto('/login') + waitFor('#username') + fill + submit
- loginAsAdmin(page) : login admin via API (page.request.post)

Scénario à tester :
[DÉCRIRE LE SCÉNARIO ÉTAPE PAR ÉTAPE]

Éléments UI disponibles :
[LISTER LES id= ET data-testid= PERTINENTS]

Résultat attendu :
[CE QUI DOIT ÊTRE VRAI À LA FIN DU TEST]

Retourne uniquement le bloc test(...) TypeScript, sans import ni helpers.
```

---

### `GEN-SQL-ANALYTICS` — Requêtes analytics SQLite

```
Écris des requêtes SQL SQLite pour les métriques suivantes.
Schéma disponible :
- users(id, username, approved, created_at INTEGER epoch_ms)
- messages(id, conversation_id, sender_id, content, created_at INTEGER epoch_ms)
- conversations(id, name, is_group, created_at INTEGER epoch_ms)
- polls(id, question, created_by, created_at INTEGER epoch_ms, is_closed)
- poll_votes(id, poll_id, option_id, user_id, voted_at INTEGER epoch_ms)

Métriques à calculer :
[LISTE DES MÉTRIQUES : ex "users actifs 7 derniers jours", "messages par jour", etc.]

Retourne uniquement les requêtes SQL avec un commentaire -- pour chaque métrique.
```

---

### `FIX-CLIPPY` — Corriger un warning clippy

```
Corrige ce warning Clippy Rust dans le fichier suivant.
Ne modifie que ce qui est nécessaire pour supprimer le warning.
Ne change pas la logique métier.

Warning :
[COLLER LE WARNING COMPLET AVEC FICHIER:LIGNE]

Fichier complet :
[COLLER LE FICHIER .rs]

Retourne le fichier complet corrigé.
```

---

### `GEN-CRUD` — Endpoint CRUD basique (pattern Axum 0.8)

```
Génère un handler Rust Axum 0.8 pour un endpoint CRUD selon ce pattern existant :

Pattern de référence (à reproduire) :
- Route : {param} (pas :param)
- Auth : extract CurrentUser depuis Extension
- DB : sqlx::query_as::<_, T>("SQL").bind(...).fetch_all(&pool).await
- Erreur : StatusCode::INTERNAL_SERVER_ERROR sur erreur DB
- Réponse : Json<T>

Endpoint à créer :
[MÉTHODE] [URL] — [DESCRIPTION]
Payload entrée : [JSON]
Réponse succès : [JSON + CODE HTTP]
Table DB concernée : [NOM TABLE + COLONNES UTILISÉES]

Retourne uniquement la fonction handler async Rust, sans les imports ni le router.
```

---

### `EXTEND-STORE` — Étendre un store Svelte 5

```
Étends ce store Svelte 5 avec les nouvelles propriétés/actions demandées.
Règles Svelte 5 obligatoires :
- $state exporté → JAMAIS de réassignation directe, muter via propriété
- Pas de writable()/readable() Svelte 4
- Pas de $derived/$effect dans ce fichier .svelte.ts (hors composant)

Store actuel :
[COLLER LE FICHIER .svelte.ts]

Ajouts demandés :
[DÉCRIRE LES NOUVELLES PROPRIÉTÉS ET ACTIONS]

Retourne le fichier complet mis à jour.
```

---

## ⚡ Workflow de délégation recommandé

```
1. Claude Sonnet identifie la tâche comme déléguable (Niveau 1 ou 2)
2. Claude Sonnet produit le prompt prêt à copier (depuis les templates ci-dessus)
   en remplissant les [...] avec le contexte exact
3. Tu copies le prompt dans l'IA gratuite choisie
4. Tu copies le résultat et le fournis à Claude Sonnet pour :
   - Vérification rapide (cohérence avec les règles Nook)
   - Intégration dans le fichier final si OK
```

> **Gain typique** : les tâches Niveau 1 représentent ~30% du volume
> mais seulement ~10% de la valeur ajoutée. Les déléguer libère du budget
> pour les analyses complexes qui nécessitent le contexte complet.

---

## 🤝 Flux inter-agents

```
← Tous agents : peuvent signaler une sous-tâche déléguable pendant leur intervention
→ 📐 ARCHITECT : décision finale si ambiguïté sur le niveau de complexité
```

---

## 📚 Apprentissages

> *Section vide à la création — se remplit avec l'expérience.*

### À documenter ici au fil des sessions :
- Quelles tâches ont été effectivement déléguées avec succès
- Quelles IAs ont mieux performé sur quels types de tâches
- Quels prompts ont nécessité des ajustements
- Cas limites où la délégation a échoué (et pourquoi)
