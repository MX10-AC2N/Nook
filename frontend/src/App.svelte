<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount } from 'svelte';
  import { Router } from '@roxi/routify';
  import { authStore, logout } from '$lib/authStore';
  import { currentTheme } from '$lib/themeStore';
  import { connectionError } from '$lib/chatStore';
  import { loadConversations } from '$lib/conversationStore';
  import { initCrypto } from '$lib/crypto';
  import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';
  import ThemeInjector from '$lib/components/ThemeInjector.svelte';
  
  // États réactifs (runes mode obligatoire)
  let loading = $state(true);
  let appError = $state(null);
  let showMenu = $state(false);

  // Initialiser au chargement
  async function initApp() {
    try {
      loading = true;
      appError = null;
      
      // Initialiser le chiffrement
      await initCrypto();
      
      // Charger les conversations si authentifié
      const user = $authStore.user;
      if (user) {
        await loadConversations();
      }
      
      loading = false;
    } catch (err) {
      appError = err.message || 'Erreur lors de l\'initialisation';
      console.error('Erreur initialisation app:', err);
      loading = false;
    }
  }

  // Réagir aux changements d'authentification (remplace la déclaration $: legacy)
  $effect(() => {
    if ($authStore.isAuthenticated !== undefined) {
      initApp();
    }
  });

  // Gestion des erreurs globales
  function handleGlobalError(error) {
    appError = error.message || 'Erreur système';
    setTimeout(() => {
      appError = null;
    }, 5000);
  }

  // Gestion du menu mobile
  function toggleMenu() {
    showMenu = !showMenu;
  }

  function closeMenu() {
    showMenu = false;
  }

  // Gestion de la déconnexion
  async function handleLogout() {
    try {
      await logout();
      closeMenu();
    } catch (err) {
      handleGlobalError(err);
    }
  }

  // Gestion des raccourcis clavier
  function handleKeydown(e) {
    // Échap pour fermer le menu
    if (e.key === 'Escape') {
      closeMenu();
    }
    // Ctrl+L ou Cmd+L pour déconnexion
    if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
      e.preventDefault();
      handleLogout();
    }
  }

  // Ajouter les écouteurs globaux
  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('error', (e) => handleGlobalError(e.error));
    window.addEventListener('unhandledrejection', (e) => handleGlobalError(e.reason));
    
    return () => {
      window.removeEventListener('keydown', handleKeydown);
      window.removeEventListener('error', (e) => handleGlobalError(e.error));
      window.removeEventListener('unhandledrejection', (e) => handleGlobalError(e.reason));
    };
  });
</script>

<ThemeInjector />

<div class="app-container theme-{$currentTheme}">
  {#if loading}
    <div class="loading-overlay">
      <LoadingSpinner size="large" />
      <p>Chargement de Nook...</p>
    </div>
  {:else if appError}
    <div class="error-overlay">
      <div class="error-content">
        <h2>❌ Erreur système</h2>
        <p>{appError}</p>
        <button onclick={() => location.reload()} class="retry-button">
          🔄 Recharger l'application
        </button>
      </div>
    </div>
  {:else}
    <header class="app-header">
      <div class="header-content">
        <button class="menu-toggle" onclick={toggleMenu} aria-label="Menu">
          ☰
        </button>

        <div class="logo">
          <span class="logo-icon">🌱</span>
          <span class="logo-text">Nook</span>
        </div>

        <div class="header-actions">
          {#if $authStore.isAuthenticated}
            <div class="user-info">
              <span>{$authStore.user?.name}</span>
              <button onclick={handleLogout} class="logout-button" title="Déconnexion (Ctrl+L)">
                🔌
              </button>
            </div>
          {/if}
        </div>
      </div>
    </header>

    <div class="app-content">
      {#if showMenu && $authStore.isAuthenticated}
        <div class="side-menu-overlay" onclick={closeMenu}></div>
        <nav class="side-menu">
          <div class="menu-header">
            <h3>Menu Nook</h3>
            <button class="close-menu" onclick={closeMenu} aria-label="Fermer le menu">
              ✕
            </button>
          </div>

          <ul class="menu-items">
            <li>
              <a href="/chat" onclick={closeMenu}>
                <span>💬</span> Chat
              </a>
            </li>
            <li>
              <a href="/admin" onclick={closeMenu}>
                <span>👑</span> Administration
              </a>
            </li>
            <li>
              <a href="/settings" onclick={closeMenu}>
                <span>⚙️</span> Paramètres
              </a>
            </li>
            <li>
              <a href="/help" onclick={closeMenu}>
                <span>❓</span> Aide
              </a>
            </li>
          </ul>

          <div class="menu-footer">
            <p>Version 3.0 • Système simplifié</p>
            <button onclick={handleLogout} class="menu-logout">
              🔌 Déconnexion
            </button>
          </div>
        </nav>
      {/if}

      <main class="main-content">
        {#if $connectionError}
          <div class="connection-error">
            <p>{$connectionError}</p>
            <button onclick={() => connectionError.set(null)} class="error-dismiss">
              ✕
            </button>
          </div>
        {/if}

        <!-- Routify 3 : le router rend directement les routes (+page.svelte, +layout.svelte, etc.) -->
        <Router />
      </main>
    </div>
  {/if}

  <footer class="app-footer">
    <p>© {new Date().getFullYear()} Nook • Messagerie privée pour la famille</p>
    <div class="theme-indicator">
      {#if $currentTheme === 'jardin-secret'}
        <span>🌿 Jardin Secret</span>
      {:else if $currentTheme === 'space-hub'}
        <span>🚀 Space Hub</span>
      {:else}
        <span>🏠 Maison Chaleureuse</span>
      {/if}
    </div>
  </footer>
</div>

<style>
  /* Ton style reste exactement identique – rien à changer */
  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    background: var(--bg-color);
    color: var(--text-color);
    overflow-x: hidden;
  }

  .app-container {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    position: relative;
  }

  .loading-overlay, .error-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(255, 255, 255, 0.95);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    backdrop-filter: blur(5px);
  }

  .loading-overlay p {
    margin-top: 1rem;
    font-size: 1.2rem;
    color: var(--primary);
  }

  .error-content {
    text-align: center;
    padding: 2rem;
    background: white;
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
  }

  .error-content h2 {
    color: #f44336;
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
    margin-top: 1rem;
    transition: all 0.2s;
  }

  .retry-button:hover {
    background: #43a047;
    transform: translateY(-1px);
  }

  .app-header {
    background: var(--header-bg);
    border-bottom: 1px solid var(--border-color);
    padding: 0.75rem 1rem;
    position: sticky;
    top: 0;
    z-index: 100;
    box-shadow: 0 2px 8px rgba(0,0,0,0.05);
  }

  .header-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }

  .menu-toggle {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--text-color);
    padding: 0.5rem;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .menu-toggle:hover {
    background: var(--hover-bg);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: bold;
    font-size: 1.25rem;
  }

  .logo-icon {
    font-size: 1.5rem;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .logout-button {
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    color: var(--text-color);
    padding: 0.5rem;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .logout-button:hover {
    background: var(--hover-bg);
    color: #f44336;
  }

  .app-content {
    display: flex;
    flex: 1;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
    padding: 1rem;
    gap: 1rem;
  }

  .side-menu-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0,0,0,0.5);
    z-index: 90;
  }

  .side-menu {
    position: fixed;
    left: 0;
    top: 0;
    bottom: 0;
    width: 280px;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border-color);
    padding: 1rem;
    z-index: 95;
    transform: translateX(0);
    transition: transform 0.3s ease;
    overflow-y: auto;
  }

  .menu-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border-color);
    margin-bottom: 1rem;
  }

  .close-menu {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--text-color);
    padding: 0.25rem;
  }

  .menu-items {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .menu-items li {
    margin-bottom: 0.5rem;
  }

  .menu-items a {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 12px;
    text-decoration: none;
    color: var(--text-color);
    transition: all 0.2s;
    font-weight: 500;
  }

  .menu-items a:hover {
    background: var(--hover-bg);
    transform: translateX(2px);
  }

  .menu-footer {
    margin-top: 2rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .menu-logout {
    background: #f44336;
    color: white;
    border: none;
    padding: 0.5rem;
    border-radius: 8px;
    width: 100%;
    cursor: pointer;
    margin-top: 0.5rem;
    transition: all 0.2s;
  }

  .menu-logout:hover {
    background: #d32f2f;
  }

  .main-content {
    flex: 1;
    min-height: calc(100vh - 120px);
    padding: 1rem;
    position: relative;
  }

  .connection-error {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: #ffebee;
    color: #c62828;
    padding: 0.75rem 1.5rem;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    display: flex;
    align-items: center;
    gap: 1rem;
    z-index: 1000;
    animation: slideUp 0.3s ease-out;
  }

  @keyframes slideUp {
    from { transform: translateX(-50%) translateY(20px); opacity: 0; }
    to { transform: translateX(-50%) translateY(0); opacity: 1; }
  }

  .error-dismiss {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    color: #c62828;
    padding: 0.25rem;
    border-radius: 50%;
  }

  .app-footer {
    background: var(--footer-bg);
    border-top: 1px solid var(--border-color);
    padding: 1rem;
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.9rem;
    margin-top: auto;
  }

  .theme-indicator {
    display: inline-block;
    margin-top: 0.25rem;
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-weight: 500;
  }

  /* Thèmes */
  .theme-jardin-secret {
    --primary: #4CAF50;
    --primary-dark: #388E3C;
    --primary-light: #E8F5E9;
    --secondary: #8BC34A;
    --bg-color: #F8FDF8;
    --text-color: #333333;
    --text-secondary: #666666;
    --header-bg: #F0F7F0;
    --footer-bg: #F8FDF8;
    --sidebar-bg: #FFFFFF;
    --border-color: #C8E6C9;
    --hover-bg: #E8F5E8;
  }

  .theme-space-hub {
    --primary: #2196F3;
    --primary-dark: #1976D2;
    --primary-light: #E3F2FD;
    --secondary: #3F51B5;
    --bg-color: #F5FAFF;
    --text-color: #333333;
    --text-secondary: #666666;
    --header-bg: #E3F2FD;
    --footer-bg: #F5FAFF;
    --sidebar-bg: #FFFFFF;
    --border-color: #BBDEFB;
    --hover-bg: #E3F2FD;
  }

  .theme-maison-chaleureuse {
    --primary: #FF9800;
    --primary-dark: #E65100;
    --primary-light: #FFF3E0;
    --secondary: #FF5722;
    --bg-color: #FFF9F5;
    --text-color: #333333;
    --text-secondary: #666666;
    --header-bg: #FFF3E0;
    --footer-bg: #FFF9F5;
    --sidebar-bg: #FFFFFF;
    --border-color: #FFE0B2;
    --hover-bg: #FFF3E0;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .side-menu {
      width: 250px;
    }
    
    .app-content {
      flex-direction: column;
      padding: 0.5rem;
    }
    
    .main-content {
      padding: 0.5rem;
    }
    
    .menu-toggle {
      display: block;
    }
  }

  @media (max-width: 480px) {
    .side-menu {
      width: 220px;
    }
    
    .logo-text {
      display: none;
    }
    
    .user-info span {
      display: none;
    }
    
    .header-actions {
      gap: 0.5rem;
    }
  }
</style>