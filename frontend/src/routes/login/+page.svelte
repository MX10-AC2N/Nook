<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '@roxi/routify';
  import { authStore } from '$lib/authStore';
  import { currentTheme } from '$lib/themeStore';
  import { login as apiLogin } from '$lib/auth';
  
  // États réactifs
  let username = '';
  let password = '';
  let loading = false;
  let error = null;
  let success = null;
  let lastAttempt = 0;
  
  // Gestion de l'auto-focus
  let usernameInput;
  let passwordInput;
  
  onMount(() => {
    // Focus sur le champ username au chargement
    if (usernameInput) {
      usernameInput.focus();
    }
    
    // Écouter les touches pour navigation clavier
    window.addEventListener('keydown', handleKeydown);
    
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
  
  function handleKeydown(e) {
    // Ctrl+L ou Cmd+L pour focus sur le login
    if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
      e.preventDefault();
      if (usernameInput) usernameInput.focus();
    }
    
    // Ctrl+P ou Cmd+P pour focus sur le password
    if ((e.ctrlKey || e.metaKey) && e.key === 'p') {
      e.preventDefault();
      if (passwordInput) passwordInput.focus();
    }
  }
  
  // Gestion de la soumission du formulaire
  async function handleSubmit() {
    // Empêcher les tentatives trop fréquentes
    const now = Date.now();
    if (now - lastAttempt < 1000) {
      error = 'Veuillez attendre avant de réessayer';
      return;
    }
    
    lastAttempt = now;
    
    if (!username.trim() || !password.trim()) {
      error = 'Veuillez remplir tous les champs';
      return;
    }
    
    try {
      loading = true;
      error = null;
      
      const success = await apiLogin(username.trim(), password.trim());
      
      if (success) {
        // Message de succès temporaire
        this.success = 'Connexion réussie ! Redirection...';
        
        // Rediriger selon le rôle
        setTimeout(() => {
          if ($authStore.isAdmin) {
            goto('/admin');
          } else {
            goto('/chat');
          }
        }, 1000);
      } else {
        error = 'Identifiant ou mot de passe incorrect';
      }
    } catch (err) {
      error = err.message || 'Erreur de connexion au serveur';
      console.error('Erreur login:', err);
    } finally {
      loading = false;
    }
  }
  
  // Gestion de l'appui sur Enter
  function handleKeyDown(e) {
    if (e.key === 'Enter' && !loading) {
      handleSubmit();
    }
  }
  
  // Effacer les messages après un délai
  onDestroy(() => {
    if (error) error = null;
    if (success) success = null;
  });
</script>

<svelte:head>
  <title>Connexion • Nook</title>
</svelte:head>

<div class="login-container theme-{$currentTheme}">
  <div class="login-card">
    <div class="logo-section">
      <div class="logo-icon">🌱</div>
      <h1 class="app-name">Nook</h1>
      <p class="app-tagline">Messagerie privée pour la famille</p>
    </div>
    
    <form on:submit|preventDefault={handleSubmit} class="login-form">
      <div class="form-group">
        <label for="username" class="form-label">Identifiant</label>
        <div class="input-wrapper">
          <span class="input-icon">👤</span>
          <input
            id="username"
            bind:value={username}
            onkeydown={handleKeyDown}
            placeholder="Votre identifiant"
            class="form-input"
            autocomplete="username"
            required
            disabled={loading}
            ref={usernameInput}
          />
        </div>
      </div>
      
      <div class="form-group">
        <label for="password" class="form-label">Mot de passe</label>
        <div class="input-wrapper">
          <span class="input-icon">🔒</span>
          <input
            id="password"
            type="password"
            bind:value={password}
            onkeydown={handleKeyDown}
            placeholder="Votre mot de passe"
            class="form-input"
            autocomplete="current-password"
            required
            disabled={loading}
            ref={passwordInput}
          />
        </div>
      </div>
      
      {#if error}
        <div class="error-message">
          <span>❌ {$error}</span>
        </div>
      {/if}
      
      {#if success}
        <div class="success-message">
          <span>✅ {$success}</span>
        </div>
      {/if}
      
      <button 
        type="submit" 
        class="submit-button"
        disabled={loading || !username.trim() || !password.trim()}
      >
        {loading ? 'Connexion en cours...' : 'Se connecter'}
      </button>
    </form>
    
    <div class="footer-links">
      <a href="/join" class="footer-link">📋 Créer un compte</a>
      <a href="/forgot-password" class="footer-link">❓ Mot de passe oublié</a>
    </div>
    
    <div class="theme-info">
      {#if $currentTheme === 'jardin-secret'}
        <span class="theme-badge jardin">🌿 Jardin Secret</span>
      {:else if $currentTheme === 'space-hub'}
        <span class="theme-badge space">🚀 Space Hub</span>
      {:else}
        <span class="theme-badge maison">🏠 Maison Chaleureuse</span>
      {/if}
    </div>
  </div>
  
  <div class="security-note">
    <p>🔒 Connexion sécurisée - Aucune donnée stockée en clair</p>
    <p>🌐 Open Source & Auto-hébergé</p>
  </div>
</div>

<style>
  .login-container {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    padding: 1rem;
    background: var(--login-bg);
  }

  .login-card {
    background: white;
    border-radius: 24px;
    box-shadow: 0 10px 30px rgba(0,0,0,0.15);
    padding: 2.5rem;
    width: 100%;
    max-width: 450px;
    transition: transform 0.3s ease, box-shadow 0.3s ease;
  }

  .login-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 15px 35px rgba(0,0,0,0.2);
  }

  .logo-section {
    text-align: center;
    margin-bottom: 2rem;
  }

  .logo-icon {
    font-size: 3rem;
    margin-bottom: 0.5rem;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .app-name {
    font-size: 2.2rem;
    font-weight: 800;
    margin: 0;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .app-tagline {
    color: var(--text-secondary);
    margin: 0.5rem 0 0 0;
    font-size: 1.1rem;
  }

  .login-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .form-label {
    font-weight: 600;
    color: var(--text-color);
    font-size: 0.95rem;
  }

  .input-wrapper {
    position: relative;
  }

  .input-icon {
    position: absolute;
    left: 1rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-secondary);
    font-size: 1.1rem;
  }

  .form-input {
    width: 100%;
    padding: 0.85rem 1rem 0.85rem 2.5rem;
    border: 2px solid var(--border-color);
    border-radius: 16px;
    font-size: 1rem;
    transition: all 0.2s;
    background: var(--input-bg);
    color: var(--text-color);
  }

  .form-input:focus {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(var(--primary-rgb), 0.2);
  }

  .form-input:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .error-message, .success-message {
    padding: 0.75rem;
    border-radius: 12px;
    font-weight: 500;
    text-align: center;
    margin: 0.5rem 0;
  }

  .error-message {
    background: #ffebee;
    color: #c62828;
    border: 1px solid #ef9a9a;
  }

  .success-message {
    background: #e8f5e9;
    color: #2e7d32;
    border: 1px solid #a5d6a7;
  }

  .submit-button {
    background: var(--primary);
    color: white;
    border: none;
    padding: 0.9rem;
    border-radius: 16px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    margin-top: 0.25rem;
  }

  .submit-button:hover:not(:disabled) {
    background: var(--primary-dark);
    transform: translateY(-1px);
  }

  .submit-button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    transform: none;
  }

  .footer-links {
    display: flex;
    justify-content: space-between;
    margin-top: 1.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border-color);
  }

  .footer-link {
    color: var(--primary);
    text-decoration: none;
    font-weight: 500;
    transition: all 0.2s;
  }

  .footer-link:hover {
    color: var(--primary-dark);
    text-decoration: underline;
  }

  .theme-info {
    margin-top: 1rem;
    text-align: center;
  }

  .theme-badge {
    display: inline-block;
    padding: 0.35rem 1rem;
    border-radius: 20px;
    font-weight: 600;
    font-size: 0.9rem;
  }

  .jardin { background: rgba(76, 175, 80, 0.15); color: #4CAF50; }
  .space { background: rgba(33, 150, 243, 0.15); color: #2196F3; }
  .maison { background: rgba(255, 152, 0, 0.15); color: #FF9800; }

  .security-note {
    text-align: center;
    margin-top: 2rem;
    color: var(--text-secondary);
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .security-note p {
    margin: 0.25rem 0;
  }

  /* Thèmes */
  .theme-jardin-secret {
    --primary: #4CAF50;
    --primary-dark: #388E3C;
    --secondary: #8BC34A;
    --login-bg: linear-gradient(135deg, #E8F5E9 0%, #F1F8E9 100%);
    --border-color: #C8E6C9;
    --input-bg: #FFFFFF;
    --text-color: #333333;
    --text-secondary: #666666;
    --primary-rgb: 76,175,80;
  }

  .theme-space-hub {
    --primary: #2196F3;
    --primary-dark: #1976D2;
    --secondary: #3F51B5;
    --login-bg: linear-gradient(135deg, #E3F2FD 0%, #BBDEFB 100%);
    --border-color: #BBDEFB;
    --input-bg: #FFFFFF;
    --text-color: #333333;
    --text-secondary: #666666;
    --primary-rgb: 33,150,243;
  }

  .theme-maison-chaleureuse {
    --primary: #FF9800;
    --primary-dark: #E65100;
    --secondary: #FF5722;
    --login-bg: linear-gradient(135deg, #FFF3E0 0%, #FFE0B2 100%);
    --border-color: #FFE0B2;
    --input-bg: #FFFFFF;
    --text-color: #333333;
    --text-secondary: #666666;
    --primary-rgb: 255,152,0;
  }

  /* Responsive */
  @media (max-width: 480px) {
    .login-card {
      padding: 1.5rem;
      margin: 0 1rem;
    }
    
    .app-name {
      font-size: 1.8rem;
    }
    
    .form-input {
      padding: 0.75rem 1rem 0.75rem 2.25rem;
    }
    
    .footer-links {
      flex-direction: column;
      gap: 0.75rem;
    }
    
    .security-note {
      font-size: 0.85rem;
    }
  }
</style>