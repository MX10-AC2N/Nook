<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { getCurrentTheme } from '$lib/ui/ThemeStore.svelte.ts';
  import { generateKeyPair, storePendingKeys } from '$lib/crypto';

  let token = $state('');
  let name = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);
  let memberId = $state('');

  onMount(() => {
    if (!browser) return;
    
    const urlParams = new URLSearchParams(window.location.search);
    token = urlParams.get('token') || '';
    
    if (!token) {
      error = 'Lien d\'invitation invalide. Demandez un nouveau lien à l\'administrateur.';
    }
  });

  async function submitRequest() {
  if (!name.trim()) {
    error = 'Veuillez entrer votre prénom';
    return;
  }

  if (name.trim().length < 2) {
    error = 'Le prénom doit contenir au moins 2 caractères';
    return;
  }

  isLoading = true;
  error = '';

  try {
    console.log('Génération des clés cryptographiques...');
    const keyPair = await generateKeyPair();
    console.log('Clés générées');

    const response = await fetch(`/api/join?token=${encodeURIComponent(token)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name: name.trim(),
        public_key: keyPair.publicKey
      })
    });

    if (response.ok) {
      const data = await response.json();
      success = data.message;
      
      const match = data.message.match(/ID: (\S+)/);
      if (match && match[1]) {
        memberId = match[1];
        
        await storePendingKeys(memberId, keyPair.publicKey, keyPair.privateKey);
        console.log('Clés pending stockées pour le membre:', memberId);
      }
      
      error = '';  // ← Une seule fois, à la fin du succès
    } else if (response.status === 400) {
      error = 'Lien d\'invitation invalide ou expiré.';
    } else if (response.status === 500) {
      error = 'Erreur serveur. Veuillez réessayer plus tard.';
    } else {
      error = 'Erreur inattendue. Code: ' + response.status;
    }
  } catch (err) {
    console.error('Erreur:', err);
    error = 'Impossible de contacter le serveur. Vérifiez votre connexion.';
  } finally {
    isLoading = false;
  }
}

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !isLoading) {
      submitRequest();
    }
  }
</script>

<svelte:head>
  <title>Rejoindre Nook</title>
</svelte:head>

<div class="page-container theme-{getCurrentTheme}">
  <div class="card">
    <div class="theme-icon">
      {#if getCurrentTheme === 'jardin-secret'}
        🌿
      {:else if getCurrentTheme === 'space-hub'}
        🚀
      {:else}
        🏠
      {/if}
    </div>

    <h1>Rejoindre Nook</h1>

    <p class="description">
      Vous avez été invité à rejoindre un espace familial privé et sécurisé
    </p>

    <div class="security-badge">
      <span class="security-icon">🔒</span>
      <span>Connexion chiffrée de bout en bout</span>
    </div>

    {#if error}
      <div class="alert error">
        <span>⚠️</span>
        <span>{error}</span>
      </div>
    {/if}

    {#if success}
      <div class="alert success">
        <div class="success-icon">✅</div>
        <p class="success-message">{success}</p>
        <p class="success-subtext">L'administrateur vous approuvera bientôt.</p>

        {#if memberId}
          <div class="member-id-box">
            <p class="member-id-label">Votre ID:</p>
            <p class="member-id-value">{memberId}</p>
            <p class="member-id-help">Conservez cet ID pour votre première connexion !</p>
          </div>
        {/if}

        <div class="success-actions">
          <a href="/" class="btn-primary">
            Retour à l'accueil
          </a>
        </div>
      </div>
    {:else if token}
      <div class="form-container">
        <div class="form-group">
          <label for="member-name">Votre prénom</label>
          <input
            id="member-name"
            type="text"
            bind:value={name}
            placeholder="Ex: Jean, Marie, Pierre..."
            class="input"
            onkeydown={handleKeydown}
            disabled={isLoading}
            maxlength="50"
          />
          <p class="help-text">
            Ce nom sera visible par les autres membres de la famille
          </p>
        </div>

        <div class="info-box">
          <p class="info-text">
            <span class="info-icon">⚠️</span>
            <span>Des clés cryptographiques seront générées pour sécuriser vos communications. Ces clés seront stockées localement dans votre navigateur.</span>
          </p>
        </div>

        <button
          onclick={submitRequest}
          disabled={isLoading || !name.trim()}
          class="submit-btn"
        >
          {#if isLoading}
            <span class="spinner"></span>
            <span>Génération des clés de sécurité...</span>
          {:else}
            <span>🔐</span>
            <span>Demander à rejoindre</span>
          {/if}
        </button>
      </div>
    {:else}
      <div class="no-token-box">
        <div class="no-token-icon">🔗</div>
        <p class="no-token-title">Vous avez besoin d'un lien d'invitation</p>
        <p class="no-token-text">Contactez l'administrateur de votre espace familial pour obtenir un lien valide.</p>
      </div>
    {/if}

    <div class="footer">
      <a href="/" class="back-link">
        <span>←</span>
        <span>Retour à l'accueil</span>
      </a>
    </div>
  </div>

  <div class="theme-switcher-container">
    <ThemeSwitcher />
  </div>
</div>

<style>
  .page-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 1.5rem;
    position: relative;
    overflow: hidden;
    background: linear-gradient(135deg, var(--bg-primary, #f0fdf4) 0%, var(--bg-secondary, #e0f2fe) 100%);
  }

  .card {
    max-width: 420px;
    width: 100%;
    padding: 2.5rem;
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.4);
    border-radius: var(--radius-2xl, 1.5rem);
    box-shadow: var(--shadow-2xl, 0 25px 50px -12px rgba(0, 0, 0, 0.25));
    text-align: center;
    animation: fade-in 0.4s ease-out;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .theme-icon {
    font-size: 3.5rem;
    margin-bottom: 1rem;
    animation: bounce 2s ease-in-out infinite;
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-8px); }
  }

  h1 {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
    margin: 0 0 0.5rem 0;
  }

  .description {
    font-size: 0.9rem;
    color: var(--text-secondary, #64748b);
    margin: 0 0 1.5rem 0;
    line-height: 1.5;
  }

  .security-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(74, 222, 128, 0.1);
    border: 1px solid rgba(74, 222, 128, 0.3);
    border-radius: var(--radius-lg, 0.75rem);
    margin-bottom: 1.5rem;
  }

  .security-icon {
    font-size: 1.25rem;
  }

  .security-badge span:last-child {
    font-size: 0.85rem;
    font-weight: 500;
    color: #4ade80;
  }

  .alert {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: var(--radius-lg, 0.75rem);
    margin-bottom: 1rem;
    text-align: left;
    animation: slide-down 0.3s ease;
  }

  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .alert.error {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
  }

  .alert.success {
    background: rgba(74, 222, 128, 0.1);
    border: 1px solid rgba(74, 222, 128, 0.3);
    color: var(--success, #22c55e);
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .success-icon {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  .success-message {
    font-weight: 600;
    font-size: 1.1rem;
    margin: 0;
  }

  .success-subtext {
    font-size: 0.85rem;
    margin: 0.5rem 0 0 0;
    opacity: 0.8;
  }

  .member-id-box {
    margin-top: 1rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.3);
    border-radius: var(--radius-md, 0.5rem);
    width: 100%;
  }

  .member-id-label {
    font-size: 0.75rem;
    color: var(--text-secondary, #64748b);
    margin: 0 0 0.25rem 0;
  }

  .member-id-value {
    font-family: var(--font-mono, monospace);
    font-size: 0.8rem;
    font-weight: 600;
    word-break: break-all;
    margin: 0;
  }

  .member-id-help {
    font-size: 0.7rem;
    color: var(--text-secondary, #64748b);
    margin: 0.5rem 0 0 0;
  }

  .success-actions {
    margin-top: 1.5rem;
  }

  .btn-primary {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    background: var(--accent, #4ade80);
    color: white;
    border-radius: var(--radius-lg, 0.75rem);
    text-decoration: none;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-primary:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .form-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-group {
    text-align: left;
  }

  label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary, #1e293b);
    margin-bottom: 0.5rem;
  }

  .input {
    width: 100%;
    padding: 0.875rem 1rem;
    font-size: 0.9rem;
    background: rgba(255, 255, 255, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.4);
    border-radius: var(--radius-lg, 0.75rem);
    color: var(--text-primary, #1e293b);
    outline: none;
    transition: all 0.2s;
  }

  .input:focus {
    border-color: var(--accent, #4ade80);
    box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.2);
  }

  .input::placeholder {
    color: var(--text-secondary, #64748b);
    opacity: 0.7;
  }

  .input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .help-text {
    font-size: 0.75rem;
    color: var(--text-secondary, #64748b);
    margin: 0.5rem 0 0 0;
  }

  .info-box {
    padding: 0.75rem 1rem;
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.3);
    border-radius: var(--radius-lg, 0.75rem);
  }

  .info-text {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: #3b82f6;
    margin: 0;
  }

  .info-icon {
    flex-shrink: 0;
  }

  .submit-btn {
    width: 100%;
    padding: 1rem 1.5rem;
    background: linear-gradient(135deg, var(--accent, #4ade80), var(--accent-dark, #22c55e));
    color: white;
    border: none;
    border-radius: var(--radius-lg, 0.75rem);
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .submit-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(74, 222, 128, 0.4);
  }

  .submit-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .submit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .no-token-box {
    padding: 1.5rem;
    background: rgba(234, 179, 8, 0.1);
    border: 1px solid rgba(234, 179, 8, 0.3);
    border-radius: var(--radius-lg, 0.75rem);
  }

  .no-token-icon {
    font-size: 2.5rem;
    margin-bottom: 0.75rem;
  }

  .no-token-title {
    font-weight: 600;
    color: #ca8a04;
    margin: 0 0 0.5rem 0;
  }

  .no-token-text {
    font-size: 0.85rem;
    color: var(--text-secondary, #64748b);
    margin: 0;
  }

  .footer {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.3);
  }

  .back-link {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    font-size: 0.9rem;
    color: var(--accent, #4ade80);
    text-decoration: none;
    transition: opacity 0.2s;
  }

  .back-link:hover {
    opacity: 0.8;
  }

  .theme-switcher-container {
    position: absolute;
    bottom: 1.5rem;
    right: 1.5rem;
  }

  @media (max-width: 480px) {
    .page-container {
      padding: 1rem;
    }

    .card {
      padding: 1.5rem;
    }

    h1 {
      font-size: 1.5rem;
    }

    .theme-icon {
      font-size: 3rem;
    }

    .theme-switcher-container {
      position: static;
      margin-top: 1.5rem;
    }
  }
</style>
