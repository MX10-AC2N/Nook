<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    chessStore,
    toAlgebraic,
    fromAlgebraic,
    decodePiece,
    PIECE_UNICODE,
    PIECE_NAMES,
    statusLabel,
    getLegalTargets,
  } from '$lib/chessStore.svelte.ts';

  const gameId = $derived($page.params.game_id);

  // ── Clic sur une case ─────────────────────────────────────────
  async function handleClick(row: number, col: number) {
    if (chessStore.isGameOver) return;
    await chessStore.selectSquare(row, col);
  }

  // ── Classes CSS d'une case ────────────────────────────────────
  function cellClass(row: number, col: number): string {
    const alg     = toAlgebraic(row, col);
    const isLight = (row + col) % 2 === 0;
    let cls = `cell ${isLight ? 'cell-light' : 'cell-dark'}`;

    // Sélection
    if (chessStore.selected?.algebraic === alg) cls += ' cell-selected';

    // Case cible légale
    if (chessStore.legalTargets.includes(alg)) {
      const board = chessStore.board();
      const hasPiece = board[row]?.[col] !== '';
      cls += hasPiece ? ' cell-capture' : ' cell-target';
    }

    // Dernier coup
    if (chessStore.lastMove?.from === alg || chessStore.lastMove?.to === alg) {
      cls += ' cell-last';
    }

    return cls;
  }

  // ── Promotion ─────────────────────────────────────────────────
  const PROMO_PIECES = ['q', 'r', 'b', 'n'] as const;
  const PROMO_LABELS: Record<string, string> = { q: '♛ Dame', r: '♜ Tour', b: '♝ Fou', n: '♞ Cavalier' };

  let showResign = $state(false);

  // ── Cycle de vie ──────────────────────────────────────────────
  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    await chessStore.loadGame(gameId);
  });

  onDestroy(() => chessStore.disconnectWebSocket());

  // ── Infos joueurs ─────────────────────────────────────────────
  const myInfo = $derived(() => {
    const g = chessStore.currentGame;
    if (!g) return null;
    const uid = authStore.user?.id;
    if (g.player1_id === uid) return { color: g.player1_color, slot: 1 };
    if (g.player2_id === uid) return { color: g.player2_color, slot: 2 };
    return null; // spectateur
  });

  // Afficher le plateau orienté selon ma couleur (noirs = retourné)
  const flipped = $derived(chessStore.myColor() === 'black');

  // Rangées et colonnes selon orientation
  const rows = $derived(flipped ? [0,1,2,3,4,5,6,7].reverse() : [0,1,2,3,4,5,6,7]);
  const cols = $derived(flipped ? [0,1,2,3,4,5,6,7].reverse() : [0,1,2,3,4,5,6,7]);

  // Historique de coups paginé (les 20 derniers)
  const recentHistory = $derived(
    (chessStore.currentGame?.move_history ?? []).slice(-20)
  );
</script>

<svelte:head>
  <title>
    {chessStore.currentGame ? `♟ Partie — Nook` : '♟ Chargement…'}
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
    {@const engine = game.engine}

    <div class="game-layout">

      <!-- ══ SIDEBAR ══ -->
      <aside class="sidebar">
        <a href="/chess" class="back-link">← Lobby</a>

        <!-- Joueurs -->
        <div class="panel players-panel">
          <h3>Joueurs</h3>

          <!-- Joueur 2 (haut — adversaire si je joue blanc) -->
          <div class="player-row"
               class:active-turn={engine?.side_to_move === game.player2_color && game.status === 'playing'}
               class:is-me={myInfo()?.slot === 2}>
            <div class="color-dot" class:dot-white={game.player2_color === 'white'}
                                   class:dot-black={game.player2_color === 'black'}></div>
            <div class="player-info">
              <span class="player-name">
                {game.player2_id
                  ? (myInfo()?.slot === 2 ? 'Vous' : (game.player2_id.slice(0,10) + '…'))
                  : (game.ai_difficulty ? `🤖 IA (${game.ai_difficulty})` : 'En attente…')}
              </span>
              <span class="player-color">{game.player2_color === 'white' ? '♙ Blancs' : '♟ Noirs'}</span>
            </div>
            {#if engine?.side_to_move === game.player2_color && game.status === 'playing'}
              <span class="turn-arrow">▶</span>
            {/if}
          </div>

          <!-- Joueur 1 (bas — moi si je joue blanc) -->
          <div class="player-row"
               class:active-turn={engine?.side_to_move === game.player1_color && game.status === 'playing'}
               class:is-me={myInfo()?.slot === 1}>
            <div class="color-dot" class:dot-white={game.player1_color === 'white'}
                                   class:dot-black={game.player1_color === 'black'}></div>
            <div class="player-info">
              <span class="player-name">
                {game.player1_id
                  ? (myInfo()?.slot === 1 ? 'Vous' : (game.player1_id.slice(0,10) + '…'))
                  : 'En attente…'}
              </span>
              <span class="player-color">{game.player1_color === 'white' ? '♙ Blancs' : '♟ Noirs'}</span>
            </div>
            {#if engine?.side_to_move === game.player1_color && game.status === 'playing'}
              <span class="turn-arrow">▶</span>
            {/if}
          </div>
        </div>

        <!-- Statut -->
        <div class="panel status-panel">
          {#if game.status === 'waiting'}
            <div class="banner waiting">🟡 En attente d'un adversaire</div>
          {:else if game.status === 'playing'}
            {#if chessStore.aiThinking}
              <div class="banner thinking">
                <span class="dots"><span></span><span></span><span></span></span>
                IA réfléchit…
              </div>
            {:else if chessStore.isMyTurn}
              <div class="banner your-turn">✅ À vous de jouer !</div>
            {:else}
              <div class="banner waiting-turn">
                ⏳ Tour des {engine?.side_to_move === 'white' ? 'Blancs' : 'Noirs'}
              </div>
            {/if}
          {:else}
            <div class="banner game-over">
              {statusLabel(game.status)}
              {#if game.winner_id}
                <small>
                  {game.winner_id === authStore.user?.id ? 'Vous avez gagné 🏆' : 'Vous avez perdu'}
                </small>
              {/if}
            </div>
            <a href="/chess" class="btn-new-game">Nouvelle partie</a>
          {/if}
        </div>

        <!-- Abandon -->
        {#if game.status === 'playing' && myInfo()}
          <div class="panel resign-panel">
            {#if !showResign}
              <button class="btn-resign" onclick={() => showResign = true}>🏳 Abandonner</button>
            {:else}
              <p class="resign-confirm-text">Confirmer l'abandon ?</p>
              <div class="resign-btns">
                <button class="rbtn-yes" onclick={() => { chessStore.resign(); showResign = false; }}>Oui</button>
                <button class="rbtn-no"  onclick={() => showResign = false}>Non</button>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Historique des coups -->
        {#if recentHistory.length > 0}
          <div class="panel history-panel">
            <h3>Historique</h3>
            <ol class="move-list">
              {#each recentHistory as move, i}
                <li class="move-item" class:white-move={move.color === 'white'}>
                  <span class="move-san">{move.san}</span>
                  <span class="move-by">{move.by}</span>
                </li>
              {/each}
            </ol>
          </div>
        {/if}

        {#if chessStore.error}
          <div class="panel error-panel">⚠️ {chessStore.error}</div>
        {/if}
      </aside>

      <!-- ══ PLATEAU ══ -->
      <main class="board-wrap">
        <div class="board-container">

          <!-- Lettres colonnes (haut) -->
          <div class="coords-top">
            {#each cols as c}
              <span>{String.fromCharCode(97 + c)}</span>
            {/each}
          </div>

          <div class="board-and-ranks">
            <!-- Chiffres rangées (gauche) -->
            <div class="coords-left">
              {#each rows as r}
                <span>{8 - r}</span>
              {/each}
            </div>

            <!-- Plateau 8×8 -->
            <div class="chess-board">
              {#each rows as row}
                {#each cols as col}
                  {@const piece = chessStore.board()[row]?.[col] ?? ''}
                  {@const decoded = decodePiece(piece)}
                  <div
                    class={cellClass(row, col)}
                    role="button"
                    tabindex="0"
                    onclick={() => handleClick(row, col)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleClick(row, col)}
                  >
                    {#if decoded}
                      <span
                        class="piece"
                        class:piece-mine={decoded.color === (chessStore.myColor() === 'white' ? 'w' : 'b')}
                        class:piece-selected={chessStore.selected?.algebraic === toAlgebraic(row, col)}
                        style="color: {decoded.color === 'w' ? '#fff' : '#1a1a1a'};
                               text-shadow: {decoded.color === 'w'
                                 ? '0 1px 3px rgba(0,0,0,0.8), 0 0 1px rgba(0,0,0,1)'
                                 : '0 1px 2px rgba(255,255,255,0.3)'};"
                      >
                        {PIECE_UNICODE[piece] ?? '?'}
                      </span>
                    {:else if chessStore.legalTargets.includes(toAlgebraic(row, col))}
                      <span class="target-dot"></span>
                    {/if}
                  </div>
                {/each}
              {/each}
            </div>

            <!-- Chiffres rangées (droite) -->
            <div class="coords-right">
              {#each rows as r}
                <span>{8 - r}</span>
              {/each}
            </div>
          </div>

          <!-- Lettres colonnes (bas) -->
          <div class="coords-bottom">
            {#each cols as c}
              <span>{String.fromCharCode(97 + c)}</span>
            {/each}
          </div>

        </div>
      </main>

    </div>

    <!-- ══ MODAL PROMOTION ══ -->
    {#if chessStore.pendingPromotion}
      <div class="modal-backdrop">
        <div class="modal-promo" role="dialog" aria-modal="true">
          <h3>Promouvoir le pion</h3>
          <div class="promo-grid">
            {#each PROMO_PIECES as p}
              <button
                class="promo-btn"
                onclick={() => chessStore.confirmPromotion(p)}
              >
                <span class="promo-piece"
                  style="color: {chessStore.myColor() === 'white' ? '#fff' : '#1a1a1a'};
                         text-shadow: {chessStore.myColor() === 'white'
                           ? '0 1px 3px rgba(0,0,0,0.8)'
                           : '0 1px 2px rgba(255,255,255,0.3)'};">
                  {PROMO_LABELS[p].split(' ')[0]}
                </span>
                <span class="promo-name">{PROMO_LABELS[p].split(' ')[1]}</span>
              </button>
            {/each}
          </div>
          <button class="promo-cancel" onclick={() => chessStore.cancelPromotion()}>
            Annuler
          </button>
        </div>
      </div>
    {/if}

  {/if}
</div>



<style>
  /* ── Page ── */
  .chess-page {
    min-height: 100vh;
    background: var(--color-bg, #f8fafc);
  }

  .loading-full, .error-state {
    min-height: 70vh;
    display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: 1rem; color: #64748b;
  }
  .spinner-lg {
    width: 40px; height: 40px;
    border: 3px solid #e2e8f0; border-top-color: #2d5a27;
    border-radius: 50%; animation: spin 1s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .btn-back {
    padding: .7rem 1.5rem; background: #2d5a27; color: #fff;
    border-radius: .5rem; text-decoration: none; font-weight: 600;
  }

  /* ── Layout ── */
  .game-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 1.5rem;
    padding: 1.25rem;
    align-items: start;
  }

  /* ── Sidebar ── */
  .sidebar {
    display: flex; flex-direction: column; gap: .75rem;
    position: sticky; top: 1rem;
  }
  .back-link { font-size: .83rem; color: #64748b; text-decoration: none; }
  .back-link:hover { color: #2d5a27; }

  .panel {
    background: #fff; border: 1px solid #e2e8f0;
    border-radius: .875rem; padding: .9rem;
  }
  .panel h3 {
    margin: 0 0 .65rem;
    font-size: .8rem; font-weight: 700;
    text-transform: uppercase; letter-spacing: .05em;
    color: #64748b;
  }

  /* Joueurs */
  .players-panel { }
  .player-row {
    display: flex; align-items: center; gap: .45rem;
    padding: .35rem .45rem; border-radius: .4rem;
    border: 2px solid transparent; margin-bottom: .25rem;
    transition: all .15s;
  }
  .player-row.active-turn { background: #f0fdf4; border-color: #86efac; }
  .player-row.is-me { font-weight: 700; }
  .color-dot {
    width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0;
    border: 1.5px solid #94a3b8;
  }
  .dot-white { background: #f8fafc; border-color: #94a3b8; }
  .dot-black { background: #1e293b; border-color: #64748b; }
  .player-info { flex: 1; min-width: 0; }
  .player-name { display: block; font-size: .82rem; font-weight: 600; color: #1e293b; }
  .player-color { font-size: .7rem; color: #94a3b8; }
  .turn-arrow { color: #22c55e; font-size: .75rem; }

  /* Bannières statut */
  .status-panel { }
  .banner {
    padding: .6rem .75rem; border-radius: .5rem;
    font-size: .82rem; font-weight: 600; text-align: center;
    line-height: 1.4;
  }
  .banner.waiting       { background: #fefce8; color: #854d0e; border: 1px solid #fde68a; }
  .banner.your-turn     { background: #f0fdf4; color: #166534; border: 1px solid #86efac; }
  .banner.waiting-turn  { background: #f8fafc; color: #475569; border: 1px solid #e2e8f0; }
  .banner.game-over     { background: #fdf4ff; color: #7e22ce; border: 1px solid #e9d5ff; }
  .banner.game-over small { display: block; font-size: .72rem; opacity: .75; margin-top: .2rem; }
  .banner.thinking {
    background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe;
    display: flex; align-items: center; justify-content: center; gap: .5rem;
  }

  /* Dots animation IA */
  .dots { display: flex; gap: 3px; }
  .dots span {
    width: 5px; height: 5px; border-radius: 50%; background: #1d4ed8;
    animation: bounce 1.2s infinite;
  }
  .dots span:nth-child(2) { animation-delay: .2s; }
  .dots span:nth-child(3) { animation-delay: .4s; }
  @keyframes bounce { 0%,80%,100% { transform: scale(0); } 40% { transform: scale(1); } }

  .btn-new-game {
    display: block; margin-top: .5rem;
    padding: .5rem; text-align: center;
    background: #2d5a27; color: #fff;
    border-radius: .45rem; text-decoration: none;
    font-size: .82rem; font-weight: 600;
    transition: background .15s;
  }
  .btn-new-game:hover { background: #3d7a37; }

  /* Historique */
  .history-panel { max-height: 180px; overflow-y: auto; }
  .move-list {
    list-style: none; margin: 0; padding: 0;
    display: flex; flex-direction: column; gap: .2rem;
  }
  .move-item {
    display: flex; justify-content: space-between; align-items: center;
    padding: .2rem .35rem; border-radius: .3rem; font-size: .78rem;
  }
  .move-item.white-move { background: #f8fafc; }
  .move-san { font-weight: 700; font-family: monospace; color: #1e293b; }
  .move-by  { color: #94a3b8; font-size: .7rem; }

  /* Erreur */
  .error-panel {
    background: #fef2f2; border-color: #fecaca;
    color: #dc2626; font-size: .82rem;
  }

  /* ── Plateau ── */
  .board-wrap {
    display: flex; justify-content: center; align-items: flex-start;
  }
  .board-container {
    display: flex; flex-direction: column; align-items: center; gap: 2px;
  }
  .board-and-ranks {
    display: flex; align-items: stretch; gap: 2px;
  }

  /* Coordonnées */
  .coords-top, .coords-bottom {
    display: grid; grid-template-columns: repeat(8, 1fr);
    width: min(74vw, 580px); padding: 0 2px;
  }
  .coords-top span, .coords-bottom span {
    text-align: center; font-size: .6rem; font-weight: 700;
    color: #94a3b8; line-height: 1.6;
  }
  .coords-left, .coords-right {
    display: flex; flex-direction: column;
  }
  .coords-left span, .coords-right span {
    flex: 1; display: flex; align-items: center; justify-content: center;
    font-size: .6rem; font-weight: 700; color: #94a3b8;
    width: 16px;
  }

  /* Grille 8×8 */
  .chess-board {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    grid-template-rows: repeat(8, 1fr);
    width: min(74vw, 580px);
    aspect-ratio: 1;
    border: 2.5px solid #374151;
    border-radius: 3px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0,0,0,.2);
  }

  /* Cases */
  .cell {
    position: relative;
    display: flex; align-items: center; justify-content: center;
    cursor: pointer;
    transition: filter .1s;
    user-select: none;
  }
  .cell:focus-visible {
    outline: 3px solid #f59e0b; outline-offset: -3px; z-index: 2;
  }
  .cell-light   { background: #f0d9b5; }
  .cell-dark    { background: #b58863; }
  .cell-selected { outline: 3px solid #f59e0b; outline-offset: -3px; z-index: 1; }
  .cell-target   { background: rgba(99, 200, 90, 0.45) !important; }
  .cell-capture  { background: rgba(220, 60, 60, 0.5) !important; }
  .cell-last     { background: rgba(255, 196, 0, 0.35) !important; }
  .cell:not(.cell-selected):hover { filter: brightness(1.08); }

  /* Pièces */
  .piece {
    font-size: clamp(1rem, 4.5vw, 2.2rem);
    line-height: 1;
    z-index: 1;
    transition: transform .1s;
    cursor: pointer;
  }
  .piece.piece-mine:hover    { transform: scale(1.12); }
  .piece.piece-selected      { transform: scale(1.25); filter: drop-shadow(0 0 6px rgba(245,158,11,.9)); }

  /* Point cible */
  .target-dot {
    width: 28%; height: 28%;
    border-radius: 50%;
    background: rgba(0,0,0,.22);
    pointer-events: none;
  }

  /* ── Modal promotion ── */
  .modal-backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,.6);
    display: flex; align-items: center; justify-content: center;
    z-index: 999;
  }
  .modal-promo {
    background: #fff; border-radius: 1rem;
    padding: 1.75rem; text-align: center;
    box-shadow: 0 16px 48px rgba(0,0,0,.3);
    min-width: 280px;
  }
  .modal-promo h3 { margin: 0 0 1.1rem; font-size: 1.1rem; color: #1e293b; }
  .promo-grid {
    display: grid; grid-template-columns: 1fr 1fr; gap: .6rem;
    margin-bottom: 1rem;
  }
  .promo-btn {
    padding: .8rem .5rem; border: 2px solid #e2e8f0; border-radius: .7rem;
    background: #fff; cursor: pointer;
    display: flex; flex-direction: column; align-items: center; gap: .2rem;
    transition: all .15s;
  }
  .promo-btn:hover { border-color: #2d5a27; background: #f0fdf4; }
  .promo-piece { font-size: 2rem; line-height: 1; }
  .promo-name  { font-size: .8rem; font-weight: 600; color: #374151; }
  .promo-cancel {
    padding: .4rem 1.2rem; background: #f1f5f9; color: #475569;
    border: none; border-radius: .45rem; cursor: pointer;
    font-size: .85rem; transition: background .15s;
  }
  .promo-cancel:hover { background: #e2e8f0; }

  /* ── Widget abandon ── */
  .resign-panel { padding: .65rem !important; }
  .btn-resign {
    width: 100%; padding: .5rem; background: #fef2f2;
    border: 1px solid #fecaca; color: #dc2626;
    border-radius: .45rem; font-size: .82rem; cursor: pointer;
    transition: background .15s;
  }
  .btn-resign:hover { background: #fee2e2; }
  .resign-confirm-text { margin: 0 0 .4rem; font-size: .82rem; color: #dc2626; font-weight: 600; }
  .resign-btns { display: flex; gap: .35rem; }
  .rbtn-yes {
    flex: 1; padding: .35rem; background: #dc2626; color: #fff;
    border: none; border-radius: .35rem; font-size: .78rem; cursor: pointer;
  }
  .rbtn-no {
    flex: 1; padding: .35rem; background: #f1f5f9; color: #475569;
    border: none; border-radius: .35rem; font-size: .78rem; cursor: pointer;
  }

  /* ── Responsive ── */
  @media (max-width: 768px) {
    .game-layout {
      grid-template-columns: 1fr;
    }
    .sidebar {
      position: static;
      flex-direction: row; flex-wrap: wrap;
    }
    .players-panel, .status-panel { flex: 1; min-width: 160px; }
    .history-panel { display: none; }
    .chess-board { width: min(94vw, 460px); }
    .coords-top, .coords-bottom { width: min(94vw, 460px); }
  }
</style>
