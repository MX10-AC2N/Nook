     1|<script lang="ts">
     2|  import { page } from '$app/stores';
     3|  import { authStore } from '$lib/authStore.svelte.js';
     4|  import { onMount } from 'svelte';
     5|  import { goto } from '$app/navigation';
     6|  import { initCryptoSystem } from '$lib/crypto';
     7|  import { sodiumState, waitForSodium } from '$lib/sodium.svelte.js';
     8|  import { cryptoStore } from '$lib/cryptoStore.svelte';
     9|  import { chatStore } from '$lib/chatStore.svelte.ts';
    10|  import CallBanner from '$lib/components/CallBanner.svelte';
    11|  import NotificationToast from '$lib/components/NotificationToast.svelte';
    12|  import Icon from '$lib/components/Icon.svelte';
    13|
    14|  let { children } = $props();
    15|  let appError        = $state<string | null>(null);
    16|  let loading         = $state(true);
    17|  let cryptoInitialized = $state(false);
    18|  let cryptoError     = $state<string | null>(null);
    19|  let menuElement     = $state<HTMLElement | undefined>(undefined);
    20|  let showMenu        = $state(false);
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
    43|</script>
    44|
    45|function handleMenuKeydown(event: KeyboardEvent) {
    46|    if (event.key === 'Escape') closeMenu();
    47|  }
    48|
    49|  async function handleLogout() {
    50|    await fetch('/api/auth/logout', { method: 'POST', credentials: 'include' });
    51|    authStore.logout();
    52|    closeMenu();
    53|    goto('/login');
    54|  }
    55|
    56|  // Guard sur `loading` UNIQUEMENT — ne pas bloquer sur cryptoInitialized.
    57|  // Si la crypto échoue (IndexedDB absent en CI / Chromium headless),
    58|  // l'app reste accessible en mode dégradé. Bloquer ici casserait tous les E2E.
    59|  $effect(() => {
    60|    if (loading) return;
    61|
    62|    const pathname = $page.url.pathname;
    63|
    64|    if (authStore.needsPasswordChange && pathname !== '/change-password') {
    65|      goto('/change-password');
    66|      return;
    67|    }
    68|
    69|    if (authStore.isAuthenticated) {
    70|      if (pathname === '/' || pathname === '/login' || pathname === '/register') {
    71|        goto('/chat');
    72|        return;
    73|      }
    74|    } else {
    75|      if (pathname !== '/login' && pathname !== '/register' && pathname !== '/help') {
    76|        goto('/login');
    77|        return;
    78|      }
    79|    }
    80|  });
    81|
    82|  onMount(async () => {
    83|    try {
    84|      await waitForSodium();
    85|      const result = await initCryptoSystem();
    86|      cryptoInitialized = result.success;
    87|      cryptoError = result.error ?? null;
    88|    } catch (e) {
    89|      cryptoError = e instanceof Error ? e.message : String(e);
    90|    } finally {
    91|      loading = false;
    92|    }
    93|  });
    94|
    95|  // ─── Hauteur header dynamique (CSS var --header-h) ──────────────────────
    96|  // Permet aux pages full-height (chat) de calculer leur hauteur exactement
    97|  let headerEl = $state<HTMLElement | undefined>(undefined);
    98|  $effect(() => {
    99|    if (headerEl) {
   100|      const ro = new ResizeObserver(() => {
   101|        document.documentElement.style.setProperty('--header-h', headerEl.offsetHeight + 'px');
   102|      });
   103|      ro.observe(headerEl);
   104|      return () => ro.disconnect();
   105|    }
   106|  });
   107|
   108|  // ─── Thème global — persisté sur toutes les pages ────────────────────────
   109|  // Appelé immédiatement dans onMount pour appliquer le thème AVANT tout rendu.
   110|  // Sans ça, le thème ne s'applique qu'à settings/+page.svelte et disparaît
   111|  // à la navigation.
   112|  function initThemeGlobal(): void {
   113|    if (typeof window === 'undefined') return;
   114|    const saved = localStorage.getItem('nook-theme');
   115|    const dark  = localStorage.getItem('nook-dark-mode') === 'true';
   116|    const theme = saved ?? 'jardin-secret';
   117|    document.body.classList.remove(
   118|      'theme-jardin-secret', 'theme-space-hub', 'theme-maison-chaleureuse'
   119|    );
   120|    document.body.classList.add(`theme-${theme}`);
   121|    document.body.classList.toggle('dark-mode', dark);
   122|    document.documentElement.setAttribute('data-theme', theme);
   123|  }
   124|
   125|  onMount(async () => {
   126|    // Thème appliqué EN PREMIER — avant tout le reste
   127|    initThemeGlobal();
   128|
   129|    if ('serviceWorker' in navigator) {
   130|      try {
   131|        const registration = await navigator.serviceWorker.register('/service-worker.js', {
   132|          scope: '/',
   133|        });
   134|        console.log('[SW] Service worker registered:', registration.scope);
   135|        
   136|        if (registration.active) {
   137|          console.log('[SW] Service worker is active');
   138|        } else if (registration.installing) {
   139|          console.log('[SW] Service worker is installing...');
   140|          registration.installing.addEventListener('statechange', () => {
   141|            console.log('[SW] Service worker state:', registration.installing?.state);
   142|            if (registration.installing?.state === 'activated') {
   143|              console.log('[SW] Service worker is now active');
   144|            }
   145|          });
   146|        }
   147|      } catch (error) {
   148|        console.error('[SW] Service worker registration failed:', error);
   149|      }
   150|    } else {
   151|      console.warn('[SW] Service workers not supported');
   152|    }
   153|
   154|    // ─────────────────────────────────────────────────────────────────────
   155|    // ARCHITECTURE : sodium en fire-and-forget, authStore.init() en priorité
   156|    //
   157|    // PROBLÈME PRÉCÉDENT (Bug R37) :
   158|    //   await waitForSodium()   ← bloquait ici (938kB WASM, >20s en CI headless)
   159|    //   await initCryptoSystem()
   160|    //   await authStore.init()
   161|    //   loading = false          ← trop tard → #username jamais visible → 75/75 timeouts
   162|    //
   163|    // SOLUTION :
   164|    //   Sodium n'est PAS nécessaire pour afficher la page de login ni vérifier
   165|    //   la session (authStore.init = fetch /api/auth/me).
   166|    //   On lance sodium en arrière-plan sans bloquer, puis on fait authStore.init()
   167|    //   immédiatement → loading = false dès que la session est vérifiée (~100ms).
   168|    //   La crypto s'active toute seule quand sodium est prêt (via unlockCrypto au login).
   169|    // ─────────────────────────────────────────────────────────────────────
   170|
   171|    // Lance sodium en arrière-plan — ne PAS await ici
   172|    waitForSodium()
   173|      .then(() => initCryptoSystem())
   174|      .then((ok) => {
   175|        cryptoInitialized = ok;
   176|        if (!ok) {
   177|          cryptoError = 'Système de chiffrement indisponible — mode dégradé activé';
   178|          console.warn('[layout] Crypto init failed — running in degraded mode (E2EE off)');
   179|        }
   180|      })
   181|      .catch((err) => {
   182|        // Sodium peut échouer en CI headless (WASM non supporté) → mode dégradé non-bloquant
   183|        cryptoError = 'Système de chiffrement indisponible — mode dégradé activé';
   184|        console.warn('[layout] waitForSodium/initCrypto error (non-bloquant) :', err);
   185|      });
   186|
   187|    // Vérification de session : c'est ça qui détermine si on est connecté ou non
   188|    // C'est la SEULE chose qui doit bloquer l'affichage
   189|    try {
   190|      await authStore.init();
   191|    } catch (err) {
   192|      console.error("Erreur d'initialisation de session :", err);
   193|      appError = 'Impossible de vérifier votre session. Réessayez.';
   194|    } finally {
   195|      // loading = false dès que authStore.init() est terminé (~100ms réseau local)
   196|      // Sodium continue en arrière-plan — cryptoError se mettra à jour quand il finit
   197|      loading = false;
   198|    }
   199|  });
   200|
   201|  // Clear cryptoError when crypto becomes ready (e.g., after login)
   202|  $effect(() => {
   203|    if (cryptoStore.ready && cryptoError) {
   204|      cryptoError = null;
   205|    }
   206|  });
   207|</script>
   208|
   209|<svelte:head>
   210|  <meta name="viewport" content="width=device-width, initial-scale=1.0">
   211|  <link rel="stylesheet" href="/app.css">
   212|</svelte:head>
   213|
   214|{#if loading}
   215|  <div class="loading-screen" data-testid="loading-screen">
   216|    <div class="loading-spinner"></div>
   217|    <p>Chargement de Nook...</p>
   218|    {#if sodiumState.error}
   219|      <p class="crypto-error">⚠️ {sodiumState.error}</p>
   220|    {/if}
   221|  </div>
   222|
   223|{:else if appError}
   224|  <div class="error-screen">
   225|    <div class="error-content">
   226|      <h1>❌ Erreur système</h1>
   227|      <p class="error-title">================</p>
   228|      <p class="error-message">{appError}</p>
   229|      <div class="error-details">
   230|        {#if sodiumState.error}
   231|          <p class="detail-item">• Libsodium : {sodiumState.error}</p>
   232|        {/if}
   233|        {#if cryptoError}
   234|          <p class="detail-item">• Cryptographie : {cryptoError}</p>
   235|        {/if}
   236|      </div>
   237|      <button onclick={() => window.location.reload()} class="retry-button">
   238|        🔄 Recharger l'application
   239|      </button>
   240|    </div>
   241|  </div>
   242|
   243|{:else}
   244|  {#if cryptoError && authStore.isAuthenticated && !cryptoStore.ready && $page.url.pathname.startsWith('/chat')}
   245|    <div class="crypto-warning-banner" role="alert">
   246|      ⚠️ Chiffrement de bout en bout indisponible — messages envoyés en clair.
   247|    </div>
   248|  {/if}
   249|
   250|  <header class="app-header" bind:this={headerEl}>
   251|    <button onclick={toggleMenu} class="menu-toggle" aria-label="Ouvrir le menu de navigation">
   252|      ☰
   253|    </button>
   254|
   255|    <h1 style="flex:1; justify-content:center;"><Icon name="logo" size="68" /></h1>
   256|
   257|    {#if authStore.isAuthenticated}
   258|      <span class="user-name">{authStore.user?.name || authStore.user?.username}</span>
   259|      <button
   260|        onclick={handleLogout}
   261|        class="logout-btn"
   262|        data-testid="logout-button"
   263|        aria-label="Déconnexion"
   264|      ><Icon name="logout" size="20" /></button>
   265|    {/if}
   266|  </header>
   267|
   268|  <CallBanner />
   269|  <NotificationToast />
   270|
   271|  {#if showMenu}
   272|    <button
   273|      class="menu-overlay"
   274|      onclick={closeMenu}
   275|      onkeydown={handleMenuKeydown}
   276|      aria-label="Fermer le menu"
   277|      aria-hidden={!showMenu}
   278|    ></button>
   279|
   280|    <div
   281|      bind:this={menuElement}
   282|      class="menu"
   283|      role="dialog"
   284|      aria-modal="true"
   285|      aria-label="Menu de navigation"
   286|      tabindex="0"
   287|      onkeydown={handleMenuKeydown}
   288|      onclick={(e) => e.stopPropagation()}
   289|    >
   290|      <div class="menu-header">
   291|        <h2>Menu Nook</h2>
   292|        <button onclick={closeMenu} class="close-menu" aria-label="Fermer le menu">✕</button>
   293|      </div>
   294|
   295|      <ul class="nav-list">
   296|        {#each navItems as item (item.path)}
   297|          {#if item.requiresAuth && !authStore.isAuthenticated}
   298|            <!-- Skip -->
   299|          {:else if item.requiresAdmin && !authStore.isAdmin}
   300|            <!-- Skip -->
   301|          {:else}
   302|            <li>
   303|              <a href={item.path} onclick={closeMenu} class:active={$page.url.pathname.startsWith(item.path)}>
   304|                {#if item.icon}<Icon name={item.icon} size="18" /> {/if}{item.label}
   305|                {#if item.path === '/chat' && totalUnread > 0}
   306|                  <span class="nav-badge">{totalUnread > 99 ? '99+' : totalUnread}</span>
   307|                {/if}
   308|              </a>
   309|            </li>
   310|          {/if}
   311|        {/each}
   312|      </ul>
   313|
   314|      <div class="menu-footer">
   315|        <p class="version">Nook v0.5 • Svelte 5 + Rust</p>
   316|        {#if authStore.isAuthenticated}
   317|          <button onclick={handleLogout} class="logout-link" aria-label="Déconnexion">
   318|            <Icon name="logout" size="18" /> Déconnexion
   319|          </button>
   320|        {/if}
   321|      </div>
   322|    </div>
   323|  {/if}
   324|
   325|  <main class="app-main">
   326|    {@render children()}
   327|  </main>
   328|
   329|  <footer class="app-footer">
   330|    <p>© {new Date().getFullYear()} Nook • Messagerie privée pour la famille</p>
   331|  </footer>
   332|{/if}
   333|
   334|<style>
   335|  :global(:root) {
   336|    --header-h: 60px; /* défaut, écrasé dynamiquement par ResizeObserver */
   337|  }
   338|  :global(body) {
   339|    margin: 0;
   340|    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
   341|    background: var(--bg-primary, #f5f7fa);
   342|    min-height: 100vh;
   343|  }
   344|
   345|  .loading-screen {
   346|    display: flex; flex-direction: column;
   347|    align-items: center; justify-content: center;
   348|    min-height: 100vh; gap: 1rem; padding: 1.5rem;
   349|  }
   350|
   351|  .loading-spinner {
   352|    width: 48px; height: 48px;
   353|    border: 4px solid var(--border, #e2e8f0);
   354|    border-top-color: var(--accent, #4ade80);
   355|    border-radius: 50%;
   356|    animation: spin 1s linear infinite;
   357|  }
   358|
   359|  .crypto-error {
   360|    color: #dc2626; font-size: 0.9rem; text-align: center;
   361|    max-width: 400px; padding: 0.5rem;
   362|    background: rgba(239, 68, 68, 0.1); border-radius: 0.5rem; margin-top: 0.5rem;
   363|  }
   364|
   365|  @keyframes spin { to { transform: rotate(360deg); } }
   366|
   367|  .error-screen {
   368|    display: flex; align-items: center; justify-content: center;
   369|    min-height: 100vh; padding: 1.5rem;
   370|  }
   371|
   372|  .error-content {
   373|    background: var(--bg-secondary, white); padding: 2.5rem; border-radius: 1rem;
   374|    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
   375|    text-align: center; max-width: 450px; width: 100%;
   376|  }
   377|
   378|  .error-content h1 { font-size: 1.5rem; margin: 0 0 0.5rem 0; color: #1e293b; }
   379|  .error-title { color: #64748b; margin: 0 0 1.25rem 0; font-family: monospace; }
   380|  .error-message { color: #dc2626; margin: 0 0 1.5rem 0; line-height: 1.5; font-weight: 500; }
   381|
   382|  .error-details {
   383|    text-align: left; margin: 1rem 0; padding: 0.75rem;
   384|    background: var(--bg-tertiary, #f8fafc); border-radius: 0.5rem; border: 1px solid var(--border, #e2e8f0);
   385|  }
   386|
   387|  .detail-item { color: var(--text-secondary, #64748b); margin: 0.25rem 0; font-size: 0.9rem; }
   388|
   389|  .retry-button {
   390|    padding: 0.75rem 1.5rem; background: var(--accent, #4ade80); color: white;
   391|    border: none; border-radius: 0.5rem; font-size: 1rem; font-weight: 600;
   392|    cursor: pointer; transition: all 0.2s;
   393|    display: inline-flex; align-items: center; gap: 0.5rem;
   394|  }
   395|
   396|  .retry-button:hover { filter: brightness(1.1); transform: translateY(-1px); }
   397|
   398|  .crypto-warning-banner {
   399|    background: #fef3c7; color: #92400e; font-size: 0.85rem;
   400|    padding: 0.5rem 1.5rem; text-align: center;
   401|    border-bottom: 1px solid #fde68a;
   402|  }
   403|
   404|  .app-header {
   405|    display: flex; align-items: center; gap: 0.5rem;
   406|    padding: 0.35rem 0.8rem; background: var(--bg-secondary, white);
   407|    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08); border-bottom: 1px solid var(--border, #e2e8f0);
   408|    position: sticky; top: 0; z-index: 100;
   409|    min-height: 64px;
   410|  }
   411|
   412|  .menu-toggle, .logout-btn {
   413|    background: none; border: none; font-size: 1rem;
   414|    cursor: pointer; padding: 0.25rem; border-radius: 0.4rem; transition: background 0.2s;
   415|  }
   416|
   417|  .menu-toggle:hover, .logout-btn:hover { background: var(--bg-tertiary, #f1f5f9); }
   418|
   419|  .app-header h1 { font-size: 0.9rem; font-weight: 600; margin: 0; display: flex; align-items: center; color: var(--text-primary, #1e293b); flex: 1; }
   420|  .user-name { font-size: 0.75rem; color: var(--text-secondary, #64748b); margin-right: 0.25rem; }
   421|
   422|  .menu-overlay {
   423|    position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5);
   424|    z-index: 200; cursor: pointer; border: none; padding: 0; margin: 0;
   425|  }
   426|
   427|  .menu {
   428|    position: fixed; top: 0; left: 0; bottom: 0; width: 300px;
   429|    max-width: 85vw; background: var(--bg-secondary, white); z-index: 201;
   430|    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.15);
   431|    display: flex; flex-direction: column;
   432|    animation: slideIn 0.25s ease-out;
   433|  }
   434|
   435|  @keyframes slideIn {
   436|    from { transform: translateX(-100%); }
   437|    to   { transform: translateX(0); }
   438|  }
   439|
   440|  .menu-header {
   441|    display: flex; justify-content: space-between; align-items: center;
   442|    padding: 1.25rem 1.5rem; border-bottom: 1px solid var(--border, #e2e8f0);
   443|  }
   444|
   445|  .menu-header h2 { font-size: 1.1rem; font-weight: 600; margin: 0; color: var(--text-primary, #1e293b); }
   446|
   447|  .close-menu {
   448|    background: none; border: none; font-size: 1.5rem; cursor: pointer;
   449|    padding: 0.5rem; border-radius: 0.5rem; color: var(--text-secondary, #64748b); transition: all 0.2s;
   450|  }
   451|
   452|  .close-menu:hover { background: var(--bg-tertiary, #f1f5f9); color: var(--text-primary, #1e293b); }
   453|
   454|  .nav-list { list-style: none; margin: 0; padding: 1rem 0; flex: 1; overflow-y: auto; }
   455|
   456|  .nav-list li a {
   457|    display: flex; align-items: center; gap: .5rem;
   458|    padding: 0.85rem 1.5rem;
   459|    color: var(--text-primary, #334155); text-decoration: none; transition: all 0.2s; font-size: 1rem;
   460|    border-radius: .5rem; margin: 0 .35rem;
   461|  }
   462|
   463|  .nav-list li a:hover, .nav-list li a.active { background: var(--bg-tertiary, #f1f5f9); color: var(--text-primary, #1e293b); }
   464|
   465|  .nav-badge {
   466|    margin-left: auto;
   467|    display: inline-flex; align-items: center; justify-content: center;
   468|    background: var(--accent, #4ade80); color: #166534;
   469|    font-size: .65rem; font-weight: 700; border-radius: 999px;
   470|    min-width: 1.25rem; height: 1.25rem; padding: 0 .3rem;
   471|  }
   472|
   473|  .menu-footer { padding: 1.25rem 1.5rem; border-top: 1px solid var(--border, #e2e8f0); }
   474|  .version { font-size: 0.8rem; color: var(--text-muted, #94a3b8); margin: 0 0 0.75rem 0; }
   475|
   476|  .logout-link {
   477|    display: flex; align-items: center; gap: 0.5rem; width: 100%;
   478|    padding: 0.75rem 1rem; background: none; border: 1px solid var(--border, #e2e8f0);
   479|    border-radius: 0.5rem; color: var(--text-secondary, #64748b); font-size: 0.95rem;
   480|    cursor: pointer; transition: all 0.2s;
   481|  }
   482|
   483|  .logout-link:hover { background: #fef2f2; border-color: #fecaca; color: #dc2626; }
   484|
   485|        .app-main { flex: 1; display: flex; flex-direction: column; padding: 0; background: var(--bg-primary, #f5f7fa); min-height: 0; }
   486|
   487|  .app-footer {
   488|    text-align: center; padding: 1.25rem; color: var(--text-secondary, #64748b);
   489|    font-size: 0.85rem; border-top: 1px solid var(--border, #e2e8f0); background: var(--bg-secondary, white);
   490|    flex-shrink: 0;
   491|  }
   492|
   493|  .app-footer p { margin: 0; }
   494|
   495|  @media (max-width: 640px) {
   496|  .app-header { padding: 0.3rem 0.6rem; min-height: 44px; }
   497|  .app-header h1 { font-size: 0.85rem; }
   498|    .app-main { padding: 0; }
   499|    .menu { width: 85vw; max-width: none; left: 0; right: auto; }
   500|    .error-content { padding: 1.5rem; }
   501|