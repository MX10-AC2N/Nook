<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount } from 'svelte';
  import { goto } from '@roxi/routify';
  import { authStore } from '$lib/authStore';
  
  // États
  import { $state } from 'svelte';
  let loading = $state(true);
  let error = $state(null);

  // Rediriger selon l'état d'authentification
  async function handleRedirect() {
    try {
      loading = true;
      
      // Attendre que l'auth soit initialisée
      await new Promise(resolve => setTimeout(resolve, 300));
      
      const isAuthenticated = $authStore.isAuthenticated;
      const isAdmin = $authStore.isAdmin;
      
      if (isAuthenticated) {
        if (isAdmin) {
          goto('/admin');
        } else {
          goto('/chat');
        }
      } else {
        goto('/login');
      }
      
      loading = false;
    } catch (err) {
      error = err.message || 'Erreur de redirection';
      loading = false;
      console.error('Erreur redirection:', err);
    }
  }

  onMount(() => {
    handleRedirect();
  });

  // Gestion des erreurs
  function handleError(err) {
    error = err.message || 'Erreur système';
    setTimeout(() => {
      error = null;
    }, 5000);
  }
</script>

<svelte:head>
  <title>Nook - Messagerie privée pour la famille</title>
  <meta name="description" content="Nook - Une messagerie sécurisée, open source et auto-hébergée pour votre famille et vos amis proches">
</svelte:head>

{#if loading}
  <div class="redirect-loading">
    <div class="spinner"></div>
    <p>Redirection...</p>
  </div>
{:else if error}
  <div class="redirect-error">
    <h2>❌ Erreur de redirection</h2>
    <p>{$error}</p>
    <div class="error-actions">
      <button onclick={() => handleRedirect()} class="retry-button">
        🔄 Réessayer
      </button>
      <button onclick={() => goto('/login')} class="login-button">
        👤 Aller à la connexion
      </button>
    </div>
  </div>
{:else}
  <div class="redirect-placeholder">
    <p>Redirection en cours...</p>
  </div>
{/if}

<style>
  .redirect-loading, .redirect-error, .redirect-placeholder {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    text-align: center;
    padding: 2rem;
    background: var(--bg-color);
    color: var(--text-color);
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 5px solid rgba(0,0,0,0.1);
    border-radius: 50%;
    border-top-color: var(--primary);
    animation: spin 1s linear infinite;
    margin-bottom: 1.5rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .redirect-loading p {
    font-size: 1.2rem;
    color: var(--primary);
    font-weight: 500;
  }

  .redirect-error h2 {
    color: #f44336;
    margin-bottom: 1rem;
    font-size: 1.8rem;
  }

  .redirect-error p {
    color: #666;
    margin-bottom: 2rem;
    font-size: 1.1rem;
  }

  .error-actions {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .retry-button, .login-button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 12px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .retry-button {
    background: #4caf50;
    color: white;
  }

  .login-button {
    background: #2196f3;
    color: white;
  }

  .retry-button:hover {
    background: #43a047;
    transform: translateY(-1px);
  }

  .login-button:hover {
    background: #1976d2;
    transform: translateY(-1px);
  }

  .redirect-placeholder p {
    color: var(--text-secondary);
    font-style: italic;
  }

  /* Thèmes */
  .theme-jardin-secret {
    --primary: #4CAF50;
  }

  .theme-space-hub {
    --primary: #2196F3;
  }

  .theme-maison-chaleureuse {
    --primary: #FF9800;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .error-actions {
      flex-direction: column;
      gap: 0.75rem;
    }
    
    .retry-button, .login-button {
      width: 100%;
    }
  }
</style>