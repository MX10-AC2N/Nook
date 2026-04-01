# Session 45 — 2026-04-01 — Fix notifications + audit échecs

## PR #23: Fix Push Notifications
**Root cause:** Service Worker never registered on any device
- svelte.config.js has serviceWorker.register: false
- Zero manual navigator.serviceWorker.register() calls anywhere in frontend
- Result: SW never runs, push subscription fails silently, notifications never arrive

**Fix:**
- Created +layout.ts with manual SW registration on window.load
- Fixed service-worker.ts icon path: /reply.png → /icons/replay.svg
- Push now works on Desktop Chrome/Edge/Firefox, Android Chrome, macOS Safari
- iOS Safari: NOT supported (Apple limitation)

## PR #23: Fix Échecs (Chess)
**Backend (chess.rs):**
- AI checkmate winner_id was always None → correctly set based on AI color
- player2_id was hardcoded null → read actual value from DB
- current_turn always 1 after AI → alternates 1↔2 correctly

**Frontend (chessStore + game page):**
- 4x $derived arrow function trap: $derived(() => ...) → $derived.by(() => ...)
  - myColor was returning a function object, not a value
  - board was returning a function object
  - kingInCheckSquare: red king check highlight NEVER shown (=== comparison always false)
  - mySlot: same trap
- Result modal never triggered → added $effect auto-show on game end
- WS refreshGame clobbered selected/legalTargets → now preserves user selection
- All 8 fetch calls now have AbortController timeouts (10-15s)
- WS reconnection limit increased 8 → 12
- Exposed wsConnected/wsReconnecting reactive state

## Audit Complet Échecs
- 12 backend files (6000 lines Rust) + 3 frontend files (1700 lines TS/Svelte)
- Engine correctness verified: castling, en passant, promotion, check/mate/stalemate/FEN/PGN all correct
- 3 backend bugs fixed (winner_id, player2_id, turn alternation)
- 6 frontend bugs fixed ($derived traps, result modal, WS clobber, fetch timeouts)
- 7 E2E tests exist, 18 scenarios not tested (castling, en passant, promotion, check, timers, etc.)

## Fichiers modifiés
- frontend/src/routes/+layout.ts (new)
- frontend/src/service-worker.ts (icon path)
- backend/src/chess.rs (3 fixes)
- frontend/src/lib/chessStore.svelte.ts (6+ fixes)
- frontend/src/routes/chess/[game_id]/+page.svelte (modal + derived fixes)
