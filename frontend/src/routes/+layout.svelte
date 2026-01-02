<script lang="ts">
  import { page } from '$app/stores';
  import { authStore, isAuthenticated, isAdmin, needsPasswordChange, initAuth } from '$lib/authStore';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  let { children } = $props();
  let showMenu = $state(false);
  let appError = $state<string | null>(null);
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

  function closeMenu() {
    showMenu = false;
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeMenu();
    }
  }

  async function handleLogout() {
    await fetch('/api/logout', { method: 'POST', credentials: 'include' });
    closeMenu();
    goto('/login');
  }

  // Redirections globales intelligentes
  $effect(() => {
    if (loading) return;

    const pathname = $page.url.pathname;

    // Si needs_password_change → forcer le changement (premier login admin)
    if ($needsPasswordChange && pathname !== '/create-password') {
      goto('/create-password');
      return;
    }

    // Si authentifié → rediriger hors des pages publiques si nécessaire
    if ($isAuthenticated) {
      if (pathname === '/' || pathname === '/login' || pathname === '/register') {
        if ($isAdmin) {
          goto('/admin');
        } else {
          goto('/chat');
        }
      }
    } else {
      // Pas authentifié → rediriger vers login (sauf pages publiques)
      const publicPaths = ['/login', '/register', '/help', '/join'];
      if (!publicPaths.some(p => pathname.startsWith(p))) {
        goto('/login');
      }
    }
  });

  onMount(async () => {
    try {
      await initAuth(); // Recharge l'état auth au montage du layout
    } catch (err) {
      console.error('Erreur init auth dans layout:', err);
      appError = 'Impossible de vérifier votre session. Réessayez.';
    } finally {
      setTimeout(() => {
        loading = false;
      }, 500);
    }
  });
</script>

{#if loading}
  <div class="loading-screen">
    <div class="loading-spinner"></div>
    <p>Chargement de Nook...</p>
  </div>
{:else if appError}
  <div class="error-screen">
    <div class="error-content">
      <h1>❌ Erreur système</h1>
      <p class="error-title">================</p>
      <p class="error-message">{appError}</p>
      <button onclick={() => window.location.reload()} class="retry-button">
        🔄 Recharger l'application
      </button>
    </div>
  </div>
{:else}
  <header class="app-header">
    <button onclick={toggleMenu} class="menu-toggle" aria-label="Ouvrir le menu de navigation">
      ☰
    </button>

    <h1>🌱 Nook</h1>

    {#if $isAuthenticated}
      <span class="user-name">{$authStore.user?.name || $authStore.user?.username}</span>
      <button onclick={handleLogout} class="logout-btn" aria-label="Déconnexion">
        🔌
      </button>
    {/if}
  </header>

  {#if showMenu}
    <button 
      class="menu-overlay" 
      onclick={closeMenu}
      aria-label="Fermer le menu"
    ></button>
    <nav 
      class="menu" 
      role="dialog" 
      aria-label="Menu de navigation"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="menu-header">
        <h2>Menu Nook</h2>
        <button onclick={closeMenu} class="close-menu" aria-label="Fermer le menu">
          ✕
        </button>
      </div>

      <ul class="nav-list">
        {#each navItems as item}
          {#if item.requiresAuth && !$isAuthenticated}
            <!-- Skip -->
          {:else if item.requiresAdmin && !$isAdmin}
            <!-- Skip -->
          {:else}
            <li>
              <a href={item.path} onclick={closeMenu}>{item.label}</a>
            </li>
          {/if}
        {/each}
      </ul>

      <div class="menu-footer">
        <p class="version">Version 3.0 • SvelteKit</p>
        {#if $isAuthenticated}
          <button onclick={handleLogout} class="logout-link" aria-label="Déconnexion">
            🔌 Déconnexion
          </button>
        {/if}
      </div>
    </nav>
  {/if}

  <main class="app-main">
    {@render children()}
  </main>

  <footer class="app-footer">
    <p>© {new Date().getFullYear()} Nook • Messagerie privée pour la famille</p>
  </footer>
{/if}

<style>
  /* Ton style existant reste inchangé – je le garde tel quel */
  :global(body) {
    margin: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: linear-gradient(135deg, #f5f7fa 0%, #e4e8ec 100%);
    min-height: 100vh;
  }

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
    border: 4px solid #e2e8f0;
    border-top-color: #4ade80;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 1.5rem;
  }

  .error-content {
    background: white;
    padding: 2.5rem;
    border-radius: 1rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    text-align: center;
    max-width: 400px;
  }

  .error-content h1 {
    font-size: 1.5rem;
    margin: 0 0 0.5rem 0;
    color: #1e293b;
  }

  .error-title {
    color: #64748b;
    margin: 0 0 1.25rem 0;
  }

  .error-message {
    color: #dc2626;
    margin: 0 0 1.5rem 0;
    line-height: 1.5;
  }

  .retry-button {
    padding: 0.75rem 1.5rem;
    background: #4ade80;
    color: white;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .retry-button:hover {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .app-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.5rem;
    background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .menu-toggle,
  .logout-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 0.5rem;
    transition: background 0.2s;
  }

  .menu-toggle:hover,
  .logout-btn:hover {
    background: #f1f5f9;
  }

  .app-header h1 {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0;
    color: #1e293b;
    flex: 1;
  }

  .user-name {
    font-size: 0.9rem;
    color: #64748b;
    margin-right: 0.5rem;
  }

  .menu-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 200;
    cursor: pointer;
    border: none;
    padding: 0;
    margin: 0;
  }

  .menu {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 300px;
    max-width: 85vw;
    background: white;
    z-index: 201;
    box-shadow: -4px 0 20px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    animation: slideIn 0.25s ease-out;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }

  .menu-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid #e2e8f0;
  }

  .menu-header h2 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0;
    color: #1e293b;
  }

  .close-menu {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 0.5rem;
    color: #64748b;
    transition: all 0.2s;
  }

  .close-menu:hover {
    background: #f1f5f9;
    color: #1e293b;
  }

  .nav-list {
    list-style: none;
    margin: 0;
    padding: 1rem 0;
    flex: 1;
    overflow-y: auto;
  }

  .nav-list li a {
    display: block;
    padding: 0.85rem 1.5rem;
    color: #334155;
    text-decoration: none;
    transition: all 0.2s;
    font-size: 1rem;
  }

  .nav-list li a:hover {
    background: #f1f5f9;
    color: #1e293b;
  }

  .menu-footer {
    padding: 1.25rem 1.5rem;
    border-top: 1px solid #e2e8f0;
  }

  .version {
    font-size: 0.8rem;
    color: #94a3b8;
    margin: 0 0 0.75rem 0;
  }

  .logout-link {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.75rem 1rem;
    background: none;
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    color: #64748b;
    font-size: 0.95rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .logout-link:hover {
    background: #fef2f2;
    border-color: #fecaca;
    color: #dc2626;
  }

  .app-main {
    min-height: calc(100vh - 140px);
    padding: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .app-footer {
    text-align: center;
    padding: 1.25rem;
    color: #64748b;
    font-size: 0.85rem;
    border-top: 1px solid #e2e8f0;
    background: white;
  }

  .app-footer p {
    margin: 0;
  }

  @media (max-width: 640px) {
    .app-header {
      padding: 0.85rem 1rem;
    }

    .app-header h1 {
      font-size: 1.1rem;
    }

    .app-main {
      padding: 1rem;
    }

    .menu {
      width: 100%;
      max-width: none;
    }
  }
</style>