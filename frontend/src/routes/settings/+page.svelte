<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    isAuthenticated,
    authUser,
    updateUser,
  } from '$lib/authStore';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let userName = $state('');
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let message = $state('');
  let error = $state('');
  let saving = $state(false);
  let activeTab = $state<'profile' | 'security' | 'appearance'>('profile');
  let selectedTheme = $state('jardin-secret');
  let darkMode = $state(false);

  // -----------------------------------------------------------------
  // 2️⃣ Thèmes disponibles
  // -----------------------------------------------------------------
  const themes = [
    {
      id: 'jardin-secret',
      name: '🌿 Jardin Secret',
      description: 'Doux, naturel, aquarelle',
    },
    {
      id: 'space-hub',
      name: '🚀 Space Hub',
      description: 'Futuriste, néon, épuré',
    },
    {
      id: 'maison-chaleureuse',
      name: '🏠 Maison Chaleureuse',
      description: 'Feutre, crayon, bois',
    },
  ];

  // -----------------------------------------------------------------
  // 3️⃣ Initialisation (auth + thème)
  // -----------------------------------------------------------------
  onMount(() => {
    if (!$isAuthenticated) {
      goto('/login');
      return;
    }

    if ($authUser) {
      userName = $authUser.name ?? '';
    }

    loadTheme();
  });

  /** Charge le thème et le mode sombre depuis le `localStorage`. */
  function loadTheme() {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('nook-theme') ?? 'jardin-secret';
      selectedTheme = saved;
      darkMode = localStorage.getItem('nook-dark-mode') === 'true';
      applyTheme();
    }
  }

  /** Applique le thème et le mode sombre sur le `<html>` et persiste. */
  function applyTheme() {
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', selectedTheme);
      document.documentElement.setAttribute('data-dark', darkMode.toString());
      localStorage.setItem('nook-theme', selectedTheme);
      localStorage.setItem('nook-dark-mode', darkMode.toString());
    }
  }

  /** Change le thème sélectionné. */
  function setTheme(themeId: string) {
    selectedTheme = themeId;
    applyTheme();
  }

  /** Bascule le mode sombre. */
  function toggleDarkMode() {
    darkMode = !darkMode;
    applyTheme();
  }

  // -----------------------------------------------------------------
  // 4️⃣ Mise à jour du profil (nom uniquement)
  // -----------------------------------------------------------------
  /**
   * Envoie le nouveau nom au backend et met à jour le store.
   */
  async function updateProfile() {
    if (!userName.trim()) {
      error = 'Le nom ne peut pas être vide';
      return;
    }

    saving = true;
    error = '';
    message = '';

    try {
      const response = await fetch('/api/user/update', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ name: userName }),
      });

      const raw = await response.text();
      let data: any = {};

      if (raw.trim()) {
        try {
          data = JSON.parse(raw);
        } catch {
          // ignore JSON parse error – keep data empty
        }
      }

      if (response.ok) {
        // Met à jour le store local
        updateUser({ name: userName });
        message = 'Profil mis à jour avec succès';
      } else {
        error = data?.message ?? `Erreur ${response.status}`;
      }
    } catch (e) {
      console.error('Erreur mise à jour profil :', e);
      error = e instanceof Error ? e.message : 'Erreur de connexion';
    } finally {
      saving = false;
    }
  }

  // -----------------------------------------------------------------
  // 5️⃣ Changement de mot de passe
  // -----------------------------------------------------------------
  /**
   * Envoie le nouveau mot de passe au backend.
   */
  async function changePassword() {
    if (!currentPassword || !newPassword || !confirmPassword) {
      error = 'Veuillez remplir tous les champs';
      return;
    }
    if (newPassword.length < 8) {
      error = 'Le nouveau mot de passe doit contenir au moins 8 caractères';
      return;
    }
    if (newPassword !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas';
      return;
    }

    saving = true;
    error = '';
    message = '';

    try {
      const response = await fetch('/api/auth/change-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          current_password: currentPassword,
          new_password: newPassword,
        }),
      });

      const raw = await response.text();
      let data: any = {};

      if (raw.trim()) {
        try {
          data = JSON.parse(raw);
        } catch {
          // ignore JSON parse error
        }
      }

      if (response.ok) {
        message = 'Mot de passe modifié avec succès';
        currentPassword = '';
        newPassword = '';
        confirmPassword = '';
      } else {
        error = data?.message ?? `Erreur ${response.status}`;
      }
    } catch (e) {
      console.error('Erreur changement mdp :', e);
      error = e instanceof Error ? e.message : 'Erreur de connexion';
    } finally {
      saving = false;
    }
  }
</script>

<svelte:head>
  <title>Paramètres - Nook</title>
</svelte:head>

<div class="settings-container">
  <!-- -----------------------------------------------------------------
       HEADER
       ----------------------------------------------------------------- -->
  <header class="page-header">
    <h1>⚙️ Paramètres</h1>
  </header>

  <!-- -----------------------------------------------------------------
       TABS
       ----------------------------------------------------------------- -->
  <div class="tabs" role="tablist" aria-label="Sections des paramètres">
    <button
      role="tab"
      class="tab"
      class:active={activeTab === 'profile'}
      aria-selected={activeTab === 'profile'}
      onclick={() => (activeTab = 'profile')}
    >
      Profil
    </button>

    <button
      role="tab"
      class="tab"
      class:active={activeTab === 'security'}
      aria-selected={activeTab === 'security'}
      onclick={() => (activeTab = 'security')}
    >
      Sécurité
    </button>

    <button
      role="tab"
      class="tab"
      class:active={activeTab === 'appearance'}
      aria-selected={activeTab === 'appearance'}
      onclick={() => (activeTab = 'appearance')}
    >
      Apparence
    </button>

  <!-- -----------------------------------------------------------------
       PROFIL
       ----------------------------------------------------------------- -->
  {#if activeTab === 'profile'}
    <div class="settings-section" role="tabpanel" aria-labelledby="profile-tab">
      <h2>Informations du profil</h2>

      <form onsubmit={(e) => { e.preventDefault(); updateProfile(); }}>
        <div class="form-group">
          <label for="userName">Prénom</label>
          <input
            type="text"
            id="userName"
            bind:value={userName}
            placeholder="Votre prénom"
          />
        </div>

        <div class="form-group">
          <label for="userId">Identifiant</label>
          <input
            type="text"
            id="userId"
            value={$authUser?.id ?? ''}
            disabled
            aria-describedby="userId-hint"
          />
          <p id="userId-hint" class="help-text">
            L'identifiant ne peut pas être modifié
          </p>
        </div>

        <button type="submit" class="btn btn-primary" disabled={saving}>
          {saving ? 'Enregistrement…' : 'Enregistrer'}
        </button>
      </form>
    </div>

    <!-- -----------------------------------------------------------------
         SÉCURITÉ
         ----------------------------------------------------------------- -->
  {:else if activeTab === 'security'}
    <div class="settings-section" role="tabpanel" aria-labelledby="security-tab">
      <h2>Changer le mot de passe</h2>

      <form onsubmit={(e) => { e.preventDefault(); changePassword(); }}>
        <div class="form-group">
          <label for="currentPassword">Mot de passe actuel</label>
          <input
            type="password"
            id="currentPassword"
            bind:value={currentPassword}
            autocomplete="current-password"
          />
        </div>

        <div class="form-group">
          <label for="newPassword">Nouveau mot de passe</label>
          <input
            type="password"
            id="newPassword"
            bind:value={newPassword}
            autocomplete="new-password"
          />
          <p class="help-text">Au moins 8 caractères</p>
        </div>

        <div class="form-group">
          <label for="confirmPassword">Confirmer le nouveau mot de passe</label>
          <input
            type="password"
            id="confirmPassword"
            bind:value={confirmPassword}
            autocomplete="new-password"
          />
        </div>

        <button type="submit" class="btn btn-primary" disabled={saving}>
          {saving ? 'Modification…' : 'Changer le mot de passe'}
        </button>
      </form>
    </section>

    <!-- -----------------------------------------------------------------
         APPARENCE
         ----------------------------------------------------------------- -->
  {:else if activeTab === 'appearance'}
    <div class="settings-section" role="tabpanel" aria-labelledby="appearance-tab">
      <h2>Thème</h2>

      <div class="themes-grid">
        {#each themes as theme}
          <button
            class="theme-card"
            class:selected={selectedTheme === theme.id}
            onclick={() => setTheme(theme.id)}
            aria-pressed={selectedTheme === theme.id}
          >
            <span class="theme-icon">
              {theme.id === 'jardin-secret'
                ? '🌿'
                : theme.id === 'space-hub'
                ? '🚀'
                : '🏠'}
            </span>
            <span class="theme-name">{theme.name}</span>
            <span class="theme-description">{theme.description}</span>
          </button>
        {/each}
      </div>

      <div class="form-group">
        <label class="toggle-label" for="darkModeToggle">
          <span>Mode sombre</span>
          <div class="toggle-switch">
            <input
              type="checkbox"
              id="darkModeToggle"
              checked={darkMode}
              onchange={toggleDarkMode}
            />
            <span class="toggle-slider"></span>
          </div>
        </label>
      </div>
    </section>
  {/if}

  <!-- -----------------------------------------------------------------
       FEEDBACK (message / error)
       ----------------------------------------------------------------- -->
  {#if message}
    <div role="alert" class="alert alert-success">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="icon"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        aria-hidden="true"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
        />
      </svg>
      <span>{message}</span>
    </div>
  {/if}

  {#if error}
    <div role="alert" class="alert alert-error">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="icon"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        aria-hidden="true"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
        />
      </svg>
      <span>{error}</span>
    </div>
  {/if}
</div>

<style>
  * { box-sizing: border-box; }

  .settings-container {
    min-height: 100vh;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
    padding: 1.5rem 1rem;
    max-width: 800px;
    margin: 0 auto;
  }

  .page-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .page-header h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0 0 0.5rem 0;
    color: #1e293b;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 2rem;
    background: white;
    padding: 0.5rem;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  }

  .tab {
    flex: 1;
    padding: 0.85rem 1rem;
    border: none;
    background: transparent;
    color: #64748b;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    border-radius: 8px;
  }

  .tab:hover {
    color: #1e293b;
    background: #f8fafc;
  }

  .tab.active {
    color: white;
    background: #2d5a27;
    box-shadow: 0 2px 8px rgba(45, 90, 39, 0.2);
  }

  .settings-section {
    background: white;
    padding: 2rem;
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    margin-bottom: 1.5rem;
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .settings-section h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0 0 1.5rem 0;
    color: #1e293b;
    padding-bottom: 0.75rem;
    border-bottom: 2px solid #f1f5f9;
  }

  .form-group {
    margin-bottom: 1.5rem;
    text-align: left;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #374151;
    font-size: 0.95rem;
  }

  .form-group input[type='text'],
  .form-group input[type='password'] {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #e2e8f0;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.2s;
    background: #f8fafc;
  }

  .form-group input[type='text']:focus,
  .form-group input[type='password']:focus {
    border-color: #2d5a27;
    box-shadow: 0 0 0 3px rgba(45, 90, 39, 0.2);
    background: white;
    outline: none;
  }

  .form-group input[type='text']:disabled {
    background: #f1f5f9;
    color: #94a3b8;
    cursor: not-allowed;
  }

  .help-text {
    margin: 0.5rem 0 0 0;
    font-size: 0.85rem;
    color: #64748b;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 0.75rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #2d5a27;
    color: white;
    box-shadow: 0 2px 8px rgba(45, 90, 39, 0.2);
  }

  .btn-primary:hover:not(:disabled) {
    background: #3d7a37;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(45, 90, 39, 0.3);
  }

  .btn-primary:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    transform: none !important;
  }

  .themes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  @media (max-width: 640px) {
    .themes-grid {
      grid-template-columns: 1fr;
    }
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding: 1.5rem 1rem;
    background: #f8fafc;
    border: 2px solid #e2e8f0;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .theme-card:hover {
    background: #f1f5f9;
    border-color: #cbd5e1;
    transform: translateY(-2px);
  }

  .theme-card.selected {
    background: #e8f5e8;
    border-color: #2d5a27;
    box-shadow: 0 4px 12px rgba(45, 90, 39, 0.15);
  }

  .theme-icon {
    font-size: 2rem;
  }

  .theme-name {
    font-weight: 600;
    color: #1e293b;
    font-size: 1rem;
  }

  .theme-description {
    font-size: 0.85rem;
    color: #64748b;
    text-align: center;
  }

  .toggle-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    margin: 0;
    padding: 1rem;
    background: #f8fafc;
    border-radius: 12px;
    border: 2px solid #e2e8f0;
  }

  .toggle-label span:first-child {
    font-weight: 600;
    color: #1e293b;
  }

  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 52px;
    height: 28px;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: #cbd5e1;
    border-radius: 34px;
    transition: .4s;
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 20px;
    width: 20px;
    left: 4px;
    bottom: 4px;
    background: white;
    border-radius: 50%;
    transition: .4s;
  }

  input:checked + .toggle-slider {
    background: #2d5a27;
  }

  input:checked + .toggle-slider:before {
    transform: translateX(24px);
  }

  .alert {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 12px;
    margin-bottom: 1.5rem;
    text-align: left;
    font-size: 0.9rem;
    animation: fadeIn 0.3s ease-out;
  }

  .alert-success {
    background: rgba(72, 187, 120, 0.1);
    border: 2px solid rgba(72, 187, 120, 0.3);
    color: #2d5a27;
  }

  .alert-error {
    background: rgba(239, 68, 68, 0.1);
    border: 2px solid rgba(239, 68, 68, 0.3);
    color: #dc2626;
  }

  .alert .icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  @media (max-width: 768px) {
    .settings-container {
      padding: 1rem 0.75rem;
    }

    .tabs {
      flex-direction: column;
    }

    .tab {
      width: 100%;
      text-align: center;
    }

    .settings-section {
      padding: 1.5rem;
    }

    .themes-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 480px) {
    .settings-section {
      padding: 1.25rem;
    }

    .toggle-label {
      flex-direction: column;
      gap: 0.75rem;
      align-items: flex-start;
    }

    .form-group input[type='text'],
    .form-group input[type='password'] {
      padding: 0.65rem;
    }

    .btn {
      width: 100%;
    }
  }
</style>