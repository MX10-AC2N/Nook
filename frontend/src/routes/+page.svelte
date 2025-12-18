<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

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

<div class="min-h-screen flex flex-col items-center justify-start md:justify-center p-4 md:p-6 relative overflow-hidden">
  <!-- Carte glassmorphism centrale -->
  <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-6 md:p-10 text-center transition-all duration-1000 animate-fade-in mt-10 md:mt-0">
    
    <!-- Emoji -->
    <div class="text-6xl md:text-8xl mb-6 md:mb-8 animate-float">
      {#if $currentTheme === 'jardin-secret'}
        🌿
      {:else if $currentTheme === 'space-hub'}
        🚀
      {:else if $currentTheme === 'maison-chaleureuse'}
        🏠
      {:else}
        🌿
      {/if}
    </div>

    <h1 class="text-4xl md:text-5xl font-extrabold mb-3 md:mb-4 bg-gradient-to-r from-[var(--accent)] to-[var(--accent-light, var(--accent))] bg-clip-text text-transparent">
      Nook
    </h1>
    
    <p class="text-base md:text-lg text-[var(--text-secondary)] mb-8 md:mb-10 opacity-90">
      Votre espace familial privé et sécurisé
    </p>

    {#if error}
      <div class="mb-4 md:mb-6 p-3 md:p-4 bg-red-500/20 text-red-600 dark:text-red-400 rounded-2xl border border-red-500/30 backdrop-blur-md">
        {error}
      </div>
    {/if}

    <input
      type="text"
      bind:value={name}
      placeholder="Votre prénom"
      class="w-full p-4 md:p-5 rounded-2xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40] focus:bg-white/50 dark:focus:bg-black/50 focus:border-[var(--accent)] transition-all duration-300 mb-4 md:mb-6 shadow-inner text-base md:text-lg"
      onkeydown={(e) => e.key === 'Enter' && join()}
    />

    <button
      onclick={join}
      class="w-full py-4 md:py-5 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-xl hover:shadow-2xl hover:scale-105 active:scale-95 transition-all duration-300 text-base md:text-lg"
    >
      Rejoindre
    </button>

    <p class="mt-6 md:mt-10 text-xs md:text-sm text-[var(--text-secondary)/80]">
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
    from { opacity: 0; transform: translateY(30px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }

  .animate-fade-in { animation: fade-in 1.2s ease-out forwards; }
  .animate-float { animation: float 5s infinite ease-in-out; }

  /* Responsive optimisations */
  @media (max-width: 767px) {
    input, button { font-size: 1.1rem; padding: 1rem; } /* Plus grand pour touch */
    h1 { font-size: 3rem; }
    .animate-float { animation-duration: 4s; } /* Plus lent pour calme */
    .backdrop-blur-2xl { backdrop-filter: blur(10px); } /* Moins lourd */
  }

  @media (min-width: 768px) and (max-width: 1024px) {
    h1 { font-size: 4rem; }
    .p-6 { padding: 2rem; } /* Tablette : espaces moyens */
  }

  @media (prefers-reduced-motion: reduce) {
    * { animation-duration: 0.01ms !important; animation-iteration-count: 1 !important; transition-duration: 0.01ms !important; }
  }
</style>