<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/Icon.svelte';

  // États
  let token = $state('');
  let username = $state('');
  let password = $state('');
  let confirmPassword = $state('');
  let fullName = $state('');
  let loading = $state(false);
  let error = $state('');
  let message = $state('');
  let tokenValid = $state(false);
  let tokenChecked = $state(false);
  let invitationData: any = $state(null);

  onMount(async () => {
    // Récupérer le token de l'URL
    const urlParams = new URLSearchParams(window.location.search);
    token = urlParams.get('token') || '';

    if (!token) {
      error = 'Token d\'invitation manquant.';
      tokenChecked = true;
      return;
    }

    // Valider le token
    await validateToken();
  });

  async function validateToken() {
    try {
      const response = await fetch(`/api/invite/validate?token=${token}`);
      
      if (response.ok) {
        invitationData = await response.json();
        tokenValid = true;
        
        // Pré-remplir le nom si disponible
        if (invitationData.name) {
          fullName = invitationData.name;
        }
      } else {
        const data = await response.json();
        error = data.message || 'Ce lien d\'invitation est invalide ou a expiré.';
      }
    } catch (err) {
      console.error('Erreur validation token:', err);
      error = 'Erreur de connexion au serveur.';
    } finally {
      tokenChecked = true;
    }
  }

  async function handleAcceptInvitation() {
    // Validation
    if (!username.trim()) {
      error = 'Veuillez choisir un identifiant';
      return;
    }
    
    if (!fullName.trim()) {
      error = 'Veuillez indiquer votre nom';
      return;
    }
    
    if (!password || !confirmPassword) {
      error = 'Veuillez remplir les deux champs de mot de passe';
      return;
    }
    
    if (password.length < 8) {
      error = 'Le mot de passe doit contenir au moins 8 caractères';
      return;
    }
    
    if (password !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas';
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await fetch('/api/invite/accept', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          token,
          username: username.trim(),
          name: fullName.trim(),
          password
        })
      });

      const data = await response.json();

      if (response.ok) {
        message = 'Compte créé avec succès ! Redirection vers la page de connexion...';
        
        // Redirection vers le login après 2 secondes
        setTimeout(() => {
          goto('/login?message=Compte créé avec succès. Vous pouvez maintenant vous connecter.');
        }, 2000);
      } else {
        error = data.message || `Erreur ${response.status}`;
      }
    } catch (err) {
      console.error('Erreur acceptation invitation:', err);
      error = 'Erreur de connexion au serveur.';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>~ Nook ~ Accepter l'invitation ~</title>
</svelte:head>

<div class="invite-page">
  <div class="invite-card">
    <div class="logo">
      <Icon name="logo" size={80} />
      <h1>Nook</h1>
    </div>

    {#if !tokenChecked}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Vérification de votre invitation...</p>
      </div>
    {:else if tokenValid}
      <p class="subtitle">
        Vous avez été invité à rejoindre <strong>{invitationData?.familyName || 'une famille'}</strong> sur Nook.
      </p>

      {#if error}
        <div class="alert error" role="alert" aria-live="polite">
          <Icon name="error" size={24} class="alert-icon" />
          <span>{error}</span>
        </div>
      {/if}

      {#if message}
        <div class="alert success" role="alert" aria-live="polite">
          <Icon name="check" size={24} class="alert-icon" />
          <span>{message}</span>
        </div>
      {:else}
        <form onsubmit={(e) => { e.preventDefault(); handleAcceptInvitation(); }} class="invite-form">
          <div class="input-group">
            <label for="fullName">
              <Icon name="user" size={20} />
              Votre nom complet
            </label>
            <input
              id="fullName"
              type="text"
              bind:value={fullName}
              placeholder="Votre nom et prénom"
              required
              disabled={loading}
            />
          </div>

          <div class="input-group">
            <label for="username">
              <Icon name="at-sign" size={20} />
              Identifiant
            </label>
            <input
              id="username"
              type="text"
              bind:value={username}
              placeholder="Choisissez votre identifiant"
              autocomplete="username"
              required
              disabled={loading}
            />
            <p class="help-text">
              Cet identifiant sera utilisé pour vous connecter
            </p>
          </div>

          <div class="input-group">
            <label for="password">
              <Icon name="lock" size={20} />
              Mot de passe
            </label>
            <input
              id="password"
              type="password"
              bind:value={password}
              placeholder="Choisissez un mot de passe sécurisé"
              autocomplete="new-password"
              required
              disabled={loading}
            />
            <p class="help-text">Minimum 8 caractères</p>
          </div>

          <div class="input-group">
            <label for="confirmPassword">
              <Icon name="lock" size={20} />
              Confirmer le mot de passe
            </label>
            <input
              id="confirmPassword"
              type="password"
              bind:value={confirmPassword}
              placeholder="Répétez le mot de passe"
              autocomplete="new-password"
              required
              disabled={loading}
            />
          </div>

          <button type="submit" class="btn-primary" disabled={loading}>
            {#if loading}
              <span class="spinner small"></span>
              Création du compte...
            {:else}
              <Icon name="check-circle" size={24} />
              Créer mon compte
            {/if}
          </button>
        </form>
      {/if}
    {:else}
      <div class="alert error" role="alert" aria-live="polite">
        <Icon name="error" size={24} class="alert-icon" />
        <span>{error || 'Lien d\'invitation invalide'}</span>
      </div>
      
      <div class="actions">
        <p>Ce lien d'invitation est invalide ou a expiré.</p>
        <a href="/" class="btn-secondary">
          <Icon name="home" size={20} />
          Retour à l'accueil
        </a>
      </div>
    {/if}
  </div>
</div>

<style>
  * { box-sizing: border-box; }

  .invite-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
  }

  .invite-card {
    background: white;
    padding: 2rem;
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
    width: 100%;
    max-width: 450px;
    text-align: center;
  }

  .logo { 
    margin-bottom: 1.5rem; 
  }

  h1 { 
    font-size: 2rem; 
    margin: 0; 
    color: #1e293b; 
  }

  .subtitle { 
    color: #64748b; 
    margin-bottom: 2rem; 
    font-size: 1.1rem;
    line-height: 1.4;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem 0;
  }

  .loading-state .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(45, 90, 39, 0.2);
    border-top-color: #2d5a27;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .alert {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 0.75rem;
    margin-bottom: 1.5rem;
    text-align: left;
    font-size: 0.9rem;
  }

  .alert.error {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #dc2626;
  }

  .alert.success {
    background: rgba(72, 187, 120, 0.1);
    border: 1px solid rgba(72, 187, 120, 0.3);
    color: #2d5a27;
  }

  .invite-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    margin-bottom: 1rem;
  }

  .input-group { 
    text-align: left; 
  }

  label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #374151;
    font-size: 0.95rem;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.2s;
  }

  input:focus {
    border-color: #2d5a27;
    box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2);
    outline: none;
  }
  input:focus-visible {
    outline: 2px solid #4f9cf9;
    outline-offset: 2px;
  }

  input:disabled { 
    opacity: 0.6; 
    cursor: not-allowed; 
  }

  .help-text {
    margin: 0.5rem 0 0 0;
    font-size: 0.85rem;
    color: #64748b;
    font-style: italic;
  }

  .btn-primary {
    width: 100%;
    padding: 1rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.75rem;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .btn-primary:hover:not(:disabled) {
    background: #3d7a37;
    transform: translateY(-1px);
  }

  .btn-primary:disabled { 
    opacity: 0.7; 
    cursor: not-allowed; 
  }

  .btn-secondary {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background: #f8fafc;
    color: #374151;
    border: 2px solid #e2e8f0;
    border-radius: 0.75rem;
    text-decoration: none;
    font-weight: 600;
    transition: all 0.2s;
  }

  .btn-secondary:hover {
    background: #f1f5f9;
    border-color: #cbd5e1;
  }

  .spinner.small {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin { 
    to { transform: rotate(360deg); } 
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    align-items: center;
    margin-top: 1rem;
  }

  .actions p {
    color: #64748b;
    margin: 0;
  }

  @media (max-width: 380px) {
    .invite-card { 
      padding: 1.5rem; 
    }
    
    input { 
      padding: 0.65rem; 
    }
  }
</style>