<script lang="ts">
  import { page } from '$app/stores';
  import { authStore, isAuthenticated, isAdmin, authLoading } from '$lib/authStore';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  let { children } = $props();

  let showMenu = $state(false);
  let appError = $state(null);
  let loading = $state(true);

  const navItems = [
    { path: '/chat', label: '💬 Chat', requiresAuth: true },
    { path: '/calendar', label: '📅 Calendrier', requiresAuth: true },
    { path: '/admin', label: '👑 Administration', requiresAuth: true, requiresAdmin: true },
    { path: '/settings', label: '⚙️ Paramètres', requiresAuth: true },
    { path: '/help', label: '❓ Aide', requiresAuth: false }
  ];

  function toggleMenu() {
    showMenu = !showMenu;
  }

  async function handleLogout() {
    await fetch('/api/logout', { method: 'POST', credentials: 'include' });
    showMenu = false;
    goto('/login');
  }

  $effect(() => {
    if (!loading && !$isAuthenticated && $page.url.pathname !== '/login' && !$page.url.pathname.startsWith('/join')) {
      goto('/login');
    }
  });

  onMount(() => {
    setTimeout(() => {
      loading = false;
    }, 500);
  });
</script>

<main>
  <div class="main-content">
    {#if loading}
      <div class="loading-screen">
        <div class="loading-spinner"></div>
        <p>Chargement de Nook...</p>
      </div>
    {:else if appError}
      <div class="error-screen">
        <h1>❌ Erreur système</h1>
        <p>{appError}</p>
        <button onclick={() => window.location.reload()} class="retry-button">
          🔄 Recharger l'application
        </button>
      </div>
    {:else}
      <header>
        <button onclick={toggleMenu} class="menu-toggle">☰</button>
        <h1>🌱 Nook</h1>
        {#if $isAuthenticated}
          <span class="user-name">{$authStore.user?.name}</span>
          <button onclick={handleLogout} class="logout-btn">🔌</button>
        {/if}
      </header>

      {#if showMenu}
        <aside class="menu-overlay" onclick={toggleMenu} role="button" tabindex="0" onkeydown={(e) => e.key === 'Escape' && toggleMenu()}>
          <aside class="menu" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="Menu de navigation">
            <div class="menu-header">
              <h2>Menu Nook</h2>
              <button onclick={toggleMenu} class="close-btn">✕</button>
            </div>
            <nav>
              {#each navItems as item}
                {#if item.requiresAuth && !$isAuthenticated}
                  <!-- Masqué si auth requise et non authentifié -->
                {:else if item.requiresAdmin && !$isAdmin}
                  <!-- Masqué si admin requis et non admin -->
                {:else}
                  <a href={item.path} onclick={toggleMenu}>{item.label}</a>
                {/if}
              {/each}
            </nav>
            <footer>
              <p class="version">Version 3.0 • SvelteKit</p>
              <button onclick={handleLogout} class="logout-button">🔌 Déconnexion</button>
            </footer>
          </aside>
        </aside>
      {/if}

      <div class="content">
        {@render children()}
      </div>

      <footer>
        <p>© {new Date().getFullYear()} Nook • Messagerie privée pour la famille</p>
      </footer>
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: var(--font-primary, Arial, sans-serif);
    background-color: var(--bg-primary, #f0f2f5);
    color: var(--text-primary, #333);
  }

  main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    text-align: center;
  }

  .main-content {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  /* Header */
  header {
    background: linear-gradient(135deg, var(--accent, #4CAF50), var(--accent-dark, #2E7D32));
    color: white;
    padding: 1rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-shadow: var(--shadow-md);
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .menu-toggle {
    background: none;
    border: none;
    color: white;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: var(--radius-md);
    transition: background-color 0.2s;
  }

  .menu-toggle:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }

  header h1 {
    margin: 0;
    font-size: 1.8rem;
    flex-grow: 1;
  }

  .user-name {
    margin-right: 1rem;
    font-weight: bold;
  }

  .logout-btn {
    background: none;
    border: none;
    color: white;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: var(--radius-md);
    transition: background-color 0.2s;
  }

  .logout-btn:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }

  /* Menu overlay */
  .menu-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 1000;
    animation: fade-in 0.2s ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .menu {
    position: fixed;
    top: 0;
    left: 0;
    width: 280px;
    height: 100%;
    background: linear-gradient(180deg, var(--bg-secondary, #333) 0%, var(--bg-primary, #1a1a1a) 100%);
    color: white;
    padding: 1.5rem;
    box-shadow: var(--shadow-xl);
    display: flex;
    flex-direction: column;
    animation: slide-in 0.3s ease;
  }

  @keyframes slide-in {
    from { transform: translateX(-100%); }
    to { transform: translateX(0); }
  }

  .menu-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
    margin-bottom: 1rem;
  }

  .menu-header h2 {
    margin: 0;
    font-size: 1.5rem;
  }

  .close-btn {
    background: none;
    border: none;
    color: white;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: var(--radius-md);
    transition: background-color 0.2s;
  }

  .close-btn:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }

  .menu nav {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }

  .menu a {
    color: white;
    text-decoration: none;
    padding: 0.8rem 1rem;
    width: 100%;
    text-align: left;
    border-radius: var(--radius-md);
    transition: all 0.2s;
    margin-bottom: 0.25rem;
  }

  .menu a:hover {
    background-color: rgba(255, 255, 255, 0.15);
    transform: translateX(5px);
  }

  .menu footer {
    margin-top: auto;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
  }

  .version {
    font-size: 0.85rem;
    opacity: 0.7;
    margin-bottom: 1rem;
  }

  .logout-button {
    background-color: var(--error, #f44336);
    color: white;
    border: none;
    padding: 0.8rem 1.5rem;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: 1rem;
    width: 100%;
    transition: background-color 0.2s;
  }

  .logout-button:hover {
    background-color: #d32f2f;
  }

  /* Content */
  .content {
    flex-grow: 1;
    padding: 1rem;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
  }

  /* Footer */
  footer {
    background: linear-gradient(135deg, var(--bg-secondary, #333) 0%, var(--bg-primary, #1a1a1a) 100%);
    color: white;
    padding: 1.5rem;
    margin-top: auto;
  }

  footer p {
    margin: 0;
    opacity: 0.8;
  }

  /* Loading screen */
  .loading-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    gap: 1rem;
  }

  .loading-spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--border, #e2e8f0);
    border-top-color: var(--accent, #4ade80);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Error screen */
  .error-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    gap: 1rem;
    padding: 2rem;
  }

  .error-screen h1 {
    color: var(--error, #ef4444);
    margin: 0;
  }

  .retry-button {
    background-color: var(--accent, #4ade80);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: 1rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .retry-button:hover {
    background-color: var(--button-hover, #22c55e);
    transform: translateY(-2px);
  }

  /* Responsive */
  @media (max-width: 480px) {
    header {
      padding: 0.75rem;
    }

    header h1 {
      font-size: 1.4rem;
    }

    .menu {
      width: 85%;
    }

    .content {
      padding: 0.75rem;
    }
  }
</style>
