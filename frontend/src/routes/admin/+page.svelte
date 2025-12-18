<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  let inviteLink = $state('');
  let members = $state([]);
  let isLoading = $state(false);
  let error = $state(null);
  let copyFeedback = $state(false);

  const invite = async () => {
    try {
      isLoading = true;
      error = null;
      const res = await fetch('/api/invite', { method: 'POST' });
      
      if (!res.ok) throw new Error(`Erreur HTTP: ${res.status}`);
      
      const data = await res.json();
      inviteLink = data.message;
    } catch (err) {
      error = `Erreur lors de la création du lien : ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur invite:', err);
    } finally {
      isLoading = false;
    }
  };

  const approve = async (id) => {
    try {
      error = null;
      const res = await fetch(`/api/members/${id}/approve`, { method: 'PATCH' });
      
      if (!res.ok) throw new Error(`Erreur HTTP: ${res.status}`);
      
      loadMembers();
    } catch (err) {
      error = `Erreur lors de l'approbation : ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur approve:', err);
    }
  };

  const loadMembers = async () => {
    try {
      isLoading = true;
      error = null;
      const res = await fetch('/api/members');
      
      if (!res.ok) throw new Error(`Erreur HTTP: ${res.status}`);
      
      const data = await res.json();
      members = data.members || [];
    } catch (err) {
      error = `Erreur lors du chargement des membres : ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur loadMembers:', err);
    } finally {
      isLoading = false;
    }
  };

  const copyLink = async () => {
    try {
      await navigator.clipboard.writeText(inviteLink);
      copyFeedback = true;
      setTimeout(() => copyFeedback = false, 2000);
    } catch (err) {
      error = "Échec de la copie (navigateur ancien ?)";
    }
  };

  onMount(loadMembers);
</script>

<svelte:head>
  <title>Admin — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-start md:justify-center p-4 md:p-6 relative overflow-hidden">
  <!-- Carte principale glassmorphism -->
  <div class="max-w-md md:max-w-2xl w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-6 md:p-10 text-center transition-all duration-1000 animate-fade-in mt-10 md:mt-0">

    <!-- Header thématique -->
    <div class="flex items-center gap-3 md:gap-4 mb-6 md:mb-8">
      <div class="text-4xl md:text-5xl">
        {#if $currentTheme === 'jardin-secret'}
          🔑
        {:else if $currentTheme === 'space-hub'}
          🔒
        {:else}
          🛡️
        {/if}
      </div>
      <h1 class="text-2xl md:text-3xl font-extrabold text-[var(--text-primary)]">Administration Nook</h1>
    </div>

    <!-- Message d'erreur -->
    {#if error}
      <div class="mb-4 md:mb-6 p-3 md:p-4 bg-red-500/20 border border-red-500/40 text-red-600 dark:text-red-400 rounded-2xl flex justify-between items-center backdrop-blur-md">
        <span>{error}</span>
        <button onclick={() => error = null} class="text-lg md:text-xl font-bold hover:scale-125 transition">×</button>
      </div>
    {/if}

    <!-- Bouton inviter -->
    <div class="mb-6 md:mb-8">
      <button
        onclick={invite}
        disabled={isLoading}
        class="w-full py-3 md:py-4 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-lg hover:shadow-2xl hover:scale-105 active:scale-95 transition-all duration-300 disabled:opacity-60 disabled:cursor-not-allowed disabled:hover:scale-100 text-base md:text-lg"
      >
        {isLoading ? 'Création en cours...' : 'Inviter un nouveau membre'}
      </button>
    </div>

    <!-- Lien d'invitation -->
    {#if inviteLink}
      <div class="mb-8 md:mb-10 p-4 md:p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 backdrop-blur-md">
        <p class="font-medium text-base md:text-lg text-[var(--text-primary)] mb-2 md:mb-3">Lien d'invitation généré :</p>
        <div class="flex flex-col md:flex-row gap-3">
          <input
            type="text"
            value={inviteLink}
            readonly
            class="flex-1 p-3 md:p-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] text-sm md:text-base"
          />
          <button
            onclick={copyLink}
            class="w-full md:w-auto px-4 md:px-6 py-3 md:py-4 bg-[var(--accent)/80] hover:bg-[var(--accent)] text-white rounded-xl transition-all hover:scale-105 text-base md:text-lg relative"
          >
            {#if copyFeedback}
              <span class="animate-ping">✓ Copié !</span>
            {:else}
              Copier
            {/if}
          </button>
        </div>
      </div>
    {/if}

    <!-- Liste des membres -->
    <h2 class="text-xl md:text-2xl font-bold mb-4 md:mb-6 text-[var(--text-primary)]">Membres de la famille</h2>

    {#if isLoading && members.length === 0}
      <div class="text-center py-6 md:py-8 text-[var(--text-secondary)] text-base md:text-lg">Chargement des membres...</div>
    {:else if members.length === 0}
      <div class="text-center py-6 md:py-8 text-[var(--text-secondary)/70] italic text-base md:text-lg">
        Aucun membre pour le moment...
      </div>
    {:else}
      <div class="space-y-3 md:space-y-4">
        {#each members as member (member.id)}
          <div class="flex flex-col md:flex-row items-start md:items-center justify-between p-4 md:p-5 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 hover:scale-[1.02] transition backdrop-blur-md">
            <div class="flex-1 mb-3 md:mb-0">
              <div class="font-semibold text-base md:text-lg">{member.name}</div>
              <div class="text-sm md:text-base text-[var(--text-secondary)/80]">ID: {member.id}</div>
            </div>

            <div class="flex items-center gap-3 md:gap-4 w-full md:w-auto">
              <span class={`px-3 md:px-4 py-1 md:py-2 rounded-full text-sm md:text-base font-medium transition-all ${
                member.approved
                  ? 'bg-green-500/30 text-green-600 dark:text-green-400 border border-green-500/50'
                  : 'bg-yellow-500/30 text-yellow-600 dark:text-yellow-400 border border-yellow-500/50 animate-pulse'
              }`}>
                {member.approved ? '✓ Approuvé' : '⏳ En attente'}
              </span>

              {#if !member.approved}
                <button
                  onclick={() => approve(member.id)}
                  class="w-full md:w-auto px-4 md:px-5 py-2 md:py-3 bg-green-500 hover:bg-green-600 text-white font-medium rounded-xl shadow-lg hover:shadow-xl hover:scale-105 transition-all text-base md:text-lg"
                >
                  Approuver
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(30px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }

  .animate-fade-in { animation: fade-in 1s ease-out forwards; }
  .animate-float { animation: float 5s infinite ease-in-out; }

  /* Responsive optimisations */
  @media (max-width: 767px) {
    h1 { font-size: 1.8rem; }
    h2 { font-size: 1.5rem; }
    button, input { font-size: 1rem; padding: 0.75rem; } /* Touch-friendly */
    .p-6 { padding: 1.25rem; } /* Espaces réduits */
    .mb-8 { margin-bottom: 1.5rem; } /* Moins d'espaces verticaux */
    .animate-float { animation-duration: 4s; } /* Plus lent pour calme */
    .backdrop-blur-2xl { backdrop-filter: blur(8px); } /* Moins lourd pour perf mobile */
  }

  @media (min-width: 768px) and (max-width: 1024px) {
    h1 { font-size: 2.5rem; }
    h2 { font-size: 1.8rem; }
    button, input { font-size: 1.1rem; padding: 1rem; }
    .p-6 { padding: 1.5rem; }
    .mb-8 { margin-bottom: 2rem; }
  }

  @media (prefers-reduced-motion: reduce) {
    * { animation-duration: 0.01ms !important; animation-iteration-count: 1 !important; transition-duration: 0.01ms !important; }
  }
</style>