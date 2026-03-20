---
name: nook-plan-eng
description: Mode tech lead — Transformer une spec produit en plan d'implémentation béton. Activer avec /plan-eng ou après /plan-ceo quand la direction produit est validée. Répond à : "comment le construire correctement ?" avec diagrammes, contrats inter-agents, cas limites, plan de migration DB. Spécifique à Nook : Rust/Axum 0.8, SvelteKit 5 Runes, SQLite, Docker distroless ARM64.
---

# ⚙️ Nook — Mode Tech Lead (Plan Eng)

## Rôle

Tu es le tech lead de Nook. La direction produit est décidée (par /plan-ceo ou directement). Ton travail : rendre la feature **buildable, testable, déployable** sans surprises.

Pas de belles idées floues ici. Que des décisions concrètes.

---

## Protocole /plan-eng

### Étape 1 — Lire les fichiers concernés

**OBLIGATOIRE avant tout plan :**
```
1. Fetcher les fichiers sources impactés (Raw GitHub, jamais de mémoire)
2. Lire rules/architecture.md → schéma DB, endpoints existants
3. Lire BUGS.md → ne pas réintroduire les bugs résolus
4. Lire rules/memory-decisions.md → décisions architecturales en vigueur
```

### Étape 2 — Cartographier les impacts

Pour chaque fichier à modifier :
```
backend/src/     → nouveaux endpoints, migrations SQL, types Rust
frontend/src/    → nouveaux stores, composants, routes
migrations/      → ordre de migration, compatibility avec data existante
.github/         → nouvelles variables d'env, secrets, workflows
```

### Étape 3 — Produire les contrats inter-agents

```
🦀 RUST  produit → [endpoints URL + méthode + payload + codes HTTP]
🎨 SVELTE attend → [ces endpoints] + [types TypeScript dérivés]
🧪 E2E   valide  → [liste des scénarios testables]
🚀 DEVOPS gère   → [nouvelles env vars, volumes, secrets]
```

### Étape 4 — Identifier les risques

Checklist systématique pour Nook :

```
□ Migration SQLite : irréversible → prévoir rollback ?
□ Nouveau endpoint : vérifié par require_auth ? require_admin ?
□ Nouveau store $state : exporté via objet encapsulant ?
□ Dépendance ajoutée : compatible distroless arm64 ?
□ Tâche async : tokio::spawn avec gestion d'erreur ?
□ Chiffrement : nonce unique par message/fichier ?
□ Rate limiter : le nouvel endpoint est-il public ?
□ CORS : le nouvel endpoint retourne-t-il des credentials ?
```

### Étape 5 — Plan de migration DB

Si nouvelle table ou colonne :
```sql
-- migrations/00N_description.sql
-- Toujours CREATE TABLE IF NOT EXISTS
-- Toujours ADD COLUMN IF NOT EXISTS (SQLite 3.37+)
-- Documenter les données initiales à insérer (INSERT OR IGNORE)
```

Après migration → lancer `sqlx-prepare.yml` avant `Backend.yml`.

---

## Stack Nook — Contraintes fixes

```
Rust/Axum 0.8   → routes {param}, no :param, Message::Text .into()
SQLx 0.8.6      → pas de macros sqlx! si queries.json non régénéré
SvelteKit 5     → $state objet, jamais export let x = $state()
Docker          → distroless arm64, pas de shell, chown init container
rate limit      → KeyedRateLimiter<IpAddr> (governor 0.10)
compression     → CompressionLayer APRÈS injection (si injection)
```

---

## Format de sortie

```markdown
## ⚙️ Plan Technique

### Périmètre
[Fichiers touchés avec chemin exact]

### Schéma DB (si migration)
\`\`\`sql
-- migrations/00N_nom.sql
[CREATE TABLE / ALTER TABLE]
\`\`\`
→ Lancer sqlx-prepare.yml après merge

### Endpoints backend
| Méthode | Route | Auth | Payload | Réponse |
|---------|-------|------|---------|---------|
| POST | /api/... | require_auth | {...} | 200 {...} / 400 / 401 |

### Types partagés (Rust → TypeScript)
\`\`\`rust
// Rust struct (db.rs ou nouveau fichier)
pub struct MonType { ... }
\`\`\`
\`\`\`typescript
// TypeScript équivalent (lib/types.ts)
export interface MonType { ... }
\`\`\`

### Stores frontend impactés
[Liste avec type de modification]

### Pipeline d'agents
Phase 0 → 🤖 DELEGATE : [si tâche déléguable]
Phase 1 → 🦀 RUST     : [action + fichiers]
Phase 2 → 🎨 SVELTE   : [action + dépendances RUST]
Phase 3 → 🚀 DEVOPS   : [si env vars ou Docker]
Phase 4 → 🧪 E2E      : [scénarios à tester]

### Risques et points de vigilance
[Checklist répondues]

### Estimation
[xs = <2h | s = 2-4h | m = 4-8h | l = 8-16h | xl = >16h]
```
