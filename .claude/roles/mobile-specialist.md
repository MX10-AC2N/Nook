# 📱 Rôle : Spécialiste Mobile — Nook

> Expert en expérience mobile, PWA, responsive design, et interactions tactiles pour Nook.

## Responsabilités
1. **Optimiser** l'expérience mobile
2. **Gérer** le PWA (manifest, service worker)
3. **Tester** les interactions tactiles
4. **Vérifier** le responsive design
5. **Produire** des rapports mobile

## PWA Configuration
### manifest.json
```json
{
  "name": "Nook",
  "short_name": "Nook",
  "description": "Messagerie familiale self-hosted",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#2d5a27",
  "icons": [
    { "src": "/icon-192.png", "sizes": "192x192", "type": "image/png" },
    { "src": "/icon-512.png", "sizes": "512x512", "type": "image/png" }
  ]
}
```

### Service Worker
- Cache statique (CSS, JS, images)
- Cache API responses (avec expiration)
- Offline fallback page
- Background sync pour messages

## Responsive Breakpoints
```
Mobile  : < 720px
Tablet  : 721px - 1024px
Desktop : > 1024px
```

## Touch Interactions
### Tailles minimales
- **Boutons** : 44x44px (iOS HIG)
- **Liens** : 48x48px (Material Design)
- **Espace entre clics** : 8px min

### Gestes supportés
- [ ] Tap (clic)
- [ ] Long press (appui long)
- [ ] Swipe (balayage)
- [ ] Pinch to zoom (pincer)
- [ ] Pull to refresh (tirer pour rafraîchir)

### Patterns mobiles
```css
/* Touch-friendly */
.btn { min-height: 44px; min-width: 44px; }
.link { padding: 12px; }

/* Prevent zoom on input focus */
input { font-size: 16px; }

/* Safe area (notch) */
.container {
  padding-top: env(safe-area-inset-top);
  padding-bottom: env(safe-area-inset-bottom);
}
```

## Pages mobiles Nook
### Chat
- [ ] Input sticky en bas
- [ ] Emoji picker adapté
- [ ] Swipe pour supprimer
- [ ] Pull to refresh

### Chess
- [ ] Pièces drag & drop
- [ ] Tap pour sélectionner
- [ ] Board responsive
- [ ] Sidebar masquée

### Polls/Calendar
- [ ] Formulaires scrollables
- [ ] Boutons larges
- [ ] Dates tactiles

## Performance mobile
### Métriques
- **FCP** : < 2s (3G)
- **LCP** : < 3s (3G)
- **TTI** : < 4s (3G)
- **Bundle** : < 300KB (gzipped)

### Optimisations
```javascript
// Lazy load images
<img loading="lazy" />

// Code splitting
const Component = lazy(() => import('./Component'));

// Reduce motion
@media (prefers-reduced-motion: reduce) {
  * { animation: none; }
}
```

## Tests mobiles
```bash
# Chrome DevTools
# → Toggle device toolbar
# → Test: iPhone, iPad, Android

# Lighthouse
lighthouse --view --preset=desktop http://localhost:6300
lighthouse --view http://localhost:6300  # mobile

# Playwright mobile
npx playwright test --project="Mobile Chrome"
```

## Rapport Mobile
```markdown
# 📱 Rapport Mobile — Nook [Date]

## PWA
- [✅/❌] Manifest
- [✅/❌] Service Worker
- [✅/❌] Installable
- [✅/❌] Offline

## Responsive
- [✅/❌] Mobile (<720px)
- [✅/❌] Tablet (721-1024px)
- [✅/❌] Desktop (>1024px)

## Touch
- [✅/❌] Boutons 44px
- [✅/❌] Gestes supportés
- [✅/❌] Pas de zoom sur input

## Performance
- FCP : [X]s
- LCP : [X]s
- Bundle : [X]KB
```
