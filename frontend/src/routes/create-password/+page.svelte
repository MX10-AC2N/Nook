<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { checkPasswordStatus, createPassword, checkAuth } from '$lib/authStore';

  let password = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);
  let memberInfo = $state(null);
  let isChecking = $state(true);

  onMount(async () => {
    // Vérifier si l'utilisateur est connecté
    const isAuth = await checkAuth();
    if (!isAuth) {
      goto('/login');
      return;
    }

    // Vérifier le statut du mot de passe
    try {
      const status = await checkPasswordStatus();
      if (status) {
        memberInfo = status;
        
        // Si l'utilisateur a déjà un mot de passe, rediriger
        if (status.has_password) {
          goto('/chat');
        }
      }
    } catch (err) {
      console.error('Erreur vérification statut:', err);
      error = 'Impossible de vérifier votre statut. Veuillez vous reconnecter.';
    } finally {
      isChecking = false;
    }
  });

  async function handleSubmit() {
    // Validation
    if (!password || !confirmPassword) {
      error = 'Veuillez remplir tous les champs';
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
    
    // Vérifier la complexité du mot de passe
    const hasUpperCase = /[A-Z]/.test(password);
    const hasLowerCase = /[a-z]/.test(password);
    const hasNumbers = /\d/.test(password);
    const hasSpecialChar = /[!@#$%^&*(),.?":{}|<>]/.test(password);
    
    if (!hasUpperCase || !hasLowerCase || !hasNumbers || !hasSpecialChar) {
      error = 'Le mot de passe doit contenir au moins une majuscule, une minuscule, un chiffre et un caractère spécial';
      return;
    }

    isLoading = true;
    error = '';

    try {
      const successResult = await createPassword(password);
      
      if (successResult) {
        success = 'Mot de passe créé avec succès !';
        password = '';
        confirmPassword = '';
        
        // Rediriger après 2 secondes
        setTimeout(() => {
          goto('/chat');
        }, 2000);
      } else {
        error = 'Erreur lors de la création du mot de passe. Veuillez réessayer.';
      }
    } catch (err) {
      console.error('Erreur création mot de passe:', err);
      error = 'Une erreur est survenue. Veuillez réessayer.';
    } finally {
      isLoading = false;
    }
  }

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && !isLoading) {
      handleSubmit();
    }
  };
</script>

<svelte:head>
  <title>Créer un mot de passe — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4">
  {#if isChecking}
    <div class="text-center">
      <div class="text-6xl mb-4 animate-spin">🌀</div>
      <p class="text-[var(--text-secondary)]">Vérification de votre compte...</p>
    </div>
  {:else if memberInfo}
    <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">
      
      <div class="text-6xl mb-6">
        🔐
      </div>

      <h1 class="text-3xl font-bold mb-2 text-[var(--text-primary)]">Créer votre mot de passe</h1>
      
      <p class="text-[var(--text-secondary)] mb-6">
        Bonjour <span class="font-semibold text-[var(--accent)]">{memberInfo.name}</span> !<br>
        Créez un mot de passe sécurisé pour votre compte.
      </p>

      {#if error}
        <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30">
          <div class="flex items-center gap-2">
            <span>⚠️</span>
            <span>{error}</span>
          </div>
        </div>
      {/if}

      {#if success}
        <div class="mb-4 p-3 bg-green-500/20 text-green-600 dark:text-green-400 rounded-xl border border-green-500/30">
          <div class="text-4xl mb-2">✅</div>
          <p class="font-semibold">{success}</p>
          <p class="text-sm mt-2">Redirection vers l'espace principal...</p>
        </div>
      {:else}
        <div class="space-y-4">
          <div>
            <label for="new-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
              Nouveau mot de passe
            </label>
            <input
              id="new-password"
              type="password"
              bind:value={password}
              placeholder="Minimum 8 caractères"
              class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
              onkeydown={handleKeyPress}
              disabled={isLoading}
            />
            <p class="text-xs text-[var(--text-secondary)] mt-2 text-left">
              • Au moins 8 caractères<br>
              • Majuscule, minuscule, chiffre et caractère spécial
            </p>
          </div>

          <div>
            <label for="confirm-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
              Confirmer le mot de passe
            </label>
            <input
              id="confirme pas sors"
              type="password"
              bind:value={confirmPassword}
              placeholder="Répétez votre mot de passe"
              class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
              onkeydown={handleKeyPress}
              disabled={isLoading}
            />
          </div>

          <div class="p-3 bg-blue-500/10 border border-blue-500/30 rounded-xl">
            <p class="text-xs text-blue-400 text-left">
              <span class="font-semibold">💡 Conseil de sécurité :</span><br>
              Utilisez un gestionnaire de mots de passe pour générer et stocker un mot de passe fort et unique.
            </p>
          </div>

          <button
            onclick={handleSubmit}
            disabled={isLoading || !password || !confirmPassword}
            class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center justify-center gap-2"
          >
            {#if isLoading}
              <span class="animate-spin">⟳</span>
              <span>Création en cours...</span>
            {:else}
              <span>🔒</span>
              <span>Créer mon mot de passe</span>
            {/if}
          </button>
        </div>
      {/if}

      <div class="mt-8 pt-4 border-t border-white/20">
        <p class="text-sm text-[var(--text-secondary)]">
          Vous avez déjà un mot de passe ? 
          <a href="/login" class="text-[var(--accent)] hover:underline ml-1">
            Se connecter
          </a>
        </p>
      </div>
    </div>
  {:else}
    <div class="text-center">
      <div class="text-6xl mb-4">🔒</div>
      <h2 class="text-xl font-bold mb-2 text-[var(--text-primary)]">Accès refusé</h2>
      <p class="text-[var(--text-secondary)] mb-4">
        Vous devez être connecté pour créer un mot de passe.
      </p>
      <a href="/login" class="inline-block px-4 py-2 bg-[var(--accent)] text-white rounded-lg hover:opacity-90 transition">
        Se connecter
      </a>
    </div>
  {/if}
</div>