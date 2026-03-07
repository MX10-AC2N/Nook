# 📅 Résumés sessions — Nook

> Contexte rapide. Détails complets dans `SESSIONS.md`.

---

## État du projet (après session 32) — COMPLET ✅

**Version** : 0.3.0-beta.3 | **Tests E2E** : 66 | **Bugs actifs** : 0

### ✅ Tout fonctionnel
- Backend Rust : compile sans warning, tous modules exposés
- Chat **temps réel WS** : `new_message`, `message_edited`, `message_deleted` broadcastés
- **WS reconnexion automatique** (backoff exponentiel) dans chatStore ET chessStore
- **Edit/Delete messages** : PATCH + DELETE `/api/conversations/{conv_id}/messages/{msg_id}`
- **Pagination messages** : `before` + `limit` utilisés, bouton "Charger plus" + scroll
- **Badges non-lus** par conversation dans la sidebar
- **Notifications navigateur** (Permission API, désactivées si app au premier plan)
- **Upload** : vérification 50 Mo côté client avec message d'erreur temporaire
- E2EE complet : keygen, chiffrement envoi, déchiffrement réception
- Rate limiting : 10 req/min sur /login, /register, /join
- libsodium : dynamic import, 0 blocage au démarrage
- Analytics admin : 8 métriques + 2 charts
- Renommage groupe inline (✏️ dans header)
- Label "Nook" pour default_global (avatar 🌿)
- Chess temps réel WS : `chess_move`, `chess_ai_move`, `chess_player_joined`
- Docker distroless arm64/amd64, CI 5 workflows
- **E2E stabilisé** : 5 bugs résolus (format réponses chess/polls, champ opponent, waitForResponse race, Rate Limit serial, loginAs retry)

### 📋 Backlog restant
- **DT-05** : WebRTC WAN instable — serveur TURN absent (LAN OK)
- Recherche de messages (endpoint + UI)
- Réactions emoji sur les messages

---

## Chronologie condensée

| Sessions | Thème | Résultat |
|----------|-------|---------| 
| 1 | Analyse initiale | CLAUDE.md créé |
| 2-5 | Rust upgrades, Docker, SQLite, CORS | Backend stable ✅ |
| 6-7 | CI Playwright, E2E_SETUP | CI infra stable ✅ |
| 8-14 | Bugs prod : UUID, CORS, SameSite, prune | Prod stable ✅ |
| 15-23 | E2E stabilisation → 43/43 | Tests verts ✅ |
| 24 | Refonte .claude/ v4 | Structure v4 ✅ |
| 25-26 | Crypto non-bloquant, Polls E2E | ✅ |
| 27 | E2EE activé (db.rs + chatStore) | ✅ |
| 28 | DT-01 (dynamic import) + DT-04 (rate limit) | ✅ |
| 29 | DT-06 (analytics enrichis) | ✅ |
| 30 | Call routing, GIF fix, Upload download, Chess mobile | ✅ |
| 31 | WS temps réel chat, edit/delete msg, pagination, badges non-lus, notifs, chess WS reconnect, renommage groupe, label Nook, upload 50Mo feedback, 66 tests E2E | ✅ |
| 32 | Fix 5 bugs E2E (format chess/polls, opponent, waitForResponse race, Rate Limit serial, loginAs retry) | ✅ |
