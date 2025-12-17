<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  let events = $state([]);
  let newEvent = $state({ title: '', date: '', time: '' });
  let showAddFeedback = $state(false);

  const loadEvents = () => {
    const stored = localStorage.getItem('nook-calendar-events');
    if (stored) {
      events = JSON.parse(stored).sort((a, b) => {
        return new Date(a.date + ' ' + a.time) - new Date(b.date + ' ' + b.time);
      });
    }
  };

  const saveEvents = () => {
    localStorage.setItem('nook-calendar-events', JSON.stringify(events));
  };

  const addEvent = () => {
    if (newEvent.title.trim() && newEvent.date && newEvent.time) {
      events = [...events, { ...newEvent, id: Date.now() }];
      saveEvents();
      newEvent = { title: '', date: '', time: '' };
      showAddFeedback = true;
      setTimeout(() => showAddFeedback = false, 1500);
    }
  };

  onMount(() => {
    loadEvents();
  });
</script>

<svelte:head>
  <title>Calendrier familial — Nook</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center p-6 relative">
  <!-- Carte principale glassmorphism -->
  <div class="max-w-2xl w-full bg-white/15 dark:bg-black/15 backdrop-blur-2xl border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 animate-fade-in overflow-hidden">

    <!-- Header thématique -->
    <div class="flex items-center gap-4 mb-10">
      <div class="text-5xl animate-float">
        {#if $currentTheme === 'jardin-secret'}
          📅
        {:else if $currentTheme === 'space-hub'}
          🪐
        {:else}
          🗓️
        {/if}
      </div>
      <h1 class="text-3xl font-extrabold text-[var(--text-primary)]">Calendrier familial</h1>
    </div>

    <!-- Formulaire d'ajout -->
    <div class="mb-10 p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 backdrop-blur-md">
      <h2 class="text-xl font-semibold mb-4 text-[var(--text-primary)]">Ajouter un événement</h2>
      <input
        type="text"
        bind:value={newEvent.title}
        placeholder="Titre de l'événement (ex: Anniversaire Mamie)"
        class="w-full p-4 mb-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
      />
      <div class="grid grid-cols-2 gap-4 mb-6">
        <input
          type="date"
          bind:value={newEvent.date}
          class="p-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
        />
        <input
          type="time"
          bind:value={newEvent.time}
          class="p-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
        />
      </div>
      <button
        on:click={addEvent}
        class="w-full py-4 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-lg hover:shadow-2xl hover:scale-105 transition-all duration-300"
      >
        Ajouter à la famille
      </button>
      {#if showAddFeedback}
        <div class="mt-4 text-center text-green-400 font-medium animate-pulse">✓ Événement ajouté !</div>
      {/if}
    </div>

    <!-- Liste des événements -->
    <h2 class="text-2xl font-bold mb-6 text-[var(--text-primary)]">Événements à venir</h2>

    {#if events.length === 0}
      <div class="text-center py-12 text-[var(--text-secondary)/70] italic">
        Aucun événement pour le moment...<br />Ajoutez-en un pour organiser la famille ! ✨
      </div>
    {:else}
      <div class="space-y-4 max-h-96 overflow-y-auto scrollbar-thin scrollbar-thumb-[var(--accent)/30]">
        {#each events as event (event.id)}
          <div class="p-5 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 hover:scale-[1.02] transition-all backdrop-blur-md animate-fade-up">
            <div class="font-semibold text-lg">{event.title}</div>
            <div class="text-[var(--text-secondary)] mt-1">
              🗓️ {new Date(event.date).toLocaleDateString('fr-FR', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}
              {' '}à {event.time}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(40px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-12px); }
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .animate-fade-in { animation: fade-in 1s ease-out forwards; }
  .animate-float { animation: float 6s infinite ease-in-out; }
  .animate-fade-up { animation: fade-up 0.5s ease-out forwards; }

  .scrollbar-thin::-webkit-scrollbar { width: 6px; }
  .scrollbar-thin::-webkit-scrollbar-thumb { background-color: rgba(255,255,255,0.3); border-radius: 3px; }

  @media (prefers-reduced-motion: reduce) {
    * { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  }
</style>