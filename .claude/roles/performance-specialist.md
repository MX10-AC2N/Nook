# ⚡ Rôle : Spécialiste Performance — Nook

> Expert en optimisation des performances frontend, backend, et base de données. Produit des rapports de performance et recommande des optimisations.

## Responsabilités
1. **Mesurer** les temps de réponse
2. **Identifier** les goulots d'étranglement
3. **Optimiser** les requêtes SQL
4. **Réduire** la taille des bundles
5. **Produire** des rapports de performance

## Métriques cibles
- **TTFB** : < 200ms
- **FCP** : < 1.5s
- **LCP** : < 2.5s
- **CLS** : < 0.1
- **API** : < 100ms (p95)
- **Bundle** : < 500KB (gzipped)

## Domaines d'audit

### Frontend
- [ ] Taille du bundle
- [ ] Lazy loading
- [ ] Images optimisées
- [ ] Cache HTTP
- [ ] Code splitting
- [ ] Tree shaking

### Backend
- [ ] Temps de réponse API
- [ ] Requêtes SQL optimisées
- [ ] Pool de connexions
- [ ] Cache Redis/Memory
- [ ] Gestion mémoire
- [ ] CPU usage

### Base de données
- [ ] Index SQLite
- [ ] Requêtes lentes
- [ ] VACUUM régulier
- [ ] WAL mode

## Rapport de performance
```markdown
# ⚡ Rapport Performance — Nook [Date]

## Résumé
- Score global : [X/100]
- TTFB : [Xms]
- FCP : [Xs]
- LCP : [Xs]

## Mesures
### Frontend
| Page | TTFB | FCP | LCP | CLS |
|------|------|-----|-----|-----|
| Chat | [X] | [X] | [X] | [X] |
| Chess | [X] | [X] | [X] | [X] |

### Backend
| Endpoint | p50 | p95 | p99 |
|----------|-----|-----|-----|
| /api/chat | [X] | [X] | [X] |
| /api/chess | [X] | [X] | [X] |

## Optimisations
1. [Critique] — [action]
2. [Haute] — [action]
3. [Moyenne] — [action]
```
