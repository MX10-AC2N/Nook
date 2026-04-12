<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    chessStore,
    toAlgebraic,
    decodePiece,
    PIECE_UNICODE,
    PIECE_NAMES,
    statusLabel,
  } from '$lib/chessStore.svelte.ts';

  const gameId = $derived($page.params.game_id);

  // ── Minuteur — formater MM:SS ──────────────────────────────────
  function formatTime(secs: number): string {
    if (secs <= 0) return '0:00';
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  // ── Trouver la case du roi en échec ───────────────────────────
  function findKingSquare(color: 'w' | 'b'): string | null {
    const board = chessStore.board;
    const king  = color + 'K';
    for (let r = 0; r < 8; r++) {
      for (let c = 0; c < 8; c++) {
        if (board[r]?.[c] === king) return toAlgebraic(r, c);
      }
    }
    return null;
  }

  // La case du roi est en échec si engine.status contient "check" ou "checkmate"
  // CORRECTION: $derived prend une expression, pas une fonction
  const kingInCheckSquare = $derived.by(() => {
    const engine = chessStore.currentGame?.engine;
    if (!engine) return null;
    const st = engine.status ?? '';
    if (!st.includes('check') && st !== 'checkmate') return null;
    const side = engine.side_to_move;
    return findKingSquare(side === 'white' ? 'w' : 'b');
  });

  async function handleClick(row: number, col: number) {
    if (chessStore.isGameOver) return;
    await chessStore.selectSquare(row, col);
  }

  function cellClass(row: number, col: number): string {
    const alg     = toAlgebraic(row, col);
    const isLight = (row + col) % 2 === 0;
    let cls = `cell ${isLight ? 'cell-light' : 'cell-dark'}`;
    if (chessStore.selected?.algebraic === alg)        cls += ' cell-selected';
    if (chessStore.legalTargets.includes(alg)) {
      const hasPiece = (chessStore.board[row]?.[col] ?? '') !== '';
      cls += hasPiece ? ' cell-capture' : ' cell-target';
    }
    if (chessStore.lastMove?.from === alg || chessStore.lastMove?.to === alg) cls += ' cell-last';
    if (kingInCheckSquare === alg) cls += ' cell-check';
    return cls;
  }

  const PROMO_PIECES = ['q', 'r', 'b', 'n'] as const;
  const PROMO_LABELS: Record<string, string> = { q: '♛ Dame', r: '♜ Tour', b: '♝ Fou', n: '♞ Cavalier' };

  let showResign = $state(false);
  // CORRECTION: Variables manquantes ajoutées
  let showResult = $state(false);
  let lastMovedFrom = $state<string | null>(null);
  let lastMovedTo = $state<string | null>(null);
  let resultDismissed = $state(false);
  let pgnCopied = $state(false);

  async function copyPgn() {
    const ok = await chessStore.copyPgn();
    if (ok) {
      pgnCopied = true;
      setTimeout(() => pgnCopied = false, 2000);
    }
  }
  // Loading LOCAL à cette page — évite la race condition avec chessStore.loading
  // partagé par loadGameList() du lobby qui peut remettre loading=false avant
  // que loadGame() ait terminé → "Partie introuvable" affiché prématurément.
  let pageLoading = $state(true);

  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    pageLoading = true;

    // Safety timeout — force pageLoading=false after 5s even if loadGame hangs
    const safetyTimer = setTimeout(() => {
      console.warn('[Chess] loadGame timeout — forcing display');
      pageLoading = false;
    }, 5000);

    try {
      await chessStore.loadGame(gameId);
      console.log('[Chess] Game loaded:', chessStore.currentGame?.id, 'status:', chessStore.currentGame?.status);
    } catch (e) {
      console.error('[Chess] loadGame threw:', e);
      chessStore.error = 'Erreur inattendue lors du chargement';
    } finally {
      clearTimeout(safetyTimer);
      pageLoading = false;
    }
  });

  // Auto-show result modal when game finishes
  $effect(() => {
    if (chessStore.currentGame?.status === 'finished' && !showResult) {
      showResult = true;
      resultDismissed = false;
    }
  });
  onDestroy(() => chessStore.disconnectWebSocket());

  // Guard : ne pas dériver si currentGame est null
  // CORRECTION: Utiliser $derived correctement
  const mySlot = $derived.by(() => {
    const g = chessStore.currentGame;
    if (!g) return null;
    const uid = authStore.user?.id;
    if (g.player1_id === uid) return 1;
    if (g.player2_id === uid) return 2;
    return null;
  });

  const flipped  = $derived(chessStore.myColor === 'black');
  const rows     = $derived(flipped ? [0,1,2,3,4,5,6,7].reverse() : [0,1,2,3,4,5,6,7]);
  const cols     = $derived(flipped ? [0,1,2,3,4,5,6,7].reverse() : [0,1,2,3,4,5,6,7]);
  const recentHistory = $derived((chessStore.currentGame?.move_history ?? []).slice(-20));
</script>

<svelte:head>
  <title>{chessStore.currentGame ? '♟ Partie — Nook' : '♟ Chargement…'}</title>
</svelte:head>

<div class="chess-page">

  {#if pageLoading}
    <div class="loading-full">
      <div class="spinner-lg"></div>
      <p>Chargement de la partie…</p>
    </div>

  {:else if chessStore.error}
    <div class="error-state">
      <p>⚠️ {chessStore.error}</p>
      <a href="/chess" class="btn-back">← Retour au lobby</a>
    </div>

  {:else if !chessStore.currentGame}
    <div class="error-state">
      <p>Partie introuvable.</p>
      <a href="/chess" class="btn-back">← Retour au lobby</a>
    </div>

  {:else}
    {@const game   = chessStore.currentGame}
    {@const engine = game?.engine ?? null}
    {@const slot   = mySlot}

    <!-- ══ BARRE COMPACTE MOBILE (status + abandon) ══ -->
    <div class="mobile-bar">
      <!-- Statut -->
      <div class="mobile-status">
        {#if (game?.status ?? '') === 'waiting'}
          <span class="badge badge-wait">🟡 En attente</span>
        {:else if (game?.status ?? '') === 'playing'}
          {#if chessStore.aiThinking}
            <span class="badge badge-think">
              <span class="dots"><span></span><span></span><span></span></span> IA…
            </span>
          {:else if chessStore.isMyTurn}
            <span class="badge badge-go">✅ À vous !</span>
          {:else}
            <span class="badge badge-wait">⏳ Adversaire</span>
          {/if}
        {:else}
          <span class="badge badge-over">{statusLabel(game?.status ?? '')}</span>
        {/if}
      </div>

      <!-- Joueurs en ligne -->
      <div class="mobile-players">
        <span class="mp" class:mp-active={engine?.side_to_move === game?.player2_color && (game?.status ?? '') === 'playing'}>
          {game?.player2_id
            ? (slot === 2 ? 'Vous' : (game?.ai_difficulty ? `🤖 IA (${game?.ai_difficulty})` : (game?.player2_name ?? 'Adv.')))
            : (game?.ai_difficulty ? `🤖 IA (${game?.ai_difficulty})` : '…')}
          <span class="mp-dot" class:dot-w={game?.player2_color === 'white'} class:dot-b={game?.player2_color === 'black'}></span>
        </span>
        <span class="mp-sep">vs</span>
        <span class="mp" class:mp-active={engine?.side_to_move === game?.player1_color && (game?.status ?? '') === 'playing'}>
          {slot === 1 ? 'Vous' : (game?.player1_id ? (game?.player1_name ?? 'Adv.') : '…')}
          <span class="mp-dot" class:dot-w={game?.player1_color === 'white'} class:dot-b={game?.player1_color === 'black'}></span>
        </span>
      </div>

      <!-- Minuteur mobile -->
      {#if chessStore.timerLimit > 0 && (game?.status ?? '') === 'playing'}
        <div class="mobile-timer">
          <span class="mt-side" class:mt-active={engine?.side_to_move === 'white'}
                class:mt-low={chessStore.whiteTime <= 30 && chessStore.whiteTime > 0}>
            ♙ {formatTime(chessStore.whiteTime)}
          </span>
          <span class="mt-sep">|</span>
          <span class="mt-side" class:mt-active={engine?.side_to_move === 'black'}
                class:mt-low={chessStore.blackTime <= 30 && chessStore.blackTime > 0}>
            ♟ {formatTime(chessStore.blackTime)}
          </span>
        </div>
      {/if}

      <!-- Abandon compact -->
      {#if (game?.status ?? '') === 'playing' && slot !== null}
        {#if !showResign}
          <button class="btn-resign-sm" onclick={() => showResign = true} title="Abandonner">🏳</button>
        {:else}
          <button class="rbtn-yes-sm" onclick={() => { chessStore.resign(); showResign = false; }}>Oui</button>
          <button class="rbtn-no-sm"  onclick={() => showResign = false}>Non</button>
        {/if}
      {/if}
      {#if (game?.status ?? '') === 'finished'}
        <a href="/chess" class="btn-new-sm">Nouvelle</a>
      {/if}
    </div>

    <div class="game-layout">

      <!-- ══ SIDEBAR (desktop) ══ -->
      <aside class="sidebar">
        <a href="/chess" class="back-link">← Lobby</a>

        <div class="panel players-panel">
          <h3>Joueurs</h3>
          <div class="player-row"
               class:active-turn={engine?.side_to_move === game?.player2_color && (game?.status ?? '') === 'playing'}
               class:is-me={slot === 2}>
            <div class="color-dot" class:dot-white={game?.player2_color === 'white'} class:dot-black={game?.player2_color === 'black'}></div>
            <div class="player-info">
              <span class="player-name">
                {game?.player2_id
                  ? (slot === 2 ? 'Vous' : (game?.player2_name ?? game?.player2_id?.slice(0,10) ?? '?'))
                  : (game?.ai_difficulty ? `🤖 IA (${game?.ai_difficulty})` : 'En attente…')}
              </span>
              <span class="player-color">{game?.player2_color === 'white' ? '♙ Blancs' : '♟ Noirs'}</span>
            </div>
            {#if engine?.side_to_move === game?.player2_color && (game?.status ?? '') === 'playing'}
              <span class="turn-arrow">▶</span>
            {/if}
          </div>

          <div class="player-row"
               class:active-turn={engine?.side_to_move === game?.player1_color && (game?.status ?? '') === 'playing'}
               class:is-me={slot === 1}>
            <div class="color-dot" class:dot-white={game?.player1_color === 'white'} class:dot-black={game?.player1_color === 'black'}></div>
            <div class="player-info">
              <span class="player-name">
                {game?.player1_id ? (slot === 1 ? 'Vous' : (game?.player1_name ?? game?.player1_id?.slice(0,10) ?? '?')) : 'En attente…'}
              </span>
              <span class="player-color">{game?.player1_color === 'white' ? '♙ Blancs' : '♟ Noirs'}</span>
            </div>
            {#if engine?.side_to_move === game?.player1_color && (game?.status ?? '') === 'playing'}
              <span class="turn-arrow">▶</span>
            {/if}
          </div>
        </div>

        <div class="panel status-panel">
          {#if chessStore.timerLimit > 0 && (game?.status ?? '') === 'playing'}
            <div class="timer-panel">
              <div class="timer-row" class:timer-active={engine?.side_to_move === 'white'}
                   class:timer-low={chessStore.whiteTime <= 30 && chessStore.whiteTime > 0}>
                <span class="timer-label">♙ Blancs</span>
                <span class="timer-val">{formatTime(chessStore.whiteTime)}</span>
              </div>
              <div class="timer-row" class:timer-active={engine?.side_to_move === 'black'}
                   class:timer-low={chessStore.blackTime <= 30 && chessStore.blackTime > 0}>
                <span class="timer-label">♟ Noirs</span>
                <span class="timer-val">{formatTime(chessStore.blackTime)}</span>
              </div>
            </div>
          {/if}
          {#if (game?.status ?? '') === 'waiting'}
            <div class="banner waiting">🟡 En attente d'un adversaire</div>
          {:else if (game?.status ?? '') === 'playing'}
            {#if chessStore.aiThinking}
              <div class="banner thinking">
                <span class="dots"><span></span><span></span><span></span></span>
                IA réfléchit…
              </div>
            {:else if chessStore.isMyTurn}
              <div class="banner your-turn">✅ À vous de jouer !</div>
            {:else}
              <div class="banner waiting-turn">⏳ Tour des {engine?.side_to_move === 'white' ? 'Blancs' : 'Noirs'}</div>
            {/if}
          {:else}
            <div class="banner game-over">
              {statusLabel(game?.status ?? '')}
              {#if game?.winner_id}
                <small>{game?.winner_id === authStore.user?.id ? 'Vous avez gagné 🏆' : 'Vous avez perdu'}</small>
              {/if}
            </div>
            <a href="/chess" class="btn-new-game">Nouvelle partie</a>
          {/if}
        </div>

        {#if (game?.status ?? '') === 'playing' && slot !== null}
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

        {#if recentHistory.length > 0}
          <div class="panel history-panel">
            <h3>Historique</h3>
            <ol class="move-list">
              {#each recentHistory as move}
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
          <div class="coords-top">
            {#each cols as c}<span>{String.fromCharCode(97 + c)}</span>{/each}
          </div>
          <div class="board-and-ranks">
            <div class="coords-left">
              {#each rows as r}<span>{8 - r}</span>{/each}
            </div>
            <div class="chess-board">
              {#each rows as row}
                {#each cols as col}
                  {@const piece   = chessStore.board[row]?.[col] ?? ''}
                  {@const decoded = decodePiece(piece)}
                  <div
                    class={cellClass(row, col)}
                    role="button" tabindex="0"
                    onclick={() => handleClick(row, col)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleClick(row, col)}
                  >
                    {#if decoded}
                      <span
                        class="piece"
                        class:piece-mine={decoded.color === (chessStore.myColor === 'white' ? 'w' : 'b')}
                        class:piece-selected={chessStore.selected?.algebraic === toAlgebraic(row, col)}
                        style="color:{decoded.color === 'w' ? '#fff' : '#1a1a1a'};
                               text-shadow:{decoded.color === 'w'
                                 ? '0 1px 3px rgba(0,0,0,0.8),0 0 1px rgba(0,0,0,1)'
                                 : '0 1px 2px rgba(255,255,255,0.3)'};"
                      >{PIECE_UNICODE[piece] ?? '?'}</span>
                    {:else if chessStore.legalTargets.includes(toAlgebraic(row, col))}
                      <span class="target-dot"></span>
                    {/if}
                  </div>
                {/each}
              {/each}
            </div>
            <div class="coords-right">
              {#each rows as r}<span>{8 - r}</span>{/each}
            </div>
          </div>
          <div class="coords-bottom">
            {#each cols as c}<span>{String.fromCharCode(97 + c)}</span>{/each}
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
              <button class="promo-btn" onclick={() => chessStore.confirmPromotion(p)}>
                <span class="promo-piece"
                  style="color:{chessStore.myColor === 'white' ? '#fff' : '#1a1a1a'};
                         text-shadow:{chessStore.myColor === 'white'
                           ? '0 1px 3px rgba(0,0,0,0.8)' : '0 1px 2px rgba(255,255,255,0.3)'};">
                  {PROMO_LABELS[p].split(' ')[0]}
                </span>
                <span class="promo-name">{PROMO_LABELS[p].split(' ')[1]}</span>
              </button>
            {/each}
          </div>
          <button class="promo-cancel" onclick={() => chessStore.cancelPromotion()}>Annuler</button>
        </div>
      </div>
    {/if}

  <!-- ══ MODAL RÉSULTAT FIN DE PARTIE ══ -->
  <!-- CORRECTION: Vérifier showResult et status 'finished' ensemble -->
  {#if showResult && (chessStore.currentGame?.status ?? '') === 'finished'}
    {@const g = chessStore.currentGame}
    {@const isWinner = g?.winner_id === authStore.user?.id}
    {@const isDraw   = !g?.winner_id}
    <div class="modal-backdrop" role="dialog" aria-modal="true">
      <div class="modal-result">
        <div class="result-icon">
          {#if isDraw}🤝{:else if isWinner}🏆{:else}👑{/if}
        </div>
        <h2 class="result-title">
          {#if isDraw}Match nul !{:else if isWinner}Victoire !{:else}Défaite{/if}
        </h2>
        <p class="result-sub">
          {#if isDraw}Égalité parfaite
          {:else if isWinner}Bien joué !
          {:else if chessStore.isVsAI}L'IA ({g?.ai_difficulty}) a gagné
          {:else}Votre adversaire a gagné
          {/if}
        </p>
        <p class="result-moves">{g?.move_history?.length ?? 0} coups joués</p>
        <div class="result-actions">
          <a href="/chess" class="result-btn result-btn-primary">Nouvelle partie</a>
          <button class="result-btn result-btn-secondary"
            onclick={() => { showResult = false; resultDismissed = true; }}>
            Voir le plateau
          </button>
        </div>
      </div>
    </div>
  {/if}

  {/if}
</div>

<style>
  /* ── Page ── */
  .chess-page { min-height: 100vh; background: var(--color-bg, #f8fafc); }

  .loading-full, .error-state {
    min-height: 70vh; display: flex; flex-direction: column;
    align-items: center; justify-content: center; gap: 1rem; color: #64748b;
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

  /* ── Barre mobile ── */
  .mobile-bar {
    display: none; /* masquée sur desktop */
  }

  /* ── Layout desktop ── */
  .game-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 1.5rem;
    padding: 1.25rem;
    align-items: start;
    min-height: calc(100vh - 60px);
  }

  /* ── Sidebar ── */
  .sidebar { display: flex; flex-direction: column; gap: .75rem; position: sticky; top: 1rem; }
  .back-link { font-size: .83rem; color: #64748b; text-decoration: none; }
  .back-link:hover { color: #2d5a27; }

  .panel {
    background: #fff; border: 1px solid #e2e8f0;
    border-radius: .875rem; padding: .9rem;
  }
  .panel h3 {
    margin: 0 0 .65rem; font-size: .8rem; font-weight: 700;
    text-transform: uppercase; letter-spacing: .05em; color: #64748b;
  }

  .player-row {
    display: flex; align-items: center; gap: .45rem;
    padding: .35rem .45rem; border-radius: .4rem;
    border: 2px solid transparent; margin-bottom: .25rem; transition: all .15s;
  }
  .player-row.active-turn { background: #f0fdf4; border-color: #86efac; }
  .player-row.is-me { font-weight: 700; }
  .color-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; border: 1.5px solid #94a3b8; }
  .dot-white { background: #f8fafc; border-color: #94a3b8; }
  .dot-black { background: #1e293b; border-color: #64748b; }
  .player-info { flex: 1; min-width: 0; }
  .player-name  { display: block; font-size: .82rem; font-weight: 600; color: #1e293b; }
  .player-color { font-size: .7rem; color: #94a3b8; }
  .turn-arrow   { color: #22c55e; font-size: .75rem; }

  .banner {
    padding: .6rem .75rem; border-radius: .5rem;
    font-size: .82rem; font-weight: 600; text-align: center; line-height: 1.4;
  }
  .banner.waiting      { background: #fefce8; color: #854d0e; border: 1px solid #fde68a; }
  .banner.your-turn    { background: #f0fdf4; color: #166534; border: 1px solid #86efac; }
  .banner.waiting-turn { background: #f8fafc; color: #475569; border: 1px solid #e2e8f0; }
  .banner.game-over    { background: #fdf4ff; color: #7e22ce; border: 1px solid #e9d5ff; }
  .banner.game-over small { display: block; font-size: .72rem; opacity: .75; margin-top: .2rem; }
  .banner.thinking {
    background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe;
    display: flex; align-items: center; justify-content: center; gap: .5rem;
  }

  .dots { display: flex; gap: 3px; }
  .dots span { width: 5px; height: 5px; border-radius: 50%; background: currentColor; animation: bounce 1.2s infinite; }
  .dots span:nth-child(2) { animation-delay: .2s; }
  .dots span:nth-child(3) { animation-delay: .4s; }
  @keyframes bounce { 0%,80%,100% { transform: scale(0); } 40% { transform: scale(1); } }

  .btn-new-game {
    display: block; margin-top: .5rem; padding: .5rem; text-align: center;
    background: #2d5a27; color: #fff; border-radius: .45rem;
    text-decoration: none; font-size: .82rem; font-weight: 600;
  }
  .btn-new-game:hover { background: #3d7a37; }

  .history-panel { max-height: 180px; overflow-y: auto; }
  .move-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: .2rem; }
  .move-item { display: flex; justify-content: space-between; align-items: center; padding: .2rem .35rem; border-radius: .3rem; font-size: .78rem; }
  .move-item.white-move { background: #f8fafc; }
  .move-san { font-weight: 700; font-family: monospace; color: #1e293b; }
  .move-by  { color: #94a3b8; font-size: .7rem; }

  .pgn-section { margin-top: .5rem; padding-top: .5rem; border-top: 1px solid var(--border, #e2e8f0); }
  .pgn-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: .3rem; }
  .pgn-label { font-size: .7rem; font-weight: 600; color: var(--text-secondary, #64748b); text-transform: uppercase; }
  .pgn-copy-btn { font-size: .7rem; padding: .2rem .5rem; border: 1px solid var(--border, #e2e8f0); border-radius: 4px; background: var(--bg-secondary, #f1f5f9); cursor: pointer; }
  .pgn-copy-btn:hover { background: var(--accent, #4ade80); color: #fff; border-color: var(--accent, #4ade80); }
  .pgn-code { display: block; font-size: .75rem; padding: .4rem; background: var(--bg-secondary, #f1f5f9); border-radius: 4px; word-break: break-all; line-height: 1.5; }

  .error-panel { background: #fef2f2; border-color: #fecaca; color: #dc2626; font-size: .82rem; }

  .resign-panel { padding: .65rem !important; }
  .btn-resign { width: 100%; padding: .5rem; background: #fef2f2; border: 1px solid #fecaca; color: #dc2626; border-radius: .45rem; font-size: .82rem; cursor: pointer; }
  .btn-resign:hover { background: #fee2e2; }
  .resign-confirm-text { margin: 0 0 .4rem; font-size: .82rem; color: #dc2626; font-weight: 600; }
  .resign-btns { display: flex; gap: .35rem; }
  .rbtn-yes { flex: 1; padding: .35rem; background: #dc2626; color: #fff; border: none; border-radius: .35rem; font-size: .78rem; cursor: pointer; }
  .rbtn-no  { flex: 1; padding: .35rem; background: #f1f5f9; color: #475569; border: none; border-radius: .35rem; font-size: .78rem; cursor: pointer; }

  /* ── Plateau ── */
  .board-wrap { display: flex; justify-content: center; align-items: center; flex: 1; padding: 1rem 0; }
  .board-container { display: flex; flex-direction: column; align-items: center; gap: 2px; }
  .board-and-ranks { display: flex; align-items: stretch; gap: 2px; }

  .coords-top, .coords-bottom {
    display: grid; grid-template-columns: repeat(8, 1fr);
    width: min(85vw, 720px); padding: 0 2px;
  }
  .coords-top span, .coords-bottom span { text-align: center; font-size: .6rem; font-weight: 700; color: #94a3b8; line-height: 1.6; }
  .coords-left, .coords-right { display: flex; flex-direction: column; }
  .coords-left span, .coords-right span { flex: 1; display: flex; align-items: center; justify-content: center; font-size: .6rem; font-weight: 700; color: #94a3b8; width: 16px; }

  .chess-board {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    grid-template-rows: repeat(8, 1fr);
    width: min(85vw, 720px);
    aspect-ratio: 1;
    border: 2.5px solid #374151;
    border-radius: 3px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0,0,0,.2);
  }

  .cell { position: relative; display: flex; align-items: center; justify-content: center; cursor: pointer; transition: filter .1s; user-select: none; }
  .cell:focus-visible { outline: 3px solid #f59e0b; outline-offset: -3px; z-index: 2; }
  .cell-light    { background: #f0d9b5; }
  .cell-dark     { background: #b58863; }
  .cell-selected { outline: 3px solid #f59e0b; outline-offset: -3px; z-index: 1; }
  .cell-target   { background: rgba(99,200,90,0.45) !important; }
  .cell-capture  { background: rgba(220,60,60,0.5) !important; }
  .cell-last     { background: rgba(255,196,0,0.35) !important; }
  .cell:not(.cell-selected):hover { filter: brightness(1.08); }

  .piece { font-size: clamp(1.8rem, 6.5vw, 3.8rem); line-height: 1; z-index: 1; transition: transform .1s; cursor: pointer; }
  .piece.piece-mine:hover { transform: scale(1.12); }
  .piece.piece-selected   { transform: scale(1.25); filter: drop-shadow(0 0 6px rgba(245,158,11,.9)); }

  .target-dot { width: 28%; height: 28%; border-radius: 50%; background: rgba(0,0,0,.22); pointer-events: none; }

  /* ── Modal promotion ── */
  .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,.6); display: flex; align-items: center; justify-content: center; z-index: 999; }
  .modal-promo { background: #fff; border-radius: 1rem; padding: 1.75rem; text-align: center; box-shadow: 0 16px 48px rgba(0,0,0,.3); min-width: 280px; }
  .modal-promo h3 { margin: 0 0 1.1rem; font-size: 1.1rem; color: #1e293b; }
  .promo-grid { display: grid; grid-template-columns: 1fr 1fr; gap: .6rem; margin-bottom: 1rem; }
  .promo-btn { padding: .8rem .5rem; border: 2px solid #e2e8f0; border-radius: .7rem; background: #fff; cursor: pointer; display: flex; flex-direction: column; align-items: center; gap: .2rem; transition: all .15s; }
  .promo-btn:hover { border-color: #2d5a27; background: #f0fdf4; }
  .promo-piece { font-size: 2rem; line-height: 1; }
  .promo-name  { font-size: .8rem; font-weight: 600; color: #374151; }
  .promo-cancel { padding: .4rem 1.2rem; background: #f1f5f9; color: #475569; border: none; border-radius: .45rem; cursor: pointer; font-size: .85rem; }

  /* ── Mise en échec ── */
  .cell-check {
    background: rgba(220, 38, 38, 0.65) !important;
    animation: pulse-check 0.8s ease-in-out infinite alternate;
    z-index: 1;
  }
  @keyframes pulse-check {
    from { background: rgba(220, 38, 38, 0.55); box-shadow: inset 0 0 0 3px rgba(220,38,38,0.8); }
    to   { background: rgba(220, 38, 38, 0.80); box-shadow: inset 0 0 0 3px rgba(220,38,38,1); }
  }

  /* ── Minuteur sidebar ── */
  .timer-panel { margin-bottom: .5rem; }
  .timer-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: .35rem .5rem; border-radius: .4rem; margin-bottom: .2rem;
    border: 1.5px solid transparent; transition: all .2s;
  }
  .timer-row.timer-active { background: #f0fdf4; border-color: #86efac; }
  .timer-row.timer-low    { background: #fef2f2; border-color: #fecaca; animation: blink-low 1s infinite; }
  .timer-label { font-size: .78rem; font-weight: 600; color: #475569; }
  .timer-val   { font-family: monospace; font-size: 1rem; font-weight: 700; color: #1e293b; }
  .timer-row.timer-active .timer-val { color: #166534; }
  .timer-row.timer-low    .timer-val { color: #dc2626; }
  @keyframes blink-low { 0%,100% { opacity: 1; } 50% { opacity: .6; } }

  /* ── Minuteur mobile ── */
  .mobile-timer {
    display: flex; align-items: center; gap: .3rem;
    font-size: .72rem; font-family: monospace; font-weight: 700;
    background: #f8fafc; border: 1px solid #e2e8f0;
    padding: .2rem .45rem; border-radius: .4rem; flex-shrink: 0;
  }
  .mt-sep  { color: #94a3b8; }
  .mt-side { color: #475569; transition: color .2s; }
  .mt-side.mt-active { color: #166534; }
  .mt-side.mt-low    { color: #dc2626; animation: blink-low 1s infinite; }

  /* ══════════════════════════════════════════
     RESPONSIVE TABLET
  ══════════════════════════════════════════ */
  @media (min-width: 721px) and (max-width: 1024px) {
    .game-layout {
      grid-template-columns: 180px 1fr;
      gap: 1rem;
      padding: 1rem;
    }
    .chess-board {
      width: min(80vw, 600px);
    }
    .coords-top, .coords-bottom {
      width: min(80vw, 600px);
    }
    .piece { font-size: clamp(1.6rem, 5.5vw, 3rem); }
  }
  /* ══════════════════════════════════════════
     RESPONSIVE MOBILE
  ══════════════════════════════════════════ */
  @media (max-width: 720px) {

    /* Barre compacte visible */
    .mobile-bar {
      display: flex;
      align-items: center;
      gap: .5rem;
      padding: .5rem .75rem;
      background: #fff;
      border-bottom: 1px solid #e2e8f0;
      flex-wrap: wrap;
      position: sticky;
      top: 0;
      z-index: 10;
    }

    /* Sidebar complètement masquée sur mobile */
    .sidebar { display: none; }

    /* Layout = colonne unique, padding minimal */
    .game-layout {
      grid-template-columns: 1fr;
      padding: .5rem .35rem;
      gap: .5rem;
    }

    /* Board occupe toute la largeur disponible */
    .chess-board {
      width: calc(100vw - 1.5rem);
      max-width: 600px;
    }
    .coords-top, .coords-bottom {
      width: calc(100vw - 1.5rem);
      max-width: 600px;
    }

    /* Badges statut */
    .badge {
      display: inline-flex; align-items: center; gap: .3rem;
      padding: .25rem .6rem; border-radius: 999px;
      font-size: .75rem; font-weight: 700; white-space: nowrap;
    }
    .badge-go   { background: #f0fdf4; color: #166534; border: 1px solid #86efac; }
    .badge-wait { background: #fefce8; color: #854d0e; border: 1px solid #fde68a; }
    .badge-think{ background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; }
    .badge-over { background: #fdf4ff; color: #7e22ce; border: 1px solid #e9d5ff; }

    /* Joueurs inline */
    .mobile-players {
      display: flex; align-items: center; gap: .35rem;
      font-size: .75rem; color: #475569; flex: 1; min-width: 0;
    }
    .mp { display: flex; align-items: center; gap: .25rem; font-weight: 600; white-space: nowrap; }
    .mp-active { color: #166534; }
    .mp-sep  { color: #94a3b8; font-size: .7rem; }
    .mp-dot  { width: 8px; height: 8px; border-radius: 50%; border: 1.5px solid #94a3b8; display: inline-block; }
    .dot-w   { background: #f8fafc; border-color: #94a3b8; }
    .dot-b   { background: #1e293b; border-color: #64748b; }

    /* Boutons abandon compacts */
    .btn-resign-sm {
      padding: .3rem .6rem; background: #fef2f2; border: 1px solid #fecaca;
      color: #dc2626; border-radius: .4rem; font-size: .8rem; cursor: pointer; flex-shrink: 0;
    }
    .rbtn-yes-sm { padding: .3rem .6rem; background: #dc2626; color: #fff; border: none; border-radius: .35rem; font-size: .78rem; cursor: pointer; }
    .rbtn-no-sm  { padding: .3rem .6rem; background: #f1f5f9; color: #475569; border: none; border-radius: .35rem; font-size: .78rem; cursor: pointer; }
    .btn-new-sm  {
      padding: .3rem .7rem; background: #2d5a27; color: #fff;
      border-radius: .4rem; text-decoration: none; font-size: .78rem; font-weight: 600; flex-shrink: 0;
    }
  }
  /* ── Modal résultat fin de partie ── */
  .modal-result {
    background: #fff; border-radius: 1.25rem; padding: 2rem 1.75rem;
    text-align: center; box-shadow: 0 20px 60px rgba(0,0,0,.35);
    min-width: 280px; max-width: 340px;
    animation: popIn .35s cubic-bezier(.34,1.56,.64,1);
  }
  @keyframes popIn {
    from { transform: scale(.7); opacity: 0; }
    to   { transform: scale(1);  opacity: 1; }
  }
  .result-icon  { font-size: 3.5rem; line-height: 1; margin-bottom: .5rem; }
  .result-title { font-size: 1.6rem; font-weight: 800; margin: 0 0 .4rem; color: #1e293b; }
  .result-sub   { font-size: .95rem; color: #64748b; margin: 0 0 .35rem; }
  .result-moves { font-size: .82rem; color: #94a3b8; margin: 0 0 1.25rem; }
  .result-actions { display: flex; gap: .75rem; justify-content: center; flex-wrap: wrap; }
  .result-btn {
    padding: .65rem 1.25rem; border-radius: .6rem; font-weight: 700;
    font-size: .9rem; cursor: pointer; text-decoration: none; border: none; transition: all .15s;
  }
  .result-btn-primary  { background: #2d5a27; color: #fff; }
  .result-btn-primary:hover  { background: #3d7a37; }
  .result-btn-secondary { background: #f1f5f9; color: #475569; }
  .result-btn-secondary:hover { background: #e2e8f0; }


  /* Piece animation on move */
  .piece {
    transition: transform 0.15s ease-out;
  }
  .piece.moving {
    animation: piece-move 0.2s ease-out;
  }
  @keyframes piece-move {
    0% { transform: scale(1.15); }
    100% { transform: scale(1); }
  }
  
  /* Better legal move indicators */
  .cell-target {
    position: relative;
  }
  .cell-target::after {
    content: '';
    position: absolute;
    width: 30%;
    height: 30%;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.15);
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }
  .cell-capture {
    background: rgba(220, 50, 50, 0.35) !important;
    box-shadow: inset 0 0 0 3px rgba(220, 50, 50, 0.5);
  }
  
  /* Selected piece highlight */
  .cell.selected {
    background: rgba(255, 215, 0, 0.5) !important;
  }
  
  /* Last move highlight */
  .cell.last-move {
    background: rgba(255, 255, 100, 0.35) !important;
  }

  /* Enhanced piece animations */
  .piece {
    transition: transform 0.15s ease-out, opacity 0.15s ease;
  }
  .piece.moving {
    animation: piece-land 0.25s ease-out;
  }
  .piece.captured {
    animation: piece-capture 0.3s ease-out forwards;
  }
  @keyframes piece-land {
    0% { transform: scale(1.2); filter: brightness(1.3); }
    100% { transform: scale(1); filter: brightness(1); }
  }
  @keyframes piece-capture {
    0% { transform: scale(1); opacity: 1; }
    100% { transform: scale(0.5) rotate(180deg); opacity: 0; }
  }
  
  /* Legal move indicators with ripple effect */
  .cell-target {
    position: relative;
  }
  .cell-target::after {
    content: '';
    position: absolute;
    width: 30%;
    height: 30%;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.2);
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    animation: target-ripple 1s ease-in-out infinite;
  }
  @keyframes target-ripple {
    0%, 100% { transform: translate(-50%, -50%) scale(1); opacity: 0.3; }
    50% { transform: translate(-50%, -50%) scale(1.2); opacity: 0.5; }
  }
  
  /* Capture indicator with danger highlight */
  .cell-capture {
    position: relative;
  }
  .cell-capture::after {
    content: '';
    position: absolute;
    inset: 4px;
    border: 3px solid rgba(220, 50, 50, 0.6);
    border-radius: 50%;
    animation: capture-pulse 0.8s ease-in-out infinite;
  }
  @keyframes capture-pulse {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
  }
  
  /* Last move highlight with glow */
  .cell.last-move {
    box-shadow: inset 0 0 0 2px rgba(255, 215, 0, 0.5);
  }
  
  /* Selected piece glow */
  .cell.selected {
    box-shadow: inset 0 0 0 3px rgba(74, 222, 128, 0.8);
  }
  
  /* Move history item animation */
  .move-item {
    transition: all 0.2s ease;
    animation: move-fade-in 0.3s ease;
  }
  .move-item:hover {
    background: var(--bg-tertiary, #f1f5f9);
  }
  @keyframes move-fade-in {
    from { opacity: 0; transform: translateY(-5px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
