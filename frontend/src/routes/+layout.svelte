<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { authStore } from '$lib/authStore.svelte.js';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { initCryptoSystem } from '$lib/crypto';
  import { sodiumState, waitForSodium } from '$lib/sodium.svelte.js';
  import { cryptoStore } from '$lib/cryptoStore.svelte';
  import { chatStore } from '$lib/chatStore.svelte.ts';
  import CallBanner from '$lib/components/CallBanner.svelte';

  let { children } = $props();
  let showMenu        = $state(false);
  let appError        = $state<string | null>(null);
  let loading         = $state(true);
  let cryptoInitialized = $state(false);
  let cryptoError     = $state<string | null>(null);
  let menuElement     = $state<HTMLElement | undefined>(undefined);

  // Badge non-lu : somme de tous les compteurs de conversations
  const totalUnread = $derived(
    Object.values(chatStore.unreadCounts).reduce((sum, n) => sum + (n ?? 0), 0)
  );

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
      const publicPaths = ['/login', '/register', '/help', '/join', '/invite'];
      if (!publicPaths.some((p) => pathname.startsWith(p))) {
        goto('/login');
      }
    }
  });

  // ─── Hauteur header dynamique (CSS var --header-h) ──────────────────────
  // Permet aux pages full-height (chat) de calculer leur hauteur exactement
  let headerEl = $state<HTMLElement | undefined>(undefined);
  $effect(() => {
    if (!headerEl) return;
    const ro = new ResizeObserver(() => {
      document.documentElement.style.setProperty('--header-h', headerEl!.offsetHeight + 'px');
    });
    ro.observe(headerEl);
    return () => ro.disconnect();
  });

  // ─── Thème global — persisté sur toutes les pages ────────────────────────
  // Appelé immédiatement dans onMount pour appliquer le thème AVANT tout rendu.
  // Sans ça, le thème ne s'applique qu'à settings/+page.svelte et disparaît
  // à la navigation.
  function initThemeGlobal(): void {
    if (typeof window === 'undefined') return;
    const saved = localStorage.getItem('nook-theme');
    const dark  = localStorage.getItem('nook-dark-mode') === 'true';
    const theme = saved ?? 'jardin-secret';
    document.body.classList.remove(
      'theme-jardin-secret', 'theme-space-hub', 'theme-maison-chaleureuse'
    );
    document.body.classList.add(`theme-${theme}`);
    document.body.classList.toggle('dark-mode', dark);
    document.documentElement.setAttribute('data-theme', theme);
  }

  onMount(async () => {
    // Thème appliqué EN PREMIER — avant tout le reste
    initThemeGlobal();

    // ─────────────────────────────────────────────────────────────────────
    // ARCHITECTURE : sodium en fire-and-forget, authStore.init() en priorité
    //
    // PROBLÈME PRÉCÉDENT (Bug R37) :
    //   await waitForSodium()   ← bloquait ici (938kB WASM, >20s en CI headless)
    //   await initCryptoSystem()
    //   await authStore.init()
    //   loading = false          ← trop tard → #username jamais visible → 75/75 timeouts
    //
    // SOLUTION :
    //   Sodium n'est PAS nécessaire pour afficher la page de login ni vérifier
    //   la session (authStore.init = fetch /api/auth/me).
    //   On lance sodium en arrière-plan sans bloquer, puis on fait authStore.init()
    //   immédiatement → loading = false dès que la session est vérifiée (~100ms).
    //   La crypto s'active toute seule quand sodium est prêt (via unlockCrypto au login).
    // ─────────────────────────────────────────────────────────────────────

    // Lance sodium en arrière-plan — ne PAS await ici
    waitForSodium()
      .then(() => initCryptoSystem())
      .then((ok) => {
        cryptoInitialized = ok;
        if (!ok) {
          cryptoError = 'Système de chiffrement indisponible — mode dégradé activé';
          console.warn('[layout] Crypto init failed — running in degraded mode (E2EE off)');
        }
      })
      .catch((err) => {
        // Sodium peut échouer en CI headless (WASM non supporté) → mode dégradé non-bloquant
        cryptoError = 'Système de chiffrement indisponible — mode dégradé activé';
        console.warn('[layout] waitForSodium/initCrypto error (non-bloquant) :', err);
      });

    // Vérification de session : c'est ça qui détermine si on est connecté ou non
    // C'est la SEULE chose qui doit bloquer l'affichage
    try {
      await authStore.init();
    } catch (err) {
      console.error("Erreur d'initialisation de session :", err);
      appError = 'Impossible de vérifier votre session. Réessayez.';
    } finally {
      // loading = false dès que authStore.init() est terminé (~100ms réseau local)
      // Sodium continue en arrière-plan — cryptoError se mettra à jour quand il finit
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
  {#if cryptoError && authStore.isAuthenticated && !cryptoStore.ready && !$page.url.pathname.startsWith('/login') && !$page.url.pathname.startsWith('/invite') && !$page.url.pathname.startsWith('/register')}
    <div class="crypto-warning-banner" role="alert">
      ⚠️ Chiffrement de bout en bout indisponible — messages envoyés en clair.
    </div>
  {/if}

  <header class="app-header" bind:this={headerEl}>
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

  <CallBanner />

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
            <li>
              <a href={item.path} onclick={closeMenu} class:active={$page.url.pathname.startsWith(item.path)}>
                {item.label}
                {#if item.path === '/chat' && totalUnread > 0}
                  <span class="nav-badge">{totalUnread > 99 ? '99+' : totalUnread}</span>
                {/if}
              </a>
            </li>
          {/if}
        {/each}
      </ul>

      <div class="menu-footer">
        <p class="version">Nook v0.5 • Svelte 5 + Rust</p>
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
  :global(:root) {
    --header-h: 60px; /* défaut, écrasé dynamiquement par ResizeObserver */
  }
  :global(body) {
    margin: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--bg-primary, #f5f7fa);
    min-height: 100vh;
  }

  .loading-screen {
    display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    min-height: 100vh; gap: 1rem; padding: 1.5rem;
  }

  .loading-spinner {
    width: 48px; height: 48px;
    border: 4px solid var(--border, #e2e8f0);
    border-top-color: var(--accent, #4ade80);
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
    background: var(--bg-secondary, white); padding: 2.5rem; border-radius: 1rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    text-align: center; max-width: 450px; width: 100%;
  }

  .error-content h1 { font-size: 1.5rem; margin: 0 0 0.5rem 0; color: #1e293b; }
  .error-title { color: #64748b; margin: 0 0 1.25rem 0; font-family: monospace; }
  .error-message { color: #dc2626; margin: 0 0 1.5rem 0; line-height: 1.5; font-weight: 500; }

  .error-details {
    text-align: left; margin: 1rem 0; padding: 0.75rem;
    background: var(--bg-tertiary, #f8fafc); border-radius: 0.5rem; border: 1px solid var(--border, #e2e8f0);
  }

  .detail-item { color: var(--text-secondary, #64748b); margin: 0.25rem 0; font-size: 0.9rem; }

  .retry-button {
    padding: 0.75rem 1.5rem; background: var(--accent, #4ade80); color: white;
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
    padding: 1rem 1.5rem; background: var(--bg-secondary, white);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1); border-bottom: 1px solid var(--border, #e2e8f0);
    position: sticky; top: 0; z-index: 100;
  }

  .menu-toggle, .logout-btn {
    background: none; border: none; font-size: 1.5rem;
    cursor: pointer; padding: 0.5rem; border-radius: 0.5rem; transition: background 0.2s;
  }

  .menu-toggle:hover, .logout-btn:hover { background: var(--bg-tertiary, #f1f5f9); }

  .app-header h1 { font-size: 1.25rem; font-weight: 700; margin: 0; color: var(--text-primary, #1e293b); flex: 1; }
  .user-name { font-size: 0.9rem; color: var(--text-secondary, #64748b); margin-right: 0.5rem; }

  .menu-overlay {
    position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5);
    z-index: 200; cursor: pointer; border: none; padding: 0; margin: 0;
  }

  .menu {
    position: fixed; top: 0; left: 0; bottom: 0; width: 300px;
    max-width: 85vw; background: var(--bg-secondary, white); z-index: 201;
    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.15);
    display: flex; flex-direction: column;
    animation: slideIn 0.25s ease-out;
  }

  @keyframes slideIn {
    from { transform: translateX(-100%); }
    to   { transform: translateX(0); }
  }

  .menu-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: 1.25rem 1.5rem; border-bottom: 1px solid var(--border, #e2e8f0);
  }

  .menu-header h2 { font-size: 1.1rem; font-weight: 600; margin: 0; color: var(--text-primary, #1e293b); }

  .close-menu {
    background: none; border: none; font-size: 1.5rem; cursor: pointer;
    padding: 0.5rem; border-radius: 0.5rem; color: var(--text-secondary, #64748b); transition: all 0.2s;
  }

  .close-menu:hover { background: var(--bg-tertiary, #f1f5f9); color: var(--text-primary, #1e293b); }

  .nav-list { list-style: none; margin: 0; padding: 1rem 0; flex: 1; overflow-y: auto; }

  .nav-list li a {
    display: flex; align-items: center; gap: .5rem;
    padding: 0.85rem 1.5rem;
    color: var(--text-primary, #334155); text-decoration: none; transition: all 0.2s; font-size: 1rem;
    border-radius: .5rem; margin: 0 .35rem;
  }

  .nav-list li a:hover, .nav-list li a.active { background: var(--bg-tertiary, #f1f5f9); color: var(--text-primary, #1e293b); }

  .nav-badge {
    margin-left: auto;
    display: inline-flex; align-items: center; justify-content: center;
    background: var(--accent, #4ade80); color: #166534;
    font-size: .65rem; font-weight: 700; border-radius: 999px;
    min-width: 1.25rem; height: 1.25rem; padding: 0 .3rem;
  }

  .menu-footer { padding: 1.25rem 1.5rem; border-top: 1px solid var(--border, #e2e8f0); }
  .version { font-size: 0.8rem; color: var(--text-muted, #94a3b8); margin: 0 0 0.75rem 0; }

  .logout-link {
    display: flex; align-items: center; gap: 0.5rem; width: 100%;
    padding: 0.75rem 1rem; background: none; border: 1px solid var(--border, #e2e8f0);
    border-radius: 0.5rem; color: var(--text-secondary, #64748b); font-size: 0.95rem;
    cursor: pointer; transition: all 0.2s;
  }

  .logout-link:hover { background: #fef2f2; border-color: #fecaca; color: #dc2626; }

        .app-main { padding: 0; background: var(--bg-primary, #f5f7fa); }

  .app-footer {
    text-align: center; padding: 1.25rem; color: var(--text-secondary, #64748b);
    font-size: 0.85rem; border-top: 1px solid var(--border, #e2e8f0); background: var(--bg-secondary, white);
    flex-shrink: 0;
  }

  .app-footer p { margin: 0; }

  @media (max-width: 640px) {
    .app-header { padding: 0.85rem 1rem; }
    .app-header h1 { font-size: 1.1rem; }
    .app-main { padding: 0; }
    .menu { width: 85vw; max-width: none; left: 0; right: auto; }
    .error-content { padding: 1.5rem; }
  }
</style>

<CallBanner />

  // Clear cryptoError when crypto becomes ready (e.g., after login)
  $effect(() => {
    if (cryptoStore.ready && cryptoError) {
      cryptoError = null;
    }
  });
