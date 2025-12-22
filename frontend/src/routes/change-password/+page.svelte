<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount } from 'svelte';
  import { goto } from '@roxi/routify';  // ← Routify au lieu de $app/navigation (SvelteKit)

  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);

  onMount(async () => {
    // Vérifier si connecté
    try {
      const res = await fetch('/api/validate-session', {
        credentials: 'include'
      });
      
      if (!res.ok) {
        goto('/login');
      }
    } catch (err) {
      goto('/login');
    }
  });

  async function handleSubmit() {
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
    
    isLoading = true;
    error = '';

    try {
      const res = await fetch('/api/change-password', {
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
      } else {
        error = 'Mot de passe actuel incorrect';
      }
    } catch (err) {
      error = 'Erreur réseau';
    } finally {
      isLoading = false;
    }
  }
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
      Vous devez changer votre mot de passe pour continuer
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
        <p class="text-sm mt-2">Redirection vers l'espace principal...</p>
      </div>
    {:else}
      <div class="space-y-4">
        <div>
          <label for="current-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Mot de passe actuel
          </label>
          <input
            id="current-password"
            type="password"
            bind:value={currentPassword}
            placeholder="Entrez votre mot de passe actuel"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)]"
          />
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
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)]"
          />
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
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)]"
          />
        </div>

        <button
          onclick={handleSubmit}
          disabled={isLoading}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 transition"
        >
          {isLoading ? 'Changement en cours...' : 'Changer le mot de passe'}
        </button>
      </div>
    {/if}
  </div>
</div>