<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';

  interface CalEvent {
    id: string; title: string; date: string; time: string;
    description: string; created_by: string;
  }

  let currentDate  = $state(new Date());
  let events       = $state<CalEvent[]>([]);
  let loading      = $state(true);
  import { notifyCalendar } from '$lib/notificationStore.svelte';
  let error        = $state<string | null>(null);
  let showAddModal = $state(false);
  let newEvent     = $state({ title: '', date: '', time: '', description: '' });
  let submitting   = $state(false);
  let formError    = $state<string | null>(null);
  let selectedDay  = $state<number | null>(null);
  let detailEvent  = $state<CalEvent | null>(null);
  let editMode     = $state(false);
  let editData     = $state({ title: '', date: '', time: '', description: '' });
  let editSaving   = $state(false);

  const today    = new Date();
  const todayStr = `${today.getFullYear()}-${String(today.getMonth()+1).padStart(2,'0')}-${String(today.getDate()).padStart(2,'0')}`;
  const monthNames = ['Janvier','Février','Mars','Avril','Mai','Juin','Juillet','Août','Septembre','Octobre','Novembre','Décembre'];

  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    await loadEvents();
  });

  async function loadEvents() {
    loading = true; error = null;
    try {
      const res = await fetch('/api/events', { credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const raw = await res.text();
      const data = raw.trim() ? JSON.parse(raw) : { events: [] };
      events = data.events ?? [];
    } catch { error = 'Impossible de charger les événements'; }
    finally { loading = false; }
  }

  async function addEvent() {
    if (!newEvent.title.trim() || !newEvent.date) { formError = 'Titre et date obligatoires'; return; }
    submitting = true; formError = null;
    try {
      const res = await fetch('/api/events', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ title: newEvent.title.trim(), date: newEvent.date,
                               time: newEvent.time, description: newEvent.description }),
      });
      if (!res.ok) { const d = await res.json().catch(() => ({})); throw new Error(d.message ?? `HTTP ${res.status}`); }
      await loadEvents();
      showAddModal = false;
      newEvent = { title: '', date: '', time: '', description: '' };
    } catch (e) { formError = e instanceof Error ? e.message : 'Erreur'; }
    finally { submitting = false; }
  }

  async function saveEdit() {
    if (!detailEvent) return;
    editSaving = true;
    try {
      const res = await fetch(`/api/events/${detailEvent.id}`, {
        method: 'PATCH', headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ title: editData.title.trim(), date: editData.date,
                               time: editData.time, description: editData.description }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      await loadEvents(); closeDetail();
    } catch (e) { alert(e instanceof Error ? e.message : 'Erreur'); }
    finally { editSaving = false; }
  }

  async function deleteEvent(id: string) {
    if (!confirm('Supprimer cet événement ?')) return;
    try {
      const res = await fetch(`/api/events/${id}`, { method: 'DELETE', credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      events = events.filter(e => e.id !== id);
    notifyCalendar('Evenement supprime', 'L evenement a ete retire du calendrier'); closeDetail();
    } catch (e) { alert(e instanceof Error ? e.message : 'Erreur'); }
  }

  function prevMonth() { currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth()-1, 1); }
  function nextMonth() { currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth()+1, 1); }
  function goToday()   { currentDate = new Date(); }

  function getDaysInMonth(d: Date) { return new Date(d.getFullYear(), d.getMonth()+1, 0).getDate(); }
  function getFirstDayOfMonth(d: Date) { const r = new Date(d.getFullYear(), d.getMonth(), 1).getDay(); return r===0?6:r-1; }
  function dayStr(day: number) {
    return `${currentDate.getFullYear()}-${String(currentDate.getMonth()+1).padStart(2,'0')}-${String(day).padStart(2,'0')}`;
  }
  function eventsForDay(day: number) { return events.filter(e => e.date === dayStr(day)); }
  function isToday(day: number)      { return dayStr(day) === todayStr; }
  function isCurrentMonth()          { return currentDate.getFullYear()===today.getFullYear() && currentDate.getMonth()===today.getMonth(); }

  const upcomingEvents = $derived(
    [...events].filter(e => e.date >= todayStr)
      .sort((a,b) => a.date.localeCompare(b.date)).slice(0, 8)
  );

  function openDay(day: number) {
    selectedDay = day;
    newEvent = { ...newEvent, date: dayStr(day) };
    const dayEvts = eventsForDay(day);
    if (dayEvts.length === 1) openDetail(dayEvts[0]);
  }
  function openDetail(evt: CalEvent) {
    detailEvent = evt; editMode = false;
    editData = { title: evt.title, date: evt.date, time: evt.time??'', description: evt.description??'' };
  }
  function closeDetail() { detailEvent = null; selectedDay = null; editMode = false; }

  function canManage(evt: CalEvent) { return evt.created_by === authStore.user?.id || authStore.isAdmin; }
  function fmtDate(ds: string) {
    if (!ds) return '';
    return new Date(ds+'T00:00:00').toLocaleDateString('fr-FR',{weekday:'long',day:'numeric',month:'long',year:'numeric'});
  }
</script>

<svelte:head><title>Calendrier — Nook</title></svelte:head>

<div class="cal-page">
  <div class="cal-header">
    <div>
      <h1>📅 Calendrier</h1>
      <p class="subtitle">Événements familiaux</p>
    </div>
    <button class="add-event-btn" onclick={() => { newEvent.date = todayStr; showAddModal = true; }}>＋ Ajouter</button>
  </div>

  {#if error}<div class="error-banner">⚠️ {error}</div>{/if}

  <div class="cal-container">
    <div class="cal-nav">
      <button class="nav-btn" onclick={prevMonth}>‹</button>
      <div class="nav-center">
        <h2>{monthNames[currentDate.getMonth()]} {currentDate.getFullYear()}</h2>
        {#if !isCurrentMonth()}<button class="today-btn" onclick={goToday}>Aujourd'hui</button>{/if}
      </div>
      <button class="nav-btn" onclick={nextMonth}>›</button>
    </div>

    <div class="calendar-grid">
      {#each ['Lun','Mar','Mer','Jeu','Ven','Sam','Dim'] as d}
        <div class="day-hdr">{d}</div>
      {/each}
      {#each Array(getFirstDayOfMonth(currentDate)) as _}
        <div class="cal-cell empty"></div>
      {/each}
      {#each Array(getDaysInMonth(currentDate)) as _, i}
        {@const day = i+1}
        {@const dayEvts = eventsForDay(day)}
        <button class="cal-cell" class:today={isToday(day)} class:has-events={dayEvts.length>0}
          class:selected-day={selectedDay===day} onclick={() => openDay(day)}>
          <span class="day-num" class:today-num={isToday(day)}>{day}</span>
          <div class="cell-events">
            {#each dayEvts.slice(0,2) as evt}
              <span class="evt-pill" onclick={(e) => { e.stopPropagation(); openDetail(evt); }} title={evt.title}>{evt.title}</span>
            {/each}
            {#if dayEvts.length > 2}<span class="evt-more">+{dayEvts.length-2}</span>{/if}
          </div>
        </button>
      {/each}
    </div>
  </div>

  <div class="upcoming">
    <h3>🗓 À venir</h3>
    {#if loading}<p class="empty-txt">Chargement…</p>
    {:else if upcomingEvents.length === 0}<p class="empty-txt">Aucun événement à venir</p>
    {:else}
      <ul class="evt-list">
        {#each upcomingEvents as evt}
          <li class="evt-item" class:today-evt={evt.date===todayStr}>
            <button class="evt-btn" onclick={() => openDetail(evt)}>
              <div class="evt-date-box">
                <span class="evt-day">{new Date(evt.date+'T00:00:00').getDate()}</span>
                <span class="evt-mon">{monthNames[new Date(evt.date+'T00:00:00').getMonth()].slice(0,3)}</span>
              </div>
              <div class="evt-info">
                <strong>{evt.title}</strong>
                {#if evt.time}<span class="evt-time">🕐 {evt.time}</span>{/if}
                {#if evt.description}<span class="evt-desc">{evt.description}</span>{/if}
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<!-- MODAL AJOUT -->
{#if showAddModal}
  <div class="modal-bg" onclick={() => showAddModal=false} role="dialog" aria-modal="true" aria-label="Ajouter un événement">
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <div class="modal-hdr"><h3>Nouvel événement</h3><button class="modal-close" onclick={() => showAddModal=false}>✕</button></div>
      <div class="modal-body">
        <label class="field">Titre *<input type="text" bind:value={newEvent.title} placeholder="Titre" maxlength="100" /></label>
        <label class="field">Date *<input type="date" bind:value={newEvent.date} /></label>
        <label class="field">Heure<input type="time" bind:value={newEvent.time} /></label>
        <label class="field">Description<textarea bind:value={newEvent.description} placeholder="Détails…" rows="2"></textarea></label>
        {#if formError}<p class="field-error">{formError}</p>{/if}
      </div>
      <div class="modal-ftr">
        <button class="btn-cancel" onclick={() => showAddModal=false}>Annuler</button>
        <button class="btn-save" onclick={addEvent} disabled={submitting}>{submitting?'Enregistrement…':'✓ Créer'}</button>
      </div>
    </div>
  </div>
{/if}

<!-- MODAL DÉTAIL/ÉDITION -->
{#if detailEvent}
  <div class="modal-bg" onclick={closeDetail} role="dialog" aria-modal="true" aria-label="Détail événement">
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <div class="modal-hdr">
        <h3>{editMode ? '✏️ Modifier' : '📌 '+detailEvent.title}</h3>
        <button class="modal-close" onclick={closeDetail}>✕</button>
      </div>
      <div class="modal-body">
        {#if editMode}
          <label class="field">Titre *<input type="text" bind:value={editData.title} maxlength="100" /></label>
          <label class="field">Date *<input type="date" bind:value={editData.date} /></label>
          <label class="field">Heure<input type="time" bind:value={editData.time} /></label>
          <label class="field">Description<textarea bind:value={editData.description} rows="2"></textarea></label>
        {:else}
          <p class="detail-row">🗓 {fmtDate(detailEvent.date)}</p>
          {#if detailEvent.time}<p class="detail-row">🕐 {detailEvent.time}</p>{/if}
          {#if detailEvent.description}<p class="detail-row desc">{detailEvent.description}</p>{/if}
        {/if}
      </div>
      <div class="modal-ftr">
        {#if editMode}
          <button class="btn-cancel" onclick={() => editMode=false}>Annuler</button>
          <button class="btn-save" onclick={saveEdit} disabled={editSaving}>{editSaving?'Enregistrement…':'✓ Sauvegarder'}</button>
        {:else}
          <button class="btn-cancel" onclick={closeDetail}>Fermer</button>
          {#if canManage(detailEvent)}
            <button class="btn-edit" onclick={() => editMode=true}>✏️ Modifier</button>
            <button class="btn-delete" onclick={() => deleteEvent(detailEvent!.id)}>🗑 Supprimer</button>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- MODAL JOUR MULTI-ÉVÉNEMENTS -->
{#if selectedDay && !detailEvent && eventsForDay(selectedDay).length > 1}
  <div class="modal-bg" onclick={() => selectedDay=null} role="dialog" aria-modal="true" aria-label="Événements du jour">
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <div class="modal-hdr"><h3>📅 {selectedDay} {monthNames[currentDate.getMonth()]}</h3><button class="modal-close" onclick={() => selectedDay=null}>✕</button></div>
      <div class="modal-body">
        <ul class="day-evts-list">
          {#each eventsForDay(selectedDay) as evt}
            <li><button class="day-evt-btn" onclick={() => openDetail(evt)}>
              <strong>{evt.title}</strong>{#if evt.time}<span class="evt-time">🕐 {evt.time}</span>{/if}
            </button></li>
          {/each}
        </ul>
      </div>
      <div class="modal-ftr">
        <button class="btn-cancel" onclick={() => selectedDay=null}>Fermer</button>
        <button class="btn-save" onclick={() => { showAddModal=true; selectedDay=null; }}>＋ Ajouter</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .cal-page { max-width: 800px; margin: 0 auto; padding: 1.25rem 1rem; }
  .cal-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 1rem; margin-bottom: 1.25rem; }
  .cal-header h1 { margin: 0; font-size: 1.5rem; color: var(--text-primary, #1e293b); }
  .subtitle { margin: .2rem 0 0; font-size: .85rem; color: var(--text-secondary, #64748b); }
  .add-event-btn { flex-shrink: 0; padding: .6rem 1.2rem; background: var(--accent, #4ade80); color: #fff; border: none; border-radius: .65rem; font-weight: 700; font-size: .9rem; cursor: pointer; transition: background .15s; }
  .add-event-btn:hover { background: var(--button-hover, #22c55e); }
  .error-banner { background: var(--error-light, #fee2e2); color: var(--error, #ef4444); padding: .75rem 1rem; border-radius: .5rem; margin-bottom: 1rem; font-size: .88rem; }
  .cal-container { background: var(--bg-primary, #fff); border: 1px solid var(--border, #e2e8f0); border-radius: 1rem; padding: 1rem; margin-bottom: 1.25rem; box-shadow: var(--depth, 0 2px 8px rgba(0,0,0,.06)); }
  .cal-nav { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .nav-center { display: flex; flex-direction: column; align-items: center; gap: .2rem; }
  .cal-nav h2 { margin: 0; font-size: 1.05rem; font-weight: 700; color: var(--text-primary, #1e293b); }
  .today-btn { padding: .2rem .6rem; font-size: .72rem; background: var(--bg-secondary, #f1f5f9); border: 1px solid var(--border, #e2e8f0); border-radius: 999px; cursor: pointer; color: var(--text-secondary, #64748b); }
  .today-btn:hover { background: var(--bg-tertiary, #e2e8f0); }
  .nav-btn { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; background: var(--bg-secondary, #f8fafc); border: 1px solid var(--border, #e2e8f0); border-radius: .5rem; font-size: 1.1rem; cursor: pointer; color: var(--text-primary, #1e293b); flex-shrink: 0; }
  .nav-btn:hover { background: var(--bg-tertiary, #e2e8f0); }
  .calendar-grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 3px; }
  .day-hdr { text-align: center; font-size: .7rem; font-weight: 700; color: var(--text-muted, #94a3b8); padding: .35rem 0; text-transform: uppercase; }
  .cal-cell { min-height: 70px; padding: .3rem .25rem; background: var(--bg-secondary, #f8fafc); border-radius: .45rem; display: flex; flex-direction: column; gap: .12rem; cursor: pointer; border: 1.5px solid transparent; transition: all .12s; text-align: left; width: 100%; }
  .cal-cell:hover { background: var(--bg-tertiary, #f1f5f9); border-color: var(--border, #e2e8f0); }
  .cal-cell.empty { background: transparent; cursor: default; border: none; }
  .cal-cell.today { background: color-mix(in srgb, var(--accent, #4ade80) 12%, var(--bg-primary, #fff)); border-color: var(--accent, #4ade80); }
  .cal-cell.selected-day { border-color: var(--accent, #4ade80); background: color-mix(in srgb, var(--accent, #4ade80) 15%, var(--bg-primary, #fff)); }
  .cal-cell.has-events { border-color: color-mix(in srgb, var(--accent, #4ade80) 35%, transparent); }
  .day-num { font-size: .78rem; font-weight: 600; color: var(--text-secondary, #64748b); align-self: flex-end; }
  .today-num { background: var(--accent, #4ade80); color: #fff; border-radius: 50%; width: 19px; height: 19px; display: flex; align-items: center; justify-content: center; font-size: .7rem; align-self: flex-end; }
  .cell-events { display: flex; flex-direction: column; gap: .1rem; width: 100%; }
  .evt-pill { display: block; font-size: .63rem; font-weight: 600; padding: .1rem .28rem; background: var(--accent, #4ade80); color: #fff; border-radius: .22rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; cursor: pointer; }
  .evt-more { font-size: .63rem; color: var(--text-muted, #94a3b8); text-align: center; }
  .upcoming { background: var(--bg-primary, #fff); border: 1px solid var(--border, #e2e8f0); border-radius: 1rem; padding: 1rem; box-shadow: var(--depth, 0 2px 8px rgba(0,0,0,.06)); }
  .upcoming h3 { margin: 0 0 .85rem; font-size: 1rem; font-weight: 700; color: var(--text-primary, #1e293b); }
  .empty-txt { text-align: center; color: var(--text-muted, #94a3b8); font-size: .88rem; padding: 1.5rem 0; }
  .evt-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: .5rem; }
  .evt-btn { display: flex; gap: .85rem; align-items: flex-start; width: 100%; background: var(--bg-secondary, #f8fafc); border: 1px solid var(--border, #e2e8f0); border-radius: .6rem; padding: .65rem .85rem; cursor: pointer; text-align: left; transition: all .12s; }
  .evt-btn:hover { background: var(--bg-tertiary, #f1f5f9); border-color: var(--accent, #4ade80); }
  .today-evt .evt-date-box { background: var(--accent, #4ade80); color: #fff; }
  .evt-date-box { display: flex; flex-direction: column; align-items: center; justify-content: center; background: var(--bg-tertiary, #e2e8f0); color: var(--text-primary, #1e293b); border-radius: .4rem; min-width: 42px; padding: .3rem .35rem; flex-shrink: 0; }
  .evt-day { font-size: 1.05rem; font-weight: 800; line-height: 1; }
  .evt-mon { font-size: .62rem; font-weight: 600; text-transform: uppercase; }
  .evt-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: .12rem; }
  .evt-info strong { font-size: .9rem; color: var(--text-primary, #1e293b); font-weight: 700; }
  .evt-time { font-size: .77rem; color: var(--accent-dark, #22c55e); font-weight: 600; }
  .evt-desc { font-size: .77rem; color: var(--text-secondary, #64748b); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .modal-bg { position: fixed; inset: 0; background: rgba(0,0,0,.5); display: flex; align-items: center; justify-content: center; z-index: 200; padding: 1rem; backdrop-filter: blur(3px); }
  .modal { background: var(--bg-primary, #fff); border-radius: 1rem; width: 100%; max-width: 440px; box-shadow: 0 20px 60px rgba(0,0,0,.25); overflow: hidden; }
  .modal-hdr { display: flex; align-items: center; justify-content: space-between; padding: 1rem 1.25rem; border-bottom: 1px solid var(--border, #e2e8f0); }
  .modal-hdr h3 { margin: 0; font-size: 1rem; font-weight: 700; color: var(--text-primary, #1e293b); }
  .modal-close { background: none; border: none; font-size: 1.1rem; cursor: pointer; color: var(--text-secondary, #64748b); padding: .2rem .4rem; border-radius: .35rem; }
  .modal-close:hover { background: var(--bg-secondary, #f1f5f9); }
  .modal-body { padding: 1rem 1.25rem; display: flex; flex-direction: column; gap: .75rem; max-height: 60vh; overflow-y: auto; }
  .modal-ftr { display: flex; gap: .5rem; justify-content: flex-end; padding: .85rem 1.25rem; border-top: 1px solid var(--border, #e2e8f0); }
  .field { display: flex; flex-direction: column; gap: .3rem; font-size: .82rem; font-weight: 600; color: var(--text-secondary, #64748b); }
  .field input, .field textarea { padding: .55rem .85rem; border: 1.5px solid var(--border, #e2e8f0); border-radius: .5rem; font-size: .9rem; background: var(--bg-secondary, #f8fafc); color: var(--text-primary, #1e293b); outline: none; transition: border-color .15s; font-family: inherit; }
  .field input:focus, .field textarea:focus { border-color: var(--accent, #4ade80); background: var(--bg-primary, #fff); }
  .field textarea { resize: vertical; }
  .field-error { font-size: .8rem; color: var(--error, #ef4444); margin: 0; }
  .detail-row { margin: 0; font-size: .92rem; color: var(--text-primary, #1e293b); }
  .detail-row.desc { color: var(--text-secondary, #64748b); line-height: 1.5; }
  .btn-cancel { padding: .55rem 1rem; background: var(--bg-secondary, #f1f5f9); border: 1px solid var(--border, #e2e8f0); border-radius: .5rem; font-size: .88rem; cursor: pointer; color: var(--text-secondary, #64748b); }
  .btn-cancel:hover { background: var(--bg-tertiary, #e2e8f0); }
  .btn-save { padding: .55rem 1.1rem; background: var(--accent, #4ade80); color: #fff; border: none; border-radius: .5rem; font-size: .88rem; font-weight: 700; cursor: pointer; }
  .btn-save:hover:not(:disabled) { background: var(--button-hover, #22c55e); }
  .btn-save:disabled { opacity: .55; cursor: not-allowed; }
  .btn-edit { padding: .55rem 1rem; background: var(--bg-secondary, #f1f5f9); border: 1px solid var(--border, #e2e8f0); border-radius: .5rem; font-size: .88rem; cursor: pointer; color: var(--text-primary, #1e293b); }
  .btn-edit:hover { background: var(--bg-tertiary, #e2e8f0); }
  .btn-delete { padding: .55rem 1rem; background: transparent; border: 1px solid #fecaca; border-radius: .5rem; font-size: .88rem; cursor: pointer; color: var(--error, #ef4444); }
  .btn-delete:hover { background: var(--error-light, #fee2e2); }
  .day-evts-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: .4rem; }
  .day-evt-btn { display: flex; flex-direction: column; gap: .15rem; width: 100%; background: var(--bg-secondary, #f8fafc); border: 1px solid var(--border, #e2e8f0); border-radius: .5rem; padding: .65rem .85rem; cursor: pointer; text-align: left; transition: all .12s; }
  .day-evt-btn:hover { background: var(--bg-tertiary, #f1f5f9); border-color: var(--accent, #4ade80); }
  .day-evt-btn strong { font-size: .9rem; color: var(--text-primary, #1e293b); }
  @media (max-width: 640px) {
    .cal-cell { min-height: 52px; padding: .2rem .12rem; }
    .evt-pill { font-size: .58rem; }
    .day-num { font-size: .7rem; }
    .cal-header { flex-direction: column; gap: .5rem; }
    .add-event-btn { width: 100%; text-align: center; }
  }
</style>
