<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isAuthenticated, authUser, updateUser } from '$lib/authStore';

  let userName = $state('');
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let message = $state('');
  let error = $state('');
  let saving = $state(false);
  let activeTab = $state<'profile' | 'security' | 'appearance'>('profile');

  const themes = [
    { id: 'jardin-secret', name: '🌿 Jardin Secret', description: 'Doux, naturel, aquarelle' },
    { id: 'space-hub', name: '🚀 Space Hub', description: 'Futuriste, néon, épuré' },
    { id: 'maison-chaleureuse', name: '🏠 Maison Chaleureuse', description: 'Feutre, crayon, bois' }
  ];

  let selectedTheme = $state('jardin-secret');
  let darkMode = $state(false);

  onMount(async () => {
    if (!$isAuthenticated) {
      goto('/login');
      return;
    }
    if (authUser) {
      userName = authUser.name || '';
    }
    loadTheme();
  });

  function loadTheme() {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('nook-theme') || 'jardin-secret';
      selectedTheme = saved;
      darkMode = localStorage.getItem('nook-dark-mode') === 'true';
      applyTheme();
    }
  }

  function applyTheme() {
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', selectedTheme);
      document.documentElement.setAttribute('data-dark', darkMode.toString());
      localStorage.setItem('nook-theme', selectedTheme);
      localStorage.setItem('nook-dark-mode', darkMode.toString());
    }
  }

  function setTheme(themeId: string) {
    selectedTheme = themeId;
    applyTheme();
  }

  function toggleDarkMode() {
    darkMode = !darkMode;
    applyTheme();
  }

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
        body: JSON.stringify({ name: userName })
      });

      if (response.ok) {
        updateUser({ name: userName });
        message = 'Profil mis à jour avec succès';
      } else {
        const data = await response.json();
        error = data.message || 'Erreur lors de la mise à jour';
      }
    } catch (err) {
      error = 'Erreur de connexion';
    } finally {
      saving = false;
    }
  }

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
      const response = await fetch('/api/change-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
      });

      if (response.ok) {
        message = 'Mot de passe modifié avec succès';
        currentPassword = '';
        newPassword = '';
        confirmPassword = '';
      } else {
        const data = await response.json();
        error = data.message || 'Erreur lors du changement de mot de passe';
      }
    } catch (err) {
      error = 'Erreur de connexion';
    } finally {
      saving = false;
    }
  }
</script>

<svelte:head>
  <title>Paramètres - Nook</title>
</svelte:head>

<div class="settings-container">
  <header class="page-header">
    <h1>⚙️ Paramètres</h1>
  </header>

  <nav class="tabs" role="tablist" aria-label="Sections des paramètres">
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
  </nav>

  {#if activeTab === 'profile'}
    <section class="settings-section" role="tabpanel" aria-labelledby="profile-tab">
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
            value={authUser?.id || ''}
            disabled
            aria-describedby="userId-hint"
          />
          <p id="userId-hint" class="help-text">L'identifiant ne peut pas être modifié</p>
        </div>

        <button type="submit" class="btn btn-primary" disabled={saving}>
          {saving ? 'Enregistrement...' : 'Enregistrer'}
        </button>
      </form>
    </section>

  {:else if activeTab === 'security'}
    <section class="settings-section" role="tabpanel" aria-labelledby="security-tab">
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
          {saving ? 'Modification...' : 'Changer le mot de passe'}
        </button>
      </form>
    </section>

  {:else if activeTab === 'appearance'}
    <section class="settings-section" role="tabpanel" aria-labelledby="appearance-tab">
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
              {theme.id === 'jardin-secret' ? '🌿' : theme.id === 'space-hub' ? '🚀' : '🏠'}
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

  {#if message}
    <div role="alert" class="alert alert-success">
      <svg xmlns="http://www.w3.org/2000/svg" class="icon" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <span>{message}</span>
    </div>
  {/if}

  {#if error}
    <div role="alert" class="alert alert-error">
      <svg xmlns="http://www.w3.org/2000/svg" class="icon" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <span>{error}</span>
    </div>
  {/if}
</div>

<style>
  .settings-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  .page-header {
    margin-bottom: 24px;
  }

  .page-header h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0;
    color: #1e293b;
  }

  .tabs {
    display: flex;
    gap: 8px;
    margin-bottom: 24px;
    border-bottom: 2px solid #e2e8f0;
    padding-bottom: 0;
  }

  .tab {
    padding: 12px 20px;
    border: none;
    background: none;
    cursor: pointer;
    font-size: 0.95rem;
    color: #64748b;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    transition: all 0.2s;
    font-weight: 500;
  }

  .tab:hover {
    color: #1e293b;
  }

  .tab.active {
    color: #4ade80;
    border-bottom-color: #4ade80;
  }

  .settings-section {
    background: white;
    padding: 24px;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(5px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .settings-section h2 {
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 20px 0;
    color: #1e293b;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: #334155;
  }

  .form-group input[type="text"],
  .form-group input[type="password"] {
    width: 100%;
    padding: 12px 14px;
    border: 1.5px solid #e2e8f0;
    border-radius: 8px;
    font-size: 1rem;
    transition: all 0.2s;
    box-sizing: border-box;
    background: white;
    color: #1e293b;
  }

  .form-group input:focus {
    outline: none;
    border-color: #4ade80;
    box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.15);
  }

  .form-group input:disabled {
    background-color: #f8fafc;
    cursor: not-allowed;
    color: #64748b;
  }

  .help-text {
    font-size: 0.85rem;
    color: #64748b;
    margin: 6px 0 0 0;
  }

  .btn {
    padding: 12px 24px;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
    font-size: 1rem;
  }

  .btn-primary {
    background-color: #4ade80;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .btn-primary:disabled {
    background-color: #9ca3af;
    cursor: not-allowed;
  }

  .themes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
  }

  .theme-card {
    padding: 16px;
    border: 2px solid #e2e8f0;
    border-radius: 12px;
    background: white;
    cursor: pointer;
    text-align: center;
    transition: all 0.2s;
  }

  .theme-card:hover {
    border-color: #cbd5e1;
    transform: translateY(-2px);
  }

  .theme-card.selected {
    border-color: #4ade80;
    background-color: rgba(74, 222, 128, 0.08);
  }

  .theme-card:focus {
    outline: 2px solid #4ade80;
    outline-offset: 2px;
  }

  .theme-icon {
    font-size: 2.5rem;
    display: block;
    margin-bottom: 10px;
  }

  .theme-name {
    font-weight: 600;
    display: block;
    margin-bottom: 6px;
    color: #1e293b;
    font-size: 1rem;
  }

  .theme-description {
    font-size: 0.85rem;
    color: #64748b;
    line-height: 1.4;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
  }

  .toggle-label span {
    font-weight: 500;
    color: #334155;
  }

  .toggle-switch {
    position: relative;
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
    inset: 0;
    background-color: #cbd5e1;
    transition: 0.3s;
    border-radius: 28px;
  }

  .toggle-slider::before {
    position: absolute;
    content: "";
    height: 22px;
    width: 22px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .toggle-switch input:checked + .toggle-slider {
    background-color: #4ade80;
  }

  .toggle-switch input:checked + .toggle-slider::before {
    transform: translateX(24px);
  }

  .toggle-switch input:focus + .toggle-slider {
    box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.2);
  }

  .alert {
    margin-top: 20px;
    padding: 14px 18px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .alert .icon {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
  }

  .alert-success {
    background-color: rgba(34, 197, 94, 0.1);
    color: #16a34a;
    border: 1px solid rgba(34, 197, 94, 0.25);
  }

  .alert-error {
    background-color: rgba(239, 68, 68, 0.1);
    color: #dc2626;
    border: 1px solid rgba(239, 68, 68, 0.25);
  }

  @media (max-width: 640px) {
    .settings-container {
      padding: 16px;
    }

    .tabs {
      overflow-x: auto;
      -webkit-overflow-scrolling: touch;
    }

    .tab {
      padding: 10px 16px;
      white-space: nowrap;
    }

    .settings-section {
      padding: 20px;
    }

    .themes-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
