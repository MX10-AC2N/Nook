<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { checkAuth } from '$lib/auth';

  let identifier = $state('');
  let password = $state('');
  let error = $state('');
  let isLoading = $state(false);
  let userStatus = $state('loading');

  onMount(async () => {
    try {
      const status = await checkAuth();
      userStatus = status.status;
      
      if (userStatus === 'approved') {
        goto('/chat');
      } else if (userStatus === 'admin') {
        goto('/admin');
      }
    } catch (err) {
      console.error('Erreur vérification auth:', err);
      userStatus = 'guest';
    }
  });

  const handleLogin = async () => {
    if (!identifier.trim() || !password.trim()) {
      error = 'Veuillez remplir tous les champs';
      return;
    }

    isLoading = true;
    error = '';

    try {
      const response = await fetch('/api/member/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          identifier: identifier.trim(), 
          password: password.trim() 
        }),
        credentials: 'include'
      });

      if (response.ok) {
        goto('/chat');
      } else if (response.status === 401) {
        error = 'Identifiants incorrects ou compte non approuvé';
      } else {
        error = 'Erreur de connexion';
      }
    } catch (err) {
      error = 'Impossible de contacter le serveur';
    } finally {
      isLoading = false;
    }
  };
</script>

<svelte:head>
  <title>Connexion — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4">
  {#if userStatus === 'loading'}
    <div class="text-center">
      <div class="inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-[var(--accent)] border-r-transparent"></div>
      <p class="mt-4 text-[var(--text-secondary)]">Vérification...</p>
    </div>
  {:else}
    <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">
      
      <div class="text-6xl mb-6">
        🔐
      </div>

      <h1 class="text-3xl font-bold mb-2 text-[var(--text-primary)]">Connexion à Nook</h1>
      
      <p class="text-[var(--text-secondary)] mb-6">
        Entrez vos identifiants pour accéder à votre espace familial
      </p>

      {#if error}
        <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30">
          {error}
        </div>
      {/if}

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            ID membre ou nom d'utilisateur
          </label>
          <input
            type="text"
            bind:value={identifier}
            placeholder="Votre ID membre ou nom d'utilisateur"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={(e) => e.key === 'Enter' && handleLogin()}
          />
        </div>

        <div>
          <label class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Mot de passe
          </label>
          <input
            type="password"
            bind:value={password}
            placeholder="Votre mot de passe"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={(e) => e.key === 'Enter' && handleLogin()}
          />
        </div>

        <button
          onclick={handleLogin}
          disabled={isLoading}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 transition"
        >
          {isLoading ? 'Connexion...' : 'Se connecter'}
        </button>
      </div>

      <div class="mt-6 p-4 bg-blue-500/10 border border-blue-500/20 rounded-xl">
        <p class="text-sm text-[var(--text-secondary)]">
          <strong>Première connexion ?</strong><br>
          1. Utilisez votre ID membre reçu après inscription<br>
          2. Si vous avez créé un nom d'utilisateur, utilisez-le à la place<br>
          3. Si vous n'avez pas encore de mot de passe, utilisez le lien reçu par email
        </p>
      </div>

      <div class="mt-6 space-y-2">
        <p class="text-sm text-[var(--text-secondary)]">
          <a href="/join" class="text-[var(--accent)] hover:underline">
            Faire une demande d'adhésion
          </a>
        </p>
        <p class="text-sm text-[var(--text-secondary)]">
          <a href="/" class="text-[var(--accent)] hover:underline">
            Retour à l'accueil
          </a>
        </p>
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  .animate-spin { animation: spin 1s linear infinite; }
</style>