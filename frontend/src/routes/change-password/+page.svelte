<script lang="ts">
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import { onMount } from 'svelte';
  import {
    getPendingKeys,
    clearPendingKeys,
    encryptPrivateKey,
    storeKeysInIndexedDB,
    registerPublicKeyOnServer,
  } from '$lib/crypto';

  let newPassword     = $state('');
  let confirmPassword = $state('');
  let error           = $state('');
  let success         = $state('');
  let isLoading       = $state(false);

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

      success = payload.message ?? 'Mot de passe mis à jour avec succès !';

      // ── Finalisation E2EE : migrer les clés pending → IndexedDB chiffré ──
      const userId = authStore.user?.id;
      if (userId) {
        const username = authStore.user?.username ?? '';
        const pending = getPendingKeys(username) ?? getPendingKeys(userId);
        if (pending) {
          try {
            const encPrivKey = await encryptPrivateKey(pending.privateKey, newPassword);
            await storeKeysInIndexedDB(userId, pending.publicKey, encPrivKey);
            await registerPublicKeyOnServer(pending.publicKey);
            clearPendingKeys(username);
            clearPendingKeys(userId);
            console.log('[E2EE] Clés migrées vers IndexedDB ✓');
          } catch (e) {
            console.error('[E2EE] Erreur migration clés:', e);
          }
        }
      }
      // ──────────────────────────────────────────────────────────

      // Met à jour le store sans re-login
      authStore.updateUser({ needs_password_change: false });

      setTimeout(() => {
        goto(authStore.user?.role === 'admin' ? '/admin' : '/chat');
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
          <input id="new-password" type="password" bind:value={newPassword}
            placeholder="Au moins 8 caractères" required disabled={isLoading}
            autocomplete="new-password" />
          <p class="help-text">Utilisez lettres, chiffres et symboles pour plus de sécurité.</p>
        </div>
        <div class="input-group">
          <label for="confirm-password">Confirmer le mot de passe</label>
          <input id="confirm-password" type="password" bind:value={confirmPassword}
            placeholder="Répétez le mot de passe" required disabled={isLoading}
            autocomplete="new-password" />
        </div>
        <button type="submit" class="btn-primary" disabled={isLoading}>
          {#if isLoading}
            <span class="spinner"></span>
            Enregistrement…
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

  .alert { display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border-radius: 0.75rem; margin-bottom: 1.5rem; text-align: left; font-size: 0.9rem; }
  .alert.error   { background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); color: #dc2626; }
  .alert.success { background: rgba(74, 222, 128, 0.1); border: 1px solid rgba(74, 222, 128, 0.3); color: #22c55e; }
  .alert-icon { font-size: 1.25rem; }
  .info-text { text-align: center; color: #64748b; font-size: 0.9rem; margin-top: 1rem; }

  .form { display: flex; flex-direction: column; gap: 1.5rem; }
  .input-group { text-align: left; }
  label { display: block; margin-bottom: 0.5rem; font-weight: 600; color: #374151; font-size: 0.95rem; }
  input { width: 100%; padding: 0.875rem 1rem; font-size: 1rem; background: #f8fafc; border: 2px solid #e2e8f0; border-radius: 0.75rem; transition: all 0.2s; outline: none; box-sizing: border-box; }
  input:focus { border-color: #2d5a27; box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2); }
  input:disabled { opacity: 0.6; cursor: not-allowed; }
  .help-text { font-size: 0.8rem; color: #64748b; margin: 0.5rem 0 0 0; }

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
