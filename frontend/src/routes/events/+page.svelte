<script lang="ts">
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore.svelte';

  // -----------------------------------------------------------------
  // 1️⃣ Types & données locales
  // -----------------------------------------------------------------
  interface EventItem {
    id: number;
    title: string;
    date: string; // format ISO YYYY‑MM‑DD
    time: string; // format HH:mm
  }

  // Liste d'événements (persistée dans le `localStorage`)
  let events = $state<EventItem[]>([]);

  // Formulaire d'ajout d'un événement
  let newEvent = $state({ title: '', date: '', time: '' });

  // Petit feedback visuel après ajout
  let showAddFeedback = $state(false);

  // -----------------------------------------------------------------
  // 2️⃣ Helpers de persistance (localStorage)
  // -----------------------------------------------------------------
  /** Trie les événements chronologiquement (date + heure). */
  function sortEvents(list: EventItem[]): EventItem[] {
    return list.sort((a, b) => {
      const da = new Date(`${a.date} ${a.time}`);
      const db = new Date(`${b.date} ${b.time}`);
      return da.getTime() - db.getTime();
    });
  }

  /** Charge les événements depuis le `localStorage`. */
  function loadEvents() {
    const stored = localStorage.getItem('nook-events');
    if (stored) {
      try {
        const parsed: EventItem[] = JSON.parse(stored);
        events = sortEvents(parsed);
      } catch {
        // Si le JSON est corrompu, on repart à zéro
        events = [];
        localStorage.removeItem('nook-events');
      }
    }
  }

  /** Sauvegarde la liste d'événements dans le `localStorage`. */
  function saveEvents() {
    localStorage.setItem('nook-events', JSON.stringify(events));
  }

  // -----------------------------------------------------------------
  // 3️⃣ Ajout d'un nouvel événement
  // -----------------------------------------------------------------
  function addEvent() {
    // Validation minimale
    if (!newEvent.title.trim() || !newEvent.date || !newEvent.time) {
      alert("Veuillez remplir le titre, la date et l'heure.");
      return;
    }

    const added: EventItem = {
      id: Date.now(),
      title: newEvent.title.trim(),
      date: newEvent.date,
      time: newEvent.time,
    };

    events = sortEvents([...events, added]);
    saveEvents();

    // Reset du formulaire
    newEvent = { title: '', date: '', time: '' };

    // Feedback visuel (2 s)
    showAddFeedback = true;
    setTimeout(() => (showAddFeedback = false), 2000);
  }

  // -----------------------------------------------------------------
  // 4️⃣ Cycle de vie – charger les événements au montage
  // -----------------------------------------------------------------
  onMount(() => {
    loadEvents();
  });
</script>

<svelte:head>
  <title>Rendez‑vous — Nook</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center p-6 relative">
  <!-- ---------------------------------------------------------------
       CARTE PRINCIPALE (glassmorphism)
       --------------------------------------------------------------- -->
  <div
    class="max-w-2xl w-full bg-white/15 dark:bg-black/15 backdrop-blur-2xl border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 animate-fade-in"
  >
    <!-- ---------------------------------------------------------------
         EN‑TÊTE THÉMATIQUE
         --------------------------------------------------------------- -->
    <div class="flex items-center gap-5 mb-10">
      <div class="text-6xl animate-float">
        {#if $currentTheme === 'jardin-secret'}
          🌸
        {:else if $currentTheme === 'space-hub'}
          🌌
        {:else}
          🎉
        {/if}
      </div>
      <h1 class="text-3xl font-extrabold text-[var(--text-primary)]">
        Rendez‑vous familiaux
      </h1>
    </div>

    <!-- ---------------------------------------------------------------
         FORMULAIRE D'AJOUT
         --------------------------------------------------------------- -->
    <div
      class="mb-10 p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 backdrop-blur-md"
    >
      <h2 class="text-xl font-semibold mb-5 text-[var(--text-primary)]">
        Planifier un nouveau moment
      </h2>

      <input
        type="text"
        bind:value={newEvent.title}
        placeholder="ex : Dîner de Noël, Appel visio, Sortie parc…"
        class="w-full p-4 mb-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40] transition-all"
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
        onclick={addEvent}
        class="w-full py-4 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-lg hover:shadow-2xl hover:scale-105 active:scale-95 transition-all duration-300"
      >
        Ajouter ce rendez‑vous
      </button>

      {#if showAddFeedback}
        <div
          class="mt-4 text-center text-green-400 font-medium text-lg animate-pulse"
        >
          ✓ Rendez‑vous ajouté avec succès !
        </div>
      {/if}
    </div>

    <!-- ---------------------------------------------------------------
         LISTE DES RENDEZ‑VOUS
         --------------------------------------------------------------- -->
    <h2 class="text-2xl font-bold mb-6 text-[var(--text-primary)]">
      Prochains moments ensemble
    </h2>

    {#if events.length === 0}
      <div
        class="text-center py-12 text-[var(--text-secondary)/70] italic text-lg"
      >
        Aucun rendez‑vous prévu pour l'instant… 
        Créez le premier moment inoubliable en famille ✨
      </div>
    {:else}
      <div
        class="space-y-5 max-h-96 overflow-y-auto scrollbar-thin scrollbar-thumb-[var(--accent)/30]"
      >
        {#each events as event (event.id)}
          <div
            class="p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 hover:scale-[1.02] transition-all backdrop-blur-md animate-fade-up"
          >
            <div class="flex justify-between items-start">
              <div>
                <div class="font-bold text-xl text-[var(--text-primary)]">
                  {event.title}
                </div>
                <div class="mt-2 text-[var(--text-secondary)] flex items-center gap-2">
                  <span>🗓️</span>
                  <span
                    >{new Date(event.date).toLocaleDateString('fr-FR', {
                    weekday: 'long',
                    year: 'numeric',
                    month: 'long',
                    day: 'numeric',
                  })}</span
                  >
                </div>
                <div class="mt-1 text-[var(--text-secondary)] flex items-center gap-2">
                  <span>🕐</span>
                  <span>à {event.time}</span>
                </div>
              </div>

              <div class="text-4xl opacity-30">
                {#if $currentTheme === 'jardin-secret'}
                  🌿
                {:else if $currentTheme === 'space-hub'}
                  ⭐
                {:else}
                  ❤️
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(40px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes float {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-15px);
    }
  }

  @keyframes fade-up {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-fade-in {
    animation: fade-in 1s ease-out forwards;
  }

  .animate-float {
    animation: float 6s infinite ease-in-out;
  }

  .animate-fade-up {
    animation: fade-up 0.5s ease-out forwards;
  }

  .scrollbar-thin::-webkit-scrollbar {
    width: 6px;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.3);
    border-radius: 3px;
  }

  @media (prefers-reduced-motion: reduce) {
    * {
      animation-duration: 0.01ms !important;
      transition-duration: 0.01ms !important;
    }
  }
</style>
