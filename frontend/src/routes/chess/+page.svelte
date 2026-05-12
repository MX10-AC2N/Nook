<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    chessStore,
    statusLabel,
    DIFFICULTY_LABELS,
    type Difficulty,
  } from '$lib/chessStore.svelte.ts';

  // ── État local formulaire ──────────────────────────────────────
  let showCreate  = $state(false);
  let creating    = $state(false);
  let opponent    = $state<'human' | Difficulty>('human');
  let myColor     = $state<'white' | 'black'>('white');
  let timerChoice = $state(0); // 0=illimité, 300=5min, 600=10min, 900=15min, 1800=30min

  onMount(async () => {
    // Attendre que authStore ait fini son init — éviter la race condition
    // qui redirigeait vers /login avant que le layout ne finisse authStore.init()
    while (authStore.loading) {
      await new Promise(r => setTimeout(r, 50));
    }
    if (!authStore.isAuthenticated) return; // Le layout gérera le redirect
    await chessStore.loadGameList();
  });

  // ── Handlers ──────────────────────────────────────────────────
  async function handleCreate() {
    creating = true;
    const gameId = await chessStore.createGame({
      opponent,
      color: myColor,
      time_limit_secs: timerChoice,
    });
    creating = false;
    if (gameId) {
      // Le timer est initialisé via loadGame → time_limit_secs du serveur
      // initTimer ici uniquement pour le créateur (avant le premier loadGame)
      if (timerChoice > 0) chessStore.initTimer(timerChoice);
      goto(`/chess/${gameId}`);
    }
  }

  async function handleJoin(id: string) {
    const ok = await chessStore.joinGame(id);
    if (ok) goto(`/chess/${id}`);
  }

  function timeAgo(ts: number): string {
    const diff = Math.floor(Date.now() / 1000 - ts);
    if (diff < 60)   return "à l'instant";
    if (diff < 3600) return `il y a ${Math.floor(diff / 60)} min`;
    return `il y a ${Math.floor(diff / 3600)} h`;
  }

  const DIFFICULTIES: Difficulty[] = ['easy', 'medium', 'hard', 'expert', 'godlike'];
</script>

<svelte:head><title>♟ Échecs — Nook</title></svelte:head>

<div class="lobby">

  <!-- ── En-tête ── -->
  <header class="lobby-header">
    <div class="title-row">
      <span class="crown"><Icon name="home" size="36" /></span>
      <h1>Échecs</h1>
    </div>
    <p class="subtitle">Échecs FIDE standard — 2 joueurs humains ou contre l'IA</p>
  </header>

  <!-- ── Bouton / formulaire création ── -->
  <div class="create-area">
    {#if !showCreate}
      <button class="btn-new" onclick={() => showCreate = true}>
        + Nouvelle partie
      </button>
    {:else}
      <div class="create-card">
        <h2>Créer une partie</h2>

        <!-- Adversaire -->
        <fieldset class="field-group">
          <legend>Adversaire</legend>
          <div class="radio-row">
            <label class="radio-opt" class:active={opponent === 'human'}>
              <input type="radio" bind:group={opponent} value="human" />
              <span class="opt-icon">👤</span>
              <span>Humain</span>
            </label>
            {#each DIFFICULTIES as d}
              <label class="radio-opt" class:active={opponent === d}>
                <input type="radio" bind:group={opponent} value={d} />
                <span class="opt-label">{DIFFICULTY_LABELS[d]}</span>
              </label>
            {/each}
          </div>
        </fieldset>

        <!-- Couleur -->
        <fieldset class="field-group">
          <legend>Votre couleur</legend>
          <div class="color-row">
            <label class="color-opt" class:active={myColor === 'white'}>
              <input type="radio" bind:group={myColor} value="white" />
              <span class="piece-preview white">♙</span>
              <span>Blancs</span>
              <small>Vous commencez</small>
            </label>
            <label class="color-opt" class:active={myColor === 'black'}>
              <input type="radio" bind:group={myColor} value="black" />
              <span class="piece-preview black">♟</span>
              <span>Noirs</span>
              {#if opponent === 'human'}
                <small>Vous attendez</small>
              {:else}
                <small>L'IA commence</small>
              {/if}
            </label>
          </div>
        </fieldset>

        <!-- Durée de la partie -->
        <fieldset class="fieldset">
          <legend>⏱ Durée par joueur</legend>
          <div class="radio-row">
            {#each [
              {val:0,     label:'∞ Illimitée'},
              {val:300,   label:'5 min'},
              {val:600,   label:'10 min'},
              {val:900,   label:'15 min'},
              {val:1800,  label:'30 min'},
            ] as t}
              <label class="radio-opt" class:active={timerChoice === t.val}>
                <input type="radio" bind:group={timerChoice} value={t.val} />
                {t.label}
              </label>
            {/each}
          </div>
        </fieldset>

        <!-- Actions -->
        <div class="form-actions">
          <button class="btn-confirm" onclick={handleCreate} disabled={creating}>
            {#if creating}
              <span class="spinner"></span> Création…
            {:else}
              Créer la partie
            {/if}
          </button>
          <button class="btn-cancel" onclick={() => showCreate = false}>Annuler</button>
        </div>

        {#if chessStore.error}
          <p class="form-error">⚠️ {chessStore.error}</p>
        {/if}
      </div>
    {/if}
  </div>

  <!-- ── Liste des parties ── -->
  <section class="games-section">
    <div class="section-top">
      <h2>Parties disponibles</h2>
      <button class="btn-refresh" onclick={() => chessStore.loadGameList()}
              disabled={chessStore.loading}>
        {chessStore.loading ? '⏳' : '↻ Actualiser'}
      </button>
    </div>

    {#if chessStore.loading && chessStore.gameList.length === 0}
      <div class="loading-state">Chargement…</div>

    {:else if chessStore.gameList.length === 0}
      <div class="empty-state">
        <span class="empty-icon">♟</span>
        <p>Aucune partie ouverte — crée-en une !</p>
      </div>

    {:else}
      <div class="games-grid">
        {#each chessStore.gameList as game (game.id)}
          <div class="game-card">
            <div class="card-top">
              <span class="status-badge">{statusLabel(game.status)}</span>
              <span class="time-ago">{timeAgo(game.updated_at)}</span>
            </div>
            <div class="card-body">
              <p class="creator">
                Créé par <strong>{game.creator_name ?? '—'}</strong>
              </p>
              <div class="badges">
                <span class="badge color-{game.creator_color}">
                  {game.creator_color === 'white' ? '♙ Blancs' : '♟ Noirs'}
                </span>
              </div>
            </div>
            <div class="card-actions">
              {#if game.status === 'waiting'}
                <button class="btn-join" onclick={() => handleJoin(game.id)}>
                  Rejoindre
                </button>
              {:else}
                <button class="btn-watch" onclick={() => goto(`/chess/${game.id}`)}>
                  Observer
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>

</div>

<style>
  .lobby {
    max-width: 860px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  /* ── Header ── */
  .lobby-header { text-align: center; margin-bottom: 2rem; }
  .title-row {
    display: flex; align-items: center; justify-content: center; gap: .75rem;
    margin-bottom: .5rem;
  }
  .crown { font-size: 2.2rem; }
  h1 { font-size: 2.1rem; font-weight: 800; color: #1e293b; margin: 0; }
  .subtitle { color: #64748b; margin: 0; }

  /* ── Zone création ── */
  .create-area { display: flex; justify-content: center; margin-bottom: 2.5rem; }
  .btn-new {
    padding: .9rem 2.5rem; background: #2d5a27; color: #fff;
    border: none; border-radius: .75rem;
    font-size: 1.05rem; font-weight: 700; cursor: pointer;
    transition: background .2s, transform .15s;
  }
  .btn-new:hover { background: #3d7a37; transform: translateY(-1px); }

  .create-card {
    background: #fff; border: 1px solid #e2e8f0; border-radius: 1rem;
    padding: 1.75rem; width: 100%; max-width: 520px;
    box-shadow: 0 4px 20px rgba(0,0,0,.07);
  }
  .create-card h2 { margin: 0 0 1.25rem; font-size: 1.2rem; color: #1e293b; }

  /* ── Fieldsets ── */
  .field-group {
    border: 1px solid #e2e8f0; border-radius: .6rem;
    padding: .75rem 1rem 1rem; margin-bottom: 1rem;
  }
  .field-group legend {
    font-size: .78rem; font-weight: 700; text-transform: uppercase;
    letter-spacing: .05em; color: #64748b; padding: 0 .35rem;
  }

  /* Adversaire — défilement horizontal */
  .radio-row {
    display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .4rem;
  }
  .radio-opt {
    display: flex; align-items: center; gap: .35rem;
    padding: .4rem .7rem; border: 2px solid #e2e8f0; border-radius: .5rem;
    cursor: pointer; font-size: .85rem; font-weight: 500;
    transition: all .15s; white-space: nowrap;
  }
  .radio-opt input[type="radio"] { display: none; }
  .radio-opt.active { border-color: #2d5a27; background: #f0fdf4; color: #166534; }
  .radio-opt:hover:not(.active) { border-color: #94a3b8; }
  .opt-icon { font-size: 1rem; }
  .opt-label { font-size: .82rem; }

  /* Couleur */
  .color-row { display: flex; gap: .75rem; margin-top: .5rem; }
  .color-opt {
    flex: 1; display: flex; flex-direction: column; align-items: center; gap: .2rem;
    padding: .8rem .5rem; border: 2px solid #e2e8f0; border-radius: .75rem;
    cursor: pointer; font-size: .9rem; font-weight: 600;
    transition: all .15s; text-align: center;
  }
  .color-opt input[type="radio"] { display: none; }
  .color-opt.active { border-color: #2d5a27; background: #f0fdf4; }
  .color-opt:hover:not(.active) { border-color: #94a3b8; }
  .piece-preview { font-size: 2rem; line-height: 1; }
  .piece-preview.white { color: #f0f0f0; text-shadow: 0 0 0 1.5px #94a3b8; }
  .piece-preview.black { color: #1e293b; }
  .color-opt small { font-size: .72rem; color: #64748b; font-weight: 400; }

  /* Actions */
  .form-actions { display: flex; gap: .65rem; margin-top: 1.25rem; }
  .btn-confirm {
    flex: 1; padding: .85rem; background: #2d5a27; color: #fff;
    border: none; border-radius: .6rem;
    font-size: 1rem; font-weight: 700; cursor: pointer;
    display: flex; align-items: center; justify-content: center; gap: .5rem;
    transition: background .2s;
  }
  .btn-confirm:hover:not(:disabled) { background: #3d7a37; }
  .btn-confirm:disabled { opacity: .6; cursor: not-allowed; }
  .btn-cancel {
    padding: .85rem 1.2rem; background: #f1f5f9; color: #475569;
    border: none; border-radius: .6rem; font-size: 1rem; cursor: pointer;
    transition: background .15s;
  }
  .btn-cancel:hover { background: #e2e8f0; }
  .form-error { color: #dc2626; font-size: .85rem; margin: .5rem 0 0; }

  .spinner {
    width: 15px; height: 15px;
    border: 2px solid rgba(255,255,255,.3); border-top-color: #fff;
    border-radius: 50%; animation: spin .7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Liste parties ── */
  .games-section { }
  .section-top {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 1rem;
  }
  .section-top h2 { margin: 0; font-size: 1.2rem; color: #1e293b; }
  .btn-refresh {
    padding: .35rem .85rem; background: #f1f5f9; border: 1px solid #e2e8f0;
    border-radius: .45rem; font-size: .83rem; cursor: pointer; color: #475569;
    transition: background .15s;
  }
  .btn-refresh:hover:not(:disabled) { background: #e2e8f0; }

  .loading-state, .empty-state {
    text-align: center; padding: 3rem; color: #94a3b8;
    display: flex; flex-direction: column; align-items: center; gap: .5rem;
  }
  .empty-icon { font-size: 2.2rem; opacity: .3; }

  .games-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }
  .game-card {
    background: #fff; border: 1px solid #e2e8f0; border-radius: .875rem;
    padding: 1.1rem; display: flex; flex-direction: column; gap: .75rem;
    transition: box-shadow .15s, transform .15s;
  }
  .game-card:hover { box-shadow: 0 4px 16px rgba(0,0,0,.08); transform: translateY(-1px); }
  .card-top { display: flex; justify-content: space-between; align-items: center; }
  .status-badge { font-size: .8rem; font-weight: 600; }
  .time-ago { font-size: .75rem; color: #94a3b8; }
  .card-body { }
  .creator { margin: 0 0 .4rem; font-size: .88rem; color: #475569; }
  .creator strong { color: #1e293b; }
  .badges { display: flex; gap: .4rem; }
  .badge {
    display: inline-flex; align-items: center; gap: .2rem;
    padding: .15rem .55rem; border-radius: 999px;
    font-size: .75rem; font-weight: 600;
  }
  .badge.color-white { background: #f8fafc; color: #374151; border: 1px solid #e2e8f0; }
  .badge.color-black { background: #1e293b; color: #f8fafc; }
  .card-actions { }
  .btn-join, .btn-watch {
    width: 100%; padding: .55rem; border: none; border-radius: .5rem;
    font-size: .88rem; font-weight: 700; cursor: pointer; transition: background .15s;
  }
  .btn-join  { background: #2d5a27; color: #fff; }
  .btn-join:hover { background: #3d7a37; }
  .btn-watch { background: #f1f5f9; color: #475569; }
  .btn-watch:hover { background: #e2e8f0; }

  @media (max-width: 560px) {
    .games-grid { grid-template-columns: 1fr; }
    h1 { font-size: 1.7rem; }
    .radio-row { gap: .3rem; }
  }
  .radio-opt {
    display: flex; align-items: center; gap: .35rem;
    padding: .35rem .65rem; border: 1.5px solid var(--border, #e2e8f0);
    border-radius: .45rem; cursor: pointer; font-size: .82rem;
    font-weight: 600; color: var(--text-secondary, #64748b);
    transition: all .15s; user-select: none;
  }
  .radio-opt.active {
    background: #f0fdf4; border-color: #86efac; color: #166534;
  }
  .radio-opt input[type=radio] { display: none; }

</style>
