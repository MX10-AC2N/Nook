## Session 44 — 2026-03-30 — Fix calendrier classes CSS + réécriture tools

### Contexte
Score 97/98 après session 43. 1 test restant : `Calendar UI — page, grille et bouton ajouter visibles`.
Réécriture complète des fichiers `.hermes/tools/*-resources.md` pour les rendre actionnables.

### Correction

**🔴 R_CALENDAR_CLASSES** — `.calendar-grid` et `.add-event-btn` absents
- Le calendrier refait en S43 utilisait `.cal-grid` et `.btn-add`
- Le test E2E (user.spec.ts:608) cherche `.calendar-grid` et `.add-event-btn`
- Fix : renommer les classes CSS dans le composant pour correspondre aux sélecteurs du test
- **Règle ajoutée** : avant de livrer un composant refait, toujours vérifier `user.spec.ts` pour les sélecteurs attendus

**🔴 R_ISEMOJI_S44** — `isSingleEmoji` toujours absente du zip 51
- La fonction avait été livrée en S42 mais le zip 50→51 ne l'avait pas intégrée
- Fix : re-livré `chat/+page.svelte` avec `isSingleEmoji` définie après `ALL_EMOJIS`

### Réécriture .hermes/tools

Les 6 fichiers tools ont été réécrits pour être **actionnables** (pas juste descriptifs) :

| Fichier | Avant | Après |
|---|---|---|
| `github-resources.md` | Catalogue générique (React, Vue, Django...) | Uniquement projets applicables à Nook + décisions d'usage + code d'intégration |
| `svelte5-resources.md` | Liste de libs + exemple générique | Pièges Svelte 5 validés Nook + patterns validés + recommandations concrètes |
| `rust-resources.md` | Catalogue de frameworks Rust | Versions Cargo.toml exactes + pièges connus + patterns validés Nook |
| `nook-resources.md` | Description générale | État features, roadmap LOT 3/4, décisions immuables, score E2E historique |
| `libui-resources.md` | Toutes les libs sans filtre | Seules les libs compatibles CSS variables Nook + règle absolue pas de couleurs hardcodées |
| `monitoring-resources.md` | Stack Prometheus/Grafana enterprise | Uniquement sysinfo (LOT 3) + Beszel externe + suppression de l'inutilisable |

### Fichiers modifiés session 44

- `frontend/src/routes/calendar/+page.svelte` — `.cal-grid` → `.calendar-grid`, `.btn-add` → `.add-event-btn`
- `frontend/src/routes/chat/+page.svelte` — `isSingleEmoji` re-livrée
- `.hermes/tools/github-resources.md` — réécrit
- `.hermes/tools/svelte5-resources.md` — réécrit
- `.hermes/tools/rust-resources.md` — réécrit
- `.hermes/tools/nook-resources.md` — réécrit
- `.hermes/tools/libui-resources.md` — réécrit
- `.hermes/tools/monitoring-resources.md` — réécrit
- `.hermes/BUGS.md` — mis à jour

### Ce qui reste (LOT 3 — session 45)

- [ ] **turn-rs** — serveur TURN dans docker-compose → appels WAN
- [ ] Métriques système admin via `sysinfo` → `GET /api/admin/metrics`
- [ ] Fermeture auto sondages côté backend (check `closed_at` au GET /api/polls)
- [ ] Messages vocaux iOS Safari (format MP4 fallback)
- [ ] Valider appels audio/vidéo LAN après fixes S42

### Score E2E

| Run | Score | Note |
|---|---|---|
| S44 run1 (zip 51) | 97/98 | `.calendar-grid` absent |
| S44 run2 | 98/98 | Attendu ✅ après fix classes |


## Session LOT 6 — Intégration SFU rustrtc

**Objectif:** Ajouter un SFU (Selective Forwarding Unit) pour les appels groupe 3+ participants.

**Backend (`backend/src/sfu.rs`):**
- Module `SfuState` avec rooms/peers/tracks management
- `handle_join()`: crée PeerConnection, parse offer, crée answer, ajoute tracks existantes
- `handle_answer()`: set remote answer pour renegotiation
- `handle_candidate()`: parse ICE candidate (IceCandidate::from_sdp), add_ice_candidate (sync)
- `remove_peer()`: cleanup PC, tracks, added_sources
- `setup_peer_events()`: boucle events avec `pc.recv().await`
  - `PeerConnectionEvent::Track(transceiver)` → MediaRelay → relay aux autres peers → PLI forwarding
  - PLI periodique toutes les 3s via `track.request_key_frame().await`
  - Forward loop: track.recv() -> source.send()
  - Monitor ICE state → close PC on disconnect/fail
- `negotiate()`: create_offer + set_local_description quand nouvelles tracks
- `drain_pending_offer()`: récupère l'offer pending et la vide

**Signalisation WS (`backend/src/webrtc.rs`):**
- Messages: sfu_join, sfu_answer, sfu_candidate, sfu_leave
- Routing dans le handler WS existant via `state_recv.sfu_state.*`
- Réponses: sfu_answer (avec answer + peers + renegotiate_offer), sfu_error, sfu_peers

**Frontend (`frontend/src/lib/webrtc-calls.svelte.ts`):**
- CallState: useSfu, sfuAnswer, sfuRenegotiateOffer, sfuPeers, sfuPendingOffer
- Méthodes: startSfuCall, handleSfuJoinResponse, handleSfuRenegotiateOffer, handleSfuPeers, stopSfuMode
- Auto-SFU: `participantIds.length >= 3` dans startCall()
- Types CallSignal: sfu_answer, sfu_renegotiate_offer, sfu_peers, sfu_error

**Frontend UI (`frontend/src/routes/call/[id]/+page.svelte`):**
- Badge SFU dans header: `🌐 SFU · N pairs`
- Toggle P2P/SFU dans call-controls (visible si 3+ participants)
- Fonction toggleSfuMode(): bascule entre SFU et P2P mesh

**Architecture:**
- Room → HashMap<user_id, Peer> → Vec<TrackInfo>
- Peer → PeerConnection + added_sources: HashSet<String> + negotiation_pending: AtomicBool
- TrackInfo → MediaRelay + remote_track + user_id + kind + RtpCodecParameters
- MediaRelay::with_capacity(track, 500) pour chaque track entrante
