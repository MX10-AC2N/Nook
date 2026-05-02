# ⚙️ Préférences de collaboration — Nook

> Préférences de livraison, format, style d'interaction

---

## 📦 Format de livraison des fichiers

| Type de fichier | Format livraison |
|-----------------|-----------------|
| `.svelte` | `.txt` — évite bugs d'affichage Claude.ai |
| `.ts` / `.svelte.ts` | `.txt` |
| `.rs` | Direct (bloc code Rust) |
| `.sql` | Direct (bloc code SQL) |
| `.yml` / `.yaml` | Direct (bloc code YAML) |
| `.json` | Direct (bloc code JSON) |
| `.md` | Direct (markdown) |

**Toujours indiquer le chemin exact** au-dessus du bloc :
```
// frontend/src/lib/chatStore.svelte.ts
```

---

## 📱 Optimisations Claude.ai Android

- **Fichiers segmentés** : un fichier par message (pas tout d'un coup)
- **Taille raisonnable** : éviter les blocs > 200 lignes sur mobile
- **Titres clairs** : "Fichier 1/3 — `frontend/src/lib/authStore.svelte.ts`"
- **Résumé d'abord** : expliquer le changement AVANT le code
- **Confirmation avant multi-fichiers** : "Je vais modifier 3 fichiers — OK ?"

---

## 🔍 Avant chaque intervention

1. Fetcher le fichier source via Raw GitHub
2. Lire `BUGS.md` — ne pas réintroduire des bugs résolus
3. Identifier les effets de bord sur d'autres fichiers
4. Corriger TOUT ce qui peut l'être avant un build (éviter les cycles inutiles)

---

## 📝 Structure d'une réponse idéale

```
## Analyse
[Cause racine, pas seulement le symptôme]

## Fichiers modifiés
- `chemin/fichier1.ts` — [raison]
- `chemin/fichier2.rs` — [raison]

## Effets de bord
[Ce que ça pourrait impacter ailleurs]

## Fichier 1/N — `chemin/fichier1.ts`
[contenu complet]

## Mise à jour .claude/
[SESSIONS.md et BUGS.md mis à jour]
```

---

## 🚫 À ne jamais faire

- ❌ Diff partiel (toujours le fichier complet)
- ❌ Supposer l'état d'un fichier vu dans une session précédente
- ❌ Réintroduire un bug listé dans `BUGS.md`
- ❌ Proposer `writable()` / `readable()` Svelte 4
- ❌ Proposer `allow_any_origin()` avec `allow_credentials(true)`
- ❌ Proposer `thread_rng()` (supprimé rand 0.9 → utiliser `rng()`)
- ❌ Proposer `rand::distributions` (déplacé vers `rand::distr` en 0.9)
- ❌ Importer sans `Rng` trait quand on utilise `sample_iter()`
- ❌ Oublier de mettre à jour `.claude/` en fin de session

---

## ✅ Upgrades UX/structure

Proposer proactivement quand c'est pertinent (pas seulement corriger) :
- Améliorations UX identifiées en passant
- Simplifications de code sans changement de comportement
- Patterns plus robustes (ex : gestion d'erreur manquante)
- Performances évidentes (ex : chunk libsodium 938 kB → dynamic import)

---

## 🗂️ Mise à jour de `.claude/` en fin de session

### SESSIONS.md — ajouter en bas :
```markdown
## Session N — YYYY-MM-DD — [Titre court]

### Contexte
[Ce qu'on faisait]

### Cause racine / Changements
[Ce qui a été fait et pourquoi]

### Fichiers modifiés session N
- `chemin/fichier` — [raison]

### État attendu après fix
[Résultat attendu]

### Ce qui reste à faire
- [ ] item 1
- [ ] item 2
```

### BUGS.md — ajouter le bug résolu dans "✅ BUGS RÉSOLUS" avec :
- Symptôme
- Cause racine
- Fix appliqué
- Chronologie si pertinente
