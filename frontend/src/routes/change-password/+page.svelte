<script lang="ts">
  import { goto } from '$app/navigation';
  import { authStore, needsPasswordChange } from '$lib/authStore';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store'; // ⬅️ AJOUTER CET IMPORT

  let newPassword = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);

  onMount(() => {
    // Utiliser get(authStore) au lieu de authStore.get()
    const store = get(authStore);
    if (!store.isAuthenticated) {
      goto('/login');
    }
    if (store.isAuthenticated && !store.needsPasswordChange) {
      goto('/chat');
    }
  });

  async function handleSubmit(event: Event) {
    event.preventDefault();
    error = '';
    success = '';

    if (newPassword !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas.';
      return;
    }

    if (newPassword.length < 8) {
      error = 'Le mot de passe doit contenir au moins 8 caractères.';
      return;
    }

    isLoading = true;

    try {
      // Utiliser get(authStore) au lieu de authStore.get()
      const store = get(authStore);
      const userId = store.user?.id;
      
      if (!userId) throw new Error('Utilisateur non identifié');

      const response = await fetch('/api/auth/change-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ 
          new_password: newPassword,
          user_id: userId 
        })
      });

      const result = await response.json();

      if (response.ok && result.success) {
        success = 'Votre mot de passe a été mis à jour avec succès !';

        // IMPORTANT: Vérifier que la session est toujours valide
      await verifySessionAndRedirect(userId);
    } else {
      error = result.message || 'Échec du changement de mot de passe.';
    }
  } catch (e: any) {
    error = e.message || 'Une erreur est survenue.';
  } finally {
    isLoading = false;
  }
}

// Nouvelle fonction pour vérifier la session après changement de mot de passe
async function verifySessionAndRedirect(userId: string) {
  try {
    // Petite pause pour laisser le cookie se mettre à jour
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Vérifier la session
    const meResponse = await fetch('/api/auth/me', { credentials: 'include' });
    
    if (meResponse.ok) {
      const meData = await meResponse.json();
      
      if (meData.authenticated && meData.user) {
        
        // Mettre à jour le store
        authStore.setAuthenticated(meData.user, meData.user.role === 'admin');
        
        // Rediriger selon le rôle
        setTimeout(() => {
          goto(meData.user.role === 'admin' ? '/admin' : '/chat');
        }, 2000);
        return;
      }
    }
    
    // Si la vérification échoue, proposer de se reconnecter
    error = 'Session expirée. Veuillez vous reconnecter.';
    setTimeout(() => goto('/login'), 3000);
    
  } catch (err) {
    console.error('Erreur vérification session:', err);
    error = 'Erreur de session. Veuillez vous reconnecter.';
    setTimeout(() => goto('/login'), 3000);
  }
}
</script>

<svelte:head>
  <title>
    {$needsPasswordChange ? 'Définir' : 'Changer'} votre mot de passe — Nook
  </title>
</svelte:head>

<div class="page-container">
  <div class="card">
    <div class="header">
      <div class="icon">🔐</div>
      <h1>
        {$needsPasswordChange ? 'Première connexion' : 'Changer le mot de passe'}
      </h1>
      <p class="description">
        {$needsPasswordChange
          ? 'Pour des raisons de sécurité, vous devez définir un nouveau mot de passe avant de continuer.'
          : 'Choisissez un mot de passe fort et unique pour protéger votre compte.'}
      </p>
    </div>

    {#if error}
      <div class="alert error" role="alert">
        <span class="alert-icon">⚠️</span>
        <span>{error}</span>
      </div>
    {/if}

    {#if success}
      <div class="alert success" role="alert">
        <span class="alert-icon">✅</span>
        <span>{success}</span>
      </div>
      <p class="info-text">Redirection en cours...</p>
    {:else}
      <form class="form" on:submit={handleSubmit}>
        <div class="input-group">
          <label for="new-password">Nouveau mot de passe</label>
          <input
            id="new-password"
            type="password"
            bind:value={newPassword}
            placeholder="Au moins 8 caractères"
            required
            disabled={isLoading}
            autocomplete="new-password"
          />
          <p class="help-text">Utilisez lettres, chiffres et symboles pour plus de sécurité</p>
        </div>

        <div class="input-group">
          <label for="confirm-password">Confirmer le mot de passe</label>
          <input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            placeholder="Répétez le mot de passe"
            required
            disabled={isLoading}
            autocomplete="new-password"
          />
        </div>

        <button type="submit" class="btn-primary" disabled={isLoading}>
          {#if isLoading}
            <span class="spinner"></span>
            Enregistrement...
          {:else}
            {$needsPasswordChange ? 'Définir le mot de passe' : 'Changer le mot de passe'}
          {/if}
        </button>
      </form>
    {/if}

    <div class="footer">
      <a href="/login" class="back-link">
        ← Retour à la connexion
      </a>
    </div>
  </div>
</div>

<style>
  .page-container {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
  }

  .card {
    background: white;
    padding: 2.5rem;
    border-radius: 1.5rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
    width: 100%;
    max-width: 420px;
    text-align: center;
  }

  .header {
    margin-bottom: 2rem;
  }

  .icon {
    font-size: 3.5rem;
    margin-bottom: 1rem;
  }

  h1 {
    font-size: 1.75rem;
    font-weight: 700;
    color: #1e293b;
    margin: 0 0 0.75rem 0;
  }

  .description {
    font-size: 0.95rem;
    color: #64748b;
    line-height: 1.5;
    margin: 0;
  }

  .alert {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 0.75rem;
    margin-bottom: 1.5rem;
    text-align: left;
    font-size: 0.9rem;
  }

  .alert.error {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #dc2626;
  }

  .alert.success {
    background: rgba(74, 222, 128, 0.1);
    border: 1px solid rgba(74, 222, 128, 0.3);
    color: #22c55e;
  }

  .alert-icon {
    font-size: 1.25rem;
  }

  .info-text {
    text-align: center;
    color: #64748b;
    font-size: 0.9rem;
    margin-top: 1rem;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .input-group {
    text-align: left;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #374151;
    font-size: 0.95rem;
  }

  input {
    width: 100%;
    padding: 0.875rem 1rem;
    font-size: 1rem;
    background: #f8fafc;
    border: 2px solid #e2e8f0;
    border-radius: 0.75rem;
    transition: all 0.2s;
    outline: none;
  }

  input:focus {
    border-color: #2d5a27;
    box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2);
  }

  input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .help-text {
    font-size: 0.8rem;
    color: #64748b;
    margin: 0.5rem 0 0 0;
  }

  .btn-primary {
    width: 100%;
    padding: 1rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.75rem;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
  }

  .btn-primary:hover:not(:disabled) {
    background: #3d7a37;
    transform: translateY(-1px);
  }

  .btn-primary:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .footer {
    margin-top: 2rem;
    padding-top: 1rem;
    border-top: 1px solid #e2e8f0;
  }

  .back-link {
    color: #64748b;
    font-size: 0.9rem;
    text-decoration: none;
    transition: color 0.2s;
  }

  .back-link:hover {
    color: #2d5a27;
  }

  @media (max-width: 480px) {
    .card {
      padding: 2rem 1.5rem;
    }
    
    .icon {
      font-size: 3rem;
    }
    
    h1 {
      font-size: 1.5rem;
    }
  }
</style>