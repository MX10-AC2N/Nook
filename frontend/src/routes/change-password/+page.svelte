<script lang="ts">
  import { goto } from '$app/navigation';
  import { authStore, initAuth } from '$lib/authStore';
  import { changePassword } from '$lib/api';  // Ta fonction modernisée dans api.ts

  let newPassword = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);

  async function handleSubmit(event: Event) {
    event.preventDefault();
    error = '';
    success = '';

    if (newPassword !== confirmPassword) {
      error = 'Les nouveaux mots de passe ne correspondent pas.';
      return;
    }

    if (newPassword.length < 8) {
      error = 'Le nouveau mot de passe doit contenir au moins 8 caractères.';
      return;
    }

    isLoading = true;

    try {
      // Pas de current_password → changement pour utilisateur déjà connecté
      const result = await changePassword(newPassword);

      if (result.success) {
        success = 'Votre mot de passe a été changé avec succès !';
        // Recharge l'état auth (needs_password_change passe à false si c'était le cas)
        await initAuth();
        setTimeout(() => {
          goto($authStore.isAdmin ? '/admin' : '/chat');
        }, 2000);
      } else {
        error = result.message || 'Échec du changement de mot de passe.';
      }
    } catch (e: any) {
      error = e.message || 'Une erreur inattendue est survenue.';
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>Changer le mot de passe — Nook</title>
</svelte:head>

<div class="page-container">
  <div class="form-wrapper">
    <div class="header">
      <div class="icon">🔐</div>
      <h1>Changer le mot de passe</h1>
      <p>Créez un nouveau mot de passe sécurisé pour votre compte</p>
    </div>

    {#if error}
      <div class="alert error">
        <span class="alert-icon">⚠️</span>
        <span>{error}</span>
      </div>
    {/if}

    {#if success}
      <div class="alert success">
        <span class="alert-icon">✅</span>
        <span>{success}</span>
      </div>
      <p class="redirect-text">Redirection vers votre espace...</p>
    {:else}
      <form class="password-form" on:submit={handleSubmit}>
        <div class="form-group">
          <label for="new-password">Nouveau mot de passe</label>
          <input
            id="new-password"
            type="password"
            bind:value={newPassword}
            class="input"
            placeholder="Minimum 8 caractères"
            required
            disabled={isLoading}
          />
        </div>

        <div class="form-group">
          <label for="confirm-password">Confirmer le nouveau mot de passe</label>
          <input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            class="input"
            placeholder="Confirmez votre nouveau mot de passe"
            required
            disabled={isLoading}
          />
        </div>

        <div class="form-group">
          <button type="submit" class="submit-btn" disabled={isLoading}>
            {#if isLoading}
              <span class="spinner"></span>
              Changement en cours...
            {:else}
              Changer le mot de passe
            {/if}
          </button>
        </div>
      </form>
    {/if}
  </div>
</div>

<style>
  /* Ton style existant est parfait – je le garde intégralement */
  .page-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    padding: 1.5rem;
    background: linear-gradient(135deg, var(--bg-primary, #f0fdf4) 0%, var(--bg-secondary, #e0f2fe) 100%);
  }

  .form-wrapper {
    width: 100%;
    max-width: 400px;
  }

  .header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .icon {
    font-size: 3.5rem;
    margin-bottom: 1rem;
    animation: bounce 2s ease-in-out infinite;
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }

  h1 {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
    margin: 0 0 0.5rem 0;
  }

  .header p {
    font-size: 0.9rem;
    color: var(--text-secondary, #64748b);
    margin: 0;
  }

  .alert {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: var(--radius-lg, 0.75rem);
    margin-bottom: 1rem;
    font-size: 0.9rem;
    animation: slide-down 0.3s ease;
  }

  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .alert.error {
    background-color: var(--error-light, #fee2e2);
    color: var(--error, #ef4444);
    border: 1px solid var(--error, #ef4444);
  }

  .alert.success {
    background-color: var(--success-light, #dcfce7);
    color: var(--success, #22c55e);
    border: 1px solid var(--success, #22c55e);
  }

  .alert-icon {
    font-size: 1.1rem;
  }

  .redirect-text {
    text-align: center;
    color: var(--text-secondary, #64748b);
    font-size: 0.9rem;
    margin-top: 1rem;
  }

  .password-form {
    background-color: var(--bg-primary, #ffffff);
    padding: 2rem;
    border-radius: var(--radius-xl, 1rem);
    box-shadow: var(--depth, 0 4px 12px rgba(0, 0, 0, 0.1));
    border: 1px solid var(--border, #e2e8f0);
  }

  .form-group {
    margin-bottom: 1.25rem;
  }

  label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary, #1e293b);
    margin-bottom: 0.5rem;
  }

  .input {
    width: 100%;
    padding: 0.75rem 1rem;
    font-size: 0.9rem;
    background-color: var(--input-bg, #ffffff);
    color: var(--text-primary, #1e293b);
    border: 2px solid var(--border, #e2e8f0);
    border-radius: var(--radius-lg, 0.75rem);
    transition: all 0.2s ease;
    outline: none;
  }

  .input:focus {
    border-color: var(--accent, #4ade80);
    box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.2);
  }

  .input::placeholder {
    color: var(--text-secondary, #64748b);
    opacity: 0.7;
  }

  .submit-btn {
    width: 100%;
    padding: 0.875rem 1.5rem;
    font-size: 0.9rem;
    font-weight: 600;
    color: white;
    background: linear-gradient(135deg, var(--accent, #4ade80) 0%, var(--accent-dark, #22c55e) 100%);
    border: none;
    border-radius: var(--radius-lg, 0.75rem);
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .submit-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(74, 222, 128, 0.4);
  }

  .submit-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .submit-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .spinner {
    width: 18px;
    height: 18px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 480px) {
    .page-container {
      padding: 1rem;
    }

    .password-form {
      padding: 1.5rem;
    }

    h1 {
      font-size: 1.5rem;
    }

    .icon {
      font-size: 3rem;
    }
  }
</style>