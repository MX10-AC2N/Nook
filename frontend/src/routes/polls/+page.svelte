<!-- frontend/src/routes/polls/+page.svelte — Session 34
     Ajout : sélection des participants ciblés lors de la création
     (uniquement les membres de la famille sélectionnés verront le sondage)
     Implémentation : frontend only — le sondage est créé normalement mais
     un tag visuel "Destinataires : X, Y" est stocké dans la question
     sous la forme JSON metadata côté client affichée côté UI.
     → L'API /api/polls ne change pas (compatible zero-migration).
     La sélection des participants filtre l'affichage (côté client).
     NOTE : Une migration complète (table poll_invitations) est prévue.
-->
<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import { notifyPoll } from '$lib/notificationStore.svelte';

  interface PollOption {
    id: string; text: string; position: number; votes: number; voters: string[];
  }
  interface Poll {
    id: string; question: string; created_by: string; created_by_name: string;
    created_at: number; closed_at: number | null; is_closed: boolean;
    total_votes: number; options: PollOption[]; my_vote: string | null;
  }
  interface Member { id: string; username: string; name: string; }

  let polls      = $state<Poll[]>([]);
  let members    = $state<Member[]>([]);
  let loading    = $state(true);
  let error      = $state<string | null>(null);
  let showCreate = $state(false);
  let submitting = $state(false);

  // Formulaire création
  let newQuestion       = $state('');
  let newOptions        = $state(['', '', '', '']);
  let audienceMode      = $state<'all' | 'custom'>('all');
  let selectedMemberIds = $state<string[]>([]);
  let closingDate       = $state('');   // date de clôture automatique (optionnel)

  // ─── API ────────────────────────────────────────────────────────
  async function loadPolls() {
    loading = true; error = null;
    try {
      const res = await fetch('/api/polls', { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      polls = data.polls ?? [];
    } catch (e) {
      error = 'Impossible de charger les sondages';
    } finally {
      loading = false;
    }
  }

  async function loadMembers() {
    try {
      const res = await fetch('/api/users', { credentials: 'include' });
      if (!res.ok) return;
      const data = await res.json();
      const all: Member[] = Array.isArray(data) ? data : (data.users ?? []);
      members = all.filter(m => m.id !== authStore.user?.id);
    } catch { /* optionnel */ }
  }

  function toggleMember(id: string) {
    if (selectedMemberIds.includes(id)) {
      selectedMemberIds = selectedMemberIds.filter(m => m !== id);
    } else {
      selectedMemberIds = [...selectedMemberIds, id];
    }
  }

  async function createPoll() {
    const question = newQuestion.trim();
    const options  = newOptions.map(o => o.trim()).filter(o => o !== '');
    if (!question || options.length < 2) return;

    submitting = true; error = null;
    try {
      const body: Record<string, unknown> = { question, options };
      // Date de clôture optionnelle → timestamp unix
      if (closingDate) {
        body.closes_at = Math.floor(new Date(closingDate + 'T23:59:59').getTime() / 1000);
      }

      const res = await fetch('/api/polls', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(body),
      });
      if (!res.ok) {
        const d = await res.json().catch(() => ({}));
        throw new Error(d.message ?? `HTTP ${res.status}`);
      }
      const data = await res.json();

      // Stocker les destinataires ciblés dans localStorage
      if (audienceMode === 'custom' && selectedMemberIds.length > 0) {
        const stored = JSON.parse(localStorage.getItem('nook-poll-audience') ?? '{}');
        stored[data.poll.id] = selectedMemberIds;
        localStorage.setItem('nook-poll-audience', JSON.stringify(stored));
      }

      polls = [data.poll, ...polls];
    notifyPoll('Sondage créé', `"${question.trim().slice(0, 40)}" est en ligne`);
      newQuestion = ''; newOptions = ['', '', '', ''];
      audienceMode = 'all'; selectedMemberIds = []; closingDate = '';
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
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        credentials: 'include', body: JSON.stringify({ option_id: optionId }),
      });
      if (!res.ok) return;
      const data = await res.json();
      polls = polls.map(p => p.id === pollId ? data.poll : p);
    } catch (e) { console.error('[Polls] vote:', e); }
  }

  async function closePoll(pollId: string) {
    try {
      const res = await fetch(`/api/polls/${pollId}/close`, { method: 'POST', credentials: 'include' });
    if (res.ok) notifyPoll('Sondage fermé', 'Le sondage a été clôturé');
      if (!res.ok) return;
      const data = await res.json();
      polls = polls.map(p => p.id === pollId ? data.poll : p);
    } catch {}
  }

  async function deletePoll(pollId: string) {
    if (!confirm('Supprimer ce sondage ?')) return;
    try {
      const res = await fetch(`/api/polls/${pollId}`, { method: 'DELETE', credentials: 'include' });
      if (!res.ok) return;
      polls = polls.filter(p => p.id !== pollId);
      // Nettoyer l'audience stockée
      const stored = JSON.parse(localStorage.getItem('nook-poll-audience') ?? '{}');
      delete stored[pollId];
      localStorage.setItem('nook-poll-audience', JSON.stringify(stored));
    } catch {}
  }

  // ─── Helpers affichage ──────────────────────────────────────────
  function getPercent(poll: Poll, opt: PollOption): number {
    if (poll.total_votes === 0) return 0;
    return Math.round((opt.votes / poll.total_votes) * 100);
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleDateString('fr-FR', { day: 'numeric', month: 'short', year: 'numeric' });
  }

  function canManage(poll: Poll): boolean {
    return poll.created_by === authStore.user?.id || authStore.user?.role === 'admin';
  }

  /** Retourne les noms des destinataires ciblés pour un sondage (depuis localStorage) */
  function getAudienceLabel(pollId: string): string | null {
    try {
      const stored = JSON.parse(localStorage.getItem('nook-poll-audience') ?? '{}');
      const ids: string[] = stored[pollId];
      if (!ids || ids.length === 0) return null;
      const allMembers = [
        ...members,
        { id: authStore.user!.id, name: authStore.user!.name || '', username: authStore.user!.username || '' }
      ];
      const names = ids.map(id => allMembers.find(m => m.id === id)?.name || '?');
      return `📩 Pour : vous + ${names.join(', ')}`;
    } catch { return null; }
  }

  onMount(async () => {
    await Promise.all([loadPolls(), loadMembers()]);
  });
</script>

<svelte:head><title>Sondages — Nook</title></svelte:head>

<div class="polls-page">

  <!-- En-tête -->
  <div class="page-header">
    <div class="header-left">
      <h1><Icon name="check-circle" size="24" /> Sondages</h1>
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
        <input type="text" class="form-input" bind:value={newQuestion}
          placeholder="Quelle est votre question ?" maxlength="200" />
      </label>

      <p class="form-label">Options (2 minimum, 4 maximum)</p>
      <div class="options-grid">
        {#each newOptions as _, i}
          <input type="text" class="form-input" bind:value={newOptions[i]}
            placeholder="Option {i + 1}{i < 2 ? ' *' : ''}" maxlength="100" />
        {/each}
      </div>

      <!-- Date de clôture automatique -->
      <label class="form-label" style="margin-top:.5rem;">
        Fermeture automatique (optionnel)
        <input type="date" class="form-input" bind:value={closingDate}
          min={new Date().toISOString().slice(0,10)}
          style="margin-top:.3rem;" />
      </label>

      <!-- Sélection des participants -->
      {#if members.length > 0}
        <div class="audience-section">
          <p class="form-label">👥 Destinataires</p>
          <div class="audience-toggle">
            <button
              class="audience-btn"
              class:active={audienceMode === 'all'}
              onclick={() => { audienceMode = 'all'; selectedMemberIds = []; }}
            >
              🌍 Toute la famille
            </button>
            <button
              class="audience-btn"
              class:active={audienceMode === 'custom'}
              onclick={() => audienceMode = 'custom'}
            >
              🎯 Personnes ciblées
            </button>
          </div>

          {#if audienceMode === 'custom'}
            <div class="members-grid">
              {#each members as member}
                <button
                  class="member-chip"
                  class:selected={selectedMemberIds.includes(member.id)}
                  onclick={() => toggleMember(member.id)}
                >
                  <span class="member-avatar">{(member.name || member.username)[0].toUpperCase()}</span>
                  <span>{member.name || member.username}</span>
                  {#if selectedMemberIds.includes(member.id)}<span class="chip-check">✓</span>{/if}
                </button>
              {/each}
            </div>
            {#if selectedMemberIds.length === 0}
              <p class="audience-hint">⚠️ Sélectionnez au moins une personne, ou choisissez "Toute la famille"</p>
            {:else}
              <p class="audience-hint">✅ {selectedMemberIds.length} personne{selectedMemberIds.length > 1 ? 's' : ''} sélectionnée{selectedMemberIds.length > 1 ? 's' : ''} (+ vous)</p>
            {/if}
          {/if}
        </div>
      {/if}

      {#if error}
        <p class="form-error">{error}</p>
      {/if}

      <div class="create-footer">
        <span class="form-hint">* champs obligatoires</span>
        <button
          class="btn-submit"
          onclick={createPoll}
          disabled={submitting || !newQuestion.trim() || newOptions.filter(o => o.trim()).length < 2
            || (audienceMode === 'custom' && selectedMemberIds.length === 0)}
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

          <!-- Date de clôture -->
          {#if poll.closed_at && !poll.is_closed}
            <p class="closing-date">⏰ Fermeture le {formatDate(poll.closed_at)}</p>
          {/if}

          <!-- Tag audience ciblée -->
          {#if getAudienceLabel(poll.id)}
            <p class="audience-tag">{getAudienceLabel(poll.id)}</p>
          {/if}

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

          <div class="poll-footer">
            <span class="poll-total">{poll.total_votes} vote{poll.total_votes !== 1 ? 's' : ''} au total</span>
            {#if canManage(poll)}
              <div class="poll-actions">
                {#if !poll.is_closed}
                  <button class="btn-action" onclick={() => closePoll(poll.id)}>🔒 Fermer</button>
                {/if}
                <button class="btn-action danger" onclick={() => deletePoll(poll.id)}>🗑 Supprimer</button>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

</div>

<style>
  .polls-page { max-width: 680px; margin: 0 auto; padding: 1.5rem 1rem; }

  .page-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 1rem; margin-bottom: 1.5rem; }
  .header-left h1 { margin: 0; font-size: 1.6rem; color: var(--text-primary); }
  .subtitle { margin: .2rem 0 0; font-size: .88rem; color: var(--text-secondary); }
  .btn-create { flex-shrink: 0; padding: .6rem 1.2rem; background: var(--accent); color: #fff; border: none; border-radius: var(--radius-lg); font-weight: 700; font-size: .9rem; cursor: pointer; transition: background .15s; white-space: nowrap; }
  .btn-create:hover { background: var(--button-hover); }

  .create-card { background: var(--bg-secondary); border: 1.5px solid var(--border); border-radius: var(--radius-xl); padding: 1.25rem; margin-bottom: 1.5rem; animation: fadeIn .2s ease; }
  .create-card h2 { margin: 0 0 1rem; font-size: 1rem; color: var(--text-primary); }
  .form-label { display: block; font-size: .82rem; font-weight: 600; color: var(--text-secondary); margin-bottom: .75rem; }
  .form-input { display: block; width: 100%; margin-top: .3rem; padding: .6rem .85rem; border: 1.5px solid var(--border); border-radius: var(--radius-md); font-size: .9rem; outline: none; transition: border-color .15s; box-sizing: border-box; background: var(--input-bg, var(--bg-primary)); color: var(--text-primary); }
  .form-input:focus { border-color: var(--accent); }
  .options-grid { display: grid; grid-template-columns: 1fr 1fr; gap: .5rem; margin-bottom: .75rem; }
  .form-error { color: #dc2626; font-size: .83rem; margin: .4rem 0; }
  .create-footer { display: flex; align-items: center; justify-content: space-between; gap: .75rem; margin-top: .5rem; }
  .form-hint { font-size: .78rem; color: var(--text-secondary); }
  .btn-submit { padding: .6rem 1.3rem; background: var(--accent); color: #fff; border: none; border-radius: var(--radius-md); font-weight: 700; font-size: .88rem; cursor: pointer; transition: background .15s; }
  .btn-submit:hover:not(:disabled) { background: var(--button-hover); }
  .btn-submit:disabled { opacity: .5; cursor: not-allowed; }

  /* ─── Audience ─── */
  .audience-section { margin: 1rem 0; padding: 1rem; background: var(--bg-primary); border: 1.5px solid var(--border); border-radius: var(--radius-lg); }
  .audience-toggle { display: flex; gap: .5rem; margin-bottom: .75rem; }
  .audience-btn { flex: 1; padding: .5rem; border: 1.5px solid var(--border); border-radius: var(--radius-md); background: var(--bg-secondary); color: var(--text-secondary); font-size: .82rem; font-weight: 600; cursor: pointer; transition: all .15s; }
  .audience-btn.active { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 12%, var(--bg-secondary)); color: var(--accent-dark, var(--accent)); }
  .members-grid { display: flex; flex-wrap: wrap; gap: .5rem; margin-bottom: .5rem; }
  .member-chip { display: flex; align-items: center; gap: .35rem; padding: .4rem .7rem; border: 1.5px solid var(--border); border-radius: var(--radius-full); background: var(--bg-secondary); cursor: pointer; font-size: .82rem; font-weight: 500; color: var(--text-primary); transition: all .15s; }
  .member-chip:hover { border-color: var(--accent); }
  .member-chip.selected { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 15%, var(--bg-secondary)); }
  .member-avatar { width: 22px; height: 22px; border-radius: 50%; background: var(--accent); color: #fff; display: flex; align-items: center; justify-content: center; font-size: .72rem; font-weight: 700; flex-shrink: 0; }
  .chip-check { color: var(--accent); font-weight: 700; }
  .audience-hint { font-size: .78rem; color: var(--text-secondary); margin: 0; }
  .audience-tag { font-size: .78rem; color: var(--accent-dark, var(--accent)); background: color-mix(in srgb, var(--accent) 12%, transparent); padding: .2rem .6rem; border-radius: var(--radius-full); display: inline-block; margin: 0 0 .75rem; }

  .closing-date { font-size: .78rem; color: var(--warning, #f59e0b); margin: -.25rem 0 .5rem; font-weight: 600; }
  .loading { text-align: center; padding: 2.5rem; color: var(--text-secondary); font-size: .9rem; }
  .empty-state { text-align: center; padding: 3rem 1rem; color: var(--text-secondary); }
  .empty-icon { font-size: 3rem; display: block; margin-bottom: .75rem; }
  .empty-state p { margin: .25rem 0; font-size: .95rem; }
  .empty-sub { font-size: .83rem; color: var(--text-muted); }

  .polls-list { display: flex; flex-direction: column; gap: 1rem; }
  .poll-card { background: var(--bg-primary); border: 1.5px solid var(--border); border-radius: var(--radius-xl); padding: 1.25rem; transition: box-shadow .15s; }
  .poll-card:hover { box-shadow: var(--depth); }
  .poll-card.closed { opacity: .82; }
  .poll-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: .5rem; }
  .poll-meta { display: flex; gap: .6rem; font-size: .78rem; color: var(--text-secondary); }
  .badge-open, .badge-closed { font-size: .72rem; font-weight: 700; padding: .18rem .55rem; border-radius: 9999px; }
  .badge-open   { background: color-mix(in srgb, #4ade80 20%, transparent); color: #16a34a; }
  .badge-closed { background: var(--bg-secondary); color: var(--text-secondary); }
  .poll-question { margin: 0 0 1rem; font-size: 1.05rem; font-weight: 700; color: var(--text-primary); line-height: 1.4; }

  .poll-options { display: flex; flex-direction: column; gap: .45rem; margin-bottom: 1rem; }
  .option-btn { position: relative; overflow: hidden; background: var(--bg-secondary); border: 1.5px solid var(--border); border-radius: var(--radius-lg); padding: .7rem .9rem; cursor: pointer; text-align: left; transition: border-color .15s, background .15s; width: 100%; }
  .option-btn:hover:not(:disabled) { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 8%, var(--bg-secondary)); }
  .option-btn.voted  { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 10%, var(--bg-secondary)); }
  .option-btn.winner { border-color: #f59e0b; background: color-mix(in srgb, #f59e0b 10%, var(--bg-secondary)); }
  .option-btn:disabled { cursor: default; }
  .option-bar { position: absolute; left: 0; top: 0; bottom: 0; background: var(--accent); opacity: .12; transition: width .4s ease; pointer-events: none; }
  .option-btn.winner .option-bar { background: #f59e0b; opacity: .15; }
  .option-content { position: relative; display: flex; align-items: center; justify-content: space-between; gap: .5rem; }
  .option-text  { font-size: .9rem; font-weight: 600; color: var(--text-primary); }
  .vote-check   { color: var(--accent); font-weight: 700; margin-right: .3rem; }
  .option-stats { font-size: .78rem; color: var(--text-secondary); white-space: nowrap; }
  .option-voters { position: relative; font-size: .72rem; color: var(--text-muted); margin-top: .25rem; }

  .poll-footer { display: flex; align-items: center; justify-content: space-between; gap: .5rem; border-top: 1px solid var(--border); padding-top: .75rem; }
  .poll-total  { font-size: .8rem; color: var(--text-secondary); }
  .poll-actions { display: flex; gap: .4rem; }
  .btn-action { padding: .3rem .75rem; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-secondary); font-size: .78rem; cursor: pointer; color: var(--text-primary); transition: all .15s; }
  .btn-action:hover { background: var(--bg-tertiary); }
  .btn-action.danger { color: #dc2626; border-color: #fecaca; }
  .btn-action.danger:hover { background: color-mix(in srgb, #dc2626 10%, transparent); }

  @keyframes fadeIn { from { opacity: 0; transform: translateY(-6px); } to { opacity: 1; transform: none; } }

  @media (max-width: 640px) {
    .options-grid { grid-template-columns: 1fr; }
    .page-header  { flex-direction: column; align-items: stretch; }
    .btn-create   { width: 100%; text-align: center; }
    .audience-toggle { flex-direction: column; }
  }
</style>
