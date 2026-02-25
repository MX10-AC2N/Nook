<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    chessStore,
    PIECE_SYMBOLS,
    PIECE_SYMBOLS_FILLED,
    PLAYER_LABELS,
    PLAYER_COLORS_HEX,
    type ChessPiece,
  } from '$lib/chessStore.svelte.ts';

  const gameId = $derived($page.params.game_id);

  let showResignConfirm  = $state(false);
  let showPromotionModal = $state(false);
  let pendingPromoMove   = $state<{ row: number; col: number } | null>(null);

  const PROMOTION_CHOICES = ['queen', 'rook', 'bishop', 'knight'] as const;
  const PROMO_LABELS: Record<string, string> = {
    queen: 'Reine', rook: 'Tour', bishop: 'Fou', knight: 'Cavalier'
  };

  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    await chessStore.loadGame(gameId);
  });

  onDestroy(() => chessStore.disconnectWebSocket());

  // ── Helpers ──────────────────────────────────────────────────

  function isCellInBounds(row: number, col: number): boolean {
    if (!chessStore.currentGame) return false;
    const pc = chessStore.currentGame.player_count;
    if (pc === 2) return true;
    if (row < 4 && col < 4) return false;
    if (row < 4 && col > 9) return false;
    if (row > 9 && col < 4) return false;
    if (row > 9 && col > 9) return false;
    return true;
  }

  function pieceAt(row: number, col: number): ChessPiece | null {
    return chessStore.currentGame?.board_state.find(
      p => p.row === row && p.col === col && p.alive
    ) ?? null;
  }

  function isValidTarget(row: number, col: number) {
    return chessStore.validMoves.find(m => m.row === row && m.col === col) ?? null;
  }

  function isSelected(piece: ChessPiece): boolean {
    return chessStore.selectedPiece?.id === piece.id;
  }

  function isLastMoveFrom(row: number, col: number): boolean {
    return chessStore.lastMoveHighlight?.from[0] === row &&
           chessStore.lastMoveHighlight?.from[1] === col;
  }

  function isLastMoveTo(row: number, col: number): boolean {
    return chessStore.lastMoveHighlight?.to[0] === row &&
           chessStore.lastMoveHighlight?.to[1] === col;
  }

  function getCellClass(row: number, col: number): string {
    if (!isCellInBounds(row, col)) return 'cell cell-void';
    const light = (row + col) % 2 === 0;
    const vm    = isValidTarget(row, col);
    let cls = `cell ${light ? 'cell-light' : 'cell-dark'}`;
    if (vm?.capture)         cls += ' cell-capture';
    else if (vm)             cls += ' cell-valid';
    if (isLastMoveFrom(row, col) || isLastMoveTo(row, col)) cls += ' cell-last-move';
    return cls;
  }

  function getPieceSymbol(piece: ChessPiece): string {
    const filled = piece.color === 'black' || piece.color === 'red';
    return filled ? PIECE_SYMBOLS_FILLED[piece.piece_type] : PIECE_SYMBOLS[piece.piece_type];
  }

  // ── Clic sur une case ────────────────────────────────────────

  function handleCellClick(row: number, col: number) {
    if (!chessStore.isMyTurn || !chessStore.currentGame) return;
    if (!isCellInBounds(row, col)) return;

    const vm = isValidTarget(row, col);
    if (vm) {
      const piece = chessStore.selectedPiece;
      if (piece?.piece_type === 'pawn') {
        const bs = chessStore.currentGame.player_count === 2 ? 8 : 14;
        const promotes =
          (piece.color === 'white' && row === 0) ||
          (piece.color === 'black' && row === bs - 1) ||
          (piece.color === 'red'   && col === bs - 1) ||
          (piece.color === 'green' && col === 0);
        if (promotes) {
          pendingPromoMove = { row, col };
          showPromotionModal = true;
          return;
        }
      }
      chessStore.playMove(row, col);
    } else {
      const p = pieceAt(row, col);
      if (p && p.color === chessStore.myColor) chessStore.selectPiece(p);
      else chessStore.selectPiece(null);
    }
  }

  async function handlePromotion(choice: string) {
    if (pendingPromoMove) await chessStore.playMove(pendingPromoMove.row, pendingPromoMove.col, choice);
    showPromotionModal = false;
    pendingPromoMove = null;
  }

  // ── Slots joueurs ─────────────────────────────────────────────

  const playerSlots = $derived(() => {
    const g = chessStore.currentGame;
    if (!g) return [];
    return [
      { slot: 1, id: g.player1_id, color: g.player1_color },
      { slot: 2, id: g.player2_id, color: g.player2_color },
      { slot: 3, id: g.player3_id, color: g.player3_color },
      { slot: 4, id: g.player4_id, color: g.player4_color },
    ].slice(0, g.player_count);
  });
</script>

<svelte:head>
  <title>
    {chessStore.currentGame ? `♟ Partie ${chessStore.currentGame.id.slice(0,8)}` : '♟ Chargement…'}
  </title>
</svelte:head>

<div class="chess-page">

  {#if chessStore.loading && !chessStore.currentGame}
    <div class="loading-full">
      <div class="spinner-lg"></div>
      <p>Chargement de la partie…</p>
    </div>

  {:else if !chessStore.currentGame}
    <div class="error-state">
      <p>Partie introuvable.</p>
      <a href="/chess" class="btn-back">← Retour au lobby</a>
    </div>

  {:else}
    {@const game = chessStore.currentGame}
    <div class="game-layout">

      <!-- ── Sidebar ── -->
      <aside class="sidebar">
        <a href="/chess" class="back-link">← Lobby</a>

        <div class="players-panel">
          <h2>Joueurs</h2>
          {#each playerSlots() as { slot, id, color } (slot)}
            <div
              class="player-row"
              class:is-turn={game.current_turn === slot && game.status === 'playing'}
              class:is-eliminated={game.eliminated.includes(slot)}
              class:is-me={chessStore.mySlot === slot}
            >
              <div class="dot" style="background:{PLAYER_COLORS_HEX[color]}"></div>
              <div class="pinfo">
                <span class="plabel">
                  {PLAYER_LABELS[slot] ?? `Joueur ${slot}`}
                  {#if chessStore.mySlot === slot}
                    <span class="you">(vous)</span>
                  {/if}
                </span>
                <span class="pid">{id ? id.slice(0,10)+'…' : 'En attente…'}</span>
              </div>
              <span class="pstatus">
                {#if game.eliminated.includes(slot)}☠
                {:else if game.current_turn === slot && game.status === 'playing'}▶
                {:else}–{/if}
              </span>
            </div>
          {/each}
        </div>

        <!-- Statut -->
        <div class="status-panel">
          {#if game.status === 'waiting'}
            <div class="banner waiting">🟡 En attente des joueurs</div>
          {:else if game.status === 'playing'}
            {#if chessStore.isMyTurn}
              <div class="banner your-turn">✅ À vous de jouer !</div>
            {:else}
              <div class="banner wait">⏳ Tour de {PLAYER_LABELS[game.current_turn] ?? `Joueur ${game.current_turn}`}</div>
            {/if}
          {:else if game.status === 'finished'}
            <div class="banner finished">
              🏆 Partie terminée !
              {#if game.winner_id}
                <small>{game.winner_id.slice(0,12)} a gagné</small>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Abandon -->
        {#if game.status === 'playing' && chessStore.mySlot !== null}
          {#if !showResignConfirm}
            <button class="btn-resign" onclick={() => showResignConfirm = true}>
              🏳 Abandonner
            </button>
          {:else}
            <div class="resign-confirm">
              <p>Confirmer l'abandon ?</p>
              <div class="resign-btns">
                <button class="rbtn-yes" onclick={() => { chessStore.resign(); showResignConfirm = false; }}>Oui</button>
                <button class="rbtn-no"  onclick={() => showResignConfirm = false}>Non</button>
              </div>
            </div>
          {/if}
        {/if}

        {#if chessStore.error}
          <div class="sidebar-error">⚠️ {chessStore.error}</div>
        {/if}
      </aside>

      <!-- ── Plateau ── -->
      <main class="board-container">
        <div
          class="chess-board"
          class:board-2p={game.player_count === 2}
          class:board-4p={game.player_count > 2}
        >
          {#each { length: chessStore.boardSize } as _, row}
            {#each { length: chessStore.boardSize } as _, col}
              {@const piece = pieceAt(row, col)}
              {@const vm    = isValidTarget(row, col)}
              <div
                class={getCellClass(row, col)}
                role="button"
                tabindex={isCellInBounds(row, col) ? 0 : -1}
                onclick={() => handleCellClick(row, col)}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') handleCellClick(row, col);
                }}
              >
                {#if isCellInBounds(row, col)}
                  {#if col === 0}
                    <span class="coord coord-r">{chessStore.boardSize - row}</span>
                  {/if}
                  {#if row === chessStore.boardSize - 1}
                    <span class="coord coord-c">{String.fromCharCode(65 + col)}</span>
                  {/if}

                  {#if piece}
                    <span
                      class="piece"
                      class:selected={isSelected(piece)}
                      class:can-move={piece.color === chessStore.myColor && chessStore.isMyTurn}
                      style="color:{PLAYER_COLORS_HEX[piece.color]}"
                    >
                      {getPieceSymbol(piece)}
                    </span>
                  {:else if vm && !vm.capture}
                    <span class="valid-dot"></span>
                  {/if}
                {/if}
              </div>
            {/each}
          {/each}
        </div>
      </main>

    </div>

    <!-- ── Modal promotion ── -->
    {#if showPromotionModal}
      <div class="modal-backdrop">
        <div class="modal-promo" role="dialog">
          <h3>Promouvoir le pion</h3>
          <div class="promo-choices">
            {#each PROMOTION_CHOICES as choice}
              <button
                class="promo-btn"
                onclick={() => handlePromotion(choice)}
                style="color:{PLAYER_COLORS_HEX[chessStore.myColor ?? 'white']}"
              >
                <span class="promo-symbol">{PIECE_SYMBOLS[choice]}</span>
                <span class="promo-label">{PROMO_LABELS[choice]}</span>
              </button>
            {/each}
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .chess-page { min-height: 100vh; background: var(--color-bg, #f8fafc); }

  .loading-full, .error-state {
    min-height: 60vh; display: flex; flex-direction: column;
    align-items: center; justify-content: center; gap: 1rem; color: #64748b;
  }
  .spinner-lg {
    width: 40px; height: 40px; border: 4px solid #e2e8f0; border-top-color: #2d5a27;
    border-radius: 50%; animation: spin 1s linear infinite;
  }
  .btn-back {
    padding: 0.75rem 1.5rem; background: #2d5a27; color: white; border-radius: 0.5rem;
    text-decoration: none; font-weight: 600;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Layout ── */
  .game-layout {
    display: grid; grid-template-columns: 220px 1fr;
    gap: 1.5rem; padding: 1.5rem; align-items: start;
  }

  /* ── Sidebar ── */
  .sidebar { display: flex; flex-direction: column; gap: 1rem; position: sticky; top: 1rem; }
  .back-link { font-size: 0.85rem; color: #64748b; text-decoration: none; }
  .back-link:hover { color: #2d5a27; }

  .players-panel {
    background: white; border: 1px solid #e2e8f0; border-radius: 0.875rem; padding: 1rem;
  }
  .players-panel h2 { margin: 0 0 0.75rem; font-size: 0.9rem; font-weight: 700; color: #1e293b; }

  .player-row {
    display: flex; align-items: center; gap: 0.5rem; padding: 0.4rem 0.5rem;
    border-radius: 0.4rem; margin-bottom: 0.25rem; border: 2px solid transparent;
    transition: all 0.15s;
  }
  .player-row.is-turn    { background: #f0fdf4; border-color: #86efac; }
  .player-row.is-eliminated { opacity: 0.35; }
  .player-row.is-me      { font-weight: 700; }

  .dot {
    width: 11px; height: 11px; border-radius: 50%; flex-shrink: 0;
    border: 1.5px solid rgba(0,0,0,0.15);
  }
  .pinfo { flex: 1; min-width: 0; }
  .plabel { display: block; font-size: 0.82rem; font-weight: 600; color: #1e293b; }
  .pid    { font-size: 0.72rem; color: #94a3b8; display: block; }
  .you    { color: #2d5a27; font-size: 0.72rem; }
  .pstatus { font-size: 0.9rem; color: #64748b; }

  .status-panel { }
  .banner {
    padding: 0.7rem 0.9rem; border-radius: 0.6rem; font-size: 0.82rem;
    font-weight: 600; text-align: center; line-height: 1.4;
  }
  .banner.waiting   { background: #fefce8; color: #854d0e; border: 1px solid #fde68a; }
  .banner.your-turn { background: #f0fdf4; color: #166534; border: 1px solid #86efac; }
  .banner.wait      { background: #f1f5f9; color: #475569; border: 1px solid #e2e8f0; }
  .banner.finished  { background: #fdf4ff; color: #7e22ce; border: 1px solid #e9d5ff; }
  .banner small     { display: block; font-size: 0.72rem; opacity: 0.7; margin-top: 0.2rem; }

  .btn-resign {
    width: 100%; padding: 0.55rem; background: #fef2f2; border: 1px solid #fecaca;
    color: #dc2626; border-radius: 0.5rem; font-size: 0.82rem; cursor: pointer;
  }
  .btn-resign:hover { background: #fee2e2; }

  .resign-confirm {
    background: #fef2f2; border: 1px solid #fecaca; border-radius: 0.6rem; padding: 0.75rem;
    font-size: 0.82rem;
  }
  .resign-confirm p { margin: 0 0 0.5rem; color: #dc2626; font-weight: 600; }
  .resign-btns { display: flex; gap: 0.4rem; }
  .rbtn-yes {
    flex: 1; padding: 0.35rem; background: #dc2626; color: white;
    border: none; border-radius: 0.35rem; font-size: 0.78rem; cursor: pointer;
  }
  .rbtn-no {
    flex: 1; padding: 0.35rem; background: #f1f5f9; color: #475569;
    border: none; border-radius: 0.35rem; font-size: 0.78rem; cursor: pointer;
  }
  .sidebar-error {
    padding: 0.55rem 0.7rem; background: #fef2f2; border: 1px solid #fecaca;
    border-radius: 0.5rem; color: #dc2626; font-size: 0.8rem;
  }

  /* ── Plateau ── */
  .board-container { display: flex; justify-content: center; align-items: flex-start; }

  .chess-board {
    display: grid;
    border: 3px solid #374151; border-radius: 4px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.18); overflow: hidden;
    width: min(75vw, 600px); aspect-ratio: 1;
  }
  .board-2p { grid-template-columns: repeat(8,  1fr); grid-template-rows: repeat(8,  1fr); }
  .board-4p { grid-template-columns: repeat(14, 1fr); grid-template-rows: repeat(14, 1fr); }

  /* ── Cases ── */
  .cell {
    position: relative; display: flex; align-items: center; justify-content: center;
    cursor: pointer; transition: background 0.1s; user-select: none;
  }
  .cell:focus-visible { outline: 2px solid #2d5a27; outline-offset: -2px; z-index: 1; }
  .cell-light    { background: #f0d9b5; }
  .cell-dark     { background: #b58863; }
  .cell-void     { background: #1a1a1a; cursor: default; }
  .cell-valid    { background: rgba(99, 190, 90, 0.5) !important; }
  .cell-capture  { background: rgba(220, 50, 50, 0.5) !important; }
  .cell-last-move { outline: 3px solid rgba(255, 196, 0, 0.7); outline-offset: -3px; }
  .cell:not(.cell-void):hover { filter: brightness(1.07); }

  .coord { position: absolute; font-size: 0.5rem; font-weight: 700; opacity: 0.45; pointer-events: none; }
  .coord-r { left: 2px; top: 2px; }
  .coord-c { right: 2px; bottom: 2px; }

  .piece {
    font-size: clamp(0.8rem, 3.2vw, 1.8rem); line-height: 1;
    text-shadow: 0 1px 3px rgba(0,0,0,0.5); cursor: pointer; z-index: 1;
    transition: transform 0.1s;
  }
  .piece.selected { transform: scale(1.3); filter: drop-shadow(0 0 5px rgba(255,220,0,0.9)); }
  .piece.can-move:hover { transform: scale(1.1); }

  .valid-dot {
    width: 30%; height: 30%; border-radius: 50%;
    background: rgba(0,0,0,0.22); pointer-events: none;
  }

  /* ── Modal promotion ── */
  .modal-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.55);
    display: flex; align-items: center; justify-content: center; z-index: 999;
  }
  .modal-promo {
    background: white; border-radius: 1rem; padding: 1.75rem; text-align: center;
    box-shadow: 0 16px 48px rgba(0,0,0,0.3); min-width: 260px;
  }
  .modal-promo h3 { margin: 0 0 1.25rem; font-size: 1.15rem; color: #1e293b; }
  .promo-choices { display: grid; grid-template-columns: 1fr 1fr; gap: 0.65rem; }
  .promo-btn {
    padding: 0.75rem; border: 2px solid #e2e8f0; border-radius: 0.7rem; background: white;
    cursor: pointer; display: flex; flex-direction: column; align-items: center; gap: 0.25rem;
    transition: all 0.15s;
  }
  .promo-btn:hover { border-color: #2d5a27; background: #f0fdf4; }
  .promo-symbol { font-size: 1.8rem; line-height: 1; }
  .promo-label  { font-weight: 600; color: #374151; font-size: 0.82rem; }

  /* ── Responsive ── */
  @media (max-width: 768px) {
    .game-layout { grid-template-columns: 1fr; }
    .sidebar { position: static; flex-direction: row; flex-wrap: wrap; }
    .players-panel { flex: 1; min-width: 180px; }
    .chess-board { width: min(95vw, 460px); }
  }
</style>
