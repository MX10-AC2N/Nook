<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { goto } from '$app/navigation';

  let token = $state('');
  let name = $state('');
  let error = $state('');
  let success = $state('');

  onMount(() => {
    // Récupère le token depuis l'URL
    const urlParams = new URLSearchParams(window.location.search);
    token = urlParams.get('token') || '';
    
    if (!token) {
      error = 'Lien d\'invitation invalide. Demandez un nouveau lien à l\'administrateur.';
    }
  });

  const submitRequest = async () => {
    if (!name.trim()) {
      error = 'Veuillez entrer votre prénom';
      return;
    }

    try {
      const response = await fetch(`/api/join?token=${token}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: name.trim(),
          public_key: 'temp_key_' + Date.now() // À remplacer par une vraie clé
        })
      });

      if (response.ok) {
        const data = await response.json();
        success = data.message; // "Demande envoyée à l'administrateur"
        error = '';
      } else {
        error = 'Lien d\'invitation invalide ou expiré.';
      }
    } catch (err) {
      error = 'Impossible de contacter le serveur.';
    }
  };
</script>

<svelte:head>
  <title>Rejoindre Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4 relative overflow-hidden">
  <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">
    
    <div class="text-6xl mb-6">
      {#if $currentTheme === 'jardin-secret'}
        🌿
      {:else if $currentTheme === 'space-hub'}
        🚀
      {:else}
        🏠
      {/if}
    </div>

    <h1 class="text-3xl font-bold mb-2 text-[var(--text-primary)]">Rejoindre Nook</h1>
    
    <p class="text-[var(--text-secondary)] mb-6">
      Vous avez été invité à rejoindre un espace familial privé
    </p>

    {#if error}
      <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30">
        {error}
      </div>
    {/if}

    {#if success}
      <div class="mb-4 p-3 bg-green-500/20 text-green-600 dark:text-green-400 rounded-xl border border-green-500/30">
        <div class="text-4xl mb-2">✅</div>
        <p class="font-semibold">{success}</p>
        <p class="text-sm mt-2">L'administrateur vous approuvera bientôt.</p>
      </div>
    {:else if token}
      <div>
        <input
          type="text"
          bind:value={name}
          placeholder="Votre prénom"
          class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] mb-4 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          onkeydown={(e) => e.key === 'Enter' && submitRequest()}
        />
        <button
          onclick={submitRequest}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 transition"
        >
          Demander à rejoindre
        </button>
      </div>
    {:else}
      <div class="p-4 bg-yellow-500/20 text-yellow-600 dark:text-yellow-400 rounded-xl">
        <p>Vous avez besoin d'un lien d'invitation pour rejoindre ce Nook.</p>
        <p class="text-sm mt-2">Contactez l'administrateur de votre famille.</p>
      </div>
    {/if}

    <p class="mt-6 text-sm text-[var(--text-secondary)]">
      <a href="/" class="text-[var(--accent)] hover:underline">Retour à l'accueil</a>
    </p>
  </div>

  <div class="absolute bottom-8 right-8">
    <ThemeSwitcher />
  </div>
</div>