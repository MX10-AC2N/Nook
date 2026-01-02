<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore, isAuthenticated, needsPasswordChange, initAuth } from '$lib/authStore';
  import { changePassword } from '$lib/api'; // Si tu utilises api.ts pour changePassword

  let newPassword = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let loading = $state(false);
  let success = $state(false);

  // Guard: si pas besoin de changement ou pas authentifié, redirige
  onMount(async () => {
    await initAuth();
    if (!$isAuthenticated) {
      goto('/login');
    } else if (!$needsPasswordChange) {
      goto($authStore.isAdmin ? '/admin' : '/chat');
    }
  });

  async function handleSubmit() {
    error = '';
    if (!newPassword || newPassword !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas ou sont vides';
      return;
    }

    loading = true;

    try {
      // Appelle l'API first-setup avec user_id et new_password
      const userId = $authStore.user?.id;
      if (!userId) throw new Error('Utilisateur non trouvé');

      const result = await changePassword(newPassword, userId); // Utilise ta fonction modernisée

      if (result.success) {
        success = true;
        // Recharge l'état auth (needs_password_change devient false)
        await initAuth();
        setTimeout(() => {
          goto($authStore.isAdmin ? '/admin' : '/chat');
        }, 2000);
      } else {
        error = result.message || 'Erreur lors du changement de mot de passe';
      }
    } catch (err) {
      error = err.message || 'Erreur serveur lors du changement';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Créer un mot de passe — Nook</title>
</svelte:head>

<div class="container">
  <div class="card">
    {#if success}
      <div class="success-message">
        <h1>✅ Succès !</h1>
        <p>Mot de passe mis à jour. Redirection dans quelques secondes...</p>
      </div>
    {:else}
      <h1>Créer votre mot de passe</h1>
      <p class="subtitle">Pour votre première connexion, veuillez définir un nouveau mot de passe sécurisé.</p>

      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="new-password">Nouveau mot de passe</label>
          <input
            type="password"
            id="new-password"
            bind:value={newPassword}
            placeholder="Entrez votre nouveau mot de passe"
            disabled={loading}
            required
          />
        </div>

        <div class="form-group">
          <label for="confirm-password">Confirmer le mot de passe</label>
          <input
            type="password"
            id="confirm-password"
            bind:value={confirmPassword}
            placeholder="Confirmez votre nouveau mot de passe"
            disabled={loading}
            required
          />
        </div>

        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        <button type="submit" class="submit-btn" disabled={loading}>
          {loading ? 'Enregistrement...' : 'Confirmer'}
        </button>
      </form>
    {/if}
  </div>
</div>

<style>
  .container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 1rem;
    background-color: var(--bg-primary, #f0fdf4);
  }

  .card {
    background: white;
    padding: 2rem;
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
    width: 100%;
    max-width: 400px;
    text-align: center;
  }

  h1 {
    font-size: 1.5rem;
    margin-bottom: 0.5rem;
    color: #1e293b;
  }

  .subtitle {
    color: #64748b;
    margin-bottom: 2rem;
  }

  .form-group {
    margin-bottom: 1.25rem;
    text-align: left;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #333;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.2s;
  }

  input:focus {
    outline: none;
    border-color: #2d5a27;
  }

  .error-message {
    background: #ffebee;
    color: #c62828;
    padding: 0.75rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }

  .submit-btn {
    width: 100%;
    padding: 0.875rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }

  .submit-btn:hover:not(:disabled) {
    background: #3d7a37;
  }

  .submit-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .success-message {
    color: #2d5a27;
    padding: 2rem;
  }

  .success-message h1 {
    font-size: 2rem;
    margin-bottom: 1rem;
  }
</style>