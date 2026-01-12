<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { state } from 'svelte'; // <-- Svelte 5 reactive state
  import {
    isAuthenticated,
    authUser,
    updateUser,
  } from '$lib/authStore';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let userName = state('');
  let currentPassword = state('');
  let newPassword = state('');
  let confirmPassword = state('');
  let message = state('');
  let error = state('');
  let saving = state(false);
  let activeTab = state<'profile' | 'security' | 'appearance'>('profile');

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

  let selectedTheme = state('jardin-secret');
  let darkMode = state(false);

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
      console.error('Erreur mise à jour profil :', e);
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
      console.error('Erreur changement mdp :', e);
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
  <nav class="tabs" role="tablist" aria-label="Sections des paramètres">
    <button
      role="tab"
      class="tab"
      class:active={activeTab === 'profile'}
      aria-selected={activeTab === 'profile'}
      on:click={() => (activeTab = 'profile')}
    >
      Profil
    </button>

    <button
      role="tab"
      class="tab"
      class:active={activeTab === 'security'}
      aria-selected={activeTab === 'security'}
      on:click={() => (activeTab = 'security')}
    >
      Sécurité
    </button>

    <button
      role="tab"
      class="tab"
      class:active={activeTab === 'appearance'}
      aria-selected={activeTab === 'appearance'}
      on:click={() => (activeTab = 'appearance')}
    >
      Apparence
    </button>
  </nav>

  <!-- -----------------------------------------------------------------
       PROFIL
       ----------------------------------------------------------------- -->
  {#if activeTab === 'profile'}
    <section class="settings-section" role="tabpanel" aria-labelledby="profile-tab">
      <h2>Informations du profil</h2>

      <form on:submit|preventDefault={updateProfile}>
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
    </section>

    <!-- -----------------------------------------------------------------
         SÉCURITÉ
         ----------------------------------------------------------------- -->
  {:else if activeTab === 'security'}
    <section class="settings-section" role="tabpanel" aria-labelledby="security-tab">
      <h2>Changer le mot de passe</h2>

      <form on:submit|preventDefault={changePassword}>
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
    <section class="settings-section" role="tabpanel" aria-labelledby="appearance-tab">
      <h2>Thème</h2>

      <div class="themes-grid">
        {#each themes as theme}
          <button
            class="theme-card"
            class:selected={selectedTheme === theme.id}
            on:click={() => setTheme(theme.id)}
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
              on:change={toggleDarkMode}
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
  /* -----------------------------------------------------------------
     CONTAINER
     ----------------------------------------------------------------- */
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

  /* -----------------------------------------------------------------
     TABS
     ----------------------------------------------------------------- */
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

  /* -----------------------------------------------------------------
     SECTION
     ----------------------------------------------------------------- */
  .settings-section {
    background: white;
    padding: 24px;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(5px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .settings-section h2 {
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 20px 0;
    color: #1e293b;
  }

  /* -----------------------------------------------------------------
     FORM
     ----------------------------------------------------------------- */
  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: #334155;
  }

  .form-group input[type='text'],
  .form-group input[type='password'] {
    width: 100%;
    padding: 12px 14px;
    border: 1.5px solid #e2e8f0;
    border-radius: 8px;
    font-size: 1rem;
    transition: all 0.2s;
    box-sizing: border-box;
    background: white;