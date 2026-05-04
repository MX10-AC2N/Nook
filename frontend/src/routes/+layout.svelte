<script lang="ts">
     2|  import '../app.css';
     3|  import { page } from '$app/stores';
     4|  import { authStore } from '$lib/authStore.svelte.js';
     5|  import { onMount } from 'svelte';
     6|  import { goto } from '$app/navigation';
     7|  import { initCryptoSystem } from '$lib/crypto';
     8|  import { sodiumState, waitForSodium } from '$lib/sodium.svelte.js';
     9|  import { cryptoStore } from '$lib/cryptoStore.svelte';
    10|  import { chatStore } from '$lib/chatStore.svelte.ts';
    11|  import CallBanner from '$lib/components/CallBanner.svelte';
    12|  import NotificationToast from '$lib/components/NotificationToast.svelte';
    13|  import Icon from '$lib/components/Icon.svelte';
    14|
    15|  let { children } = $props();
    16|  let appError        = $state<string | null>(null);
    17|  let loading         = $state(true);
    18|  let cryptoInitialized = $state(false);
    19|  let cryptoError     = $state<string | null>(null);
    20|  let menuElement     = $state<HTMLElement | undefined>(undefined);
    21|
    22|  // Badge non-lu : somme de tous les compteurs de conversations
    23|  const totalUnread = $derived(
    24|    Object.values(chatStore.unreadCounts).reduce((sum, n) => sum + (n ?? 0), 0)
    25|  );
    26|
    27|  const navItems = [
    28|    { path: '/chat',      label: 'Chat',            icon: 'chat',       requiresAuth: true  },
    29|    { path: '/chess',     label: 'Échecs',           icon: 'chess',      requiresAuth: true  },
    30|    { path: '/calendar',  label: 'Calendrier',       icon: 'calendar',   requiresAuth: true  },
    31|    { path: '/polls',     label: 'Sondages',         icon: 'check-circle', requiresAuth: true  },
    32|    { path: '/admin',     label: 'Administration',   icon: 'user',       requiresAuth: true, requiresAdmin: true },
    33|    { path: '/settings',  label: 'Paramètres',       icon: 'settings',   requiresAuth: true  },
    34|    { path: '/help',      label: 'Aide',             icon: 'help',       requiresAuth: false },
    35|  ];
    36|
    37|  function toggleMenu() {
    38|    showMenu = !showMenu;
    39|    if (showMenu && menuElement) menuElement.focus();
    40|  }
    41|
    42|  function closeMenu() { showMenu = false; }
    43|
    44|  function handleMenuKeydown(event: KeyboardEvent) {
    45|    if (event.key === 'Escape') closeMenu();
    46|  }
    47|
    48|  async function handleLogout() {
    49|    await fetch('/api/auth/logout', { method: 'POST', credentials: 'include' });
    50|    authStore.logout();
    51|    closeMenu();
    52|    goto('/login');
    53|  }
    54|
    55|  // Guard sur `loading` UNIQUEMENT — ne pas bloquer sur cryptoInitialized.
    56|  // Si la crypto échoue (IndexedDB absent en CI / Chromium headless),
    57|  // l'app reste accessible en mode dégradé. Bloquer ici casserait tous les E2E.
    58|  $effect(() => {
    59|    if (loading) return;
    60|
    61|    const pathname = $page.url.pathname;
    62|
    63|    if (authStore.needsPasswordChange && pathname !== '/change-password') {
    64|      goto('/change-password');
    65|      return;
    66|    }
    67|
    68|    if (authStore.isAuthenticated) {
    69|      if (pathname === '/' || pathname === '/login' || pathname === '/register') {
    70|        goto('/chat');
    71|        return;
    72|      }
    73|    } else {
    74|      if (pathname !== '/login' && pathname !== '/register' && pathname !== '/help') {
    75|        goto('/login');
    76|        return;
    77|      }
    78|    }
    79|  });
    80|
    81|  onMount(async () => {
    82|    try {
    83|      await waitForSodium();
    84|      const result = await initCryptoSystem();
    85|      cryptoInitialized = result.success;
    86|      cryptoError = result.error ?? null;
    87|    } catch (e) {
    88|      cryptoError = e instanceof Error ? e.message : String(e);
    89|    } finally {
    90|      loading = false;
    91|    }
    92|  });
    94|
</script>

    95|<svelte:head>
    96|  <meta name="viewport" content="width=device-width, initial-scale=1.0">
    97|</svelte:head>

    98|
    99|
   100|  // ─── Hauteur header dynamique (CSS var --header-h) ──────────────────────
   101|  // Permet aux pages full-height (chat) de calculer leur hauteur exactement
   102|  let headerEl = $state<HTMLElement | undefined>(undefined);
   103|  $effect(() => {
   104|    if (headerEl) {
   105|      const ro = new ResizeObserver(() => {
   106|        document.documentElement.style.setProperty('--header-h', headerEl.offsetHeight + 'px');
   107|      });
   108|      ro.observe(headerEl);
   109|      return () => ro.disconnect();
   110|    }
   111|  });
   112|
   113|  // ─── Thème global — persisté sur toutes les pages ────────────────────────
   114|  // Appelé immédiatement dans onMount pour appliquer le thème AVANT tout rendu.
   115|  // Sans ça, le thème ne s'applique qu'à settings/+page.svelte et disparaît
   116|  // à la navigation.
   117|  function initThemeGlobal(): void {
   118|    if (typeof window === 'undefined') return;
   119|    const saved = localStorage.getItem('nook-theme');
   120|    const dark  = localStorage.getItem('nook-dark-mode') === 'true';
   121|    const theme = saved ?? 'jardin-secret';
   122|    document.body.classList.remove(
   123|      'theme-jardin-secret', 'theme-space-hub', 'theme-maison-chaleureuse'
   124|    );
   125|    document.body.classList.add(`theme-${theme}`);
   126|    document.body.classList.toggle('dark-mode', dark);
   127|    document.documentElement.setAttribute('data-theme', theme);
   128|  }
   129|
   130|  onMount(async () => {
   131|    // Thème appliqué EN PREMIER — avant tout le reste
   132|    initThemeGlobal();
   133|
   134|    // ─────────────────────────────────────────────────────────────────────
   135|    // Service Worker Registration (pour les notifications push)
   136|    // ─────────────────────────────────────────────────────────────────────
   137|    if ('serviceWorker' in navigator) {
   138|      try {
   139|        const registration = await navigator.serviceWorker.register('/service-worker.js', {
   140|          scope: '/',
   141|        });
   142|        console.log('[SW] Service worker registered:', registration.scope);
   143|        
   144|        if (registration.active) {
   145|          console.log('[SW] Service worker is active');
   146|        } else if (registration.installing) {
   147|          console.log('[SW] Service worker is installing...');
   148|          registration.installing.addEventListener('statechange', () => {
   149|            console.log('[SW] Service worker state:', registration.installing?.state);
   150|            if (registration.installing?.state === 'activated') {
   151|              console.log('[SW] Service worker is now active');
   152|            }
   153|          });
   154|        }
   155|      } catch (error) {
   156|        console.error('[SW] Service worker registration failed:', error);
   157|      }
   158|    } else {
   159|      console.warn('[SW] Service workers not supported');
   160|    }
   161|
   162|    // ─────────────────────────────────────────────────────────────────────
   163|    // ARCHITECTURE : sodium en fire-and-forget, authStore.init() en priorité
   164|    //
   165|    // PROBLÈME PRÉCÉDENT (Bug R37) :
   166|    //   await waitForSodium()   ← bloquait ici (938kB WASM, >20s en CI headless)
   167|    //   await initCryptoSystem()
   168|    //   await authStore.init()
   169|    //   loading = false          ← trop tard → #username jamais visible → 75/75 timeouts
   170|    //
   171|    // SOLUTION :
   172|    //   Sodium n'est PAS nécessaire pour afficher la page de login ni vérifier
   173|    //   la session (authStore.init = fetch /api/auth/me).
   174|    //   On lance sodium en arrière-plan sans bloquer, puis on fait authStore.init()
   175|    //   immédiatement → loading = false dès que la session est vérifiée (~100ms).
   176|    //   La crypto s'active toute seule quand sodium est prêt (via unlockCrypto au login).
   177|    // ─────────────────────────────────────────────────────────────────────
   178|
   179|    // Lance sodium en arrière-plan — ne PAS await ici
   180|    waitForSodium()
   181|      .then(() => initCryptoSystem())
   182|      .then((ok) => {
   183|        cryptoInitialized = ok;
   184|        if (!ok) {
   185|          cryptoError = 'Système de chiffrement indisponible — mode dégradé activé';
   186|          console.warn('[layout] Crypto init failed — running in degraded mode (E2EE off)');
   187|        }
   188|      })
   189|      .catch((err) => {
   190|        // Sodium peut échouer en CI headless (WASM non supporté) → mode dégradé non-bloquant
   191|        cryptoError = 'Système de chiffrement indisponible — mode dégradé activé';
   192|        console.warn('[layout] waitForSodium/initCrypto error (non-bloquant) :', err);
   193|      });
   194|
   195|    // Vérification de session : c'est ça qui détermine si on est connecté ou non
   196|    // C'est la SEULE chose qui doit bloquer l'affichage
   197|    try {
   198|      await authStore.init();
   199|    } catch (err) {
   200|      console.error("Erreur d'initialisation de session :", err);
   201|      appError = 'Impossible de vérifier votre session. Réessayez.';
   202|    } finally {
   203|      // loading = false dès que authStore.init() est terminé (~100ms réseau local)
   204|      // Sodium continue en arrière-plan — cryptoError se mettra à jour quand il finit
   205|      loading = false;
   206|    }
   207|  });
   208|
   209|  // Clear cryptoError when crypto becomes ready (e.g., after login)
   210|  $effect(() => {
   211|    if (cryptoStore.ready && cryptoError) {
   212|      cryptoError = null;
   213|    }
   214|  });
   215|
   216|</script>
   217|
   218|{#if loading}
   219|  <div class="loading-screen" data-testid="loading-screen">
   220|    <div class="loading-spinner"></div>
   221|    <p>Chargement de Nook...</p>
   222|    {#if sodiumState.error}
   223|      <p class="crypto-error">⚠️ {sodiumState.error}</p>
   224|    {/if}
   225|  </div>
   226|
   227|{:else if appError}
   228|  <div class="error-screen">
   229|    <div class="error-content">
   230|      <h1>❌ Erreur système</h1>
   231|      <p class="error-title">================</p>
   232|      <p class="error-message">{appError}</p>
   233|      <div class="error-details">
   234|        {#if sodiumState.error}
   235|          <p class="detail-item">• Libsodium : {sodiumState.error}</p>
   236|        {/if}
   237|        {#if cryptoError}
   238|          <p class="detail-item">• Cryptographie : {cryptoError}</p>
   239|        {/if}
   240|      </div>
   241|      <button onclick={() => window.location.reload()} class="retry-button">
   242|        🔄 Recharger l'application
   243|      </button>
   244|    </div>
   245|  </div>
   246|
   247|{:else}
   248|  {#if cryptoError && authStore.isAuthenticated && !cryptoStore.ready && $page.url.pathname.startsWith('/chat')}
   249|    <div class="crypto-warning-banner" role="alert">
   250|      ⚠️ Chiffrement de bout en bout indisponible — messages envoyés en clair.
   251|    </div>
   252|  {/if}
   253|
   254|  <header class="app-header" bind:this={headerEl}>
   255|    <button onclick={toggleMenu} class="menu-toggle" aria-label="Ouvrir le menu de navigation">
   256|      ☰
   257|    </button>
   258|
   259|    <h1 style="flex:1; justify-content:center;"><Icon name="logo" size="68" /></h1>
   260|
   261|    {#if authStore.isAuthenticated}
   262|      <span class="user-name">{authStore.user?.name || authStore.user?.username}</span>
   263|      <button
   264|        onclick={handleLogout}
   265|        class="logout-btn"
   266|        data-testid="logout-button"
   267|        aria-label="Déconnexion"
   268|      ><Icon name="logout" size="20" /></button>
   269|    {/if}
   270|  </header>
   271|
   272|  <CallBanner />
   273|  <NotificationToast />
   274|
   275|  {#if showMenu}
   276|    <button
   277|      class="menu-overlay"
   278|      onclick={closeMenu}
   279|      onkeydown={handleMenuKeydown}
   280|      aria-label="Fermer le menu"
   281|      aria-hidden={!showMenu}
   282|    ></button>
   283|
   284|    <div
   285|      bind:this={menuElement}
   286|      class="menu"
   287|      role="dialog"
   288|      aria-modal="true"
   289|      aria-label="Menu de navigation"
   290|      tabindex="0"
   291|      onkeydown={handleMenuKeydown}
   292|      onclick={(e) => e.stopPropagation()}
   293|    >
   294|      <div class="menu-header">
   295|        <h2>Menu Nook</h2>
   296|        <button onclick={closeMenu} class="close-menu" aria-label="Fermer le menu">✕</button>
   297|      </div>
   298|
   299|      <ul class="nav-list">
   300|        {#each navItems as item}
   301|          {#if item.requiresAuth && !authStore.isAuthenticated}
   302|            <!-- Skip -->
   303|          {:else if item.requiresAdmin && !authStore.isAdmin}
   304|            <!-- Skip -->
   305|          {:else}
   306|            <li>
   307|              <a href={item.path} onclick={closeMenu} class:active={$page.url.pathname.startsWith(item.path)}>
   308|                {#if item.icon}<Icon name={item.icon} size="18" /> {/if}{item.label}
   309|                {#if item.path === '/chat' && totalUnread > 0}
   310|                  <span class="nav-badge">{totalUnread > 99 ? '99+' : totalUnread}</span>
   311|                {/if}
   312|              </a>
   313|            </li>
   314|          {/if}
   315|        {/each}
   316|      </ul>
   317|
   318|      <div class="menu-footer">
   319|        <p class="version">Nook v0.5 • Svelte 5 + Rust</p>
   320|        {#if authStore.isAuthenticated}
   321|          <button onclick={handleLogout} class="logout-link" aria-label="Déconnexion">
   322|            <Icon name="logout" size="18" /> Déconnexion
   323|          </button>
   324|        {/if}
   325|      </div>
   326|    </div>
   327|  {/if}
   328|
   329|  <main class="app-main">
   330|    {@render children()}
   331|  </main>
   332|
   333|  <footer class="app-footer">
   334|    <p>© {new Date().getFullYear()} Nook • Messagerie privée pour la famille</p>
   335|  </footer>
   336|{/if}
   337|
   338|<style>
   339|  :global(:root) {
   340|    --header-h: 60px; /* défaut, écrasé dynamiquement par ResizeObserver */
   341|  }
   342|  :global(body) {
   343|    margin: 0;
   344|    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
   345|    background: var(--bg-primary, #f5f7fa);
   346|    min-height: 100vh;
   347|  }
   348|
   349|  .loading-screen {
   350|    display: flex; flex-direction: column;
   351|    align-items: center; justify-content: center;
   352|    min-height: 100vh; gap: 1rem; padding: 1.5rem;
   353|  }
   354|
   355|  .loading-spinner {
   356|    width: 48px; height: 48px;
   357|    border: 4px solid var(--border, #e2e8f0);
   358|    border-top-color: var(--accent, #4ade80);
   359|    border-radius: 50%;
   360|    animation: spin 1s linear infinite;
   361|  }
   362|
   363|  .crypto-error {
   364|    color: #dc2626; font-size: 0.9rem; text-align: center;
   365|    max-width: 400px; padding: 0.5rem;
   366|    background: rgba(239, 68, 68, 0.1); border-radius: 0.5rem; margin-top: 0.5rem;
   367|  }
   368|
   369|  @keyframes spin { to { transform: rotate(360deg); } }
   370|
   371|  .error-screen {
   372|    display: flex; align-items: center; justify-content: center;
   373|    min-height: 100vh; padding: 1.5rem;
   374|  }
   375|
   376|  .error-content {
   377|    background: var(--bg-secondary, white); padding: 2.5rem; border-radius: 1rem;
   378|    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
   379|    text-align: center; max-width: 450px; width: 100%;
   380|  }
   381|
   382|  .error-content h1 { font-size: 1.5rem; margin: 0 0 0.5rem 0; color: #1e293b; }
   383|  .error-title { color: #64748b; margin: 0 0 1.25rem 0; font-family: monospace; }
   384|  .error-message { color: #dc2626; margin: 0 0 1.5rem 0; line-height: 1.5; font-weight: 500; }
   385|
   386|  .error-details {
   387|    text-align: left; margin: 1rem 0; padding: 0.75rem;
   388|    background: var(--bg-tertiary, #f8fafc); border-radius: 0.5rem; border: 1px solid var(--border, #e2e8f0);
   389|  }
   390|
   391|  .detail-item { color: var(--text-secondary, #64748b); margin: 0.25rem 0; font-size: 0.9rem; }
   392|
   393|  .retry-button {
   394|    padding: 0.75rem 1.5rem; background: var(--accent, #4ade80); color: white;
   395|    border: none; border-radius: 0.5rem; font-size: 1rem; font-weight: 600;
   396|    cursor: pointer; transition: all 0.2s;
   397|    display: inline-flex; align-items: center; gap: 0.5rem;
   398|  }
   399|
   400|  .retry-button:hover { filter: brightness(1.1); transform: translateY(-1px); }
   401|
   402|  .crypto-warning-banner {
   403|    background: #fef3c7; color: #92400e; font-size: 0.85rem;
   404|    padding: 0.5rem 1.5rem; text-align: center;
   405|    border-bottom: 1px solid #fde68a;
   406|  }
   407|
   408|  .app-header {
   409|    display: flex; align-items: center; gap: 0.5rem;
   410|    padding: 0.35rem 0.8rem; background: var(--bg-secondary, white);
   411|    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08); border-bottom: 1px solid var(--border, #e2e8f0);
   412|    position: sticky; top: 0; z-index: 100;
   413|    min-height: 64px;
   414|  }
   415|
   416|  .menu-toggle, .logout-btn {
   417|    background: none; border: none; font-size: 1rem;
   418|    cursor: pointer; padding: 0.25rem; border-radius: 0.4rem; transition: background 0.2s;
   419|  }
   420|
   421|  .menu-toggle:hover, .logout-btn:hover { background: var(--bg-tertiary, #f1f5f9); }
   422|
   423|  .app-header h1 { font-size: 0.9rem; font-weight: 600; margin: 0; display: flex; align-items: center; color: var(--text-primary, #1e293b); flex: 1; }
   424|  .user-name { font-size: 0.75rem; color: var(--text-secondary, #64748b); margin-right: 0.25rem; }
   425|
   426|  .menu-overlay {
   427|    position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5);
   428|    z-index: 200; cursor: pointer; border: none; padding: 0; margin: 0;
   429|  }
   430|
   431|  .menu {
   432|    position: fixed; top: 0; left: 0; bottom: 0; width: 300px;
   433|    max-width: 85vw; background: var(--bg-secondary, white); z-index: 201;
   434|    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.15);
   435|    display: flex; flex-direction: column;
   436|    animation: slideIn 0.25s ease-out;
   437|  }
   438|
   439|  @keyframes slideIn {
   440|    from { transform: translateX(-100%); }
   441|    to   { transform: translateX(0); }
   442|  }
   443|
   444|  .menu-header {
   445|    display: flex; justify-content: space-between; align-items: center;
   446|    padding: 1.25rem 1.5rem; border-bottom: 1px solid var(--border, #e2e8f0);
   447|  }
   448|
   449|  .menu-header h2 { font-size: 1.1rem; font-weight: 600; margin: 0; color: var(--text-primary, #1e293b); }
   450|
   451|  .close-menu {
   452|    background: none; border: none; font-size: 1.5rem; cursor: pointer;
   453|    padding: 0.5rem; border-radius: 0.5rem; color: var(--text-secondary, #64748b); transition: all 0.2s;
   454|  }
   455|
   456|  .close-menu:hover { background: var(--bg-tertiary, #f1f5f9); color: var(--text-primary, #1e293b); }
   457|
   458|  .nav-list { list-style: none; margin: 0; padding: 1rem 0; flex: 1; overflow-y: auto; }
   459|
   460|  .nav-list li a {
   461|    display: flex; align-items: center; gap: .5rem;
   462|    padding: 0.85rem 1.5rem;
   463|    color: var(--text-primary, #334155); text-decoration: none; transition: all 0.2s; font-size: 1rem;
   464|    border-radius: .5rem; margin: 0 .35rem;
   465|  }
   466|
   467|  .nav-list li a:hover, .nav-list li a.active { background: var(--bg-tertiary, #f1f5f9); color: var(--text-primary, #1e293b); }
   468|
   469|  .nav-badge {
   470|    margin-left: auto;
   471|    display: inline-flex; align-items: center; justify-content: center;
   472|    background: var(--accent, #4ade80); color: #166534;
   473|    font-size: .65rem; font-weight: 700; border-radius: 999px;
   474|    min-width: 1.25rem; height: 1.25rem; padding: 0 .3rem;
   475|  }
   476|
   477|  .menu-footer { padding: 1.25rem 1.5rem; border-top: 1px solid var(--border, #e2e8f0); }
   478|  .version { font-size: 0.8rem; color: var(--text-muted, #94a3b8); margin: 0 0 0.75rem 0; }
   479|
   480|  .logout-link {
   481|    display: flex; align-items: center; gap: 0.5rem; width: 100%;
   482|    padding: 0.75rem 1rem; background: none; border: 1px solid var(--border, #e2e8f0);
   483|    border-radius: 0.5rem; color: var(--text-secondary, #64748b); font-size: 0.95rem;
   484|    cursor: pointer; transition: all 0.2s;
   485|  }
   486|
   487|  .logout-link:hover { background: #fef2f2; border-color: #fecaca; color: #dc2626; }
   488|
   489|        .app-main { flex: 1; display: flex; flex-direction: column; padding: 0; background: var(--bg-primary, #f5f7fa); min-height: 0; }
   490|
   491|  .app-footer {
   492|    text-align: center; padding: 1.25rem; color: var(--text-secondary, #64748b);
   493|    font-size: 0.85rem; border-top: 1px solid var(--border, #e2e8f0); background: var(--bg-secondary, white);
   494|    flex-shrink: 0;
   495|  }
   496|
   497|  .app-footer p { margin: 0; }
   498|
   499|  @media (max-width: 640px) {
   500|  .app-header { padding: 0.3rem 0.6rem; min-height: 44px; }
   501|