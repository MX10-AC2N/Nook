<script lang="ts">
  import { goto } from '$app/navigation';
  import { isAuthenticated, needsPasswordChange } from '$lib/authStore';
  import { onMount } from 'svelte';

  let username = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleLogin() {
    if (!username || !password) {
      error = 'Veuillez remplir tous les champs';
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ username, password })
      });

      if (response.ok) {
        const data = await response.json();
        
        if (data.user?.needs_password_change) {
          goto('/change-password');
        } else {
          goto('/chat');
        }
      } else {
        const data = await response.json();
        error = data.message || 'Identifiants incorrects ou compte en attente d\'approbation';
      }
    } catch (err) {
      error = 'Erreur de connexion au serveur. Vérifiez votre réseau.';
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    if ($isAuthenticated) {
      if ($needsPasswordChange) {
        goto('/change-password');
      } else {
        goto('/chat');
      }
    }
  });
</script>

<svelte:head>
  <title>Connexion - Nook</title>
</svelte:head>

<div class="login-page">
  <div class="login-card">
    <div class="logo">
      <span class="logo-icon">🌱</span>
      <h1>Nook</h1>
    </div>
    
    <p class="subtitle">Bienvenue dans votre espace familial sécurisé</p>

    {#if error}
      <div class="alert error" role="alert">
        <span class="alert-icon">⚠️</span>
        <span>{error}</span>
      </div>
    {/if}

    <form on:submit|preventDefault={handleLogin} class="login-form">
      <div class="input-group">
        <label for="username">Identifiant</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          placeholder="Votre identifiant unique"
          autocomplete="username"
          required
          disabled={loading}
        />
      </div>

      <div class="input-group">
        <label for="password">Mot de passe</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          placeholder="Votre mot de passe"
          autocomplete="current-password"
          required
          disabled={loading}
        />
      </div>

      <button type="submit" class="btn-primary" disabled={loading}>
        {#if loading}
          <span class="spinner"></span>
          Connexion en cours...
        {:else}
          Se connecter
        {/if}
      </button>
    </form>

    <div class="actions">
      <a href="/register" class="action-link primary">
        ➕ Créer un compte (attente approbation admin)
      </a>
      
      <a href="/help" class="action-link subtle">
        Besoin d'aide ?
      </a>
    </div>
  </div>
</div>

<style>
  * { box-sizing: border-box; }  /* ← Fix global overflow */

  .login-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
  }

  .login-card {
    background: white;
    padding: 2rem;
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
    width: 100%;
    max-width: 400px;
    text-align: center;
  }

  .logo {
    margin-bottom: 1.5rem;
  }

  .logo-icon {
    font-size: 3.5rem;
    display: block;
    margin-bottom: 0.5rem;
  }

  h1 {
    font-size: 2rem;
    margin: 0;
    color: #1e293b;
  }

  .subtitle {
    color: #64748b;
    margin-bottom: 2rem;
    font-size: 1rem;
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

  .alert-icon {
    font-size: 1.25rem;
  }

  .login-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    margin-bottom: 2rem;
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
    padding: 0.75rem;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.2s;
  }

  input:focus {
    border-color: #2d5a27;
    box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2);
  }

  input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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

  .actions {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .action-link {
    font-size: 0.95rem;
    text-decoration: none;
    transition: opacity 0.2s;
  }

  .action-link:hover {
    opacity: 0.8;
  }

  .action-link.primary {
    color: #2d5a27;
    font-weight: 600;
  }

  .action-link.subtle {
    color: #64748b;
    font-size: 0.85rem;
  }

   @media (max-width: 380px) {
    .login-card {
      padding: 1.5rem;
    }
    input {
      padding: 0.65rem;
    }
  }
</style>