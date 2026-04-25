# 🛠️ WORKFLOW-CATALOG — GitHub Actions Nook

> **Inventaire complet des workflows** | Mis à jour : **2026-04-02 (S46)**
> 20 fichiers `.github/workflows/*.yml`

---

## 📊 Vue d'ensemble

| # | Fichier | Nom | Lignes | Déclencheurs | Critique |
|---|---------|------|--------|-------------|----------|
| 1 | `test-nook.yml` | test-nook | ~382 | push/PR/dispatch | 🔴 OUI |
| 2 | `Backend.yml` | Backend Build & Artifact | 255 | dispatch | 🟡 UTILE |
| 3 | `Frontend.yml` | Frontend Build & Artifact | 177 | dispatch | 🟡 UTILE |
| 4 | `Docker.yml` | Docker Build & Push | 223 | dispatch | 🟡 UTILE |
| 5 | `ci-new2.yml` | CI Multi-Arch Distroless | 244 | dispatch | 🟡 REDONDANT |
| 6 | `e2e-targeted.yml` | E2E ciblé | 304 | dispatch | 🟡 UTILE |
| 7 | `Release.yml` | Release — Bump Version | 111 | dispatch | 🟡 UTILE |
| 8 | `sqlx-prepare.yml` | SQLx queries.json | 98 | push/dispatch | 🟢 UTILITAIRE |
| 9 | `update-cargo-lock.yml` | Update Cargo.lock | 52 | push/dispatch | 🟢 UTILITAIRE |
| 10 | `update-frontend-lock.yml` | Update package-lock | 32 | dispatch | 🟢 UTILITAIRE |
| 11 | `bundle-analysis.yml` | Bundle Analysis | 155 | push/dispatch | 🟢 UTILITAIRE |
| 12 | `npm-audit-report.yml` | Audit sécurité npm | 143 | dispatch/cron(désactivé) | 🟢 UTILITAIRE |
| 13 | `npm-update-deps.yml` | Update deps npm | 112 | dispatch/cron(désactivé) | 🟢 UTILITAIRE |
| 14 | `ghcr-cleanup.yml` | GHCR Nettoyage | 220 | wf_run/dispatch | 🟢 UTILITAIRE |
| 15 | `clear-cache.yml` | Nettoyage cache GA | 106 | dispatch/cron | 🟢 UTILITAIRE |
| 16 | `fetch-gifs.yml` | Fetch GIFs | 107 | dispatch | 🟢 UTILITAIRE |
17 | `generate-pwa-icons.yml` | Generation icônes PWA | 226 | push/dispatch | 🟢 UTILITAIRE |
18 | `security-audit.yml` | Security Audit auto | 160 | dispatch/cron | 🆕 NOUVEAU |
19 | `auto-svelte5-migration.yml` | Check Svelte 5 | 234 | dispatch | 🔴 OBSOLÈTE |
20 | `fix-svelte5-runes.yml` | Purify Svelte 5 | 174 | dispatch | 🔴 OBSOLÈTE |
21 | `generate-android-instruction.yml` | Instruction Android | 106 | push/dispatch | ⚪ FAIBLE |

---

## 🔴 À SUPPRIMER (candidats cleanup)

### `auto-svelte5-migration.yml` — OBSOLÈTE

**Pourquoi** : La migration Svelte 4 → 5 est terminée depuis la session 37.
Ce workflow vérifiait la syntaxe S5 pendant la migration. Plus aucun fichier S4 ne reste.
**Impact** : Zéro — personne ne l'utilise plus.

### `fix-svelte5-runes.yml` — OBSOLÈTE

**Pourquoi** : Idem — purifiait automatiquement les runes Svelte 5
pendant la migration. Migration terminée, toutes les runes sont correctes.
**Impact** : Zéro — doublon fonctionnel avec `auto-svelte5-migration.yml`.

### `generate-android-instruction.yml` — FAIBLE VALEUR

**Pourquoi** : Génère l'instruction Android pour Claude.ai à partir
du fichier VERSION. Usage ponctuel (une fois au setup), pas besoin
d'un workflow CI dédié. Peux être fait manuellement ou par un agent.
**Impact** : Faible — fichier léger, mais encombre l'interface.

---

## 🟡 À FUSIONNER (optimisation)

### `ci-new2.yml` → fusion avec `Backend.yml` + `Docker.yml`

**Pourquoi** : `ci-new2.yml` fait exactement la même chose que
`Backend.yml` + `Frontend.yml` + `Docker.yml` combinés. C'est un
workflow "tout en un" créé pour itérer rapidement pendant le dev.
Maintenant que les workflows séparés fonctionnent, `ci-new2` est redondant.

**Option** : Garder `ci-new2` comme workflow "rapide" pour les dispatch manuels
et supprimer les workflows séparés, OU l'inverse.

### `update-cargo-lock.yml` + `update-frontend-lock.yml` → un seul

**Pourquoi** : Deux micro-workflows (52 et 32 lignes) qui font la même chose :
mettre à jour un lockfile après changement de dependencies. Peuvent être
fusionnés en un seul `update-lockfiles.yml`.

---

## 🟢 À CONSERVER (utiles)

| Workflow | Pourquoi le garder |
|----------|-------------------|
| `test-nook.yml` | CI principale — 7 sections shell consolidees en 1 bloc — indispensable (shell tests consolidés en un seul bloc run:) |
| `Backend.yml` | Build backend (artefact) — utile pour debug |
| `Frontend.yml` | Build frontend (artefact) — utile pour debug |
| `Docker.yml` | Build Docker multi-arch — critique pour deploy |
| `e2e-targeted.yml` | Debug rapide d'une suite E2E spécifique |
| `Release.yml` | Gestion version/tag — processus release |
| `sqlx-prepare.yml` | Auto-régénère queries.json après modifs SQL |
| `bundle-analysis.yml` | Surveille la taille du bundle frontend |
| `npm-audit-report.yml` | Sécurité npm (cron désactivé mais dispatch utile) |
| `npm-update-deps.yml` | Màj deps npm (cron désactivé mais dispatch utile) |
| `ghcr-cleanup.yml` | Nettoyage auto des images GHCR |
| `clear-cache.yml` | Nettoyage cache GitHub Actions |
| `fetch-gifs.yml` | Collection GIFs de base |
| `generate-pwa-icons.yml` | Icônes PWA auto-générées |

---

## 🤖 POUR L'AGENT — Comment utiliser les workflows

### Exécuter un workflow manuellement

```bash
# Via l'API GitHub
export GITHUB_TOKEN=$(grep GITHUB_TOKEN= ~/.hermes/.env | cut -d= -f2 | tr -d '\n')
OWNER=MX10-AC2N
REPO=Nook

# Ex: lancer le pipeline E2E complet
curl -X POST \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  "https://api.github.com/repos/$OWNER/$REPO/actions/workflows/test-nook.yml/dispatches" \
  -d '{"ref":"fix/notifications-and-chess-audit"}'

# Ex: lancer un test E2E ciblé (upload suite)
curl -X POST \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  "https://api.github.com/repos/$OWNER/$REPO/actions/workflows/e2e-targeted.yml/dispatches" \
  -d '{"ref":"fix/notifications-and-chess-audit","inputs":{"test_suite":"Upload"}}'
```

### Vérifier le statut CI

```bash
# Voir les derniers runs
curl -s \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  "https://api.github.com/repos/$OWNER/$REPO/actions/runs?branch=fix/notifications-and-chess-audit&per_page=5" \
  | python3 -c "
import sys, json
data = json.load(sys.stdin)
for r in data.get('workflow_runs', []):
    status = r['status']
    conclusion = r.get('conclusion') or 'in_progress'
    print(f\"  {r['name']:<40} {status:<12} {conclusion}\")"
```

---

## 📋 Plan de nettoyage recommandé

| Priorité | Action | Workflow(s) | Effort |
|----------|--------|-------------|--------|
| 1 | Supprimer | `auto-svelte5-migration.yml` | 1 min |
| 2 | Supprimer | `fix-svelte5-runes.yml` | 1 min |
| 3 | Supprimer | `generate-android-instruction.yml` | 1 min |
| 4 | Fusionner | `update-cargo-lock.yml` + `update-frontend-lock.yml` → `update-lockfiles.yml` | 10 min |
| 5 | Décider | Garder ou supprimer `ci-new2.yml` | 5 min |

**Économie** : ~5 fichiers supprimés, ~900 lignes de YAML en moins.

---

*Catalogue généré automatiquement — S46 — 2026-04-02*

### Notifications (Session 48)
- `frontend/src/lib/notificationStore.svelte.ts` — Store central avec types de notification
- `frontend/src/lib/components/NotificationToast.svelte` — Composant toast + historique
- Fonctions disponibles: notify(), notifyMessage(), notifyChess(), notifyPoll(), notifyCalendar(), notifyCall(), notifyAdmin()
