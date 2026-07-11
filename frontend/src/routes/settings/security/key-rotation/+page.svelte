<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { cryptoStore } from '$lib/cryptoStore.svelte';
  import { authStore } from '$lib/authStore.svelte.js';
  import {
    getCurrentKeyVersion,
    getKeyHistoryFromStore,
    saveKeyRotation,
    rotateKeyOnServer,
    generateKeyPair,
    encryptPrivateKey,
    decryptPrivateKey,
    registerPublicKeyOnServer,
    encryptPrivateKeyForArchive,
    decryptPrivateKeyFromArchive,
    type ArchivedKeyEntry,
  } from '$lib/crypto';

  interface KeyHistoryDisplay {
    version: number;
    createdAt: string;
    isArchived: boolean;
  }

  let currentVersion = $state(1);
  let keyHistory = $state<KeyHistoryDisplay[]>([]);
  let password = $state('');
  let confirmPassword = $state('');
  let showPasswordInput = $state(false);
  let rotating = $state(false);
  let message = $state('');
  let error = $state('');

  // Backup state
  let backupPassphrase = $state('');
  let confirmBackupPassphrase = $state('');
  let showingBackup = $state(false);
  let backupResult = $state<string | null>(null);
  let backingUp = $state(false);
  let restoreB64 = $state('');
  let restorePassphrase = $state('');
  let restoring = $state(false);
  let restoreResult = $state('');

  onMount(async () => {
    if (!cryptoStore.ready || !cryptoStore.userId) {
      error = 'Veuillez déverrouiller vos clés E2EE d\'abord.';
      return;
    }
    await loadKeyState();
  });

  async function loadKeyState() {
    if (!cryptoStore.userId) return;
    try {
      currentVersion = cryptoStore.currentKeyVersion;
      const history = await getKeyHistoryFromStore(cryptoStore.userId);
      keyHistory = history
        .map((e: ArchivedKeyEntry) => ({
          version: e.version,
          createdAt: new Date(e.createdAt).toLocaleDateString('fr-FR', {
            year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
          }),
          isArchived: true,
        }))
        .reverse();
    } catch (e: any) {
      console.error('[key-rotation] loadKeyState:', e);
    }
  }

  async function handleRotate() {
    if (!cryptoStore.userId || !cryptoStore.ready) {
      error = 'Clés non déverrouillées.';
      return;
    }
    if (!password || password.length < 4) {
      error = 'Veuillez entrer votre mot de passe.';
      return;
    }

    rotating = true;
    error = '';
    message = '';

    try {
      // 1. Données actuelles — on utilise le mot de passe pour déchiffrer l'ancienne clé
      const { loadKeysFromIndexedDB, getCurrentKeyVersion } = await import('$lib/crypto');
      const currentKp = await loadKeysFromIndexedDB(cryptoStore.userId, password);
      if (!currentKp) throw new Error('Impossible de charger les clés actuelles.');

      const oldPubB64 = btoa(String.fromCharCode(...currentKp.publicKey));

      // 2. Générer une nouvelle paire
      const newKp = await generateKeyPair();
      const newVersion = currentVersion + 1;

      // 3. Chiffrer la nouvelle privée avec le mot de passe
      const newEncPriv = await encryptPrivateKey(newKp.privateKey, password);

      // 4. Archiver l'ancienne clé en local
      const oldEncPriv = await encryptPrivateKey(currentKp.privateKey, password);
      await saveKeyRotation(
        cryptoStore.userId,
        newKp.publicKey,
        newEncPriv,
        oldEncPriv,
        oldPubB64,
        newVersion
      );

      // 5. Rotate on server (archives old key, registers new public key)
      const result = await rotateKeyOnServer(
        btoa(String.fromCharCode(...newKp.publicKey)),
        newEncPriv,
        password
      );

      // 6. Update store state
      cryptoStore.currentKeyVersion = result.version;

      message = `Clés E2EE mises à jour → version ${result.version} ✓`;
      showPasswordInput = false;
      password = '';
      confirmPassword = '';
      await loadKeyState();
    } catch (e: any) {
      error = 'Rotation échouée : ' + (e?.message ?? String(e));
      console.error('[key-rotation] handleRotate:', e);
    } finally {
      rotating = false;
    }
  }

  // ── Backup ────────────────────────────────────────────────────────────
  async function handleBackup() {
    if (backupPassphrase !== confirmBackupPassphrase) {
      error = 'Les passphrases ne correspondent pas.';
      return;
    }
    if (backupPassphrase.length < 8) {
      error = 'Le passphrase doit faire au moins 8 caractères.';
      return;
    }

    backingUp = true;
    error = '';
    backupResult = null;

    try {
      const { loadKeysFromIndexedDB } = await import('$lib/crypto');
      if (!cryptoStore.userId) throw new Error('userId absent');
      const kp = await loadKeysFromIndexedDB(cryptoStore.userId, backupPassphrase);
      if (!kp) throw new Error('Mot de passe incorrect ou clés introuvables.');

      const archiveB64 = await encryptPrivateKeyForArchive(kp.privateKey, backupPassphrase);
      backupResult = archiveB64;
      message = 'Copie de sauvegarde générée ✓';
    } catch (e: any) {
      error = 'Sauvegarde échouée : ' + (e?.message ?? String(e));
    } finally {
      backingUp = false;
    }
  }

  async function handleRestore() {
    if (!restoreB64 || !restorePassphrase) {
      error = 'Veuillez fournir le blob et le passphrase.';
      return;
    }
    restoring = true;
    error = '';
    restoreResult = '';

    try {
      const privateKey = await decryptPrivateKeyFromArchive(restoreB64, restorePassphrase);
      if (!privateKey) throw new Error('Déchiffrement échoué — mauvais passphrase ou données corrompues.');

      restoreResult = `Clé privée restaurée ✓ (${privateKey.length} bytes). Vous pouvez maintenant la ré-enregistrer.`;
      message = 'Restauration réussie ✓';
    } catch (e: any) {
      error = 'Restauration échouée : ' + (e?.message ?? String(e));
    } finally {
      restoring = false;
    }
  }

  function copyBackup() {
    if (backupResult) {
      navigator.clipboard.writeText(backupResult).then(
        () => { message = 'Blob copié dans le presse-papier ✓'; },
        () => { error = 'Impossible de copier.'; }
      );
    }
  }

  function downloadBackup() {
    if (!backupResult) return;
    const blob = new Blob([backupResult], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `nook-e2ee-backup-${new Date().toISOString().split('T')[0]}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="key-rotation-page">
  <nav class="breadcrumb">
    <a href="/settings" onclick={(e) => { e.preventDefault(); goto('/settings'); }}>Paramètres</a>
    <span>›</span>
    <span>Clés E2EE</span>
  </nav>

  <h1>🔐 Gestion des clés E2EE</h1>

  {#if error}
    <div role="alert" class="alert alert-error">❌ {error}</div>
  {/if}
  {#if message}
    <div role="alert" class="alert alert-success">✓ {message}</div>
  {/if}

  <!-- ── État actuel ────────────────────────────────────────── -->
  <section class="section">
    <h2>État des clés</h2>
    <div class="key-status">
      <div class="key-stat">
        <span class="stat-label">Version actuelle</span>
        <span class="stat-value">{currentVersion}</span>
      </div>
      <div class="key-stat">
        <span class="stat-label">Statut</span>
        <span class="stat-value" class:ready={cryptoStore.ready} class:not-ready={!cryptoStore.ready}>
          {cryptoStore.ready ? '✅ Prêt' : '🔒 Verrouillé'}
        </span>
      </div>
    </div>
  </section>

  <!-- ── Rotation ───────────────────────────────────────────── -->
  <section class="section">
    <h2>Rotation de clé</h2>
    <p class="section-desc">
      La rotation génère une nouvelle paire de clés Curve25519 et archive l'ancienne.
      Les anciens messages restent lisibles en utilisant la clé archivée.
    </p>

    {#if !showPasswordInput}
      <button class="btn btn-warning" onclick={() => { showPasswordInput = true; error = ''; message = ''; }}>
        🔄 Lancer une rotation
      </button>
    {:else}
      <form onsubmit={(e) => { e.preventDefault(); handleRotate(); }}>
        <div class="form-group">
          <label for="rotate-password">Mot de passe Nook</label>
          <input id="rotate-password" type="password" bind:value={password} autocomplete="current-password" required />
          <p class="help-text">Nécessaire pour déchiffrer et rechiffrer votre clé privée.</p>
        </div>
        <div class="form-group">
          <label for="rotate-confirm">Confirmer le mot de passe</label>
          <input id="rotate-confirm" type="password" bind:value={confirmPassword} autocomplete="current-password" required />
        </div>
        <div class="form-actions">
          <button type="submit" class="btn btn-primary" disabled={rotating}>
            {rotating ? 'Rotation en cours…' : 'Confirmer la rotation'}
          </button>
          <button type="button" class="btn btn-ghost" onclick={() => { showPasswordInput = false; password = ''; confirmPassword = ''; }}>
            Annuler
          </button>
        </div>
      </form>
    {/if}
  </section>

  <!-- ── Historique ─────────────────────────────────────────── -->
  <section class="section">
    <h2>Historique des clés archivées</h2>
    {#if keyHistory.length === 0}
      <p class="section-desc text-muted">Aucune rotation effectuée — c'est votre clé d'origine (version 1).</p>
    {:else}
      <div class="history-list">
        {#each keyHistory as entry}
          <div class="history-item">
            <span class="history-version">v{entry.version}</span>
            <span class="history-date">{entry.createdAt}</span>
            <span class="history-badge archivée">Archivée</span>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <!-- ── Backup / Restore cross-device ──────────────────────── -->
  <section class="section">
    <h2>💾 Backup & Restore</h2>
    <p class="section-desc">
      Sauvegardez votre clé privée chiffrée avec un passphrase dédié pour la restaurer
      sur un autre appareil ou en cas de perte.
    </p>

    <details class="collapsible">
      <summary onclick={() => {}}>📤 Créer une sauvegarde</summary>
      <div class="backup-form">
        <p class="help-text">
          Entrez un passphrase de <strong>8 caractères minimum</strong>.
          Ce passphrase est indépendant de votre mot de passe Nook.
        </p>
        <div class="form-group">
          <label for="backup-pass">Passphrase de sauvegarde</label>
          <input id="backup-pass" type="password" bind:value={backupPassphrase} />
        </div>
        <div class="form-group">
          <label for="backup-confirm">Confirmer le passphrase</label>
          <input id="backup-confirm" type="password" bind:value={confirmBackupPassphrase} />
        </div>
        <button class="btn btn-primary" onclick={handleBackup} disabled={backingUp}>
          {backingUp ? 'Génération…' : 'Générer la sauvegarde'}
        </button>

        {#if backupResult}
          <div class="backup-result">
            <p class="success-text">✓ Sauvegarde générée !</p>
            <div class="backup-actions">
              <button class="btn btn-sm" onclick={copyBackup}>📋 Copier</button>
              <button class="btn btn-sm" onclick={downloadBackup}>⬇️ Télécharger</button>
            </div>
            <pre class="backup-blob">{backupResult}</pre>
            <p class="help-text">Conservez ce blob et le passphrase en lieu sûr. Sans le passphrase, le blob est inutilisable.</p>
          </div>
        {/if}
      </div>
    </details>

    <details class="collapsible">
      <summary>📥 Restaurer une sauvegarde</summary>
      <div class="restore-form">
        <p class="help-text">
          Collez le blob de sauvegarde et le passphrase utilisé lors de sa création.
        </p>
        <div class="form-group">
          <label for="restore-b64">Blob de sauvegarde</label>
          <textarea id="restore-b64" bind:value={restoreB64} rows="4" placeholder="Collez le blob base64…"></textarea>
        </div>
        <div class="form-group">
          <label for="restore-pass">Passphrase</label>
          <input id="restore-pass" type="password" bind:value={restorePassphrase} />
        </div>
        <button class="btn btn-primary" onclick={handleRestore} disabled={restoring}>
          {restoring ? 'Restauration…' : 'Restaurer'}
        </button>

        {#if restoreResult}
          <div class="restore-result">
            <p class="success-text">✓ {restoreResult}</p>
          </div>
        {/if}
      </div>
    </details>
  </section>
</div>

<style>
  .key-rotation-page {
    max-width: 640px;
    margin: 0 auto;
    padding: 1.5rem;
  }

  .breadcrumb {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    margin-bottom: 1rem;
    font-size: 0.9rem;
    color: var(--text-muted, #888);
  }
  .breadcrumb a {
    color: var(--accent, #6c5ce7);
    text-decoration: none;
  }
  .breadcrumb a:hover {
    text-decoration: underline;
  }

  h1 {
    font-size: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .section {
    background: var(--card-bg, #1e1e2e);
    border-radius: 12px;
    padding: 1.25rem;
    margin-bottom: 1rem;
    border: 1px solid var(--border, #2d2d44);
  }

  .section h2 {
    font-size: 1.1rem;
    margin: 0 0 0.5rem;
  }

  .section-desc {
    font-size: 0.9rem;
    color: var(--text-muted, #888);
    margin: 0 0 1rem;
    line-height: 1.5;
  }

  .key-status {
    display: flex;
    gap: 2rem;
  }

  .key-stat {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .stat-label {
    font-size: 0.8rem;
    color: var(--text-muted, #888);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .stat-value.ready { color: var(--success, #27ae60); }
  .stat-value.not-ready { color: var(--warning, #f39c12); }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    font-size: 0.9rem;
    margin-bottom: 0.35rem;
    color: var(--text, #ddd);
  }

  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 0.65rem 0.75rem;
    border-radius: 8px;
    border: 1px solid var(--border, #2d2d44);
    background: var(--bg, #151520);
    color: var(--text, #eee);
    font-size: 0.9rem;
    box-sizing: border-box;
  }

  .form-group textarea {
    font-family: 'Fira Code', 'JetBrains Mono', monospace;
    font-size: 0.8rem;
  }

  .help-text {
    font-size: 0.8rem;
    color: var(--text-muted, #888);
    margin: 0.3rem 0 0;
  }

  .form-actions {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .btn {
    padding: 0.6rem 1.2rem;
    border-radius: 8px;
    border: none;
    font-size: 0.9rem;
    cursor: pointer;
    transition: opacity 0.2s;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-primary { background: var(--accent, #6c5ce7); color: #fff; }
  .btn-warning { background: var(--warning, #f39c12); color: #151520; }
  .btn-ghost { background: transparent; color: var(--text-muted, #888); border: 1px solid var(--border, #2d2d44); }
  .btn-sm { padding: 0.35rem 0.75rem; font-size: 0.8rem; }

  .alert {
    padding: 0.75rem 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }
  .alert-success { background: rgba(39, 174, 96, 0.15); color: var(--success, #27ae60); border: 1px solid rgba(39, 174, 96, 0.3); }
  .alert-error   { background: rgba(231, 76, 60, 0.15); color: var(--error, #e74c3c); border: 1px solid rgba(231, 76, 60, 0.3); }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .history-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 0.75rem;
    background: var(--bg, #151520);
    border-radius: 8px;
  }

  .history-version {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    min-width: 2.5rem;
  }

  .history-date {
    font-size: 0.85rem;
    color: var(--text-muted, #888);
    flex: 1;
  }

  .history-badge {
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    background: rgba(108, 92, 231, 0.15);
    color: var(--accent, #6c5ce7);
  }

  .collapsible {
    margin-bottom: 0.75rem;
  }

  .collapsible summary {
    cursor: pointer;
    font-size: 0.95rem;
    padding: 0.5rem 0;
    color: var(--accent, #6c5ce7);
    font-weight: 500;
  }

  .backup-form,
  .restore-form {
    padding: 1rem 0 0.5rem;
  }

  .backup-result,
  .restore-result {
    margin-top: 1rem;
    padding: 0.75rem;
    background: var(--bg, #151520);
    border-radius: 8px;
  }

  .success-text {
    color: var(--success, #27ae60);
    font-weight: 600;
    margin: 0 0 0.5rem;
  }

  .backup-actions {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .backup-blob {
    font-size: 0.7rem;
    word-break: break-all;
    white-space: pre-wrap;
    max-height: 120px;
    overflow-y: auto;
    padding: 0.5rem;
    background: var(--bg-deep, #0a0a12);
    border-radius: 4px;
    color: var(--text-muted, #888);
    margin: 0;
  }

  .text-muted { color: var(--text-muted, #888); }
</style>
