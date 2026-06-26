---
name: nook-performance-specialist
description: Skill spécialisé pour l'audit de performance du projet Nook. Utiliser pour analyser le bundle size (SvelteKit/Vite), le lazy loading, l'optimisation d'images, les requêtes SQLx, le WebSocket Axum, les temps de réponse et la charge. Génère un rapport avec score sur 100 en comparant avec les rapports précédents.
---

# ⚡ Nook — Performance Specialist Skill

## Périmètre d'audit

```
Frontend (SvelteKit 5 + Vite)
├── Bundle size analysis (chunk sizes, gzip, brotli)
├── Code splitting (manualChunks configuration)
├── Lazy loading (dynamic imports, modulepreload)
├── Image optimization (WebP/AVIF, responsive)
├── Cache strategies (Cache-Control, ETags)
└── CSS optimization (unused selectors)

Backend (Rust Axum + SQLx)
├── SQL query performance (indexes, N+1 problems)
├── WebSocket efficiency (message size, encryption overhead)
├── Rate limiting configuration
├── Compression middleware (gzip, brotli)
└── SQLite WAL mode and pragma settings

Metrics to collect
├── Build time and bundle sizes
├── Database query execution plans
├── HTTP response times
├── WebSocket latency
└── Memory usage (Rust release build)
```

## Outils d'analyse

### Frontend
```bash
# Bundle analysis
cd frontend && npx vite build --mode production

# Check chunk sizes
du -sh build/_app/immutable/chunks/*.js | sort -rh

# Check for modulepreload in HTML
grep -i "modulepreload\|preload" build/index.html

# Image analysis
find static -name "*.png" -o -name "*.jpg" | xargs ls -lah
```

### Backend
```bash
# Check SQL indexes
grep -r "CREATE INDEX" backend/migrations/

# Analyze SQLx queries
cd backend && cargo sqlx prepare --check

# Check WebSocket message handling
grep -r "64.*KB\|message.*size" backend/src/
```

## Grille de scoring (sur 100)

| Catégorie | Poids | Critères |
|-----------|-------|----------|
| **Bundle Size** | 25% | Chunks < 600kB, code splitting effectif |
| **Lazy Loading** | 15% | Dynamic imports, modulepreload configurés |
| **Images** | 15% | WebP/AVIF, responsive, compression |
| **Backend SQL** | 20% | Indexes présents, requêtes optimisées |
| **WebSocket** | 10% | Message size limit, encryption efficient |
| **Caching** | 10% | Cache-Control headers, etag |
| **Code Quality** | 5% | Pas de CSS inutilisé, a11y warnings |

## Calcul du score

```
Score = (Bundle_Score × 0.25) + (LazyLoad_Score × 0.15) + (Images_Score × 0.15) 
       + (SQL_Score × 0.20) + (WS_Score × 0.10) + (Cache_Score × 0.10)
       + (Quality_Score × 0.05)
```

### Barèmes détaillés

**Bundle Size (0-25 points):**
- 25 pts: Tous chunks < 600kB, code splitting optimal
- 20 pts: 1-2 chunks > 600kB mais < 1MB
- 15 pts: 1-2 chunks > 1MB
- 10 pts: 3+ chunks > 600kB
- 5 pts: Bundle monolithique > 2MB

**Lazy Loading (0-15 points):**
- 15 pts: Dynamic imports partout, modulepreload configuré
- 12 pts: Modulepreload présent, quelques imports statiques lourds
- 8 pts: Modulepreload partiel
- 4 pts: Pas de code splitting

**Images (0-15 points):**
- 15 pts: WebP/AVIF + responsive + compression
- 12 pts: WebP/AVIF présent, pas de responsive
- 8 pts: Compression mais format ancien (PNG/JPG)
- 4 pts: Pas d'optimisation

**SQL (0-20 points):**
- 20 pts: Indexes sur tous les WHERE/JOIN, pas de N+1
- 15 pts: Indexes présents mais quelques requêtes sous-optimales
- 10 pts: Indexes partiels
- 5 pts: Pas d'indexes

**WebSocket (0-10 points):**
- 10 pts: Message limit + encryption efficient + cleanup
- 7 pts: Message limit + encryption
- 4 pts: Message limit seulement
- 2 pts: Pas de limite

**Cache (0-10 points):**
- 10 pts: Cache-Control avec max-age adaptatif (hash=1y, html=0)
- 7 pts: Cache-Control présent (max-age fixe)
- 4 pts: Cache partiel
- 0 pts: Pas de cache

**Code Quality (0-5 points):**
- 5 pts: 0 warning
- 3 pts: Quelques warnings non-critiques
- 1 pts: Beaucoup de warnings (CSS inutilisé, a11y)

## Comparaison historique

| Date | Score | Problèmes critiques |
|------|-------|---------------------|
| 2026-04-09 | 81/100 | P1: Pas de code splitting |
| 2026-04-21 (S50) | 82/100 | +1 pt (amélioration marginale) |
| **Actuel** | **À calculer** | **Voir rapport ci-dessous** |

## Checklist d'audit

- [ ] Lancer `vite build` et noter les tailles de chunks
- [ ] Vérifier manualChunks dans vite.config.js
- [ ] Checker les dynamic imports (sodium, chess, etc.)
- [ ] Analyser les images dans /static (format, taille)
- [ ] Vérifier les indexes SQL dans /migrations
- [ ] Checker les Cache-Control headers dans main.rs
- [ ] Analyser les WebSocket message handling
- [ ] Compter les warnings CSS/a11y dans le build
- [ ] Générer le score final sur 100
- [ ] Créer le rapport dans `.hermes/archive/reports/audits/PERFORMANCE-REPORT.md`
