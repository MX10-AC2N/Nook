<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { isAuthenticated } from '$lib/authStore';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let currentDate = $state(new Date()); // mois affiché
  let events = $state<
    Array<{
      id: number;
      title: string;
      date: string; // YYYY‑MM‑DD
      time: string;
      description: string;
    }>
  >([]);

  let showAddModal = $state(false);
  let newEvent = $state({ title: '', date: '', time: '', description: '' });
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Référence pour focus sur le premier input
  let modalOverlay = $state<HTMLElement | undefined>(undefined);

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
  // 2️⃣ Cycle de vie – vérif auth + chargement des événements
  // -----------------------------------------------------------------
  onMount(async () => {
    // CORRECTION : isAuthenticated sans $
    if (!isAuthenticated) {
      goto('/login');
      return;
    }

    try {
      await loadEvents();
    } catch (e) {
      console.error('Erreur chargement événements :', e);
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
      console.error('Erreur chargement événements :', err);
      events = [];
      throw err;
    }
  }

  // -----------------------------------------------------------------
  // 4️⃣ Ajout d'un nouvel événement
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
      console.error('Erreur création événement :', err);
      alert(err instanceof Error ? err.message : 'Erreur serveur');
    } finally {
      // Reset du formulaire même en cas d'erreur
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

  // Focus automatique sur le premier input à l'ouverture
  $effect(() => {
    if (showAddModal && modalOverlay) {
      const firstInput = modalOverlay.querySelector('#eventTitle') as HTMLInputElement | null;
      if (firstInput) {
        firstInput.focus();
      }
    }
  });

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
    return new Date(date.getFullYear(), date.getMonth(), 1).getDay();
  }

  function getEventsForDay(day: number) {
    const year = currentDate.getFullYear();
    const month = String(currentDate.getMonth() + 1).padStart(2, '0');
    const dayStr = String(day).padStart(2, '0');
    // CORRECTION : syntaxe de template literal
    const dateStr = `${year}-${month}-${dayStr}`;
    return events.filter((e) => e.date === dateStr);
  }

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
  <div class="page-header">
    <h1>📅 Calendrier</h1>
    <p class="subtitle">Gérez les événements familiaux</p>
  </div>

  <!-- -----------------------------------------------------------------
       ADD EVENT BUTTON
       ----------------------------------------------------------------- -->
  <button class="add-event-btn" onclick={() => (showAddModal = true)}>
    ➕ Ajouter un événement
  </button>

  <!-- -----------------------------------------------------------------
       CALENDAR CONTAINER
       ----------------------------------------------------------------- -->
  <div class="calendar-container">
    <div class="calendar-nav">
      <button class="nav-btn" onclick={prevMonth}>←</button>
      <h2 class="current-month">
        {monthNames[currentDate.getMonth()]} {currentDate.getFullYear()}
      </h2>
      <button class="nav-btn" onclick={nextMonth}>→</button>
    </div>

      <!-- -----------------------------------------------------------------
     CALENDAR GRID
     ----------------------------------------------------------------- -->
<div class="calendar-grid">
  <div class="day-header">Lun</div>
  <div class="day-header">Mar</div>
  <div class="day-header">Mer</div>
  <div class="day-header">Jeu</div>
  <div class="day-header">Ven</div>
  <div class="day-header">Sam</div>
  <div class="day-header">Dim</div>

  <!-- Jours vides du début du mois -->
  {#each Array.from({ length: getFirstDayOfMonth(currentDate) }) as _, i}
    <div class="calendar-day empty"></div>
  {/each}

  <!-- Jours du mois -->
  {#each Array.from({ length: getDaysInMonth(currentDate) }) as _, i}
    {@const day = i + 1}
    {@const dayEvents = getEventsForDay(day)}
    <div class="calendar-day">
      <div class="day-number">{day}</div>
      <div class="day-events">
        {#each dayEvents.slice(0, 2) as event}
          <div class="event-badge">{event.title}</div>
        {/each}
        {#if dayEvents.length > 2}
          <div class="event-more">+{dayEvents.length - 2}</div>
        {/if}
      </div>
    </div>
  {/each}
</div>

  <!-- -----------------------------------------------------------------
       UPCOMING EVENTS
       ----------------------------------------------------------------- -->
  <div class="upcoming-events">
    <h3>🕓 Événements à venir</h3>
    {#if getUpcomingEvents().length === 0}
      <p class="no-events">Aucun événement à venir</p>
    {:else}
      <ul class="events-list">
        {#each getUpcomingEvents() as event}
          <li class="event-item">
            <div class="event-date">
              <span class="event-day">{new Date(event.date).getDate()}</span>
              <span class="event-month">
                {monthNames[new Date(event.date).getMonth()].slice(0, 3)}
              </span>
            </div>
            <div class="event-details">
              <h4>{event.title}</h4>
              <p class="event-time">{event.time || 'Toute la journée'}</p>
              <p class="event-desc">{event.description}</p>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <!-- -----------------------------------------------------------------
       MODAL D'AJOUT D'ÉVÉNEMENT
       ----------------------------------------------------------------- -->
  {#if showAddModal}
    <div
      bind:this={modalOverlay}
      class="modal-overlay"
      onclick={closeModal}
      role="dialog"
      aria-modal="true"
      aria-label="Nouvel événement"
      tabindex="-1"
      onkeydown={handleModalKeydown}
    >
      <div class="modal" role="document" onclick={(e) => e.stopPropagation()}>
        <h3>Nouvel événement</h3>

        <form onsubmit={(e) => { e.preventDefault(); addEvent(); }}>
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
            <input
              type="date"
              id="eventDate"
              bind:value={newEvent.date}
              required
            />
          </div>

          <div class="form-group">
            <label for="eventTime">Heure (optionnel)</label>
            <input
              type="time"
              id="eventTime"
              bind:value={newEvent.time}
            />
          </div>

          <div class="form-group">
            <label for="eventDescription">Description (optionnel)</label>
            <textarea
              id="eventDescription"
              bind:value={newEvent.description}
              placeholder="Détails de l'événement..."
            ></textarea>
          </div>

          <div class="form-actions">
            <button type="button" class="cancel-btn" onclick={closeModal}>
              Annuler
            </button>
            <button type="submit" class="submit-btn">
              Créer l'événement
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Ton style reste 100% inchangé */
  * { box-sizing: border-box; }

  .calendar-page {
    min-height: 100vh;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
    padding: 1.5rem 1rem;
  }

  /* -----------------------------------------------------------------
     HEADER
     ----------------------------------------------------------------- */
  .page-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .page-header h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0 0 0.5rem 0;
    color: #1e293b;
  }

  .subtitle {
    color: #64748b;
    margin: 0;
    font-size: 1rem;
  }

  /* -----------------------------------------------------------------
     ADD EVENT BUTTON
     ----------------------------------------------------------------- */
  .add-event-btn {
    display: block;
    width: 100%;
    max-width: 300px;
    margin: 0 auto 2rem;
    padding: 0.85rem 1.5rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 4px 12px rgba(45, 90, 39, 0.2);
  }

  .add-event-btn:hover {
    background: #3d7a37;
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(45, 90, 39, 0.3);
  }

  /* -----------------------------------------------------------------
     CALENDAR CONTAINER
     ----------------------------------------------------------------- */
  .calendar-container {
    background: white;
    border-radius: 16px;
    padding: 1.5rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    margin-bottom: 2rem;
  }

  .calendar-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .nav-btn {
    background: #f8fafc;
    border: 2px solid #e2e8f0;
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    color: #64748b;
  }

  .nav-btn:hover {
    background: #f1f5f9;
    border-color: #cbd5e1;
  }

  .current-month {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
    color: #1e293b;
  }

  /* -----------------------------------------------------------------
     CALENDAR GRID
     ----------------------------------------------------------------- */
  .calendar-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 0.5rem;
  }

  .day-header {
    text-align: center;
    font-weight: 600;
    color: #64748b;
    padding: 0.75rem 0.5rem;
    font-size: 0.875rem;
    border-bottom: 2px solid #f1f5f9;
  }

  .calendar-day {
    aspect-ratio: 1;
    padding: 0.75rem 0.5rem;
    background: #f8fafc;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    cursor: pointer;
    transition: all 0.2s;
    min-height: 85px;
    border: 2px solid transparent;
  }

  .calendar-day:hover {
    background: #f1f5f9;
    border-color: #e2e8f0;
  }

  .calendar-day:focus {
    outline: 2px solid #2d5a27;
    outline-offset: 2px;
  }

  .calendar-day.empty {
    background: transparent;
    cursor: default;
    border: none;
  }

  .calendar-day.empty:hover {
    background: transparent;
  }

  .day-number {
    font-weight: 600;
    color: #334155;
    font-size: 0.9rem;
    align-self: flex-end;
  }

  .day-events {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    overflow: hidden;
    margin-top: auto;
  }

  .event-badge {
    font-size: 0.7rem;
    padding: 0.15rem 0.35rem;
    background: #e8f5e8;
    color: #2d5a27;
    border-radius: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }

  .event-more {
    font-size: 0.7rem;
    color: #64748b;
    text-align: center;
    font-weight: 500;
  }

  /* -----------------------------------------------------------------
     UPCOMING EVENTS
     ----------------------------------------------------------------- */
  .upcoming-events {
    background: white;
    border-radius: 16px;
    padding: 1.5rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    margin-bottom: 2rem;
  }

  .upcoming-events h3 {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0 0 1.5rem 0;
    color: #1e293b;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .no-events {
    text-align: center;
    color: #64748b;
    padding: 2rem;
    background: #f8fafc;
    border-radius: 0.75rem;
    margin: 0;
  }

  .events-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .event-item {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    background: #f8fafc;
    border-radius: 12px;
    transition: all 0.2s;
    border: 2px solid transparent;
  }

  .event-item:hover {
    background: #f1f5f9;
    border-color: #e2e8f0;
    transform: translateY(-2px);
  }

  .event-date {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #2d5a27;
    color: white;
    padding: 0.5rem 0.75rem;
    border-radius: 8px;
    min-width: 60px;
  }

  .event-day {
    font-size: 1.25rem;
    font-weight: 700;
    line-height: 1;
  }

  .event-month {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .event-details {
    flex: 1;
  }

  .event-details h4 {
    margin: 0 0 0.25rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: #1e293b;
  }

  .event-time {
    color: #2d5a27;
    font-size: 0.875rem;
    font-weight: 500;
    margin: 0 0 0.5rem 0;
  }

  .event-desc {
    color: #64748b;
    font-size: 0.875rem;
    margin: 0;
    line-height: 1.4;
  }

  /* -----------------------------------------------------------------
     MODAL
     ----------------------------------------------------------------- */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal {
    background: white;
    border-radius: 16px;
    padding: 2rem;
    width: 100%;
    max-width: 500px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  }

  .modal h3 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0 0 1.5rem 0;
    color: #1e293b;
  }

  .form-group {
    margin-bottom: 1.25rem;
    text-align: left;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #374151;
    font-size: 0.95rem;
  }

  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #e2e8f0;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.2s;
    background: #f8fafc;
  }

  .form-group input:focus,
  .form-group textarea:focus {
    border-color: #2d5a27;
    box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2);
    background: white;
    outline: none;
  }

  .form-group textarea {
    resize: vertical;
    min-height: 80px;
    font-family: inherit;
  }

  .form-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 2rem;
  }

  .cancel-btn {
    padding: 0.75rem 1.5rem;
    background: #f1f5f9;
    color: #64748b;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .cancel-btn:hover {
    background: #e2e8f0;
  }

  .submit-btn {
    padding: 0.75rem 1.5rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .submit-btn:hover {
    background: #3d7a37;
    transform: translateY(-1px);
  }

  /* -----------------------------------------------------------------
     RESPONSIVE
     ----------------------------------------------------------------- */
  @media (max-width: 768px) {
    .calendar-page {
      padding: 1rem 0.75rem;
    }

    .calendar-container,
    .upcoming-events {
      padding: 1rem;
    }

    .calendar-grid {
      gap: 0.25rem;
    }

    .calendar-day {
      min-height: 70px;
      padding: 0.5rem 0.25rem;
    }

    .event-badge {
      font-size: 0.65rem;
      padding: 0.1rem 0.25rem;
    }

    .event-item {
      padding: 0.75rem;
    }

    .event-date {
      min-width: 50px;
      padding: 0.4rem 0.6rem;
    }

    .modal {
      padding: 1.5rem;
    }
  }

  @media (max-width: 480px) {
    .calendar-grid {
      grid-template-columns: repeat(7, 1fr);
    }

    .calendar-day {
      min-height: 60px;
      padding: 0.25rem;
    }

    .day-number {
      font-size: 0.8rem;
    }

    .form-actions {
      flex-direction: column;
    }

    .cancel-btn,
    .submit-btn {
      width: 100%;
    }
  }
</style>