<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { goto } from '@roxi/routify';

  let name = $state('');
  let username = $state('');
  let password = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);

  onMount(() => {
    // Vérifier si déjà connecté
    fetch('/api/validate-session', { credentials: 'include' })
      .then(res => {
        if (res.ok) {
          goto('/chat');
        }
      })
      .catch(() => {});
  });

  const submitRegister = async () => {
    // Validation
    if (!name.trim() || !username.trim() || !password || !confirmPassword) {
      error = 'Veuillez remplir tous les champs';
      return;
    }
    
    if (username.length < 3) {
      error = 'L\'identifiant doit contenir au moins 3 caractères';
      return;
    }
    
    if (password.length < 8) {
      error = 'Le mot de passe doit contenir au moins 8 caractères';
      return;
    }
    
    if (password !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas';
      return;
    }

    isLoading = true;
    error = '';

    try {
      const response = await fetch('/api/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: name.trim(),
          username: username.trim(),
          password: password
        })
      });

      const data = await response.json();
      
      if (data.success) {
        success = data.message;
        name = '';
        username = '';
        password = '';
        confirmPassword = '';
      } else {
        error = data.message || 'Erreur lors de l\'inscription';
      }
    } catch (err) {
      error = 'Impossible de contacter le serveur';
    } finally {
      isLoading = false;
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && !isLoading) {
      submitRegister();
    }
  };
</script>

<svelte:head>
  <title>Inscription — Nook</title>
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

    <h1 class="text-3xl font-bold mb-2 text-[var(--text-primary)]">Inscription à Nook</h1>
    
    <p class="text-[var(--text-secondary)] mb-6">
      Demandez à rejoindre l'espace familial
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
        <p class="text-sm mt-2">Vous recevrez un email lorsque votre compte sera approuvé.</p>
        <div class="mt-4">
          <a href="/login" class="inline-block px-4 py-2 bg-[var(--accent)] text-white rounded-lg hover:opacity-90 transition">
            Aller à la connexion
          </a>
        </div>
      </div>
    {:else}
      <div class="space-y-4">
        <div>
          <label for="name" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Votre nom
          </label>
          <input
            id="name"
            type="text"
            bind:value={name}
            placeholder="Ex: Jean Dupont"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
          />
        </div>
        
        <div>
          <label for="username" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Identifiant
          </label>
          <input
            id="username"
            type="text"
            bind:value={username}
            placeholder="Ex: jean.dupont"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
          />
          <p class="text-xs text-[var(--text-secondary)] mt-1 text-left">
            Minimum 3 caractères. Utilisez-le pour vous connecter.
          </p>
        </div>

        <div>
          <label for="password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Mot de passe
          </label>
          <input
            id="password"
            type="password"
            bind:value={password}
            placeholder="Minimum 8 caractères"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
          />
        </div>

        <div>
          <label for="confirm-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Confirmer le mot de passe
          </label>
          <input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            placeholder="Répétez le mot de passe"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
          />
        </div>

        <button
          onclick={submitRegister}
          disabled={isLoading}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 transition"
        >
          {isLoading ? 'Inscription en cours...' : 'Demander l\'inscription'}
        </button>
      </div>
    {/if}

    <div class="mt-6 pt-4 border-t border-white/20">
      <p class="text-sm text-[var(--text-secondary)]">
        Déjà un compte ? 
        <a href="/login" class="text-[var(--accent)] hover:underline ml-1">
          Se connecter
        </a>
      </p>
    </div>
  </div>

  <div class="absolute bottom-8 right-8">
    <ThemeSwitcher />
  </div>
</div>