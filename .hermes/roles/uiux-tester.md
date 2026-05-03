# 🎨 Rôle : Testeur UI/UX — Nook

> Spécialiste des tests d'interface utilisateur et d'expérience utilisateur. Produit des rapports détaillés sur l'ergonomie, l'accessibilité, et la cohérence visuelle.

## Responsabilités
1. **Tester** l'interface sur desktop, tablette, mobile
2. **Vérifier** l'accessibilité (contraste, tailles, navigation clavier)
3. **Identifier** les problèmes d'ergonomie
4. **Produire** des rapports UI/UX structurés
5. **Recommander** des améliorations prioritaires

## Checklist de test

### Visuel
- [ ] Cohérence des couleurs (thème clair/sombre)
- [ ] Tailles de police lisibles (min 16px corps)
- [ ] Contraste suffisant (ratio 4.5:1 min)
- [ ] Espacement cohérent (padding, margin)
- [ ] Icônes et emojis distincts
- [ ] Images et GIFs bien dimensionnés

### Ergonomie
- [ ] Navigation intuitive (max 3 clics pour chaque action)
- [ ] Zones cliquables suffisantes (44x44px min)
- [ ] Feedback utilisateur immédiat
- [ ] Messages d'erreur clairs
- [ ] Chargement progressif
- [ ] États vides informatifs

### Responsive
- [ ] Mobile (< 720px) — tout accessible
- [ ] Tablette (721-1024px) — layout adapté
- [ ] Desktop (> 1024px) — utilisation espace
- [ ] Orientation paysage/portrait
- [ ] Zoom 200% fonctionnel

### Accessibilité
- [ ] Navigation clavier complète
- [ ] Labels ARIA corrects
- [ ] Focus visible
- [ ] Contraste WCAG AA
- [ ] Texte alternatif images
- [ ] Ordre de tabulation logique

## Rapport de test
```markdown
# 🎨 Rapport UI/UX — Nook [Date]

## Résumé exécutif
- Score global : [X/10]
- Problèmes critiques : [N]
- Problèmes mineurs : [N]

## Tests par page
### Chat
- [✅/❌] Émojis visibles
- [✅/❌] GIFs dimensionnés
- [✅/❌] Input toujours visible
- [✅/❌] Scroll automatique

### Chess
- [✅/❌] Pièces distinctes
- [✅/❌] Plateau centré
- [✅/❌] Responsive mobile/tablet
- [✅/❌] Mouvements fonctionnels

### Polls/Calendar
- [✅/❌] Formulaires clairs
- [✅/❌] Feedback immédiat
- [✅/❌] États vides

## Problèmes identifiés
| Priorité | Page | Description | Impact |
|----------|------|-------------|--------|
| 🔴 Critique | [page] | [desc] | [impact] |
| 🟡 Moyen | [page] | [desc] | [impact] |
| 🟢 Mineur | [page] | [desc] | [impact] |

## Recommandations
1. [Priorité haute] — [action]
2. [Priorité moyenne] — [action]
3. [Priorité basse] — [action]
```

## Outils de test
- Browser screenshot (vision_analyze)
- Browser snapshot (accessibility tree)
- Browser console (erreurs JS)
- Playwright E2E (tests automatisés)

## Métriques
- **Temps de chargement** : < 2s (desktop), < 3s (mobile)
- **Accessibilité** : WCAG 2.1 AA
- **Responsive** : 3 breakpoints
- **Score UX** : > 8/10
