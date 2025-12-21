<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { page } from '@roxi/routify';
  import { onMount } from 'svelte';
  import { goto } from '@roxi/routify';
  import { authStore } from '$lib/authStore';
  import { currentTheme } from '$lib/themeStore';
  
  // États (runes mode obligatoire)
  let loading = $state(true);
  let error = $state(null);
  let previousPath = $state('');

  // Vérifier l'authentification et rediriger si nécessaire
  function checkAuth() {
    const currentPath = $page.path;
    const isAuthenticated = $authStore.isAuthenticated;
    const isAdmin = $authStore.isAdmin;
    
    // Sauvegarder le chemin précédent pour le retour après login
    if (currentPath !== previousPath) {
      previousPath = currentPath;
    }
    
    // Pages publiques accessibles sans authentification
    const publicPaths = [
      '/login',
      '/join',
      '/create-password',
      '/change-password'
    ];
    
    // Pages admin accessibles uniquement aux admins
    const adminPaths = ['/admin'];
    
    // Rediriger si non authentifié et sur une page privée
    if (!isAuthenticated && !publicPaths.includes(currentPath)) {
      goto('/login');
      return;
    }
    
    // Rediriger si non admin et sur une page admin
    if (isAuthenticated && adminPaths.includes(currentPath) && !isAdmin) {
      goto('/chat');
      return;
    }
    
    // Rediriger si authentifié vers la page d'accueil
    if (isAuthenticated && publicPaths.includes(currentPath)) {
      goto('/chat');
      return;
    }
  }

  // Initialiser au montage
  onMount(() => {
    try {
      checkAuth();
      loading = false;
    } catch (err) {
      error = err.message || 'Erreur de chargement';
      loading = false;
      console.error('Erreur layout:', err);
    }
  });

  // Réagir aux changements de route (remplace la déclaration $: legacy)
  $effect(() => {
    if ($page.path) {
      checkAuth();
    }
  });

  // Gestion des erreurs
  function handleError(err) {
    error = err.message || 'Erreur système';
    setTimeout(() => {
      error = null;
    }, 5000);
  }
</script>

{#if loading}
  <div class="layout-loading">
    <div class="spinner"></div>
    <p>Chargement...</p>
  </div>
{:else if error}
  <div class="layout-error">
    <p>{error}</p>
    <button onclick={() => window.location.reload()} class="retry-button">
      🔄 Recharger
    </button>
  </div>
{:else}
  <div class="layout-container theme-{$currentTheme}">
    {#if $page.path.startsWith('/admin') && !$authStore.isAdmin}
      <div class="admin-warning">
        <p>⚠️ Accès administrateur requis</p>
        <button onclick={() => goto('/chat')} class="back-button">
          Retour au chat
        </button>
      </div>
    {/if}

    <slot />
  </div>
{/if}

<style>
  .layout-loading, .layout-error {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    text-align: center;
    padding: 2rem;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(0,0,0,0.1);
    border-radius: 50%;
    border-top-color: var(--primary);
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .layout-error p {
    color: #f44336;
    font-size: 1.2rem;
    margin-bottom: 1rem;
  }

  .retry-button {
    background: #4caf50;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .retry-button:hover {
    background: #43a047;
    transform: translateY(-1px);
  }

  .layout-container {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .admin-warning {
    background: #fff8e1;
    color: #ff8f00;
    padding: 1rem;
    text-align: center;
    border-bottom: 1px solid #ffd54f;
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .back-button {
    background: #ff8f00;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    margin-top: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-button:hover {
    background: #ff6f00;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .layout-container {
      padding: 0.5rem;
    }
    
    .admin-warning {
      padding: 0.75rem;
      font-size: 0.9rem;
    }
  }
</style>