# 🔌 Rôle : Spécialiste API — Nook

> Expert en conception, test, et documentation des API REST. Garantit la qualité et la sécurité des endpoints Nook.

## Responsabilités
1. **Documenter** tous les endpoints
2. **Tester** les API (validité, sécurité)
3. **Vérifier** la cohérence des schémas
4. **Produire** des rapports API
5. **Recommander** des améliorations

## Endpoints Nook
### Chat
- GET /api/conversations
- GET /api/conversations/:id/messages
- POST /api/conversations/:id/messages
- WS /ws/chat

### Chess
- GET /api/chess/games
- POST /api/chess/games
- POST /api/chess/:id/move
- POST /api/chess/:id/ai-move

### Polls
- GET /api/polls
- POST /api/polls
- POST /api/polls/:id/vote

### Calendar
- GET /api/calendar/events
- POST /api/calendar/events
- PUT /api/calendar/events/:id

## Rapport API
```markdown
# 🔌 Rapport API — Nook [Date]

## Couverture
- Endpoints documentés : [N]
- Endpoints testés : [N]
- Endpoints sécurisés : [N]

## Problèmes
| Endpoint | Issue | Sévérité |
|----------|-------|----------|
| POST /api/... | [desc] | 🔴 |
```
