<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import { chessStore, PLAYER_LABELS, type GameListItem } from '$lib/chessStore.svelte.ts';

  let creating    = $state(false);
  let playerCount = $state(2);
  let gameName    = $state('');
  let showCreate  = $state(false);

  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    await chessStore.loadGameList();
  });

  async function handleCreate() {
    creating = true;
    const gameId = await chessStore.createGame(playerCount, gameName || undefined);
    creating = false;
    if (gameId) goto(`/chess/${gameId}`);
  }

  async function handleJoin(game: GameListItem) {
    const ok = await chessStore.joinGame(game.id);
    if (ok) goto(`/chess/${game.id}`);
  }

  function handleSpectate(game: GameListItem) {
    goto(`/chess/${game.id}`);
  }

  function statusLabel(status: string): string {
    return status === 'waiting' ? '🟡 En attente' : '🟢 En cours';
  }

  function timeAgo(ts: number): string {
    const diff = Math.floor(Date.now() / 1000 - ts);
    if (diff < 60) return "à l'instant";
    if (diff < 3600) return `il y a ${Math.floor(diff / 60)} min`;
    return `il y a ${Math.floor(diff / 3600)} h`;
  }
</script>

<svelte:head>
  <title>♟ Échecs — Nook</title>
</svelte:head>

<div class="chess-lobby">
  <header class="lobby-header">
    <div class="title-group">
      <span class="icon">♟</span>
      <h1>Échecs</h1>
    </div>
    <p class="subtitle">Jouez en 2, 3 ou 4 joueurs sur un plateau adapté</p>
  </header>

  <div class="create-section">
    {#if !showCreate}
      <button class="btn-create" onclick={() => showCreate = true}>
        + Nouvelle partie
      </button>
    {:else}
      <div class="create-form">
        <h2>Créer une partie</h2>

        <div class="field">
          <label>Nombre de joueurs</label>
          <div class="player-count-selector">
            {#each [2, 3, 4] as n}
              <button
                class="count-btn"
                class:active={playerCount === n}
                onclick={() => playerCount = n}
              >
                {n} joueurs
              </button>
            {/each}
          </div>
        </div>

        <div class="field">
          <label for="game-name">Nom de la partie (optionnel)</label>
          <input
            id="game-name"
            type="text"
            bind:value={gameName}
            placeholder="ex: Partie famille dimanche"
            maxlength="40"
          />
        </div>

        <div class="board-info">
          {#if playerCount === 2}
            <span class="info-badge">🏁 Plateau 8×8 classique</span>
          {:else if playerCount === 3}
            <span class="info-badge">🔶 Plateau 14×14 — 3 coins actifs</span>
          {:else}
            <span class="info-badge">🔷 Plateau 14×14 — 4 joueurs</span>
          {/if}
        </div>

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
      </div>
    {/if}
  </div>

  {#if chessStore.error}
    <div class="alert-error" role="alert">⚠️ {chessStore.error}</div>
  {/if}

  <section class="games-section">
    <div class="section-header">
      <h2>Parties disponibles</h2>
      <button
        class="btn-refresh"
        onclick={() => chessStore.loadGameList()}
        disabled={chessStore.loading}
      >
        {chessStore.loading ? '⏳' : '↻ Actualiser'}
      </button>
    </div>

    {#if chessStore.loading}
      <div class="loading-state">Chargement des parties…</div>
    {:else if chessStore.gameList.length === 0}
      <div class="empty-state">
        <span class="empty-icon">♟</span>
        <p>Aucune partie en cours. Crée-en une !</p>
      </div>
    {:else}
      <div class="games-grid">
        {#each chessStore.gameList as game (game.id)}
          <div class="game-card">
            <div class="game-header">
              <span class="game-status">{statusLabel(game.status)}</span>
              <span class="game-time">{timeAgo(game.updated_at)}</span>
            </div>
            <div class="game-body">
              <p class="game-creator">
                Créé par <strong>{game.creator_name ?? game.created_by.slice(0, 8)}</strong>
              </p>
              <div class="game-meta">
                <span class="meta-badge players">{game.player_count} joueurs</span>
                <span class="meta-badge turn">Tour {game.current_turn}</span>
              </div>
            </div>
            <div class="game-actions">
              {#if game.status === 'waiting'}
                <button class="btn-join" onclick={() => handleJoin(game)}>Rejoindre</button>
              {:else}
                <button class="btn-spectate" onclick={() => handleSpectate(game)}>Observer</button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .chess-lobby { max-width: 900px; margin: 0 auto; padding: 2rem 1rem; }

  .lobby-header { text-align: center; margin-bottom: 2.5rem; }
  .title-group { display: flex; align-items: center; justify-content: center; gap: 0.75rem; }
  .icon { font-size: 2.5rem; }
  h1 { font-size: 2.2rem; font-weight: 800; color: var(--color-text, #1e293b); margin: 0; }
  .subtitle { color: var(--color-muted, #64748b); margin: 0.5rem 0 0; }

  .create-section { margin-bottom: 2.5rem; display: flex; justify-content: center; }
  .btn-create {
    padding: 0.9rem 2.5rem; background: #2d5a27; color: white; border: none;
    border-radius: 0.75rem; font-size: 1.1rem; font-weight: 600; cursor: pointer;
    transition: background 0.2s, transform 0.15s;
  }
  .btn-create:hover { background: #3d7a37; transform: translateY(-1px); }

  .create-form {
    background: white; border: 1px solid #e2e8f0; border-radius: 1rem; padding: 2rem;
    width: 100%; max-width: 480px; box-shadow: 0 4px 20px rgba(0,0,0,0.08);
  }
  .create-form h2 { margin: 0 0 1.5rem; font-size: 1.3rem; color: #1e293b; }

  .field { margin-bottom: 1.25rem; }
  .field label { display: block; font-weight: 600; color: #374151; margin-bottom: 0.5rem; font-size: 0.9rem; }
  .field input {
    width: 100%; padding: 0.7rem 0.9rem; border: 2px solid #e2e8f0; border-radius: 0.5rem;
    font-size: 1rem; box-sizing: border-box; transition: border-color 0.2s;
  }
  .field input:focus { border-color: #2d5a27; outline: none; box-shadow: 0 0 0 3px rgba(45,90,39,0.15); }

  .player-count-selector { display: flex; gap: 0.5rem; }
  .count-btn {
    flex: 1; padding: 0.6rem; border: 2px solid #e2e8f0; border-radius: 0.5rem;
    background: white; font-size: 0.95rem; cursor: pointer; transition: all 0.15s; font-weight: 500;
  }
  .count-btn.active { border-color: #2d5a27; background: #f0fdf4; color: #2d5a27; font-weight: 700; }
  .count-btn:hover:not(.active) { border-color: #9ca3af; }

  .board-info { margin-bottom: 1.25rem; }
  .info-badge {
    display: inline-block; padding: 0.4rem 0.9rem; background: #f0fdf4;
    border: 1px solid #bbf7d0; border-radius: 999px; font-size: 0.85rem; color: #166534; font-weight: 500;
  }

  .form-actions { display: flex; gap: 0.75rem; }
  .btn-confirm {
    flex: 1; padding: 0.875rem; background: #2d5a27; color: white; border: none;
    border-radius: 0.6rem; font-size: 1rem; font-weight: 600; cursor: pointer;
    display: flex; align-items: center; justify-content: center; gap: 0.5rem; transition: background 0.2s;
  }
  .btn-confirm:hover:not(:disabled) { background: #3d7a37; }
  .btn-confirm:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-cancel {
    padding: 0.875rem 1.25rem; background: #f1f5f9; color: #475569; border: none;
    border-radius: 0.6rem; font-size: 1rem; cursor: pointer; transition: background 0.2s;
  }
  .btn-cancel:hover { background: #e2e8f0; }

  .spinner {
    width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white; border-radius: 50%; animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .alert-error {
    background: #fef2f2; border: 1px solid #fecaca; color: #dc2626;
    padding: 0.9rem 1rem; border-radius: 0.6rem; margin-bottom: 1.5rem; font-size: 0.9rem;
  }

  .section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.25rem; }
  .section-header h2 { margin: 0; font-size: 1.3rem; color: #1e293b; }
  .btn-refresh {
    padding: 0.4rem 0.9rem; background: #f1f5f9; border: 1px solid #e2e8f0;
    border-radius: 0.5rem; font-size: 0.85rem; cursor: pointer; color: #475569; transition: background 0.15s;
  }
  .btn-refresh:hover:not(:disabled) { background: #e2e8f0; }

  .loading-state { text-align: center; padding: 3rem; color: #64748b; }
  .empty-state {
    text-align: center; padding: 3rem; color: #64748b;
    display: flex; flex-direction: column; align-items: center; gap: 0.75rem;
  }
  .empty-icon { font-size: 2.5rem; opacity: 0.4; }

  .games-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem; }
  .game-card {
    background: white; border: 1px solid #e2e8f0; border-radius: 0.875rem; padding: 1.25rem;
    display: flex; flex-direction: column; gap: 0.875rem; transition: box-shadow 0.15s, transform 0.15s;
  }
  .game-card:hover { box-shadow: 0 4px 16px rgba(0,0,0,0.08); transform: translateY(-1px); }

  .game-header { display: flex; justify-content: space-between; align-items: center; }
  .game-status { font-size: 0.85rem; font-weight: 600; }
  .game-time { font-size: 0.8rem; color: #94a3b8; }
  .game-creator { margin: 0 0 0.5rem; font-size: 0.9rem; color: #475569; }
  .game-creator strong { color: #1e293b; }
  .game-meta { display: flex; gap: 0.5rem; }
  .meta-badge { padding: 0.2rem 0.6rem; border-radius: 999px; font-size: 0.78rem; font-weight: 600; }
  .meta-badge.players { background: #eff6ff; color: #1d4ed8; }
  .meta-badge.turn    { background: #fef9c3; color: #854d0e; }

  .game-actions { display: flex; gap: 0.5rem; }
  .btn-join, .btn-spectate {
    flex: 1; padding: 0.6rem; border: none; border-radius: 0.5rem;
    font-size: 0.9rem; font-weight: 600; cursor: pointer; transition: background 0.15s;
  }
  .btn-join       { background: #2d5a27; color: white; }
  .btn-join:hover  { background: #3d7a37; }
  .btn-spectate   { background: #f1f5f9; color: #475569; }
  .btn-spectate:hover { background: #e2e8f0; }

  @media (max-width: 600px) {
    .games-grid { grid-template-columns: 1fr; }
    h1 { font-size: 1.75rem; }
  }
</style>
