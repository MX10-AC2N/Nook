# 🔧 Optimisation Profile security-auditor — Nook

> **Profil**: security-auditor
> **Date**: 2026-09-11
> **Objectif**: Optimiser la config, profiles, skills, et utilité du profile security-auditor

> **Pour Hermes**: Utiliser le skill `plan` pour l'exécution détaillée. Ce document est le plan d'optimisation.

---

## Constat Actuel

| Aspect | État | Problème |
|--------|------|----------|
| **SOUL.md** | ✅ Existe | Profil bien défini mais pourrait être plus léger |
| **MCP Servers** | ⚠️ 2/8 | SOUL.md dit 7/8 mais `.mcp.json` n'a que `plur` + `rust-analyser` |
| **Skills** | ⚠️ 50+ | Tous génériques, aucun spécifique security-auditor |
| **Config.yaml** | ✅ Fonctionnel | MOA avec reference_models, fanout |
| **LSP** | 🔴 33MB | Potentiellement réductible |
| **Logs** | 🟡 2.5MB | Accumulation |
| **Cache** | ✅ 784K | OK |
| **Roles** | ✅ Existe | `security-crypto.md` détaillé |
| **Critical Pitfalls** | ✅ Existe | Complet |
| **Nook Context** | ✅ Existe | `nook-context.md` à jour |
| **PLUR Engrams** | ✅ Fonctionnel | 1 engram de session |
| **Memories** | ✅ Fonctionnel | MEMORY.md avec contexte |

---

## Plan d'Optimisation — 4 Axes

### Axe 1: MCP Servers (CRITIQUE)

**Problème**: SOUL.md référence 7/8 MCP servers mais `.mcp.json` n'en configure que 2.

**Actions**:

1. **Auditer les MCP attendus** (security-auditor):
   - `github` ✅ (PRs, issues, releases)
   - `filesystem` ✅ (lecture fichiers)
   - `sqlite` ✅ (base de données)
   - `playwright` ✅ (E2E testing)
   - `svelte` ✅ (frontend)
   - `svelte-docs` ✅ (docs Svelte)
   - `rust-analyzer` ✅ (déjà configuré)
   - `plur` ✅ (déjà configuré)
   - **→ 8 serveurs attendus, 2 configurés → 6 manquants**

2. **Ajouter les MCP manquants** à `.mcp.json`:
   - `github` — accès aux repos Nook
   - `filesystem` — navigation fichiers
   - `sqlite` — queries DB Nook
   - `playwright` — tests E2E
   - `svelte` — inspection composants
   - `svelte-docs` — documentation Svelte

3. **Vérifier la compatibilité** après ajout

### Axe 2: Skills Spécifiques (HAUTE PRIORITÉ)

**Problème**: 50+ skills génériques, ZERO skill spécifique security-auditor/Nook.

**Actions**:

1. **Créer des skills spécifiques Nook**:

   | Skill Name | Description | Scope |
   |------------|-------------|-------|
   | `nook-security-audit` | Audit E2EE, crypto, threat model | security-auditor |
   | `nook-crypto-review` | Revue implémentation crypto (e2ee.rs, auth.rs) | security-auditor |
   | `nook-rate-limit` | DT-04 governor + Redis rate limiting | security-auditor |
   | `nook-e2ee-verify` | Vérification E2EE (X25519, XChaCha20, Double Ratchet) | security-auditor |
   | `nook-threat-model` | Maintenance STRIDE threat model | security-auditor |
   | `nook-dep-dependency` | cargo audit, pnpm audit, osv-scanner | security-auditor |
   | `nook-playwright-sec` | Tests E2E sécurité (Playwright) | security-auditor |

2. **Structure de chaque skill**:
   - `SKILL.md` avec frontmatter (name, description, tags)
   - `references/` pour docs techniques
   - `scripts/` pour vérifications automatisées
   - `templates/` pour rapports d'audit

3. **Déplacer/dedupliquer les skills existants**:
   - Les skills `autonomous-ai-agents/*` sont déjà installés
   - Créer un namespace `nook-*` pour les skills Nook-spécifiques

### Axe 3: Config Optimisation (MOYENNE PRIORITÉ)

**Problème**: config.yaml a des couches MOA complexes, LSP 33MB, logs 2.5MB.

**Actions**:

1. **LSP cleanup**:
   - `lsp/` = 33MB → vérifier ce qui est installé
   - Si c'est npm packages de dev → considérer nettoyage
   - Sauvegarder avant toute modification

2. **Log rotation**:
   - `logs/agent.log` = 2.3MB → rotation automatique
   - `logs/errors.log` = 184KB → archiver
   - Ajouter logrotate ou rotation par taille dans config

3. **Cache optimization**:
   - `cache/` = 784K → OK pour le moment
   - Vérifier les stale entries

4. **Config.yaml refinement**:
   - Le `moa` config est complexe (reference_models, fanout, aggregator)
   - Vérifier que le `delegation.model: nemotron-3.5-lightning-free` est pertinent
   - Le `max_tokens: 4096` et `moa.max_tokens: 4096` — cohérent ?
   - **Note**: Le THROTTLE ACTION récent a réduit à `minimax-m3-free (2048 max_tokens)` → vérifier config cohérence

### Axe 4: Utilité & Productivité (HAUTE PRIORITÉ)

**Problème**: Le profile a des outils mais manque d'automatisation pour les tâches récurrentes de security-auditor.

**Actions**:

1. **Cron jobs utiles**:
   - `cargo audit` hebdomadaire (déjà dans CI mais pas en cron local)
   - `cargo clippy` sur les changements
   - Vérification DT status hebdomadaire
   - Nettoyage logs mensuel

2. **Scripts utilitaires** à ajouter dans `bin/`:
   - `bin/security-check.sh` — run cargo audit + pnpm audit + clippy
   - `bin/e2ee-verify.sh` — vérifier la crypto E2EE
   - `bin/dt-status.sh` — afficher le statut de tous les DT items
   - `bin/rate-limit-test.sh` — tester le rate limiting

3. **Templates de rapports** dans `reports/`:
   - `security-audit-template.md`
   - `crypto-review-template.md`
   - `threat-model-template.md`
   - `incident-response-template.md`

4. **Memory optimization**:
   - MEMORY.md consolidé (déjà fait)
   - Vérifier `nook-context.md` n'est pas obsolète
   - Ajouter entries récentes si nécessaire

---

## Priorisation & Dépendances

```
Phase 1 (URGENT):   Axe 1 — MCP Servers (sans ça, le profile est incomplet)
Phase 2 (HIGH):     Axe 2 — Skills Spécifiques (le profile a besoin de son identité)
Phase 3 (MEDIUM):   Axe 4 — Utilité (cron, scripts, templates)
Phase 4 (LOW):      Axe 3 — Config Optimization (LSP, logs, refinement)
```

## Fichiers à Créer/Modifier

### Créer:
- `.mcp.json` — ajouter 6 MCP servers manquants
- `skills/nook-security-audit/SKILL.md`
- `skills/nook-crypto-review/SKILL.md`
- `skills/nook-rate-limit/SKILL.md`
- `skills/nook-e2ee-verify/SKILL.md`
- `skills/nook-threat-model/SKILL.md`
- `skills/nook-dep-dependency/SKILL.md`
- `skills/nook-playwright-sec/SKILL.md`
- `bin/security-check.sh`
- `bin/e2ee-verify.sh`
- `bin/dt-status.sh`
- `bin/rate-limit-test.sh`
- `reports/security-audit-template.md`
- `reports/crypto-review-template.md`
- `reports/threat-model-template.md`
- `reports/incident-response-template.md`

### Modifier:
- `.mcp.json` — ajouter les serveurs MCP
- `config.yaml` — ajuster log rotation, cache settings
- `SOUL.md` — mettre à jour la référence MCP (7/8 → cohérent)
- `logs/` — archiver anciens logs

### Vérifier:
- `lsp/` — identifier et nettoyer si nécessaire
- `cache/` — stale entries
- `cron/` — jobs existants vs nouveaux besoins

## Validation

- [ ] MCP servers fonctionnels (test connectivité)
- [ ] Skills chargés et accessibles
- [ ] Scripts exécutables et fonctionnels
- [ ] Logs rotatés
- [ ] Config cohérente (throttle + moa)
- [ ] PLUR engram écrit

## Risques

- Ajouter des MCP servers peut casser des connexions existantes
- Les scripts binaires nécessitent `chmod +x`
- Les skills Nook-spécifiques doivent éviter les conflits avec les skills génériques
- La config MOA est complexe — modifications risquées sans comprendre l'impact

## PLUR Engram

À écrire après exécution du plan:
```yaml
type: procedural
scope: global
domain: nook.security-auditor.profile-optimization
tags: [nook, security-auditor, 2026-09-11, profile-optimization]
statement: "Profile security-auditor optimisé: 6 MCP servers ajoutés, 7 skills Nook créés, scripts utilitaires ajoutés"
```
