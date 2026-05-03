# 🏠 Nook Resources — État & Roadmap

> Source de vérité sur l'état du projet, les décisions techniques et la roadmap.
> Complémentaire à `BUGS.md` et `memory-sessions.md`.
> Mis à jour : session 44

---

## 📊 État des features v0.5

| Feature | État | Score E2E | Notes |
|---|---|---|---|
| 💬 Chat texte | ✅ Stable | 97/98 | scroll auto S43, badge S43 |
| 📎 Upload fichiers | ✅ Fixé S43 | — | DefaultBodyLimit 52MB |
| 🎙️ Messages vocaux | ⚠️ Fragile | — | iOS Safari à tester |
| 🌐 Appels SFU groupe | ✅ LOT 6 | — | rustrtc intégré, auto-activé 3+ participants |
| 📞 Appels audio/vidéo | ✅ WS fixé S42 | — | P2P mesh + TURN, WAN fonctionnel |
| ♟️ Échecs | ✅ Solide | — | Timer, IA, WS, noms joueurs |
| 📅 Calendrier | ✅ Refait S43 | 97/98¹ | click/édition/suppression |
| 📊 Sondages | ✅ Date clôture S43 | — | fermeture auto backend à faire |
| 🎨 Thèmes | ✅ 4 thèmes S43 | — | nuit-douce ajouté |
| 🔐 E2EE | ✅ Non-bloquant S42 | — | ready=true avant sodium |
| 🔔 Notifications push | ✅ VAPID | — | stable |
| 🔗 Invitations | ✅ Flux complet S41 | — | /invite page publique |

¹ `calendar-grid` et `add-event-btn` classes CSS doivent correspondre aux sélecteurs E2E

---

## 🗺️ Roadmap LOT 3 (prochaine session)

### P1 — turn-rs (appels WAN)
```
Durée estimée : 2h
Impact : Appels fonctionnels hors LAN (toute la famille même à distance)
Fichiers : docker-compose.yml + webrtc-calls.svelte.ts (iceServers)
Voir : github-resources.md → turn-rs
```

### P2 — Fermeture auto sondages côté backend
```
Durée estimée : 1h
Impact : Sondages se ferment automatiquement à la date définie
Fichiers : backend/src/polls.rs → check closed_at à GET /api/polls
```

### P3 — Métriques système dans l'admin
```
Durée estimée : 2h
Impact : Admin peut voir CPU/RAM/disque du Zimaboard depuis Nook
Fichiers : backend/src/admin.rs (route GET /api/admin/metrics via sysinfo)
           frontend/src/routes/admin/+page.svelte
```

---

## 🗺️ Roadmap LOT 4 (sessions suivantes)

- **Android TWA** — packager la PWA Nook pour le Play Store familial
- **Export PGN échecs** — `GET /api/chess/{id}/pgn` via pgn.rs existant
- **Backup chiffré SQLite** — cron quotidien dans le container
- ~~**SFU groupe** — rustrtc pour appels famille 4+ personnes~~ ✅ FAIT LOT 6
- **Sondages : notification WS** — `new_poll` broadcast à la création

---

## 🏗️ Décisions techniques immuables

| Décision | Raison | Ne pas changer |
|---|---|---|
| **Axum 0.8** (pas Actix) | API moderne, équipe Tokio, compatibilité tower | ✅ |
| **SQLite** (pas PostgreSQL) | Zimaboard homeserver, pas de migration réseau, backup trivial | ✅ |
| **SvelteKit 5 Runes** (pas React/Vue) | Migration faite S30+, pièges documentés | ✅ |
| **Docker distroless** | Surface d'attaque minimale, pas de shell | ✅ |
| **WebSocket unique `/ws`** | Simplifie auth, routing via to_user_id | ✅ |
| **Workflows manuels CI** | Projet familial, pas de CI automatique sur push | ✅ |

---

## 🔑 Variables d'environnement — Zimaboard

```bash
# .env sur le Zimaboard (référence)
NOOK_IMAGE=ghcr.io/mx10-ac2n/nook:latest
HOST_PORT=6300
DATA_DIR=/media/docker_Nook/nook-data
LOGS_DIR=/media/docker_Nook/nook-logs
PUBLIC_SITE_URL=http://192.168.X.X:6300
ALLOWED_ORIGINS=http://192.168.X.X:6300,https://nook.mondomaine.com
RUST_LOG=info
TZ=Europe/Paris
E2E_SETUP=0
# À ajouter LOT 3 :
# TURN_SECRET=<secret>
```

---

## 🧪 Score E2E — Historique

| Session | Score | Test qui échouait | Fix |
|---|---|---|---|
| S41 | 43/43 | — | Stable |
| S42 | 73/74 | Chat : isSingleEmoji non définie | Ajout fonction |
| S43 | 73/74 | Chat : isSingleEmoji non définie (non intégré) | Re-livré |
| S44 run1 | 97/98 | Calendrier : `.calendar-grid` absent | Renommage classes CSS |
| S44 run2 | 98/98 | — | Attendu ✅ |

---

## 📁 Structure fichiers .hermes

```
.hermes/
├── BUGS.md              ← bugs actifs + résolus
├── SESSIONS.md          ← historique sessions (référence longue durée)
├── TEST_REPORT.md       ← dernier rapport E2E (généré par CI)
├── rules/
│   ├── memory-sessions.md   ← résumé sessions récentes
│   ├── memory-decisions.md  ← décisions architecturales
│   ├── critical-pitfalls.md ← pièges à ne jamais reproduire
│   ├── api-and-db.md        ← surface API complète
│   ├── frontend-and-business.md ← stores + règles métier
│   └── debug-and-glossary.md    ← patterns de debug + glossaire
├── tools/
│   ├── nook-resources.md    ← ce fichier : état + roadmap
│   ├── github-resources.md  ← projets GitHub applicables
│   ├── rust-resources.md    ← crates Rust + pièges
│   ├── svelte5-resources.md ← patterns Svelte 5 + libs UI
│   ├── libui-resources.md   ← bibliothèques UI Svelte détaillées
│   └── monitoring-resources.md ← outils monitoring
└── roles/               ← rôles agents (architect, rust-backend, etc.)
```
