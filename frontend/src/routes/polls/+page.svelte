<script lang="ts">
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  interface Poll {
    id: number;
    question: string;
    options: string[];
    votes: number[];
  }

  let polls: Poll[] = $state([]);
  let newPoll = $state({ question: '', option1: '', option2: '', option3: '', option4: '' });
  let showCreateFeedback = $state(false);

  const loadPolls = () => {
    const stored = localStorage.getItem('nook-polls');
    if (stored) {
      polls = JSON.parse(stored);
    }
  };

  const savePolls = () => {
    localStorage.setItem('nook-polls', JSON.stringify(polls));
  };

  const createPoll = () => {
    const options = [
      newPoll.option1,
      newPoll.option2,
      newPoll.option3,
      newPoll.option4
    ].filter(opt => opt.trim() !== '');

    if (newPoll.question.trim() && options.length >= 2) {
      polls = [{
        id: Date.now(),
        question: newPoll.question.trim(),
        options,
        votes: new Array(options.length).fill(0)
      }, ...polls];

      savePolls();
      newPoll = { question: '', option1: '', option2: '', option3: '', option4: '' };
      showCreateFeedback = true;
      setTimeout(() => showCreateFeedback = false, 2000);
    }
  };

  const vote = (pollId: number, optionIndex: number) => {
    const poll = polls.find(p => p.id === pollId);
    if (poll) {
      poll.votes[optionIndex]++;
      polls = polls; // trigger reactivity
      savePolls();
    }
  };

  const getTotalVotes = (poll: Poll) => {
    return poll.votes.reduce((a, b) => a + b, 0);
  };

  const getPercentage = (poll: Poll, index: number) => {
    const total = getTotalVotes(poll);
    return total === 0 ? 0 : Math.round((poll.votes[index] / total) * 100);
  };

  onMount(() => {
    loadPolls();
  });
</script>

<svelte:head>
  <title>Sondages — Nook</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center p-6 relative">
  <!-- Carte principale glassmorphism -->
  <div class="max-w-2xl w-full bg-white/15 dark:bg-black/15 backdrop-blur-2xl border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 animate-fade-in">

    <!-- Header thématique -->
    <div class="flex items-center gap-5 mb-10">
      <div class="text-6xl animate-float">
        {#if $currentTheme === 'jardin-secret'}
          🗳️
        {:else if $currentTheme === 'space-hub'}
          🪐
        {:else}
          📊
        {/if}
      </div>
      <h1 class="text-3xl font-extrabold text-[var(--text-primary)]">Sondages familiaux</h1>
    </div>

    <!-- Création de sondage -->
    <div class="mb-12 p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 backdrop-blur-md">
      <h2 class="text-xl font-semibold mb-5 text-[var(--text-primary)]">Lancer un nouveau sondage</h2>
      
      <input
        type="text"
        bind:value={newPoll.question}
        placeholder="Votre question ? (ex: Quel film ce soir ?)"
        class="w-full p-4 mb-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
      />
      
      <div class="grid grid-cols-2 gap-4 mb-4">
        <input
          type="text"
          bind:value={newPoll.option1}
          placeholder="Option 1 (obligatoire)"
          class="p-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
        />
        <input
          type="text"
          bind:value={newPoll.option2}
          placeholder="Option 2 (obligatoire)"
          class="p-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
        />
        <input
          type="text"
          bind:value={newPoll.option3}
          placeholder="Option 3 (facultatif)"
          class="p-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
        />
        <input
          type="text"
          bind:value={newPoll.option4}
          placeholder="Option 4 (facultatif)"
          class="p-4 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40]"
        />
      </div>

      <button
        onclick={createPoll}
        class="w-full py-4 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-lg hover:shadow-2xl hover:scale-105 transition-all duration-300"
      >
        Créer le sondage
      </button>

      {#if showCreateFeedback}
        <div class="mt-4 text-center text-green-400 font-medium text-lg animate-pulse">
          ✓ Sondage créé ! Tout le monde peut voter maintenant 🎉
        </div>
      {/if}
    </div>

    <!-- Liste des sondages -->
    <h2 class="text-2xl font-bold mb-6 text-[var(--text-primary)]">Sondages en cours</h2>

    {#if polls.length === 0}
      <div class="text-center py-12 text-[var(--text-secondary)/70] italic text-lg">
        Aucun sondage pour le moment...<br />
        Lancez le premier et décidez ensemble ! 🗳️✨
      </div>
    {:else}
      <div class="space-y-8">
        {#each polls as poll (poll.id)}
          <div class="p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 backdrop-blur-md hover:scale-[1.01] transition-all animate-fade-up">
            <h3 class="text-xl font-bold mb-5 text-[var(--text-primary)]">{poll.question}</h3>
            
            <div class="space-y-4">
              {#each poll.options as option, j}
                <div class="group">
                  <button
                    onclick={() => vote(poll.id, j)}
                    class="w-full text-left p-4 rounded-xl bg-white/20 dark:bg-black/20 border border-white/30 hover:bg-[var(--accent)/20] hover:border-[var(--accent)/50] hover:scale-105 transition-all flex items-center justify-between"
                  >
                    <span class="font-medium">{option}</span>
                    <span class="text-2xl opacity-70 group-hover:opacity-100 transition">🗳️</span>
                  </button>
                  
                  <!-- Barre de progression -->
                  <div class="mt-2 h-8 bg-white/10 rounded-full overflow-hidden border border-white/20">
                    <div
                      class="h-full bg-gradient-to-r from-[var(--accent)] to-[var(--accent-light, var(--accent))] transition-all duration-1000 ease-out flex items-center justify-end pr-3 text-white font-bold"
                      style="width: {getPercentage(poll, j)}%"
                    >
                      {getPercentage(poll, j)}%
                    </div>
                  </div>
                  
                  <div class="text-right mt-1 text-sm text-[var(--text-secondary)/80]">
                    {poll.votes[j]} vote{poll.votes[j] > 1 ? 's' : ''}
                  </div>
                </div>
              {/each}
            </div>
            
            <div class="mt-5 text-center text-sm text-[var(--text-secondary)/70]">
              Total : {getTotalVotes(poll)} vote{getTotalVotes(poll) > 1 ? 's' : ''}
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
    50% { transform: translateY(-15px); }
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .animate-fade-in { animation: fade-in 1s ease-out forwards; }
  .animate-float { animation: float 6s infinite ease-in-out; }
  .animate-fade-up { animation: fade-up 0.6s ease-out forwards; }

  @media (prefers-reduced-motion: reduce) {
    * { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  }
</style>