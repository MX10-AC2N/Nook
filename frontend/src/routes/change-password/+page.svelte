<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);
  let needsChange = $state(false);

  onMount(async () => {
    // Vérifier si le changement est nécessaire
    try {
      const res = await fetch('/api/member/check-password-change', {
        credentials: 'include'
      });
      
      if (res.ok) {
        const data = await res.json();
        needsChange = data.needs_password_change;
        
        if (!needsChange) {
          // Rediriger vers le chat si pas besoin de changement
          goto('/chat');
        }
      } else if (res.status === 401) {
        // Non connecté, rediriger vers login
        goto('/login');
      }
    } catch (err) {
      console.error('Erreur vérification:', err);
      goto('/login');
    }
  });

  async function handleSubmit() {
    // Validation
    if (!currentPassword || !newPassword || !confirmPassword) {
      error = 'Veuillez remplir tous les champs';
      return;
    }
    
    if (newPassword.length < 8) {
      error = 'Le nouveau mot de passe doit contenir au moins 8 caractères';
      return;
    }
    
    if (newPassword !== confirmPassword) {
      error = 'Les nouveaux mots de passe ne correspondent pas';
      return;
    }
    
    // Vérifier la complexité
    const hasUpperCase = /[A-Z]/.test(newPassword);
    const hasLowerCase = /[a-z]/.test(newPassword);
    const hasNumbers = /\d/.test(newPassword);
    const hasSpecialChar = /[!@#$%^&*(),.?":{}|<>]/.test(newPassword);
    
    if (!hasUpperCase || !hasLowerCase || !hasNumbers) {
      error = 'Le mot de passe doit contenir au moins une majuscule, une minuscule et un chiffre';
      return;
    }

    isLoading = true;
    error = '';

    try {
      const res = await fetch('/api/member/change-temporary-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          current_password: currentPassword,
          new_password: newPassword
        }),
        credentials: 'include'
      });
      
      if (res.ok) {
        const data = await res.json();
        success = 'Mot de passe changé avec succès !';
        
        // Rediriger après 2 secondes
        setTimeout(() => {
          goto('/chat');
        }, 2000);
      } else if (res.status === 401) {
        error = 'Mot de passe actuel incorrect';
      } else {
        error = 'Erreur lors du changement de mot de passe';
      }
    } catch (err) {
      console.error('Erreur:', err);
      error = 'Erreur réseau';
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
  <title>Changer le mot de passe — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4">
  <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">
    
    <div class="text-6xl mb-6">
      🔐
    </div>

    <h1 class="text-3xl font-bold mb-2 text-[var(--text-primary)]">Première connexion</h1>
    
    <p class="text-[var(--text-secondary)] mb-8">
      Vous devez changer votre mot de passe temporaire pour continuer
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
    {:else if needsChange}
      <div class="space-y-4">
        <div>
          <label for="current-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Mot de passe temporaire
          </label>
          <input
            id="current-password"
            type="password"
            bind:value={currentPassword}
            placeholder="Entrez le mot de passe temporaire"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
            disabled={isLoading}
          />
          <p class="text-xs text-[var(--text-secondary)] mt-1 text-left">
            Utilisez le mot de passe fourni par l'administrateur
          </p>
        </div>

        <div>
          <label for="new-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Nouveau mot de passe
          </label>
          <input
            id="new-password"
            type="password"
            bind:value={newPassword}
            placeholder="Minimum 8 caractères"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
            disabled={isLoading}
          />
          <p class="text-xs text-[var(--text-secondary)] mt-2 text-left">
            • Au moins 8 caractères<br>
            • Majuscule, minuscule et chiffre requis
          </p>
        </div>

        <div>
          <label for="confirm-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Confirmer le nouveau mot de passe
          </label>
          <input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            placeholder="Répétez le nouveau mot de passe"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
            disabled={isLoading}
          />
        </div>

        <button
          onclick={handleSubmit}
          disabled={isLoading || !currentPassword || !newPassword || !confirmPassword}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center justify-center gap-2"
        >
          {#if isLoading}
            <span class="animate-spin">⟳</span>
            <span>Changement en cours...</span>
          {:else}
            <span>🔒</span>
            <span>Changer le mot de passe</span>
          {/if}
        </button>
      </div>
    {:else}
      <div class="text-center py-4">
        <div class="text-4xl mb-2">⏳</div>
        <p class="text-[var(--text-secondary)]">Vérification en cours...</p>
      </div>
    {/if}
  </div>
</div>