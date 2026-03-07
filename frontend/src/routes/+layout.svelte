<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { authStore } from '$lib/authStore.svelte.js';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { initCryptoSystem } from '$lib/crypto';
  import { sodiumState, waitForSodium } from '$lib/sodium.svelte.js';

  let { children } = $props();
  let showMenu        = $state(false);
  let appError        = $state<string | null>(null);
  let loading         = $state(true);
  let cryptoInitialized = $state(false);
  let cryptoError     = $state<string | null>(null);
  let menuElement     = $state<HTMLElement | undefined>(undefined);

  const navItems = [
    { path: '/chat',      label: '💬 Chat',           requiresAuth: true  },
    { path: '/chess',     label: '♟️ Échecs',          requiresAuth: true  },
    { path: '/calendar',  label: '📅 Calendrier',      requiresAuth: true  },
    { path: '/polls',     label: '📊 Sondages',        requiresAuth: true  },
    { path: '/admin',     label: '👑 Administration',   requiresAuth: true, requiresAdmin: true },
    { path: '/settings',  label: '⚙️ Paramètres',      requiresAuth: true  },
    { path: '/help',      label: '❓ Aide',             requiresAuth: false },
  ];

  function toggleMenu() {
    showMenu = !showMenu;
    if (showMenu && menuElement) menuElement.focus();
  }

  function closeMenu() { showMenu = false; }

  function handleMenuKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') closeMenu();
  }

  async function handleLogout() {
    await fetch('/api/auth/logout', { method: 'POST', credentials: 'include' });
    authStore.logout();
    closeMenu();
    goto('/login');
  }

  // Guard sur `loading` UNIQUEMENT — ne pas bloquer sur cryptoInitialized.
  // Si la crypto échoue (IndexedDB absent en CI / Chromium headless),
  // l'app reste accessible en mode dégradé. Bloquer ici casserait tous les E2E.
  $effect(() => {
    if (loading) return;

    const pathname = $page.url.pathname;

    if (authStore.needsPasswordChange && pathname !== '/change-password') {
      goto('/change-password');
      return;
    }

    if (authStore.isAuthenticated) {
      if (pathname === '/' || pathname === '/login' || pathname === '/register') {
        goto(authStore.isAdmin ? '/admin' : '/chat');
      }
    } else {
      const publicPaths = ['/login', '/register', '/help', '/join'];
      if (!publicPaths.some((p) => pathname.startsWith(p))) {
        goto('/login');
      }
    }
  });

  onMount(async () => {
    try {
      await waitForSodium();

      cryptoInitialized = await initCryptoSystem();
      if (!cryptoInitialized) {
        // NON BLOQUANT : mode dégradé, on continue sans E2EE
        cryptoError = "Système de chiffrement indisponible — mode dégradé activé";
        console.warn('[layout] Crypto init failed — running in degraded mode (E2EE off)');
        // Ne PAS throw ici → authStore.init() continue quand même
      }

      await authStore.init();

    } catch (err) {
      console.error("Erreur d'initialisation globale :", err);
      if (sodiumState.error) {
        appError = 'Erreur de chargement des bibliothèques de sécurité.';
      } else {
        appError = 'Impossible de vérifier votre session. Réessayez.';
      }
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <div class="loading-screen" data-testid="loading-screen">
    <div class="loading-spinner"></div>
    <p>Chargement de Nook...</p>
    {#if sodiumState.error}
      <p class="crypto-error">⚠️ {sodiumState.error}</p>
    {/if}
  </div>

{:else if appError}
  <div class="error-screen">
    <div class="error-content">
      <h1>❌ Erreur système</h1>
      <p class="error-title">================</p>
      <p class="error-message">{appError}</p>
      <div class="error-details">
        {#if sodiumState.error}
          <p class="detail-item">• Libsodium : {sodiumState.error}</p>
        {/if}
        {#if cryptoError}
          <p class="detail-item">• Cryptographie : {cryptoError}</p>
        {/if}
      </div>
      <button onclick={() => window.location.reload()} class="retry-button">
        🔄 Recharger l'application
      </button>
    </div>
  </div>

{:else}
  {#if cryptoError}
    <div class="crypto-warning-banner" role="alert">
      ⚠️ Chiffrement de bout en bout indisponible — messages envoyés en clair.
    </div>
  {/if}

  <header class="app-header">
    <button onclick={toggleMenu} class="menu-toggle" aria-label="Ouvrir le menu de navigation">
      ☰
    </button>

    <h1>🌱 Nook</h1>

    {#if authStore.isAuthenticated}
      <span class="user-name">{authStore.user?.name || authStore.user?.username}</span>
      <button
        onclick={handleLogout}
        class="logout-btn"
        data-testid="logout-button"
        aria-label="Déconnexion"
      >🔌</button>
    {/if}
  </header>

  {#if showMenu}
    <button
      class="menu-overlay"
      onclick={closeMenu}
      onkeydown={handleMenuKeydown}
      aria-label="Fermer le menu"
      aria-hidden={!showMenu}
    ></button>

    <div
      bind:this={menuElement}
      class="menu"
      role="dialog"
      aria-modal="true"
      aria-label="Menu de navigation"
      tabindex="0"
      onkeydown={handleMenuKeydown}
      onclick={(e) => e.stopPropagation()}
    >
      <div class="menu-header">
        <h2>Menu Nook</h2>
        <button onclick={closeMenu} class="close-menu" aria-label="Fermer le menu">✕</button>
      </div>

      <ul class="nav-list">
        {#each navItems as item}
          {#if item.requiresAuth && !authStore.isAuthenticated}
            <!-- Skip -->
          {:else if item.requiresAdmin && !authStore.isAdmin}
            <!-- Skip -->
          {:else}
            <li><a href={item.path} onclick={closeMenu}>{item.label}</a></li>
          {/if}
        {/each}
      </ul>

      <div class="menu-footer">
        <p class="version">Version 3.0 • SvelteKit</p>
        {#if authStore.isAuthenticated}
          <button onclick={handleLogout} class="logout-link" aria-label="Déconnexion">
            🔌 Déconnexion
          </button>
        {/if}
      </div>
    </div>
  {/if}

  <main class="app-main">
    {@render children()}
  </main>

  <footer class="app-footer">
    <p>© {new Date().getFullYear()} Nook • Messagerie privée pour la famille</p>
  </footer>
{/if}

<style>
  :global(body) {
    margin: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: linear-gradient(135deg, #f5f7fa 0%, #e4e8ec 100%);
    min-height: 100vh;
  }

  .loading-screen {
    display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    min-height: 100vh; gap: 1rem; padding: 1.5rem;
  }

  .loading-spinner {
    width: 48px; height: 48px;
    border: 4px solid #e2e8f0;
    border-top-color: #4ade80;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .crypto-error {
    color: #dc2626; font-size: 0.9rem; text-align: center;
    max-width: 400px; padding: 0.5rem;
    background: rgba(239, 68, 68, 0.1); border-radius: 0.5rem; margin-top: 0.5rem;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  .error-screen {
    display: flex; align-items: center; justify-content: center;
    min-height: 100vh; padding: 1.5rem;
  }

  .error-content {
    background: white; padding: 2.5rem; border-radius: 1rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    text-align: center; max-width: 450px; width: 100%;
  }

  .error-content h1 { font-size: 1.5rem; margin: 0 0 0.5rem 0; color: #1e293b; }
  .error-title { color: #64748b; margin: 0 0 1.25rem 0; font-family: monospace; }
  .error-message { color: #dc2626; margin: 0 0 1.5rem 0; line-height: 1.5; font-weight: 500; }

  .error-details {
    text-align: left; margin: 1rem 0; padding: 0.75rem;
    background: #f8fafc; border-radius: 0.5rem; border: 1px solid #e2e8f0;
  }

  .detail-item { color: #64748b; margin: 0.25rem 0; font-size: 0.9rem; }

  .retry-button {
    padding: 0.75rem 1.5rem; background: #4ade80; color: white;
    border: none; border-radius: 0.5rem; font-size: 1rem; font-weight: 600;
    cursor: pointer; transition: all 0.2s;
    display: inline-flex; align-items: center; gap: 0.5rem;
  }

  .retry-button:hover { filter: brightness(1.1); transform: translateY(-1px); }

  .crypto-warning-banner {
    background: #fef3c7; color: #92400e; font-size: 0.85rem;
    padding: 0.5rem 1.5rem; text-align: center;
    border-bottom: 1px solid #fde68a;
  }

  .app-header {
    display: flex; align-items: center; gap: 0.75rem;
    padding: 1rem 1.5rem; background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    position: sticky; top: 0; z-index: 100;
  }

  .menu-toggle, .logout-btn {
    background: none; border: none; font-size: 1.5rem;
    cursor: pointer; padding: 0.5rem; border-radius: 0.5rem; transition: background 0.2s;
  }

  .menu-toggle:hover, .logout-btn:hover { background: #f1f5f9; }

  .app-header h1 { font-size: 1.25rem; font-weight: 700; margin: 0; color: #1e293b; flex: 1; }
  .user-name { font-size: 0.9rem; color: #64748b; margin-right: 0.5rem; }

  .menu-overlay {
    position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5);
    z-index: 200; cursor: pointer; border: none; padding: 0; margin: 0;
  }

  .menu {
    position: fixed; top: 0; right: 0; bottom: 0; width: 300px;
    max-width: 85vw; background: white; z-index: 201;
    box-shadow: -4px 0 20px rgba(0, 0, 0, 0.15);
    display: flex; flex-direction: column;
    animation: slideIn 0.25s ease-out;
  }

  @keyframes slideIn {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
  }

  .menu-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: 1.25rem 1.5rem; border-bottom: 1px solid #e2e8f0;
  }

  .menu-header h2 { font-size: 1.1rem; font-weight: 600; margin: 0; color: #1e293b; }

  .close-menu {
    background: none; border: none; font-size: 1.5rem; cursor: pointer;
    padding: 0.5rem; border-radius: 0.5rem; color: #64748b; transition: all 0.2s;
  }

  .close-menu:hover { background: #f1f5f9; color: #1e293b; }

  .nav-list { list-style: none; margin: 0; padding: 1rem 0; flex: 1; overflow-y: auto; }

  .nav-list li a {
    display: block; padding: 0.85rem 1.5rem;
    color: #334155; text-decoration: none; transition: all 0.2s; font-size: 1rem;
  }

  .nav-list li a:hover { background: #f1f5f9; color: #1e293b; }

  .menu-footer { padding: 1.25rem 1.5rem; border-top: 1px solid #e2e8f0; }
  .version { font-size: 0.8rem; color: #94a3b8; margin: 0 0 0.75rem 0; }

  .logout-link {
    display: flex; align-items: center; gap: 0.5rem; width: 100%;
    padding: 0.75rem 1rem; background: none; border: 1px solid #e2e8f0;
    border-radius: 0.5rem; color: #64748b; font-size: 0.95rem;
    cursor: pointer; transition: all 0.2s;
  }

  .logout-link:hover { background: #fef2f2; border-color: #fecaca; color: #dc2626; }

  .app-main { min-height: calc(100vh - 140px); padding: 1.5rem; max-width: 1200px; margin: 0 auto; }

  .app-footer {
    text-align: center; padding: 1.25rem; color: #64748b;
    font-size: 0.85rem; border-top: 1px solid #e2e8f0; background: white;
  }

  .app-footer p { margin: 0; }

  @media (max-width: 640px) {
    .app-header { padding: 0.85rem 1rem; }
    .app-header h1 { font-size: 1.1rem; }
    .app-main { padding: 1rem; }
    .menu { width: 100%; max-width: none; }
    .error-content { padding: 1.5rem; }
  }
</style>