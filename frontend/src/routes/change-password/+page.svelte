<script lang="ts">
  import { goto } from '$app/navigation';
  import PasswordInput from '$lib/components/PasswordInput.svelte';
  import { authStore } from '$lib/authStore.svelte.js';
  import { onMount } from 'svelte';
  import { unlockCrypto, cryptoStore } from '$lib/cryptoStore.svelte.ts';

  // Flux E2EE pour le changement de mot de passe obligatoire (admin ou invité) :
  //   1. Changement de mot de passe validé côté serveur
  //   2. unlockCrypto(userId, newPassword) :
  //        → si aucune clé en IndexedDB → génération initiale (nouveau compte)
  //        → si clés existantes → déchiffrement (cas re-chiffrement après reset)
  //   3. La clé publique est enregistrée sur le serveur
  //   4. La clé privée est chiffrée avec le NOUVEAU mot de passe et stockée en IndexedDB
  //
  // Pourquoi newPassword et non l'ancien ?
  //   Le chiffrement de la clé privée utilise le mot de passe comme dérivation de clé (Argon2).
  //   On chiffre TOUJOURS avec le mot de passe actif, jamais avec l'ancien mot de passe temporaire.
  //   Ici c'est la première génération → aucune clé existante → newPassword est le seul mot de passe.

  let newPassword     = $state('');
  let confirmPassword = $state('');
  let error           = $state('');
  let success         = $state('');
  let isLoading       = $state(false);

  let e2eeStatus      = $state<'idle' | 'generating' | 'done' | 'error'>('idle');

  onMount(() => {
    if (!authStore.isAuthenticated) {
      goto('/login');
      return;
    }
    if (!authStore.needsPasswordChange) {
      goto('/chat');
    }
  });

  async function handleSubmit() {
    error   = '';
    success = '';

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
      const userId = authStore.user?.id;
      if (!userId) throw new Error('Utilisateur non identifié');

      // ── Étape 1 : Changer le mot de passe côté serveur ───────────────────
      const response = await fetch('/api/auth/change-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ new_password: newPassword, user_id: userId }),
      });

      const raw = await response.text();
      let payload: any = {};
      if (raw.trim()) {
        try { payload = JSON.parse(raw); }
        catch { payload.message = raw; }
      }

      if (!response.ok) {
        error = payload.message ?? `Erreur ${response.status}`;
        return;
      }

      // ── Étape 2 : Générer et enregistrer les clés E2EE ───────────────────
      // unlockCrypto avec le NOUVEAU mot de passe :
      //   • Cherche les clés dans IndexedDB → absentes (premier login)
      //   • Génère une paire Curve25519
      //   • Chiffre la clé privée avec newPassword (Argon2 + XSalsa20)
      //   • Stocke dans IndexedDB
      //   • Envoie la clé publique à /api/e2ee/register-key
      e2eeStatus = 'generating';
      const e2eeOk = await unlockCrypto(userId, newPassword);

      if (!e2eeOk) {
        // Echec E2EE non bloquant : on laisse continuer mais on prévient
        console.error('[change-password] E2EE init failed:', cryptoStore.error);
        e2eeStatus = 'error';
        // On ne bloque pas — le changement de mot de passe a réussi,
        // l'E2EE sera réessayé au prochain login
      } else {
        e2eeStatus = 'done';
      }

      // ── Étape 3 : Finaliser ──────────────────────────────────────────────
      success = payload.message ?? 'Mot de passe mis à jour avec succès !';
      authStore.updateUser({ needs_password_change: false });

      setTimeout(async () => {
        // Logout and redirect to login page
      await authStore.logout();
      goto('/login');
      }, 2000);

    } catch (e: any) {
      error = e?.message ?? 'Une erreur est survenue.';
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>{authStore.needsPasswordChange ? 'Définir' : 'Changer'} votre mot de passe — Nook</title>
</svelte:head>

<div class="page-container">
  <div class="card">
    <div class="header">
      <div class="icon">🔐</div>
      <h1>{authStore.needsPasswordChange ? 'Première connexion' : 'Changer le mot de passe'}</h1>
      <p class="description">
        {authStore.needsPasswordChange
          ? 'Pour des raisons de sécurité, définissez un nouveau mot de passe. Vos clés de chiffrement seront générées automatiquement.'
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
      {#if e2eeStatus === 'done'}
        <div class="e2ee-badge">
          <span>🔒</span>
          <span>Chiffrement de bout en bout activé</span>
        </div>
      {:else if e2eeStatus === 'error'}
        <div class="e2ee-badge warn">
          <span>⚠️</span>
          <span>Chiffrement activé au prochain login</span>
        </div>
      {/if}
      <p class="info-text">Redirection en cours…</p>
    {:else}
      <form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
        <div class="input-group">
          <label for="new-password">Nouveau mot de passe</label>
          <PasswordInput id="new-password" bind:value={newPassword} placeholder="Au moins 8 caractères" autocomplete="new-password" required disabled={isLoading} />
          <p class="help-text">Utilisez lettres, chiffres et symboles pour plus de sécurité.</p>
        </div>
        <div class="input-group">
          <label for="confirm-password">Confirmer le mot de passe</label>
          <PasswordInput id="confirm-password" bind:value={confirmPassword} placeholder="Répétez le mot de passe" autocomplete="new-password" required disabled={isLoading} />
        </div>

        {#if isLoading}
          <div class="progress-steps">
            <div class="step" class:active={isLoading}>
              <span class="spinner-sm"></span>
              {#if e2eeStatus === 'generating'}
                Génération des clés de chiffrement…
              {:else}
                Enregistrement du mot de passe…
              {/if}
            </div>
          </div>
        {/if}

        <button type="submit" class="btn-primary" disabled={isLoading}>
          {#if isLoading}
            <span class="spinner"></span>
            En cours…
          {:else}
            {authStore.needsPasswordChange ? 'Définir le mot de passe' : 'Changer le mot de passe'}
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
  .page-container { min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 1.5rem; background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%); }
  .card { background: white; padding: 2.5rem; border-radius: 1.5rem; box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1); width: 100%; max-width: 420px; text-align: center; }
  .header { margin-bottom: 2rem; }
  .icon { font-size: 3.5rem; margin-bottom: 1rem; }
  h1 { font-size: 1.75rem; font-weight: 700; color: #1e293b; margin: 0 0 0.75rem 0; }
  .description { font-size: 0.95rem; color: #64748b; line-height: 1.5; margin: 0; }

  .alert { display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border-radius: 0.75rem; margin-bottom: 1rem; text-align: left; font-size: 0.9rem; }
  .alert.error   { background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); color: #dc2626; }
  .alert.success { background: rgba(74, 222, 128, 0.1); border: 1px solid rgba(74, 222, 128, 0.3); color: #22c55e; }
  .alert-icon { font-size: 1.25rem; flex-shrink: 0; }

  .e2ee-badge {
    display: flex; align-items: center; justify-content: center; gap: 0.5rem;
    padding: 0.5rem 1rem; border-radius: 0.5rem; margin-bottom: 0.75rem;
    font-size: 0.85rem; font-weight: 500;
    background: rgba(74, 222, 128, 0.1); border: 1px solid rgba(74, 222, 128, 0.3);
    color: #16a34a;
  }
  .e2ee-badge.warn {
    background: rgba(234, 179, 8, 0.1); border-color: rgba(234, 179, 8, 0.3);
    color: #ca8a04;
  }

  .info-text { text-align: center; color: #64748b; font-size: 0.9rem; margin-top: 0.5rem; }

  .form { display: flex; flex-direction: column; gap: 1.5rem; }
  .input-group { text-align: left; }
  label { display: block; margin-bottom: 0.5rem; font-weight: 600; color: #374151; font-size: 0.95rem; }
  input { width: 100%; padding: 0.875rem 1rem; font-size: 1rem; background: #f8fafc; border: 2px solid #e2e8f0; border-radius: 0.75rem; transition: all 0.2s; outline: none; box-sizing: border-box; }
  input:focus { border-color: #2d5a27; box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2); }
  input:disabled { opacity: 0.6; cursor: not-allowed; }
  .help-text { font-size: 0.8rem; color: #64748b; margin: 0.5rem 0 0 0; }

  .progress-steps {
    padding: 0.75rem 1rem;
    background: rgba(59, 130, 246, 0.05);
    border: 1px solid rgba(59, 130, 246, 0.2);
    border-radius: 0.75rem;
    font-size: 0.875rem;
    color: #3b82f6;
  }
  .step { display: flex; align-items: center; gap: 0.5rem; }
  .step.active { font-weight: 500; }

  .spinner-sm {
    width: 14px; height: 14px; flex-shrink: 0;
    border: 2px solid rgba(59, 130, 246, 0.3);
    border-top-color: #3b82f6;
    border-radius: 50%; animation: spin 0.8s linear infinite;
  }

  .btn-primary { width: 100%; padding: 1rem; background: #2d5a27; color: white; border: none; border-radius: 0.75rem; font-size: 1.1rem; font-weight: 600; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; justify-content: center; gap: 0.75rem; }
  .btn-primary:hover:not(:disabled) { background: #3d7a37; transform: translateY(-1px); }
  .btn-primary:disabled { opacity: 0.7; cursor: not-allowed; }

  .spinner { width: 20px; height: 20px; border: 2px solid rgba(255, 255, 255, 0.3); border-top-color: white; border-radius: 50%; animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .footer { margin-top: 2rem; padding-top: 1rem; border-top: 1px solid #e2e8f0; }
  .back-link { color: #64748b; font-size: 0.9rem; text-decoration: none; transition: color 0.2s; }
  .back-link:hover { color: #2d5a27; }

  @media (max-width: 480px) {
    .card { padding: 2rem 1.5rem; }
    .icon { font-size: 3rem; }
    h1 { font-size: 1.5rem; }
  }

</style>
