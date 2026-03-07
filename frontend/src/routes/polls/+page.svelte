<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';

  // ─────────────────────────────────────────────────────────────────
  // Types
  // ─────────────────────────────────────────────────────────────────
  interface PollOption {
    id: string;
    text: string;
    position: number;
    votes: number;
    voters: string[];
  }

  interface Poll {
    id: string;
    question: string;
    created_by: string;
    created_by_name: string;
    created_at: number;
    closed_at: number | null;
    is_closed: boolean;
    total_votes: number;
    options: PollOption[];
    my_vote: string | null;
  }

  // ─────────────────────────────────────────────────────────────────
  // État
  // ─────────────────────────────────────────────────────────────────
  let polls       = $state<Poll[]>([]);
  let loading     = $state(true);
  let error       = $state<string | null>(null);
  let showCreate  = $state(false);
  let submitting  = $state(false);

  // Formulaire création
  let newQuestion = $state('');
  let newOptions  = $state(['', '', '', '']);

  // ─────────────────────────────────────────────────────────────────
  // API
  // ─────────────────────────────────────────────────────────────────
  async function loadPolls() {
    loading = true;
    error = null;
    try {
      const res = await fetch('/api/polls', { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      polls = data.polls ?? [];
    } catch (e) {
      error = 'Impossible de charger les sondages';
      console.error('[Polls] loadPolls:', e);
    } finally {
      loading = false;
    }
  }

  async function createPoll() {
    const question = newQuestion.trim();
    const options = newOptions.map(o => o.trim()).filter(o => o !== '');
    if (!question || options.length < 2) return;

    submitting = true;
    try {
      const res = await fetch('/api/polls', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ question, options }),
      });
      if (!res.ok) {
        const d = await res.json().catch(() => ({}));
        throw new Error(d.message ?? `HTTP ${res.status}`);
      }
      const data = await res.json();
      polls = [data.poll, ...polls];
      newQuestion = '';
      newOptions = ['', '', '', ''];
      showCreate = false;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur création';
    } finally {
      submitting = false;
    }
  }

  async function vote(pollId: string, optionId: string) {
    try {
      const res = await fetch(`/api/polls/${pollId}/vote`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ option_id: optionId }),
      });
      if (!res.ok) return;
      const data = await res.json();
      polls = polls.map(p => p.id === pollId ? data.poll : p);
    } catch (e) {
      console.error('[Polls] vote:', e);
    }
  }

  async function closePoll(pollId: string) {
    try {
      const res = await fetch(`/api/polls/${pollId}/close`, {
        method: 'POST',
        credentials: 'include',
      });
      if (!res.ok) return;
      const data = await res.json();
      polls = polls.map(p => p.id === pollId ? data.poll : p);
    } catch (e) {
      console.error('[Polls] close:', e);
    }
  }

  async function deletePoll(pollId: string) {
    if (!confirm('Supprimer ce sondage ?')) return;
    try {
      const res = await fetch(`/api/polls/${pollId}`, {
        method: 'DELETE',
        credentials: 'include',
      });
      if (!res.ok) return;
      polls = polls.filter(p => p.id !== pollId);
    } catch (e) {
      console.error('[Polls] delete:', e);
    }
  }

  // ─────────────────────────────────────────────────────────────────
  // Helpers affichage
  // ─────────────────────────────────────────────────────────────────
  function getPercent(poll: Poll, opt: PollOption): number {
    if (poll.total_votes === 0) return 0;
    return Math.round((opt.votes / poll.total_votes) * 100);
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleDateString('fr-FR', {
      day: 'numeric', month: 'short', year: 'numeric'
    });
  }

  function canManage(poll: Poll): boolean {
    return poll.created_by === authStore.user?.id || authStore.user?.role === 'admin';
  }

  onMount(loadPolls);
</script>

<svelte:head><title>Sondages — Nook</title></svelte:head>

<div class="polls-page">

  <!-- En-tête -->
  <div class="page-header">
    <div class="header-left">
      <h1>📊 Sondages</h1>
      <p class="subtitle">Décidez ensemble</p>
    </div>
    <button class="btn-create" onclick={() => showCreate = !showCreate}>
      {showCreate ? '✕ Annuler' : '＋ Nouveau sondage'}
    </button>
  </div>

  <!-- Formulaire création -->
  {#if showCreate}
    <div class="create-card">
      <h2>Nouveau sondage</h2>

      <label class="form-label">
        Question
        <input
          type="text"
          class="form-input"
          bind:value={newQuestion}
          placeholder="Quelle est votre question ?"
          maxlength="200"
        />
      </label>

      <p class="form-label">Options (2 minimum, 4 maximum)</p>
      <div class="options-grid">
        {#each newOptions as _, i}
          <input
            type="text"
            class="form-input"
            bind:value={newOptions[i]}
            placeholder="Option {i + 1}{i < 2 ? ' *' : ''}"
            maxlength="100"
          />
        {/each}
      </div>

      {#if error}
        <p class="form-error">{error}</p>
      {/if}

      <div class="create-footer">
        <span class="form-hint">* champs obligatoires</span>
        <button
          class="btn-submit"
          onclick={createPoll}
          disabled={submitting || !newQuestion.trim() || newOptions.filter(o => o.trim()).length < 2}
        >
          {submitting ? 'Création…' : 'Créer le sondage'}
        </button>
      </div>
    </div>
  {/if}

  <!-- Liste des sondages -->
  {#if loading}
    <div class="loading">Chargement des sondages…</div>
  {:else if polls.length === 0}
    <div class="empty-state">
      <span class="empty-icon">🗳️</span>
      <p>Aucun sondage pour le moment</p>
      <p class="empty-sub">Créez le premier sondage familial !</p>
    </div>
  {:else}
    <div class="polls-list">
      {#each polls as poll (poll.id)}
        <div class="poll-card" class:closed={poll.is_closed}>

          <!-- En-tête du sondage -->
          <div class="poll-header">
            <div class="poll-meta">
              <span class="poll-author">Par {poll.created_by_name}</span>
              <span class="poll-date">{formatDate(poll.created_at)}</span>
            </div>
            {#if poll.is_closed}
              <span class="badge-closed">Fermé</span>
            {:else}
              <span class="badge-open">Ouvert</span>
            {/if}
          </div>

          <h2 class="poll-question">{poll.question}</h2>

          <!-- Options + barres de progression -->
          <div class="poll-options">
            {#each poll.options as opt (opt.id)}
              {@const percent = getPercent(poll, opt)}
              {@const isMyVote = poll.my_vote === opt.id}
              <button
                class="option-btn"
                class:voted={isMyVote}
                class:winner={poll.is_closed && opt.votes === Math.max(...poll.options.map(o => o.votes)) && opt.votes > 0}
                disabled={poll.is_closed}
                onclick={() => vote(poll.id, opt.id)}
              >
                <div class="option-bar" style="width: {percent}%"></div>
                <div class="option-content">
                  <span class="option-text">
                    {#if isMyVote}<span class="vote-check">✓</span>{/if}
                    {opt.text}
                  </span>
                  <span class="option-stats">
                    {opt.votes} vote{opt.votes !== 1 ? 's' : ''}
                    {#if poll.total_votes > 0} · {percent}%{/if}
                  </span>
                </div>
                {#if opt.voters.length > 0}
                  <div class="option-voters">{opt.voters.join(', ')}</div>
                {/if}
              </button>
            {/each}
          </div>

          <!-- Pied de carte -->
          <div class="poll-footer">
            <span class="poll-total">
              {poll.total_votes} vote{poll.total_votes !== 1 ? 's' : ''} au total
            </span>
            {#if canManage(poll)}
              <div class="poll-actions">
                {#if !poll.is_closed}
                  <button class="btn-action" onclick={() => closePoll(poll.id)}>
                    🔒 Fermer
                  </button>
                {/if}
                <button class="btn-action danger" onclick={() => deletePoll(poll.id)}>
                  🗑 Supprimer
                </button>
              </div>
            {/if}
          </div>

        </div>
      {/each}
    </div>
  {/if}

</div>

<style>
  .polls-page {
    max-width: 680px;
    margin: 0 auto;
    padding: 1.5rem 1rem;
  }

  /* ── En-tête ── */
  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }
  .header-left h1 {
    margin: 0;
    font-size: 1.6rem;
    color: var(--text-primary, #1e293b);
  }
  .subtitle {
    margin: .2rem 0 0;
    font-size: .88rem;
    color: var(--text-secondary, #64748b);
  }
  .btn-create {
    flex-shrink: 0;
    padding: .6rem 1.2rem;
    background: var(--accent, #4ade80);
    color: #fff;
    border: none;
    border-radius: .6rem;
    font-weight: 700;
    font-size: .9rem;
    cursor: pointer;
    transition: background .15s;
    white-space: nowrap;
  }
  .btn-create:hover { background: var(--button-hover, #22c55e); }

  /* ── Formulaire ── */
  .create-card {
    background: var(--bg-secondary, #f8fafc);
    border: 1.5px solid var(--border, #e2e8f0);
    border-radius: 1rem;
    padding: 1.25rem;
    margin-bottom: 1.5rem;
    animation: fadeIn .2s ease;
  }
  .create-card h2 {
    margin: 0 0 1rem;
    font-size: 1rem;
    color: var(--text-primary, #1e293b);
  }
  .form-label {
    display: block;
    font-size: .82rem;
    font-weight: 600;
    color: var(--text-secondary, #64748b);
    margin-bottom: .75rem;
  }
  .form-input {
    display: block;
    width: 100%;
    margin-top: .3rem;
    padding: .6rem .85rem;
    border: 1.5px solid var(--border, #e2e8f0);
    border-radius: .5rem;
    font-size: .9rem;
    outline: none;
    transition: border-color .15s;
    box-sizing: border-box;
    background: var(--bg-primary, #fff);
    color: var(--text-primary, #1e293b);
  }
  .form-input:focus { border-color: var(--accent, #4ade80); }
  .options-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: .5rem;
    margin-bottom: .75rem;
  }
  .form-error {
    color: #dc2626;
    font-size: .83rem;
    margin: .4rem 0;
  }
  .create-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: .75rem;
    margin-top: .5rem;
  }
  .form-hint {
    font-size: .78rem;
    color: var(--text-secondary, #94a3b8);
  }
  .btn-submit {
    padding: .6rem 1.3rem;
    background: var(--accent, #4ade80);
    color: #fff;
    border: none;
    border-radius: .5rem;
    font-weight: 700;
    font-size: .88rem;
    cursor: pointer;
    transition: background .15s;
  }
  .btn-submit:hover:not(:disabled) { background: var(--button-hover, #22c55e); }
  .btn-submit:disabled { opacity: .5; cursor: not-allowed; }

  /* ── États loading / empty ── */
  .loading {
    text-align: center;
    padding: 2.5rem;
    color: var(--text-secondary, #94a3b8);
    font-size: .9rem;
  }
  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    color: var(--text-secondary, #64748b);
  }
  .empty-icon { font-size: 3rem; display: block; margin-bottom: .75rem; }
  .empty-state p { margin: .25rem 0; font-size: .95rem; }
  .empty-sub { font-size: .83rem; color: var(--text-secondary, #94a3b8); }

  /* ── Cards sondages ── */
  .polls-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .poll-card {
    background: var(--bg-primary, #fff);
    border: 1.5px solid var(--border, #e2e8f0);
    border-radius: 1rem;
    padding: 1.25rem;
    transition: box-shadow .15s;
  }
  .poll-card:hover { box-shadow: 0 4px 16px rgba(0,0,0,.07); }
  .poll-card.closed { opacity: .82; }

  .poll-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: .5rem;
  }
  .poll-meta {
    display: flex;
    gap: .6rem;
    font-size: .78rem;
    color: var(--text-secondary, #64748b);
  }
  .badge-open, .badge-closed {
    font-size: .72rem;
    font-weight: 700;
    padding: .18rem .55rem;
    border-radius: 9999px;
  }
  .badge-open   { background: #dcfce7; color: #16a34a; }
  .badge-closed { background: #f1f5f9; color: #64748b; }

  .poll-question {
    margin: 0 0 1rem;
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
    line-height: 1.4;
  }

  /* ── Options ── */
  .poll-options {
    display: flex;
    flex-direction: column;
    gap: .45rem;
    margin-bottom: 1rem;
  }
  .option-btn {
    position: relative;
    overflow: hidden;
    background: var(--bg-secondary, #f8fafc);
    border: 1.5px solid var(--border, #e2e8f0);
    border-radius: .6rem;
    padding: .7rem .9rem;
    cursor: pointer;
    text-align: left;
    transition: border-color .15s, background .15s;
    width: 100%;
  }
  .option-btn:hover:not(:disabled) {
    border-color: var(--accent, #4ade80);
    background: #f0fdf4;
  }
  .option-btn.voted {
    border-color: var(--accent, #4ade80);
    background: #f0fdf4;
  }
  .option-btn.winner {
    border-color: #f59e0b;
    background: #fffbeb;
  }
  .option-btn:disabled { cursor: default; }

  .option-bar {
    position: absolute;
    left: 0; top: 0; bottom: 0;
    background: var(--accent, #4ade80);
    opacity: .12;
    transition: width .4s ease;
    pointer-events: none;
  }
  .option-btn.winner .option-bar {
    background: #f59e0b;
    opacity: .15;
  }

  .option-content {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: .5rem;
  }
  .option-text {
    font-size: .9rem;
    font-weight: 600;
    color: var(--text-primary, #1e293b);
  }
  .vote-check {
    color: var(--accent, #4ade80);
    font-weight: 700;
    margin-right: .3rem;
  }
  .option-stats {
    font-size: .78rem;
    color: var(--text-secondary, #64748b);
    white-space: nowrap;
  }
  .option-voters {
    position: relative;
    font-size: .72rem;
    color: var(--text-secondary, #94a3b8);
    margin-top: .25rem;
  }

  /* ── Pied de carte ── */
  .poll-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: .5rem;
    border-top: 1px solid var(--border, #e2e8f0);
    padding-top: .75rem;
  }
  .poll-total {
    font-size: .8rem;
    color: var(--text-secondary, #64748b);
  }
  .poll-actions {
    display: flex;
    gap: .4rem;
  }
  .btn-action {
    padding: .3rem .75rem;
    border: 1px solid var(--border, #e2e8f0);
    border-radius: .4rem;
    background: var(--bg-secondary, #f8fafc);
    font-size: .78rem;
    cursor: pointer;
    transition: all .15s;
  }
  .btn-action:hover { background: var(--bg-tertiary, #e2e8f0); }
  .btn-action.danger { color: #dc2626; border-color: #fecaca; }
  .btn-action.danger:hover { background: #fee2e2; }

  @keyframes fadeIn { from { opacity: 0; transform: translateY(-6px); } to { opacity: 1; transform: none; } }

  /* ── Mobile ── */
  @media (max-width: 640px) {
    .options-grid { grid-template-columns: 1fr; }
    .page-header { flex-direction: column; align-items: stretch; }
    .btn-create { width: 100%; text-align: center; }
  }
</style>
