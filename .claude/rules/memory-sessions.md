## Session 43 — 2026-03-29 — LOT 1+2 : Upload, Scroll, Badge, Calendrier, Mode sombre, Polls

### Contexte
Suite du plan CEO v0.5. Objectif : stabilisation + expérience. 7 fichiers produits en une seule passe.

### Corrections LOT 1 (stabilisation)

**🔴 R_UPLOAD_7MO** — Upload > 7Mo : "failed to read"
- Axum `DefaultBodyLimit` par défaut = 2MB → tout upload > 2Mo échoue avec 413
- Fix : `DefaultBodyLimit::max(52 * 1024 * 1024)` appliqué sur le Router global dans main.rs
- La validation métier 50Mo reste dans upload.rs
- Layer appliqué AVANT `.layer(cors_layer)` pour éviter rejet prématuré

**🔴 R_SCROLL_CHAT** — Scroll chat ne suit pas les nouveaux messages
- `$effect` existant recalculait scrollTop mais sans vérifier si l'user avait remonté
- Fix : tolérance 150px — si `scrollHeight - scrollTop - clientHeight < 150` → scroll auto
- Ajout scroll forcé après `loadMessages` dans `selectConversation`

**🔴 R_BADGE_MENU** — Badge non-lu absent dans le menu navigation
- `unreadCounts` existait dans chatStore mais jamais affiché dans le menu burger
- Fix : `totalUnread = $derived(Object.values(chatStore.unreadCounts).reduce(...))` dans +layout.svelte
- Badge vert `.nav-badge` sur l'item Chat, lien actif `.active` avec CSS

### Corrections LOT 2 (expérience)

**🟡 R_CALENDAR_EDIT** — Calendrier : click, édition, suppression
- Calendrier refait entièrement : click sur case → modal détail/édition/suppression
- Jour actuel mis en avant (fond accent + numéro en cercle)
- Multi-événements par jour → modal liste intermédiaire
- Nouveau handler backend `update_event` (PATCH /api/events/{id}) dans db.rs
- Route ajoutée dans main.rs : `axum::routing::patch(db::update_event)`

**🟡 R_THEME_DARK** — Mode sombre global absent
- Nouveau thème `nuit-douce` : gris ardoise profond + violet lavande, adapté pour le soir
- Fichier CSS : `frontend/src/lib/ui/themes/nuit-douce.css`
- ThemeStore : type étendu + thème ajouté dans `availableThemes` + `applyTheme` mis à jour
- app.css : import ajouté
- `getSystemTheme()` : préférence dark → `nuit-douce` (au lieu de space-hub)

**🟡 R_POLLS_CLOSING** — Sondages : date de clôture automatique
- Nouveau champ `closingDate` (date HTML) dans le formulaire de création
- Converti en timestamp unix `closes_at` envoyé au backend
- Affiché sur la carte sondage si non encore fermé

### Fichiers modifiés session 43

- `backend/src/main.rs` — DefaultBodyLimit 52MB + route PATCH /events/{id}
- `backend/src/db.rs` — `update_event` handler PATCH
- `frontend/src/routes/+layout.svelte` — badge non-lu + chatStore import + nav actif + version
- `frontend/src/routes/chat/+page.svelte` — scroll intelligent (tolérance 150px)
- `frontend/src/routes/calendar/+page.svelte` — refonte complète avec modals
- `frontend/src/routes/polls/+page.svelte` — date clôture automatique
- `frontend/src/lib/ui/ThemeStore.svelte.ts` — thème nuit-douce
- `frontend/src/lib/ui/themes/nuit-douce.css` — NOUVEAU fichier thème sombre
- `frontend/src/app.css` — import nuit-douce.css

### Ce qui reste (LOT 3 — session 44)

- [ ] Messages vocaux : tester iOS Safari + format MP4 fallback
- [ ] Appels audio/vidéo : valider après fixes S42 en conditions réelles LAN
- [ ] Serveur TURN (turn-rs) pour appels WAN — docker-compose + iceServers config
- [ ] Sondages : fermeture auto côté backend (cron ou check à la lecture)
- [ ] GIFs : documenter workflow fetch-gifs.yml ou pack open-source
- [ ] Android TWA / Capacitor
- [ ] Export PGN parties d'échecs
- [ ] Backup chiffré automatique DB
