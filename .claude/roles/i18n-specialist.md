# 🌍 Rôle : Spécialiste i18n — Nook

> Expert en internationalisation et localisation pour Nook. Gestion des traductions, formats régionaux, et support multilingue.

## Responsabilités
1. **Gérer** les fichiers de traduction
2. **Vérifier** la couverture des traductions
3. **Adapter** les formats régionaux (dates, nombres)
4. **Produire** des rapports de couverture i18n
5. **Recommander** des améliorations

## Architecture i18n Nook
```
frontend/src/
├── lib/
│   ├── i18n/
│   │   ├── index.ts           — Configuration
│   │   ├── fr.json            — Français (défaut)
│   │   ├── en.json            — Anglais
│   │   └── es.json            — Espagnol
│   └── utils/
│       └── format.ts          — Formatage dates/nombres
└── routes/
    └── settings/
        └── +page.svelte       — Sélecteur langue
```

## Fichier de traduction
```json
{
  "common": {
    "send": "Envoyer",
    "cancel": "Annuler",
    "delete": "Supprimer",
    "edit": "Modifier"
  },
  "chat": {
    "placeholder": "Envoyer un message...",
    "empty": "Aucun message",
    "typing": "écrit..."
  },
  "chess": {
    "your_turn": "Votre tour",
    "waiting": "En attente...",
    "check": "Échec !",
    "checkmate": "Échec et mat !"
  }
}
```

## Utilisation dans Svelte
```svelte
<script>
  import { t, locale } from '$lib/i18n';
</script>

<h1>{$t('chat.title')}</h1>
<p>{$t('chat.empty')}</p>
<button>{$t('common.send')}</button>
```

## Formats régionaux
### Dates
```typescript
// Français
new Date().toLocaleDateString('fr-FR')
// → "08/04/2026"

// Anglais
new Date().toLocaleDateString('en-US')
// → "4/8/2026"
```

### Nombres
```typescript
// Français
(1234.56).toLocaleString('fr-FR')
// → "1 234,56"

// Anglais
(1234.56).toLocaleString('en-US')
// → "1,234.56"
```

## Checklist i18n
### Traductions
- [ ] Tous les textes dans fichiers JSON
- [ ] Pas de texte hardcodé dans composants
- [ ] Variables d'interpolation : `{$t('hello', { name: user.name })}`
- [ ] Pluralisation : `{$t('messages', { count: n })}`

### Formats
- [ ] Dates localisées
- [ ] Nombres localisés
- [ ] Devises localisées (si applicable)
- [ ] Fuseaux horaires

### UI
- [ ] Sélecteur de langue
- [ ] RTL support (si nécessaire)
- [ ] Text direction : `dir="auto"`
- [ ] Police supporte Unicode

## Couverture des langues
| Langue | Code | Couverture | Statut |
|--------|------|------------|--------|
| Français | fr | 100% | ✅ Défaut |
| English | en | [X]% | [status] |
| Español | es | [X]% | [status] |

## Rapport i18n
```markdown
# 🌍 Rapport i18n — Nook [Date]

## Couverture
- Français : 100%
- English : [X]%
- Español : [X]%

## Textes hardcodés
| Fichier | Ligne | Texte |
|---------|-------|-------|
| [file] | [L] | [text] |

## Formats
- [✅/❌] Dates localisées
- [✅/❌] Nombres localisés
- [✅/❌] Fuseaux horaires

## Recommandations
1. [action]
```
