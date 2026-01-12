<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { state } from 'svelte'; // <-- Svelte 5 reactive state
  import { isAuthenticated } from '$lib/authStore';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let currentDate = state(new Date()); // mois affiché
  let events = state<Array<{ id: number; title: string; date: string; time: string; description: string }>>([]);
  let showAddModal = state(false);
  let newEvent = state({ title: '', date: '', time: '', description: '' });
  let loading = state(true);
  let error = state<string | null>(null);

  const monthNames = [
    'Janvier',
    'Février',
    'Mars',
    'Avril',
    'Mai',
    'Juin',
    'Juillet',
    'Août',
    'Septembre',
    'Octobre',
    'Novembre',
    'Décembre',
  ];

  // -----------------------------------------------------------------
  // 2️⃣ Cycle de vie – vérification auth + chargement des événements
  // -----------------------------------------------------------------
  onMount(async () => {
    if (!$isAuthenticated) {
      goto('/login');
      return;
    }

    try {
      await loadEvents();
    } catch (e) {
      console.error('Erreur chargement événements :', e);
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  // -----------------------------------------------------------------
  // 3️⃣ Chargement des événements depuis le backend
  // -----------------------------------------------------------------
  async function loadEvents() {
    try {
      const response = await fetch('/api/events', { credentials: 'include' });
      const raw = await response.text();

      if (!response.ok) {
        throw new Error(`Erreur ${response.status}`);
      }

      const data = raw.trim() ? JSON.parse(raw) : { events: [] };
      events = data.events || [];
    } catch (err) {
      console.error('Erreur chargement événements :', err);
      events = [];
      throw err;
    }
  }

  // -----------------------------------------------------------------
  // 4️⃣ Ajout d’un nouvel événement
  // -----------------------------------------------------------------
  async function addEvent() {
    if (!newEvent.title || !newEvent.date) {
      alert('Veuillez remplir le titre et la date');
      return;
    }

    try {
      const response = await fetch('/api/events', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          title: newEvent.title,
          date: newEvent.date,
          time: newEvent.time,
          description: newEvent.description,
        }),
      });

      if (!response.ok) {
        const raw = await response.text();
        const data = raw.trim() ? JSON.parse(raw) : {};
        throw new Error(data.message ?? `Erreur ${response.status}`);
      }

      // Recharger la liste après création réussie
      await loadEvents();
      showAddModal = false;
    } catch (err) {
      console.error('Erreur création événement :', err);
      alert(err instanceof Error ? err.message : 'Erreur serveur');
    } finally {
      // Reset du formulaire même en cas d’erreur
      newEvent = { title: '', date: '', time: '', description: '' };
    }
  }

  // -----------------------------------------------------------------
  // 5️⃣ Gestion du modal (ouverture / fermeture)
  // -----------------------------------------------------------------
  function closeModal() {
    showAddModal = false;
  }

  function handleModalKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }

  // -----------------------------------------------------------------
  // 6️⃣ Navigation entre les mois
  // -----------------------------------------------------------------
  function prevMonth() {
    currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() - 1, 1);
  }

  function nextMonth() {
    currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 1);
  }

  // -----------------------------------------------------------------
  // 7️⃣ Helpers calendrier
  // -----------------------------------------------------------------
  function getDaysInMonth(date: Date): number {
    return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
  }

  function getFirstDayOfMonth(date: Date): number {
    // 0 = dimanche, 1 = lundi, …
    return new Date(date.getFullYear(), date.getMonth(), 1).getDay();
  }

  /** Retourne les événements du jour (format ISO `YYYY-MM-DD`). */
  function getEventsForDay(day: number) {
    const year = currentDate.getFullYear();
    const month = String(currentDate.getMonth() + 1).padStart(2, '0');
    const dayStr = String(day).padStart(2, '0');
    const dateStr = `${year}-${month}-${dayStr}`;
    return events.filter((e) => e.date === dateStr);
  }

  /** Retourne les prochains événements (aujourd’hui inclus). */
  function getUpcomingEvents() {
    const today = new Date();
    return events
      .filter((e) => new Date(e.date) >= today)
      .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());
  }
</script>

<svelte:head>
  <title>Calendrier - Nook</title>
</svelte:head>

<div class="calendar-page">
  <!-- -----------------------------------------------------------------
       HEADER
       ----------------------------------------------------------------- -->
  <header class="page-header">
    <h1>📅 Calendrier Familial</h1>
    <p class="subtitle">=====================</p>
  </header>

  <!-- -----------------------------------------------------------------
       BOUTON AJOUT EVENT
       ----------------------------------------------------------------- -->
  <button on:click={() => (showAddModal = true)} class="add-event-btn">
    + Ajouter un événement
  </button>

  <!-- -----------------------------------------------------------------
       CALENDRIER
       ----------------------------------------------------------------- -->
  <div class="calendar-container">
    <div class="calendar-nav">
      <button on:click={prevMonth} class="nav-btn" aria-label="Mois précédent">
        ◀
      </button>
      <h2 class="current-month">
        {monthNames[currentDate.getMonth()]} {currentDate.getFullYear()}
      </h2>
      <button on:click={nextMonth} class="nav-btn" aria-label="Mois suivant">
        ▶
      </button>
    </div>

    <div class="calendar-grid" role="grid">
      <!-- Days of week header -->
      <div class="day-header" role="columnheader">Dim</div>
      <div class="day-header" role="columnheader">Lun</div>
      <div class="day-header" role="columnheader">Mar</div>
      <div class="day-header" role="columnheader">Mer</div>
      <div class="day-header" role="columnheader">Jeu</div>
      <div class="day-header" role="columnheader">Ven</div>
      <div class="day-header" role="columnheader">Sam</div>

      <!-- Empty cells before first day -->
      {#each Array(getFirstDayOfMonth(currentDate)) as _}
        <div class="calendar-day empty" role="gridcell"></div>
      {/each}

      <!-- Days of month -->
      {#each Array(getDaysInMonth(currentDate)) as _, i}
        {@const dayEvents = getEventsForDay(i + 1)}
        <div
          class="calendar-day"
          role="gridcell"
          tabindex="0"
          aria-label={`Jour ${i + 1}, ${dayEvents.length} événement${dayEvents.length > 1 ? 's' : ''}`}
        >
          <span class="day-number">{i + 1}</span>

          {#if dayEvents.length > 0}
            <div class="day-events">
              {#each dayEvents.slice(0, 2) as event}
                <span class="event-badge">{event.title}</span>
              {/each}
              {#if dayEvents.length > 2}
                <span class="event-more">+{dayEvents.length - 2}</span>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- -----------------------------------------------------------------
       PROCHAINS ÉVÉNEMENTS
       ----------------------------------------------------------------- -->
  <section class="upcoming-events">
    <h3>### Événements à venir</h3>

    {#if getUpcomingEvents().length === 0}
      <p class="no-events">Aucun événement à venir</p>
    {:else}
      <ul class="events-list">
        {#each getUpcomingEvents() as event}
          <li class="event-item">
            <div class="event-date">
              <span class="event-day">{new Date(event.date).getDate()}</span>
              <span class="event-month">{monthNames[new Date(event.date).getMonth()].slice(0, 3)}</span>
            </div>

            <div class="event-details">
              <h4>{event.title}</h4>
              <p class="event-time">{event.time || 'Toute la journée'}</p>
              {#if event.description}
                <p class="event-desc">{event.description}</p>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- -----------------------------------------------------------------
       MODAL AJOUT EVENT
       ----------------------------------------------------------------- -->
  {#if showAddModal}
    <div
      class="modal-overlay"
      on:click={closeModal}
      role="button"
      tabindex="0"
      on:keydown={handleModalKeydown}
    >
      <div
        class="modal"
        on:click|stopPropagation
        role="dialog"
        aria-label="Nouvel événement"
        tabindex="-1"
      >
        <h3>Nouvel événement</h3>

        <form on:submit|preventDefault={addEvent}>
          <div class="form-group">
            <label for="eventTitle">Titre</label>
            <input
              type="text"
              id="eventTitle"
              bind:value={newEvent.title}
              placeholder="Nom de l'événement"
              required
            />
          </div>

          <div class="form-group">
            <label for="eventDate">Date</label>
            <input type="date" id="eventDate" bind:value={newEvent.date} required />
          </div>

          <div class="form-group">
            <label for="eventTime">Heure</label>
            <input type="time" id="eventTime" bind:value={newEvent.time} />
          </div>

          <div class="form-group">
            <label for="eventDescription">Description (optionnel)</label>
            <textarea
              id="eventDescription"
              bind:value={newEvent.description}
              placeholder="Détails de l'événement"
              rows="3"
            ></textarea>
          </div>

          <div class="form-actions">
            <button type="button" on:click={closeModal} class="cancel-btn">
              Annuler
            </button>
            <button type="submit" class="submit-btn">Créer</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  /* -----------------------------------------------------------------
     PAGE LAYOUT
     ----------------------------------------------------------------- */
  .calendar-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 1.5rem;
  }

  .page-header {
    text-align: center;
    margin-bottom: 1.5rem;
  }

  .page-header h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0 0 0.25rem 0;
    color: #1e293b;
  }

  .subtitle {
    color: #64748b;
    margin: 0;
    font-size: 0.95rem;
  }

  .add-event-btn {
    display: block;
    width: 100%;
    max-width: 300px;
    margin: 0 auto 1.5rem;
    padding: 0.85rem 1.5rem;
    background: #4ade80;
    color: white;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .add-event-btn:hover {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  /* -----------------------------------------------------------------
     CALENDAR CONTAINER
     ----------------------------------------------------------------- */
  .calendar-container {
    background: white;
    border-radius: 1rem;
    padding: 1.5rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    margin-bottom: 2rem;
  }

  .calendar-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.25rem;
  }

  .nav-btn {
    background: #f1f5f9;
    border: none;
    padding: 0.6rem 1rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .nav-btn:hover {
    background: #e2e8f0;
  }

  .current-month {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
    color: #1e293b;
  }

  .calendar-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 0.5rem;
  }

  .day-header {
    text-align: center;
    font-weight: 600;
    color: #64748b;
    padding: 0.5rem;
    font-size: 0.85rem;
  }

  .calendar-day {
    aspect-ratio: 1;
    padding: 0.5rem;
    background: #f8fafc;
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    cursor: pointer;
    transition: all 0.2s;
    min-height: 70px;
  }

  .calendar-day:hover {
    background: #f1f5f9;
  }

  .calendar-day:focus {
    outline: 2px solid #4ade80;
    outline-offset: 2px;
  }

  .calendar-day.empty {
    background: transparent;
    cursor: default;
  }

  .calendar-day.empty:hover {
    background: transparent;
  }

  .day-number {
    font-weight: 600;
    color: #334155;
    font-size: 0.9rem;
  }

  .day-events {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    overflow: hidden;
  }

  .event-badge {
    font-size: 0.7rem;
    padding: 0.15rem 0.35rem;
    background: #dbeafe;
    color: #1d4ed8;
    border-radius: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .event-more {
    font-size: 0.7rem;
    color: #64748b;
    text-align: center;
  }

  /* -----------------------------------------------------------------
     UPCOMING EVENTS
     ----------------------------------------------------------------- */
  .upcoming-events h3 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
    color: #1e293b;
  }

  .no-events {
    text-align: center;
    color: #64748b;
    padding: 2rem;
    background: #f8fafc;
    border-radius: 0.75rem;
  }

  .events-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .event-item {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    background: white;