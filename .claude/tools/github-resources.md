# 🔗 GitHub Resources — Nook

> Projets GitHub **directement applicables** à Nook. Pour chaque entrée :
> décision d'usage, priorité, et comment l'intégrer concrètement.
> Mis à jour : session 44

---

## 📞 WebRTC & Appels (priorité haute)

### turn-rs — Serveur TURN/STUN Rust pur
**Repo :** https://github.com/mycrl/turn-rs  
**Décision :** ✅ **À intégrer LOT 3** — résout les appels WAN (actuellement instables hors LAN)  
**Pourquoi :** Rust pur, <35µs latence, tourne sur Zimaboard ARM64, licence MIT  
**Intégration :**
```yaml
# docker-compose.yml — ajouter après le service nook
turn:
  image: ghcr.io/mycrl/turn-rs:latest
  ports: ["3478:3478/udp", "3478:3478/tcp"]
  environment:
    TURN_REALM: nook.local
    TURN_SECRET: ${TURN_SECRET}
```
```typescript
// webrtc-calls.svelte.ts — iceServers
iceServers: [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'turn:192.168.X.X:3478', username: 'nook', credential: '${TURN_SECRET}' },
]
```

### rustrtc — SFU WebRTC haute performance
**Repo :** https://github.com/restsend/rustrtc  
**Décision :** 🔵 **LOT 4** — remplace le mesh P2P pour les appels groupe (4+ personnes)  
**Pourquoi :** 2.8x plus rapide que pion, SFU intégré, Rust natif  
**Note :** Intégration lourde — prioriser turn-rs d'abord

---

## 📊 Monitoring (priorité moyenne)

### rustmon — Dashboard monitoring web
**Repo :** https://github.com/imdadareeph/rustmon  
**Décision :** 🟡 **Optionnel** — utile pour une page admin `/admin/system`  
**Pourquoi :** Stack Rust + Axum (identique à Nook), WebSocket temps réel, Docker stats  
**Intégration :** Exposer `/api/metrics` dans Nook via `sysinfo` crate, afficher dans admin

### Beszel — Dashboard multi-serveurs
**Repo :** https://github.com/henrygd/beszel  
**Décision :** 🟡 **Outil externe** — pas intégré dans Nook, déployé séparément sur le Zimaboard  
**Usage :** Surveiller CPU/RAM/Docker du Zimaboard depuis un navigateur

---

## ♟️ Échecs (référence moteur)

### Walleye — Moteur UCI Rust
**Repo :** https://github.com/MitchelPaulin/Walleye  
**Décision :** 📌 **Référence** — notre moteur actuel est custom, Walleye sert de comparaison  
**Usage :** Comparer les optimisations (Killer Moves, MVV-LVA, PV Search)

---

## 💬 Messagerie (référence architecture)

### rustchat — Chat Rust + Axum (comme Nook)
**Repo :** https://github.com/rustchatio/rustchat  
**Décision :** 📌 **Référence architecture** — même stack (Axum + SQLx + WebSocket)  
**Usage :** S'inspirer des patterns pour le temps réel, la gestion des rooms, les notifications

---

## ❌ Non retenus pour Nook

| Projet | Raison |
|---|---|
| videocall-rs | Stack incompatible (Actix + NATS + PostgreSQL) — trop lourd |
| chess-tui | TUI terminal — pas utile pour une app web |
| React/Vue/Express/Django/FastAPI | Nook est Svelte 5 + Rust, pas de migration prévue |
| webpack | Nook utilise Vite, pas webpack |

---

## 📝 Notes

- **Priorité LOT 3 :** turn-rs en docker-compose → appels WAN fonctionnels
- **Priorité LOT 4 :** rustrtc SFU → appels groupe famille
- **À ne pas changer :** Stack core Axum 0.8 + SQLite + SvelteKit 5 — mature et stable
