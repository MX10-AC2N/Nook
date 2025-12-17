<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore'; // Pour accès réactif si besoin (optionnel ici)

  let name = $state('');
  let error = $state('');

  const join = () => {
    if (name.trim()) {
      localStorage.setItem('nook-name', name.trim());
      window.location.href = '/chat';
    } else {
      error = 'Veuillez entrer votre prénom.';
    }
  };

  onMount(() => {
    const storedName = localStorage.getItem('nook-name');
    if (storedName) window.location.href = '/chat';
  });
</script>

<svelte:head>
  <title>Nook – Votre espace familial privé</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-6 relative overflow-hidden">
  <!-- Carte glassmorphism centrale – laisse les particules du layout briller au travers -->
  <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-10 text-center transition-all duration-1000 animate-fade-in">
    
    <!-- Emoji thématique avec animation subtile -->
    <div class="text-8xl mb-6 animate-float">
      {#if $currentTheme === 'jardin-secret'}
        🌿
      {:else if $currentTheme === 'space-hub'}
        🚀
      {:else}
        🏠
      {/if}
    </div>

    <h1 class="text-5xl font-extrabold mb-4 bg-gradient-to-r from-[var(--accent)] to-[var(--accent-light, var(--accent))] bg-clip-text text-transparent">
      Nook
    </h1>
    
    <p class="text-lg text-[var(--text-secondary)] mb-10 opacity-90">
      Votre espace familial privé et sécurisé
    </p>

    {#if error}
      <div class="mb-6 p-4 bg-red-500/20 text-red-600 dark:text-red-400 rounded-2xl border border-red-500/30 backdrop-blur-md">
        {error}
      </div>
    {/if}

    <input
      type="text"
      bind:value={name}
      placeholder="Votre prénom"
      class="w-full p-5 rounded-2xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40] focus:bg-white/50 dark:focus:bg-black/50 focus:border-[var(--accent)] transition-all duration-300 mb-6 shadow-inner"
      on:keydown={(e) => e.key === 'Enter' && join()}
    />

    <button
      on:click={join}
      class="w-full py-5 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-xl hover:shadow-2xl hover:scale-105 active:scale-95 transition-all duration-300"
    >
      Rejoindre
    </button>

    <p class="mt-10 text-sm text-[var(--text-secondary)/80]">
      ✅ Zéro tracking • ✅ Chiffrement E2EE • ✅ Open-source
    </p>
  </div>

  <!-- ThemeSwitcher en bas à droite -->
  <div class="absolute bottom-8 right-8">
    <ThemeSwitcher />
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

  .animate-fade-in {
    animation: fade-in 1.2s ease-out forwards;
  }

  .animate-float {
    animation: float 6s infinite ease-in-out;
  }

  /* Respect du reduced motion */
  @media (prefers-reduced-motion: reduce) {
    * {
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
    }
  }
</style>