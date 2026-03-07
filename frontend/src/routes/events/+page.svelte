<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { getCurrentTheme } from '$lib/ui/ThemeStore.svelte.ts';

  // ─────────────────────────────────────────────────────────────────
  // Types
  // ─────────────────────────────────────────────────────────────────
  interface EventItem {
    id: string;
    title: string;
    date: string;   // format ISO YYYY-MM-DD
    time: string;   // format HH:MM (optionnel)
    description?: string;
    created_by?: string;
  }

  // ─────────────────────────────────────────────────────────────────
  // État
  // ─────────────────────────────────────────────────────────────────
  let events      = $state<EventItem[]>([]);
  let loading     = $state(true);
  let error       = $state<string | null>(null);
  let submitting  = $state(false);

  // Formulaire d'ajout
  let newEvent = $state({ title: '', date: '', time: '', description: '' });
  let showAddFeedback = $state(false);

  // ─────────────────────────────────────────────────────────────────
  // API
  // ─────────────────────────────────────────────────────────────────
  function sortEvents(list: EventItem[]): EventItem[] {
    return [...list].sort((a, b) => {
      const da = new Date(`${a.date}T${a.time || '00:00'}`);
      const db = new Date(`${b.date}T${b.time || '00:00'}`);
      return da.getTime() - db.getTime();
    });
  }

  async function loadEvents() {
    loading = true;
    error = null;
    try {
      const res = await fetch('/api/events', { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      events = sortEvents(data.events ?? data ?? []);
    } catch (e) {
      error = 'Impossible de charger les événements.';
      console.error('[Events] loadEvents:', e);
    } finally {
      loading = false;
    }
  }

  async function addEvent() {
    if (!newEvent.title.trim() || !newEvent.date) {
      error = 'Le titre et la date sont obligatoires.';
      return;
    }
    error = null;
    submitting = true;
    try {
      const res = await fetch('/api/events', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          title: newEvent.title.trim(),
          date: newEvent.date,
          time: newEvent.time || null,
          description: newEvent.description.trim() || null,
        }),
      });
      if (!res.ok) {
        const d = await res.json().catch(() => ({}));
        throw new Error(d.message ?? `HTTP ${res.status}`);
      }
      // Recharger la liste pour avoir l'ID serveur et l'ordre correct
      await loadEvents();
      newEvent = { title: '', date: '', time: '', description: '' };
      showAddFeedback = true;
      setTimeout(() => (showAddFeedback = false), 2000);
    } catch (e: any) {
      error = e?.message ?? "Erreur lors de l'ajout.";
      console.error('[Events] addEvent:', e);
    } finally {
      submitting = false;
    }
  }

  async function deleteEvent(id: string) {
    try {
      const res = await fetch(`/api/events/${id}`, {
        method: 'DELETE',
        credentials: 'include',
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      events = events.filter(e => e.id !== id);
    } catch (e) {
      console.error('[Events] deleteEvent:', e);
    }
  }

  onMount(loadEvents);
</script>

<svelte:head>
  <title>Rendez‑vous — Nook</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center p-6 relative">
  <div
    class="max-w-2xl w-full bg-white/15 dark:bg-black/15 backdrop-blur-2xl border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 animate-fade-in"
  >
    <!-- En-tête -->
    <div class="flex items-center gap-5 mb-10">
      <div class="text-6xl animate-float">
        {#if getCurrentTheme === 'jardin-secret'}
          🌸
        {:else if getCurrentTheme === 'space-hub'}
          🌌
        {:else}
          🎉
        {/if}
      </div>
      <h1 class="text-3xl font-extrabold text-[var(--text-primary)]">
        Rendez‑vous familiaux
      </h1>
    </div>

    <!-- Message d'erreur -->
    {#if error}
      <div class="mb-6 p-4 bg-red-500/20 border border-red-400/40 rounded-xl text-red-300 text-sm">
        ⚠️ {error}
      </div>
    {/if}

    <!-- Formulaire d'ajout -->
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

      <div class="grid grid-cols-2 gap-4 mb-4">
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

      <input
        type="text"
        bind:value={newEvent.description}
        placeholder="Description (optionnelle)"
        class="w-full p-4 mb-6 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40] transition-all"
      />

      <button
        onclick={addEvent}
        disabled={submitting}
        class="w-full py-4 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-lg hover:shadow-2xl hover:scale-105 active:scale-95 transition-all duration-300 disabled:opacity-60 disabled:cursor-not-allowed disabled:hover:scale-100"
      >
        {submitting ? '⏳ Ajout en cours…' : 'Ajouter ce rendez‑vous'}
      </button>

      {#if showAddFeedback}
        <div class="mt-4 text-center text-green-400 font-medium text-lg animate-pulse">
          ✓ Rendez‑vous ajouté avec succès !
        </div>
      {/if}
    </div>

    <!-- Liste des événements -->
    <h2 class="text-2xl font-bold mb-6 text-[var(--text-primary)]">
      Prochains moments ensemble
    </h2>

    {#if loading}
      <div class="text-center py-12 text-[var(--text-secondary)/70] italic text-lg">
        Chargement…
      </div>
    {:else if events.length === 0}
      <div class="text-center py-12 text-[var(--text-secondary)/70] italic text-lg">
        Aucun rendez‑vous prévu pour l'instant…
        Créez le premier moment inoubliable en famille ✨
      </div>
    {:else}
      <div class="space-y-5 max-h-96 overflow-y-auto scrollbar-thin scrollbar-thumb-[var(--accent)/30]">
        {#each events as event (event.id)}
          <div
            class="p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 hover:scale-[1.02] transition-all backdrop-blur-md animate-fade-up"
          >
            <div class="flex justify-between items-start">
              <div class="flex-1">
                <div class="font-bold text-xl text-[var(--text-primary)]">
                  {event.title}
                </div>
                <div class="mt-2 text-[var(--text-secondary)] flex items-center gap-2">
                  <span>🗓️</span>
                  <span>
                    {new Date(event.date).toLocaleDateString('fr-FR', {
                      weekday: 'long',
                      year: 'numeric',
                      month: 'long',
                      day: 'numeric',
                    })}
                  </span>
                </div>
                {#if event.time}
                  <div class="mt-1 text-[var(--text-secondary)] flex items-center gap-2">
                    <span>🕐</span>
                    <span>à {event.time}</span>
                  </div>
                {/if}
                {#if event.description}
                  <div class="mt-2 text-sm text-[var(--text-secondary)/80] italic">
                    {event.description}
                  </div>
                {/if}
              </div>

              <div class="flex flex-col items-center gap-3 ml-4">
                <div class="text-4xl opacity-30">
                  {#if getCurrentTheme === 'jardin-secret'}
                    🌿
                  {:else if getCurrentTheme === 'space-hub'}
                    ⭐
                  {:else}
                    ❤️
                  {/if}
                </div>
                <button
                  onclick={() => deleteEvent(event.id)}
                  class="text-xs text-[var(--text-secondary)/60] hover:text-red-400 transition-colors"
                  aria-label="Supprimer cet événement"
                >
                  🗑️
                </button>
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
    from { opacity: 0; transform: translateY(40px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50%       { transform: translateY(-15px); }
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(20px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .animate-fade-in { animation: fade-in 1s ease-out forwards; }
  .animate-float   { animation: float 6s infinite ease-in-out; }
  .animate-fade-up { animation: fade-up 0.5s ease-out forwards; }

  .scrollbar-thin::-webkit-scrollbar { width: 6px; }
  .scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.3);
    border-radius: 3px;
  }

  @media (prefers-reduced-motion: reduce) {
    * { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  }
</style>
