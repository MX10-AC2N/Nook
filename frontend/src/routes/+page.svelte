<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { goto } from '$app/navigation';

  onMount(() => {
    // Redirige vers la page de demande de rejoindre
    // (les admins iront directement sur /admin)
    goto('/join');
  });
</script>

<svelte:head>
  <title>Nook – Votre espace familial privé</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center">
  <div class="text-center">
    <div class="inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-[var(--accent)] border-r-transparent"></div>
    <p class="mt-4 text-[var(--text-secondary)]">Redirection vers Nook...</p>
  </div>
  
  <div class="absolute bottom-8 right-8">
    <ThemeSwitcher />
  </div>
</div>

<style>
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  .animate-spin { animation: spin 1s linear infinite; }
</style>