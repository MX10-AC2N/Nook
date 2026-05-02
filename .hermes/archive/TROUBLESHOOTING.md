# 🔧 Troubleshooting Guide — Nook

## CI/CD Issues

### Build Fails
1. **Import manquant** → Vérifier tous les imports de notificationStore
2. **Svelte 5 syntax** → Utiliser `onclick={(e) => {}}` pas `onclick|`
3. **Rand crate** → rand 0.9: `rng()` pas `thread_rng()`, `distr::` pas `distributions::`

### Docker Issues
1. **Container unhealthy** → Vérifier healthcheck dans Dockerfile
2. **Permission denied** → Vérifier UID/GID 1000
3. **Config non montée** → Vérifier volume mount et path

## Frontend Issues

### Chess
1. **Pas de mouvement** → `this.myColor` (pas `this.myColor()`)
2. **Pièces trop petites** → Vérifier CSS `.piece { font-size: clamp(...) }`
3. **Board pas centré** → Vérifier `.board-wrap { align-items: center; }`

### Chat
1. **Emojis trop petits** → `.emoji-only { font-size: 4rem !important; }`
2. **Input pas visible** → `.input-area { position: sticky; bottom: 0; }`
3. **GIFs trop petits** → `.chat-gif { max-width: 600px; max-height: 600px; }`

### Notifications
1. **notifyPoll is not defined** → Import manquant dans le composant
2. **Pas de son** → AudioContext non initialisé (nécessite interaction utilisateur)
3. **Web Push** → Nécessite HTTPS (pas dispo en HTTP/LAN)

## Backend Issues

### Database
1. **SQLite locked** → Vérifier les connexions ouvertes
2. **Migration failed** → Vérifier sqlx-data.json

### API
1. **401 Unauthorized** → Vérifier session/cookie
2. **500 Internal** → Vérifier logs avec `RUST_LOG=debug`

## Performance

### Frontend
1. **Scroll lent** → Vérifier `$effect` avec dépendances stables
2. **Renders multiples** → Vérifier que les stores ne se réinitialisent pas

### Backend
1. **Memory leak** → Vérifier les connexions non fermées
2. **CPU haute** → Vérifier les boucles infinies ou polling excessif
