<!-- frontend/src/routes/settings/+page.svelte — Session 39
     Ajout : section notifications push dans l'onglet Sécurité
-->
<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import PasswordInput from '$lib/components/PasswordInput.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import { getPushState, subscribeToPush, unsubscribePush, type PushState } from '$lib/push';

  let userName         = $state('');
  let currentPassword  = $state('');
  let newPassword      = $state('');
  let confirmPassword  = $state('');
  let message          = $state('');
  let error            = $state('');
  let saving           = $state(false);
  let activeTab        = $state<'profile' | 'security' | 'appearance'>('profile');
  let selectedTheme    = $state('jardin-secret');
  let avatarUrl        = $state<string | null>(null);
  let avatarUploading  = $state(false);
  let avatarError      = $state('');

  // ── DiceBear avatar styles ───────────────────────────────────────
  let selectedAvatarStyle = $state<string>('adventurer');
  let selectedAvatarSeed = $state<string>('');
  let avatarGridSeeds = $state<string[]>([]);

  const avatarStyles = [
    { id: 'adventurer',  label: 'Aventurier',  icon: '🎮' },
    { id: 'avataaars',   label: 'Cartoon',     icon: '😊' },
    { id: 'open-peeps',  label: 'Illustré',    icon: '✏️' },
    { id: 'notionists',  label: 'Minimaliste', icon: '🎨' },
    { id: 'fun-emoji',   label: 'Emoji',       icon: '😄' },
    { id: 'big-smile',   label: 'Sourire',     icon: '😁' },
    { id: 'lorelei',     label: 'Portrait',    icon: '🧑' },
    { id: 'personas',    label: 'Personas',    icon: '💼' },
    { id: 'bottts',      label: 'Robot',       icon: '🤖' },
    { id: 'initials',    label: 'Initiales',   icon: '🔤' },
  ];

  const NICKNAMES = [
    'Luna','Felix','Nova','Cosmo','Pixel','Spark','Ziggy','Bree','Mochi','Nori',
    'Kiki','Zara','Remy','Juno','Ash','Sky','Rio','Sol','Mika','Leo',
    'Ava','Kai','Zoe','Max','Sam','Ivy','Theo','Lily','Oscar','Nina',
    'Hugo','Ella','Finn','Mila','Axel','Rosa','Cleo','Jazz','Vega','Ludo',
  ];

  function regenerateGrid() {
    const shuffled = [...NICKNAMES].sort(() => Math.random() - 0.5);
    avatarGridSeeds = shuffled.slice(0, 20);
  }

  function selectStyle(styleId: string) {
    selectedAvatarStyle = styleId;
    selectedAvatarSeed = '';
    regenerateGrid();
  }

  function selectSeed(seed: string) {
    selectedAvatarSeed = seed;
  }

  let darkMode         = $state(false);

  // ── Push notifications ────────────────────────────────────────────────────
  let pushState        = $state<PushState | null>(null);
  let pushLoading      = $state(false);
  let pushMessage      = $state('');
  let pushError        = $state('');

  const themes = [
    {
      id: 'jardin-secret',
      name: 'Jardin Secret',
      icon: '🌿',
      description: 'Doux, naturel, aquarelle',
      preview: { bg: '#f0fdf4', accent: '#4ade80', text: '#1e293b', bubble: '#dcfce7' },
    },
    {
      id: 'space-hub',
      name: 'Space Hub',
      icon: '🌌',
      description: 'Sombre, futuriste, cosmique',
      preview: { bg: '#0f172a', accent: '#8b5cf6', text: '#f1f5f3', bubble: '#334155' },
    },
    {
      id: 'maison-chaleureuse',
      name: 'Maison Chaleureuse',
      icon: '🏠',
      description: 'Chaleureux, ambre, feu',
      preview: { bg: '#fdf2e9', accent: '#ea580c', text: '#7c2d12', bubble: '#ffedd5' },
    },
  ];

  onMount(async () => {
    if (!authStore.isAuthenticated) { goto('/login'); return; }
    if (authStore.user) {
      userName = authStore.user.name ?? '';
      selectedAvatarStyle = authStore.user.avatar_style ?? 'adventurer';
      selectedAvatarSeed = authStore.user.avatar_seed ?? '';
    }
    loadTheme();
    pushState = await getPushState();
    regenerateGrid();
  });

  function loadTheme() {
    if (typeof window !== 'undefined') {
      selectedTheme = localStorage.getItem('nook-theme') ?? 'jardin-secret';
      darkMode      = localStorage.getItem('nook-dark-mode') === 'true';
      applyTheme();
    }
  }

  function applyTheme() {
    if (typeof document !== 'undefined') {
      const body = document.body;
      body.classList.remove('theme-jardin-secret', 'theme-space-hub', 'theme-maison-chaleureuse');
      body.classList.add(`theme-${selectedTheme}`);
      body.classList.toggle('dark-mode', darkMode);
      document.documentElement.setAttribute('data-theme', selectedTheme);
      document.documentElement.setAttribute('data-dark',  darkMode.toString());
      localStorage.setItem('nook-theme',     selectedTheme);
      localStorage.setItem('nook-dark-mode', darkMode.toString());
    }
  }

  function setTheme(themeId: string) { selectedTheme = themeId; applyTheme(); }
  function toggleDarkMode()          { darkMode = !darkMode; applyTheme(); }

  async function updateProfile() {
    if (!userName.trim()) { error = 'Le nom ne peut pas être vide'; return; }
    saving = true; error = ''; message = '';
    try {
      const response = await fetch('/api/user/update', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        credentials: 'include', body: JSON.stringify({ name: userName, avatar_style: selectedAvatarStyle, avatar_seed: selectedAvatarSeed }),
      });
      const raw = await response.text();
      let data: any = {};
      if (raw.trim()) { try { data = JSON.parse(raw); } catch {} }
      if (response.ok) {
        authStore.updateUser({ name: userName, avatar_style: selectedAvatarStyle, avatar_seed: selectedAvatarSeed });
        message = 'Profil mis à jour avec succès';
      }
      else error = data?.message ?? `Erreur ${response.status}`;
    } catch (e) { error = e instanceof Error ? e.message : 'Erreur de connexion'; }
    finally { saving = false; }
  }

  async function changePassword() {
    if (!currentPassword || !newPassword || !confirmPassword) { error = 'Veuillez remplir tous les champs'; return; }
    if (newPassword.length < 8) { error = 'Au moins 8 caractères requis'; return; }
    if (newPassword !== confirmPassword) { error = 'Les mots de passe ne correspondent pas'; return; }
    saving = true; error = ''; message = '';
    try {
      const response = await fetch('/api/auth/change-password', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
      });
      const raw = await response.text();
      let data: any = {};
      if (raw.trim()) { try { data = JSON.parse(raw); } catch {} }
      if (response.ok) { message = 'Mot de passe modifié avec succès'; currentPassword = ''; newPassword = ''; confirmPassword = ''; }
      else error = data?.message ?? `Erreur ${response.status}`;
    } catch (e) { error = e instanceof Error ? e.message : 'Erreur de connexion'; }
    finally { saving = false; }
  }

  // ── Push handlers ─────────────────────────────────────────────────────────
  async function handlePushToggle() {
    pushLoading = true; pushMessage = ''; pushError = '';
    try {
      if (pushState?.subscribed) {
        const res = await unsubscribePush();
        if (res.success) { pushMessage = 'Notifications désactivées'; }
        else             { pushError = res.error ?? 'Erreur désabonnement'; }
      } else {
        const res = await subscribeToPush();
        if (res.success) { pushMessage = 'Notifications activées 🔔'; }
        else             { pushError = res.error ?? 'Erreur activation'; }
      }
      pushState = await getPushState();
    } finally {
      pushLoading = false;
    }
  }
</script>

<svelte:head><title>Paramètres - Nook</title></svelte:head>

<div class="settings-container">
  <header class="page-header">
    <h1><Icon name="settings" size="24" /> Paramètres</h1>
  </header>

  <div class="tabs" role="tablist">
    <button role="tab" class="tab" class:active={activeTab === 'profile'}    onclick={() => (activeTab = 'profile')}>Profil</button>
    <button role="tab" class="tab" class:active={activeTab === 'security'}   onclick={() => (activeTab = 'security')}><Icon name="lock" size="18" /> Sécurité</button>
    <button role="tab" class="tab" class:active={activeTab === 'appearance'} onclick={() => (activeTab = 'appearance')}>Apparence</button>
  </div>

  <!-- PROFIL -->
  {#if activeTab === 'profile'}
    <div class="settings-section">
      <!-- Avatar -->
      <div class="avatar-section">
        <label>Avatar</label>
        <p class="help-text">Choisissez un style puis sélectionnez votre avatar parmi les propositions.</p>
        <div class="avatar-preview">
          <Avatar username={authStore.user?.username ?? ''} name={userName} size={80} userId={authStore.user?.id} style={selectedAvatarStyle} seed={selectedAvatarSeed} />
        </div>

        <!-- Style selector -->
        <label>Sélectionner un style</label>
        <div class="avatar-style-grid">
          {#each avatarStyles as opt}
            <button
              type="button"
              class="avatar-style-option"
              class:selected={selectedAvatarStyle === opt.id}
              onclick={() => selectStyle(opt.id)}
              title={opt.label}
            >
              <img
                src="https://api.dicebear.com/9.x/{opt.id}/svg?seed=Nook&size=36"
                alt={opt.label}
                class="avatar-style-img"
                loading="lazy"
              />
              <span class="avatar-style-label">{opt.label}</span>
            </button>
          {/each}
        </div>

        <!-- Seed grid (avatars within chosen style) -->
        <label>Choisissez votre avatar</label>
        <div class="avatar-seed-grid">
          {#each avatarGridSeeds as s}
            <button
              type="button"
              class="avatar-seed-option"
              class:selected={selectedAvatarSeed === s}
              onclick={() => selectSeed(s)}
              title={s}
            >
              <img
                src="https://api.dicebear.com/9.x/{selectedAvatarStyle}/svg?seed={s}&size=48"
                alt={s}
                class="avatar-seed-img"
                loading="lazy"
              />
            </button>
          {/each}
        </div>
        <button type="button" class="regenerate-btn" onclick={regenerateGrid}>🔄 Autres propositions</button>
      </div>

      <h2>Informations du profil</h2>
      <form onsubmit={(e) => { e.preventDefault(); updateProfile(); }}>
        <div class="form-group">
          <label for="userName">Prénom / Nom affiché</label>
          <input type="text" id="userName" bind:value={userName} placeholder="Votre prénom" />
          <p class="help-text">Ce nom sera visible par les autres membres</p>
        </div>
        <div class="form-group">
          <label for="userUsername">Identifiant de connexion</label>
          <input type="text" id="userUsername" value={authStore.user?.username ?? ''} disabled />
          <p class="help-text">⚠️ L'identifiant de connexion ne peut pas être modifié. Vous devez toujours utiliser <strong>{authStore.user?.username ?? 'admin'}</strong> pour vous connecter, même si vous avez changé votre nom affiché.</p>
        </div>
        <button type="submit" class="btn btn-primary" disabled={saving}>
          {saving ? 'Enregistrement…' : 'Enregistrer'}
        </button>
      </form>
    </div>

  <!-- SÉCURITÉ -->
  {:else if activeTab === 'security'}
    <div class="settings-section">
      <h2>Changer le mot de passe</h2>
      <form onsubmit={(e) => { e.preventDefault(); changePassword(); }}>
        <div class="form-group">
          <label for="currentPassword">Mot de passe actuel</label>
          <PasswordInput id="currentPassword" bind:value={currentPassword} autocomplete="current-password" />
        </div>
        <div class="form-group">
          <label for="newPassword">Nouveau mot de passe</label>
          <PasswordInput id="newPassword" bind:value={newPassword} autocomplete="new-password" />
          <p class="help-text">Au moins 8 caractères</p>
        </div>
        <div class="form-group">
          <label for="confirmPassword">Confirmer le nouveau mot de passe</label>
          <PasswordInput id="confirmPassword" bind:value={confirmPassword} autocomplete="new-password" />
        </div>
        <button type="submit" class="btn btn-primary" disabled={saving}>
          {saving ? 'Modification…' : 'Changer le mot de passe'}
        </button>
      </form>
    </div>

    <!-- ── Notifications push ────────────────────────────────────────── -->
    <div class="settings-section push-section">
      <h2>🔔 Notifications push</h2>
      <p class="section-desc">
        Recevez une notification sur cet appareil quand un message est envoyé dans Nook,
        même quand l'application est en arrière-plan.
      </p>

      {#if pushState === null}
        <p class="push-loading">Chargement…</p>

      {:else if !pushState.supported}
        <div class="push-unsupported">
          <span class="push-icon">🚫</span>
          <p>Les notifications push ne sont pas supportées par ce navigateur.</p>
          <p class="help-text">Essayez Chrome, Firefox ou Safari (iOS 16.4+).</p>
        </div>

      {:else if pushState.permission === 'denied'}
        <div class="push-blocked">
          <span class="push-icon">🔕</span>
          <p>Les notifications sont bloquées dans les paramètres du navigateur.</p>
          <p class="help-text">Pour les réactiver : cliquez sur l'icône 🔒 dans la barre d'adresse → Notifications → Autoriser.</p>
        </div>

      {:else}
        <div class="push-toggle-row">
          <div class="push-info">
            <span class="push-icon">{pushState.subscribed ? '🔔' : '🔕'}</span>
            <div>
              <p class="push-status">
                {pushState.subscribed ? 'Notifications activées sur cet appareil' : 'Notifications désactivées'}
              </p>
              <p class="help-text">
                {pushState.subscribed
                  ? 'Vous recevrez une notification pour chaque nouveau message.'
                  : 'Activez pour être notifié même quand Nook est fermé.'}
              </p>
            </div>
          </div>
          <button
            class="btn {pushState.subscribed ? 'btn-danger' : 'btn-primary'}"
            onclick={handlePushToggle}
            disabled={pushLoading}
          >
            {#if pushLoading}
              {pushState.subscribed ? 'Désactivation…' : 'Activation…'}
            {:else}
              {pushState.subscribed ? 'Désactiver' : 'Activer les notifications'}
            {/if}
          </button>
        </div>

        {#if pushMessage}
          <div role="alert" class="alert alert-success">{pushMessage}</div>
        {/if}
        {#if pushError}
          <div role="alert" class="alert alert-error">❌ {pushError}</div>
        {/if}
      {/if}
    </div>

  <!-- APPARENCE -->
  {:else if activeTab === 'appearance'}
    <div class="settings-section">
      <h2>Thème de l'application</h2>
      <p class="section-desc">Choisissez l'ambiance visuelle de Nook. Le changement est immédiat sur toutes les pages.</p>
      <div class="themes-grid">
        {#each themes as theme}
          <button
            class="theme-card"
            class:selected={selectedTheme === theme.id}
            onclick={() => setTheme(theme.id)}
            aria-pressed={selectedTheme === theme.id}
            style="--preview-bg:{theme.preview.bg}; --preview-accent:{theme.preview.accent}; --preview-text:{theme.preview.text}; --preview-bubble:{theme.preview.bubble};"
          >
            <div class="theme-preview">
              <div class="preview-header">
                <span class="preview-dot" style="background:{theme.preview.accent}"></span>
                <span class="preview-title" style="color:{theme.preview.text}">Nook</span>
              </div>
              <div class="preview-bubble" style="background:{theme.preview.bubble}; color:{theme.preview.text}">Bonjour !</div>
              <div class="preview-btn" style="background:{theme.preview.accent}">→</div>
            </div>
            <div class="theme-info">
              <span class="theme-icon">{theme.icon}</span>
              <span class="theme-name">{theme.name}</span>
              <span class="theme-description">{theme.description}</span>
            </div>
            {#if selectedTheme === theme.id}
              <span class="selected-badge">✓ Actif</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="form-group">
        <label class="toggle-label" for="darkModeToggle">
          <div>
            <span class="toggle-title">🌙 Mode sombre</span>
            <p class="help-text">Superpose un filtre sombre sur le thème actuel</p>
          </div>
          <div class="toggle-switch">
            <input type="checkbox" id="darkModeToggle" checked={darkMode} onchange={toggleDarkMode} />
            <span class="toggle-slider"></span>
          </div>
        </label>
      </div>
    </div>
  {/if}

  <!-- FEEDBACK global -->
  {#if message}
    <div role="alert" class="alert alert-success">✅ {message}</div>
  {/if}
  {#if error}
    <div role="alert" class="alert alert-error">❌ {error}</div>
  {/if}
</div>

<style>
  * { box-sizing: border-box; }

  .settings-container {
    min-height: 100vh;
    padding: 1.5rem 1rem;
    max-width: 800px;
    margin: 0 auto;
    color: var(--text-primary);
  }

  .page-header { text-align: center; margin-bottom: 2rem; }
  .page-header h1 { font-size: 1.75rem; font-weight: 700; margin: 0 0 0.5rem 0; color: var(--text-primary); }

  .tabs {
    display: flex; gap: 0.5rem; margin-bottom: 2rem;
    background: var(--bg-secondary); padding: 0.5rem;
    border-radius: var(--radius-xl); border: 1px solid var(--border);
    box-shadow: var(--depth);
  }

  .tab {
    flex: 1; padding: 0.85rem 1rem; border: none; background: transparent;
    color: var(--text-secondary); font-size: 0.9rem; font-weight: 600;
    cursor: pointer; transition: all 0.2s; border-radius: var(--radius-lg);
  }
  .tab:hover  { background: var(--bg-tertiary); color: var(--text-primary); }
  .tab.active { background: var(--accent); color: #fff; box-shadow: var(--shadow-md); }

  .settings-section {
    background: var(--bg-primary); padding: 2rem;
    border-radius: var(--radius-xl); border: 1px solid var(--border);
    box-shadow: var(--depth); margin-bottom: 1.5rem;
    animation: var(--fade-in);
  }

  .settings-section h2 {
    font-size: 1.2rem; font-weight: 700; margin: 0 0 0.5rem 0;
    color: var(--text-primary); padding-bottom: 0.75rem;
    border-bottom: 2px solid var(--border);
  }

  .section-desc { color: var(--text-secondary); font-size: 0.9rem; margin: 0 0 1.5rem 0; }

  .form-group { margin-bottom: 1.5rem; }
  .form-group label { display: block; margin-bottom: 0.5rem; font-weight: 600; color: var(--text-primary); font-size: 0.95rem; }

  .form-group input[type='text'],
  .form-group input[type='password'] {
    width: 100%; padding: 0.75rem; border: 2px solid var(--border);
    border-radius: var(--radius-lg); font-size: 1rem; background: var(--input-bg);
    color: var(--text-primary); transition: border-color 0.2s, box-shadow 0.2s; outline: none;
  }
  .form-group input[type='text']:focus,
  .form-group input[type='password']:focus {
    border-color: var(--accent); box-shadow: var(--glow-sm);
  }
  .form-group input:disabled { opacity: 0.6; cursor: not-allowed; }
  .help-text { margin: 0.4rem 0 0 0; font-size: 0.82rem; color: var(--text-secondary); }
  .help-text strong { color: var(--accent-dark, var(--accent)); }

  .btn {
    padding: 0.75rem 1.5rem; border: none; border-radius: var(--radius-lg);
    font-size: 1rem; font-weight: 600; cursor: pointer; transition: all 0.2s;
  }
  .btn-primary {
    background: var(--accent); color: #fff; box-shadow: var(--shadow-md);
  }
  .btn-primary:hover:not(:disabled) {
    background: var(--button-hover); transform: translateY(-1px); box-shadow: var(--shadow-lg);
  }
  .btn-danger {
    background: #ef4444; color: #fff; box-shadow: var(--shadow-md);
  }
  .btn-danger:hover:not(:disabled) {
    background: #dc2626; transform: translateY(-1px);
  }
  .btn:disabled { opacity: 0.6; cursor: not-allowed; transform: none !important; }

  /* ── Push notifications ── */
  .push-section { margin-top: 1.5rem; }

  .push-loading { color: var(--text-secondary); font-style: italic; }

  .push-unsupported,
  .push-blocked {
    display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
    padding: 1.5rem; background: var(--bg-secondary);
    border-radius: var(--radius-lg); text-align: center;
    border: 1px solid var(--border);
  }

  .push-toggle-row {
    display: flex; justify-content: space-between; align-items: center;
    gap: 1rem; padding: 1rem 1.25rem;
    background: var(--bg-secondary); border-radius: var(--radius-xl);
    border: 2px solid var(--border); flex-wrap: wrap;
  }

  .push-info {
    display: flex; align-items: center; gap: 1rem; flex: 1; min-width: 0;
  }

  .push-icon { font-size: 2rem; flex-shrink: 0; }

  .push-status { font-weight: 600; color: var(--text-primary); margin: 0 0 0.2rem 0; }

  /* ─── Grille de thèmes avec aperçu live ─── */
  .themes-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 1rem; margin-bottom: 2rem; }

  .theme-card {
    position: relative;
    display: flex; flex-direction: column; align-items: center; gap: 0.75rem;
    padding: 1rem; background: var(--preview-bg, var(--bg-secondary));
    border: 2px solid var(--border); border-radius: var(--radius-xl);
    cursor: pointer; transition: all 0.25s; overflow: hidden;
  }
  .theme-card:hover { transform: translateY(-3px); box-shadow: var(--shadow-lg); }
  .theme-card.selected {
    border-color: var(--preview-accent, var(--accent));
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--preview-accent, var(--accent)) 30%, transparent);
  }

  .theme-preview {
    width: 100%; background: var(--preview-bg); border-radius: var(--radius-lg);
    padding: 0.5rem; display: flex; flex-direction: column; gap: 0.4rem;
    border: 1px solid color-mix(in srgb, var(--preview-text) 15%, transparent);
  }
  .preview-header { display: flex; align-items: center; gap: 0.4rem; }
  .preview-dot    { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .preview-title  { font-size: 0.7rem; font-weight: 700; }
  .preview-bubble {
    align-self: flex-end; font-size: 0.65rem; padding: 0.25rem 0.5rem;
    border-radius: 8px 8px 0 8px; max-width: 80%;
  }
  .preview-btn {
    align-self: flex-end; font-size: 0.6rem; color: #fff; width: 20px; height: 20px;
    border-radius: 50%; display: flex; align-items: center; justify-content: center;
  }

  .theme-info { display: flex; flex-direction: column; align-items: center; gap: 0.2rem; }
  .theme-icon        { font-size: 1.5rem; }
  .theme-name        { font-weight: 700; font-size: 0.9rem; color: var(--preview-text); }
  .theme-description { font-size: 0.75rem; color: color-mix(in srgb, var(--preview-text) 70%, transparent); text-align: center; }

  .selected-badge {
    position: absolute; top: 0.5rem; right: 0.5rem;
    background: var(--preview-accent, var(--accent)); color: #fff;
    font-size: 0.65rem; font-weight: 700; padding: 0.15rem 0.4rem;
    border-radius: var(--radius-full);
  }

  /* ─── Toggle mode sombre ─── */
  .toggle-label {
    display: flex; justify-content: space-between; align-items: center;
    cursor: pointer; margin: 0; padding: 1rem 1.25rem;
    background: var(--bg-secondary); border-radius: var(--radius-xl);
    border: 2px solid var(--border); transition: border-color 0.2s;
  }
  .toggle-label:hover { border-color: var(--accent); }
  .toggle-title { font-weight: 600; color: var(--text-primary); }
  .toggle-switch { position: relative; display: inline-block; width: 52px; height: 28px; flex-shrink: 0; }
  .toggle-switch input { opacity: 0; width: 0; height: 0; }
  .toggle-slider {
    position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0;
    background: var(--text-muted); border-radius: 34px; transition: .4s;
  }
  .toggle-slider:before {
    position: absolute; content: ""; height: 20px; width: 20px;
    left: 4px; bottom: 4px; background: white; border-radius: 50%; transition: .4s;
  }
  input:checked + .toggle-slider { background: var(--accent); }
  input:checked + .toggle-slider:before { transform: translateX(24px); }

  /* ─── Alertes feedback ─── */
  .alert {
    padding: 1rem 1.25rem; border-radius: var(--radius-lg); margin-bottom: 1.5rem;
    font-size: 0.9rem; animation: var(--fade-in);
  }
  .alert-success {
    background: color-mix(in srgb, #4ade80 15%, transparent);
    border: 1px solid color-mix(in srgb, #4ade80 40%, transparent);
    color: var(--text-primary);
  }
  .alert-error {
    background: color-mix(in srgb, #f87171 15%, transparent);
    border: 1px solid color-mix(in srgb, #f87171 40%, transparent);
    color: var(--text-primary);
  }

  :global(.dark-mode) .settings-section,
  :global(.dark-mode) .tabs {
    filter: brightness(0.85);
  }

  @media (max-width: 768px) {
    .tabs { flex-direction: column; }
    .tab  { text-align: center; }
    .settings-section { padding: 1.25rem; }
    .themes-grid { grid-template-columns: 1fr 1fr; }
    .push-toggle-row { flex-direction: column; align-items: flex-start; }
  }
  @media (max-width: 480px) {
    .themes-grid { grid-template-columns: 1fr; }
    .settings-container { padding: 1rem 0.75rem; }
    .btn { width: 100%; }
    .toggle-label { flex-direction: row; }
  }

  .avatar-section {
    margin-bottom: 1.5rem;
  }
  .avatar-section > label {
    display: block;
    font-weight: 600;
    margin-bottom: 0.25rem;
    margin-top: 1rem;
    color: var(--text-primary);
  }
  .avatar-section > label:first-child {
    margin-top: 0;
  }
  .avatar-preview {
    margin: 1rem 0;
    display: flex;
    justify-content: center;
  }
  /* Style selector (horizontal row) */
  .avatar-style-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 1rem;
  }
  .avatar-style-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 6px 8px;
    border-radius: 10px;
    border: 2px solid var(--border-color, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    cursor: pointer;
    transition: all 0.2s;
  }
  .avatar-style-option:hover {
    border-color: var(--accent-color, #4ade80);
    transform: scale(1.05);
  }
  .avatar-style-option.selected {
    border-color: var(--accent-color, #4ade80);
    border-width: 3px;
    background: var(--bg-accent, #dcfce7);
  }
  .avatar-style-img {
    width: 36px;
    height: 36px;
    border-radius: 50%;
  }
  .avatar-style-label {
    font-size: 0.65rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }
  /* Seed grid (clickable avatars) */
  .avatar-seed-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 8px;
    margin-bottom: 0.75rem;
  }
  .avatar-seed-option {
    padding: 4px;
    border-radius: 12px;
    border: 3px solid transparent;
    background: var(--bg-secondary, #f8fafc);
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .avatar-seed-option:hover {
    border-color: var(--accent-color, #4ade80);
    transform: scale(1.08);
  }
  .avatar-seed-option.selected {
    border-color: var(--accent-color, #4ade80);
    background: var(--bg-accent, #dcfce7);
    box-shadow: 0 0 8px rgba(74, 222, 128, 0.4);
  }
  .avatar-seed-img {
    width: 48px;
    height: 48px;
    border-radius: 50%;
  }
  .regenerate-btn {
    padding: 6px 16px;
    border-radius: 8px;
    border: 1px solid var(--border-color, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.2s;
  }
  .regenerate-btn:hover {
    background: var(--bg-tertiary, #e2e8f0);
  }
</style>
