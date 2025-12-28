<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { isAuthenticated } from '$lib/authStore';

  let currentDate = $state(new Date());
  let events = $state<Array<{id: number; title: string; date: string; time: string; description: string}>>([]);
  let showAddModal = $state(false);
  let newEvent = $state({ title: '', date: '', time: '', description: '' });
  let loading = $state(true);

  const monthNames = ['Janvier', 'Février', 'Mars', 'Avril', 'Mai', 'Juin', 'Juillet', 'Août', 'Septembre', 'Octobre', 'Novembre', 'Décembre'];

  onMount(async () => {
    if (!$isAuthenticated) {
      goto('/login');
      return;
    }
    await loadEvents();
    loading = false;
  });

  async function loadEvents() {
    try {
      const response = await fetch('/api/events', { credentials: 'include' });
      if (response.ok) {
        const data = await response.json();
        events = data.events || [];
      }
    } catch (err) {
      console.error('Erreur chargement événements:', err);
      events = [];
    }
  }

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
          description: newEvent.description
        })
      });
      if (response.ok) {
        await loadEvents();
        showAddModal = false;
        newEvent = { title: '', date: '', time: '', description: '' };
      }
    } catch (err) {
      console.error('Erreur création événement:', err);
    }
  }

  function closeModal() {
    showAddModal = false;
  }

  function handleModalKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }

  function prevMonth() {
    currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() - 1, 1);
  }

  function nextMonth() {
    currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 1);
  }

  function getDaysInMonth(date: Date): number {
    return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
  }

  function getFirstDayOfMonth(date: Date): number {
    return new Date(date.getFullYear(), date.getMonth(), 1).getDay();
  }

  function getEventsForDay(day: number): Array<{id: number; title: string; date: string; time: string; description: string}> {
    const year = currentDate.getFullYear();
    const month = String(currentDate.getMonth() + 1).padStart(2, '0');
    const dayStr = String(day).padStart(2, '0');
    const dateStr = `${year}-${month}-${dayStr}`;
    return events.filter(e => e.date === dateStr);
  }

  function getUpcomingEvents(): Array<{id: number; title: string; date: string; time: string; description: string}> {
    return events
      .filter(e => new Date(e.date) >= new Date())
      .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());
  }
</script>

<svelte:head>
  <title>Calendrier - Nook</title>
</svelte:head>

<div class="calendar-page">
  <header class="page-header">
    <h1>📅 Calendrier Familial</h1>
    <p class="subtitle">=====================</p>
  </header>

  <button onclick={() => showAddModal = true} class="add-event-btn">
    + Ajouter un événement
  </button>

  <div class="calendar-container">
    <div class="calendar-nav">
      <button onclick={prevMonth} class="nav-btn" aria-label="Mois précédent">◀</button>
      <h2 class="current-month">{monthNames[currentDate.getMonth()]} {currentDate.getFullYear()}</h2>
      <button onclick={nextMonth} class="nav-btn" aria-label="Mois suivant">▶</button>
    </div>

    <div class="calendar-grid" role="grid">
      <div class="day-header" role="columnheader">Dim</div>
      <div class="day-header" role="columnheader">Lun</div>
      <div class="day-header" role="columnheader">Mar</div>
      <div class="day-header" role="columnheader">Mer</div>
      <div class="day-header" role="columnheader">Jeu</div>
      <div class="day-header" role="columnheader">Ven</div>
      <div class="day-header" role="columnheader">Sam</div>

      {#each Array(getFirstDayOfMonth(currentDate)) as _}
        <div class="calendar-day empty" role="gridcell"></div>
      {/each}

      {#each Array(getDaysInMonth(currentDate)) as _, i}
        {@const dayEvents = getEventsForDay(i + 1)}
        <div class="calendar-day" role="gridcell" tabindex="0">
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

  {#if showAddModal}
    <div 
      class="modal-overlay" 
      onclick={closeModal}
      role="button"
      tabindex="0"
      onkeydown={handleModalKeydown}
    >
      <div 
        class="modal" 
        onclick={(e) => e.stopPropagation()}
        role="dialog"
        aria-label="Nouvel événement"
        tabindex="-1"
      >
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
            <label for="eventTime">Heure</label>
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
              placeholder="Détails de l'événement"
              rows="3"
            ></textarea>
          </div>

          <div class="form-actions">
            <button type="button" onclick={closeModal} class="cancel-btn">
              Annuler
            </button>
            <button type="submit" class="submit-btn">
              Créer
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
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
    border-radius: 0.75rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  }

  .event-date {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-width: 50px;
    padding: 0.5rem;
    background: #4ade80;
    border-radius: 0.5rem;
    color: white;
  }

  .event-day {
    font-size: 1.25rem;
    font-weight: 700;
    line-height: 1;
  }

  .event-month {
    font-size: 0.7rem;
    text-transform: uppercase;
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
    margin: 0;
    font-size: 0.85rem;
    color: #64748b;
  }

  .event-desc {
    margin: 0.5rem 0 0 0;
    font-size: 0.85rem;
    color: #64748b;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .modal {
    background: white;
    border-radius: 1rem;
    padding: 1.5rem;
    width: 100%;
    max-width: 420px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  }

  .modal h3 {
    margin: 0 0 1.25rem 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #1e293b;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    font-weight: 500;
    color: #334155;
    margin-bottom: 0.4rem;
    font-size: 0.9rem;
  }

  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1.5px solid #e2e8f0;
    border-radius: 0.5rem;
    font-size: 1rem;
    transition: all 0.2s;
    box-sizing: border-box;
  }

  .form-group input:focus,
  .form-group textarea:focus {
    outline: none;
    border-color: #4ade80;
    box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.15);
  }

  .form-group textarea {
    resize: vertical;
    min-height: 80px;
  }

  .form-actions {
    display: flex;
    gap: 0.75rem;
    margin-top: 1.5rem;
  }

  .cancel-btn,
  .submit-btn {
    flex: 1;
    padding: 0.85rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .cancel-btn {
    background: #f1f5f9;
    color: #64748b;
  }

  .cancel-btn:hover {
    background: #e2e8f0;
  }

  .submit-btn {
    background: #4ade80;
    color: white;
  }

  .submit-btn:hover {
    filter: brightness(1.1);
  }

  @media (max-width: 640px) {
    .calendar-page {
      padding: 1rem;
    }

    .calendar-container {
      padding: 1rem;
    }

    .calendar-day {
      min-height: 50px;
      padding: 0.35rem;
    }

    .day-number {
      font-size: 0.8rem;
    }

    .event-badge {
      font-size: 0.6rem;
      padding: 0.1rem 0.25rem;
    }
  }
</style>
