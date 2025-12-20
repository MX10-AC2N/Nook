<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  onMount(() => {
    // Vérifier si déjà connecté
    fetch('/api/validate-session', { credentials: 'include' })
      .then(res => {
        if (res.ok) {
          res.json().then(data => {
            if (data.role === 'admin') {
              window.location.href = '/admin';
            } else {
              window.location.href = '/chat';
            }
          });
        }
      })
      .catch(() => {});
  });
</script>

<svelte:head>
  <title>Nook – Votre espace familial privé</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4 md:p-6 relative overflow-hidden">
  <!-- Carte glassmorphism centrale -->
  <div class="max-w-2xl w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-6 md:p-10 text-center transition-all duration-1000 animate-fade-in">
    
    <!-- Logo et emoji -->
    <div class="text-8xl md:text-9xl mb-6 md:mb-8 animate-float">
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

    <h1 class="text-5xl md:text-6xl font-extrabold mb-4 bg-gradient-to-r from-[var(--accent)] to-[var(--accent-light, var(--accent))] bg-clip-text text-transparent">
      Nook
    </h1>
    
    <p class="text-xl md:text-2xl text-[var(--text-secondary)] mb-8 md:mb-12 opacity-90">
      Votre espace familial privé et sécurisé
    </p>

    <!-- Cartes d'action -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 md:gap-8 mb-10">
      <!-- Carte Connexion -->
      <div class="backdrop-blur-xl bg-white/15 dark:bg-black/15 border border-white/30 rounded-2xl p-6 md:p-8 hover:scale-105 transition-transform duration-300">
        <div class="text-5xl mb-4">🔐</div>
        <h3 class="text-xl font-bold mb-3 text-[var(--text-primary)]">Connexion</h3>
        <p class="text-[var(--text-secondary)] mb-6">
          Accédez à votre espace familial
        </p>
        <a href="/login" class="inline-block w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 transition">
          Se connecter
        </a>
      </div>

      <!-- Carte Inscription -->
      <div class="backdrop-blur-xl bg-white/15 dark:bg-black/15 border border-white/30 rounded-2xl p-6 md:p-8 hover:scale-105 transition-transform duration-300">
        <div class="text-5xl mb-4">📝</div>
        <h3 class="text-xl font-bold mb-3 text-[var(--text-primary)]">Inscription</h3>
        <p class="text-[var(--text-secondary)] mb-6">
          Demandez à rejoindre l'espace familial
        </p>
        <a href="/register" class="inline-block w-full py-3 bg-white/20 dark:bg-black/20 text-[var(--text-primary)] font-semibold rounded-xl border border-white/30 hover:bg-white/30 transition">
          Créer un compte
        </a>
      </div>
    </div>

    <!-- Informations sécurité -->
    <div class="inline-flex flex-wrap justify-center gap-4 md:gap-6 text-sm md:text-base">
      <span class="flex items-center gap-2 text-green-500">
        <span class="text-lg">✅</span>
        <span>Zéro localStorage</span>
      </span>
      <span class="flex items-center gap-2 text-green-500">
        <span class="text-lg">✅</span>
        <span>Chiffrement E2EE</span>
      </span>
      <span class="flex items-center gap-2 text-green-500">
        <span class="text-lg">✅</span>
        <span>Open-source</span>
      </span>
      <span class="flex items-center gap-2 text-green-500">
        <span class="text-lg">✅</span>
        <span>Auto-hébergé</span>
      </span>
    </div>
  </div>

  <!-- ThemeSwitcher -->
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
    50% { transform: translateY(-20px); }
  }
  .animate-fade-in { animation: fade-in 1.2s ease-out forwards; }
  .animate-float { animation: float 6s infinite ease-in-out; }
</style>