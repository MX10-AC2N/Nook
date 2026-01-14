<script lang="ts">
  import { goto } from '$app/navigation';
  import { authStore, needsPasswordChange } from '$lib/authStore';
  import { onMount } from 'svelte';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let newPassword = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);

  // -----------------------------------------------------------------
  // 2️⃣ Redirection automatique si l'utilisateur n'est pas autorisé
  // -----------------------------------------------------------------
  onMount(() => {
    // Si l'utilisateur n'est pas authentifié → retour à la page login
    if (!$authStore.isAuthenticated) {
      goto('/login');
    }

    // Si l'utilisateur n'a pas besoin de changer son mot de passe → chat
    if ($authStore.isAuthenticated && !$needsPasswordChange) {
      goto('/chat');
    }
  });

  // -----------------------------------------------------------------
  // 3️⃣ Soumission du formulaire
  // -----------------------------------------------------------------
  /**
   * Envoie la requête de changement de mot de passe au backend.
   */
  async function handleSubmit() {
    // Réinitialiser les messages
    error = '';
    success = '';

    // Validation côté client
    if (newPassword !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas.';
      return;
    }
    if (newPassword.length < 8) {
      error = 'Le mot de passe doit contenir au moins 8 caractères.';
      return;
    }

    isLoading = true;

    try {
      // L'identifiant de l'utilisateur provient du store
      const userId = $authStore.user?.id;
      if (!userId) throw new Error('Utilisateur non identifié');

      const response = await fetch('/api/auth/change-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          new_password: newPassword,
          user_id: userId,
        }),
      });

      // Le corps peut être vide → on le traite en texte d'abord
      const raw = await response.text();

      let payload: any = {};
      if (raw.trim()) {
        try {
          payload = JSON.parse(raw);
        } catch {
          // Si le JSON est invalide, on garde le texte brut comme message
          payload.message = raw;
        }
      }

      if (!response.ok) {
        error = payload.message ?? `Erreur ${response.status}`;
        return;
      }

      // Succès
      success = payload.message ?? 'Mot de passe mis à jour avec succès !';
      // Met à jour le store (le backend indique que le flag `needs_password_change` est maintenant `false`)
      if ($authStore.user) {
        $authStore.user.needs_password_change = false;
      }

      // Redirection après un court délai (pour laisser le message s'afficher)
      setTimeout(() => {
        // Rediriger selon le rôle (admin → /admin, sinon → /chat)
        const target = $authStore.user?.role === 'admin' ? '/admin' : '/chat';
        goto(target);
      }, 2000);
    } catch (e: any) {
      error = e?.message ?? 'Une erreur est survenue.';
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>{$needsPasswordChange ? 'Définir' : 'Changer'} votre mot de passe — Nook</title>
</svelte:head>

<div class="page-container">
  <div class="card">
    <div class="header">
      <div class="icon">🔐</div>
      <h1>{$needsPasswordChange ? 'Première connexion' : 'Changer le mot de passe'}</h1>
      <p class="description">
        {$needsPasswordChange
          ? 'Pour des raisons de sécurité, vous devez définir un nouveau mot de passe avant de continuer.'
          : 'Choisissez un mot de passe fort et unique pour protéger votre compte.'}
      </p>
    </div>

    {#if error}
      <div class="alert error" role="alert" aria-live="polite">
        <span class="alert-icon">⚠️</span>
        <span>{error}</span>
      </div>
    {/if}

    {#if success}
      <div class="alert success" role="alert" aria-live="polite">
        <span class="alert-icon">✅</span>
        <span>{success}</span>
      </div>
      <p class="info-text">Redirection en cours…</p>
    {:else}
      <form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
        <div class="input-group">
          <label for="new-password">Nouveau mot de passe</label>
          <input
            id="new-password"
            type="password"
            bind:value={newPassword}
            placeholder="Au moins 8 caractères"
            required
            disabled={isLoading}
            autocomplete="new-password"
          />
          <p class="help-text">Utilisez lettres, chiffres et symboles pour plus de sécurité.</p>
        </div>

        <div class="input-group">
          <label for="confirm-password">Confirmer le mot de passe</label>
          <input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            placeholder="Répétez le mot de passe"
            required
            disabled={isLoading}
            autocomplete="new-password"
          />
        </div>

        <button type="submit" class="btn-primary" disabled={isLoading}>
          {#if isLoading}
            <span class="spinner"></span>
            Enregistrement…
          {:else}
            {$needsPasswordChange ? 'Définir le mot de passe' : 'Changer le mot de passe'}
          {/if}
        </button>
      </form>
    {/if}

    <div class="footer">
      <a href="/login" class="back-link">← Retour à la connexion</a>
    </div>
  </div>
</div>

<style>
  /* -----------------------------------------------------------------
     PAGE LAYOUT
     ----------------------------------------------------------------- */
  .page-container {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
  }

  .card {
    background: white;
    padding: 2.5rem;
    border-radius: 1.5rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
    width: 100%;
    max-width: 420px;
    text-align: center;
  }

  .header {
    margin-bottom: 2rem;
  }

  .icon {
    font-size: 3.5rem;
    margin-bottom: 1rem;
  }

  h1 {
    font-size: 1.75rem;
    font-weight: 700;
    color: #1e293b;
    margin: 0 0 0.75rem 0;
  }

  .description {
    font-size: 0.95rem;
    color: #64748b;
    line-height: 1.5;
    margin: 0;
  }

  /* -----------------------------------------------------------------
     ALERTS
     ----------------------------------------------------------------- */
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
    background: rgba(74, 222, 128, 0.1);
    border: 1px solid rgba(74, 222, 128, 0.3);
    color: #22c55e;
  }

  .alert-icon {
    font-size: 1.25rem;
  }

  .info-text {
    text-align: center;
    color: #64748b;
    font-size: 0.9rem;
    margin-top: 1rem;
  }

  /* -----------------------------------------------------------------
     FORM
     ----------------------------------------------------------------- */
  .form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .input-group {
    text-align: left;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #374151;
    font-size: 0.95rem;
  }

  input {
    width: 100%;
    padding: 0.875rem 1rem;
    font-size: 1rem;
    background: #f8fafc;
    border: 2px solid #e2e8f0;
    border-radius: 0.75rem;
    transition: all 0.2s;
    outline: none;
  }

  input:focus {
    border-color: #2d5a27;
    box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2);
  }

  input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .help-text {
    font-size: 0.8rem;
    color: #64748b;
    margin: 0.5rem 0 0 0;
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
  }

  .btn-primary:hover:not(:disabled) {
    background: #3d7a37;
    transform: translateY(-1px);
  }

  .btn-primary:disabled {
    opacity: 0.7;
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
    to {
      transform: rotate(360deg);
    }
  }

  /* -----------------------------------------------------------------
     FOOTER
     ----------------------------------------------------------------- */
  .footer {
    margin-top: 2rem;
    padding-top: 1rem;
    border-top: 1px solid #e2e8f0;
  }

  .back-link {
    color: #64748b;
    font-size: 0.9rem;
    text-decoration: none;
    transition: color 0.2s;
  }

  .back-link:hover {
    color: #2d5a27;
  }

  /* -----------------------------------------------------------------
     RESPONSIVE
     ----------------------------------------------------------------- */
  @media (max-width: 480px) {
    .card {
      padding: 2rem 1.5rem;
    }

    .icon {
      font-size: 3rem;
    }

    h1 {
      font-size: 1.5rem;
    }
  }
</style>