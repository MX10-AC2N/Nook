<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { goto } from '$app/navigation';

  // États pour le formulaire de connexion
  let username = $state('');
  let password = $state('');
  let loginError = $state(null);
  let isLoggingIn = $state(false);

  // États pour le panneau admin (après connexion)
  let isAdminAuthenticated = $state(false);
  let inviteLink = $state('');
  let members = $state([]);
  let isLoading = $state(false);
  let adminError = $state(null);
  let copyFeedback = $state(false);
  let needsPasswordChange = $state(false); // <-- NOUVEAU : détection première connexion

  // Vérifie au chargement si une session admin existe déjà
  onMount(async () => {
    try {
      const checkResponse = await fetch('/api/admin/validate', {
        credentials: 'include'
      });
      
      if (checkResponse.ok) {
        // Vérifie si c'est la première connexion (mot de passe par défaut)
        const firstLoginCheck = await fetch('/api/admin/check-first-login', {
          credentials: 'include'
        });
        
        if (firstLoginCheck.ok) {
          const data = await firstLoginCheck.json();
          if (data.needs_password_change) {
            needsPasswordChange = true;
            return; // Montre le formulaire de changement de mot de passe
          }
        }
        
        isAdminAuthenticated = true;
        loadMembers();
      }
    } catch (err) {
      console.warn("Échec de validation de session admin:", err);
    }
  });

  // --- LOGIN ADMIN ---
  const handleAdminLogin = async () => {
    isLoggingIn = true;
    loginError = null;
    try {
      const response = await fetch('/api/admin/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
        credentials: 'include'
      });

      if (response.ok) {
        // Vérifie si c'est la première connexion
        const firstLoginCheck = await fetch('/api/admin/check-first-login', {
          credentials: 'include'
        });
        
        if (firstLoginCheck.ok) {
          const data = await firstLoginCheck.json();
          if (data.needs_password_change) {
            needsPasswordChange = true;
            return;
          }
        }
        
        isAdminAuthenticated = true;
        loadMembers();
      } else {
        loginError = "Identifiant ou mot de passe incorrect.";
      }
    } catch (err) {
      loginError = "Impossible de se connecter au serveur.";
      console.error('Login error:', err);
    } finally {
      isLoggingIn = false;
    }
  };

  // --- FONCTIONS DU PANNEAU ADMIN ---
  const createInvite = async () => {
    try {
      isLoading = true;
      adminError = null;
      const res = await fetch('/api/admin/invite', {
        method: 'POST',
        credentials: 'include'
      });
      if (!res.ok) throw new Error(`Erreur HTTP: ${res.status}`);
      const data = await res.json();
      inviteLink = data.message;
    } catch (err) {
      adminError = `Erreur : ${err.message}`;
    } finally {
      isLoading = false;
    }
  };

  const approveMember = async (id) => {
    try {
      adminError = null;
      const res = await fetch(`/api/admin/members/${id}/approve`, {
        method: 'PATCH',
        credentials: 'include'
      });
      if (!res.ok) throw new Error(`Erreur HTTP: ${res.status}`);
      loadMembers();
    } catch (err) {
      adminError = `Erreur : ${err.message}`;
    }
  };

  const loadMembers = async () => {
    try {
      isLoading = true;
      adminError = null;
      const res = await fetch('/api/admin/members', {
        credentials: 'include'
      });
      if (!res.ok) throw new Error(`Erreur HTTP: ${res.status}`);
      const data = await res.json();
      members = data.members || [];
    } catch (err) {
      adminError = `Erreur : ${err.message}`;
      if (err.message.includes('401')) {
        isAdminAuthenticated = false;
      }
    } finally {
      isLoading = false;
    }
  };

  const copyLink = async () => {
    try {
      await navigator.clipboard.writeText(inviteLink);
      copyFeedback = true;
      setTimeout(() => copyFeedback = false, 2000);
    } catch (err) {
      adminError = "Échec de la copie.";
    }
  };

  const handleLogout = async () => {
    await fetch('/api/admin/logout', { method: 'POST', credentials: 'include' });
    isAdminAuthenticated = false;
    username = '';
    password = '';
    members = [];
    inviteLink = '';
    needsPasswordChange = false;
  };

  // --- CHANGEMENT DE MOT DE PASSE ---
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let changePasswordError = $state('');
  let isChangingPassword = $state(false);

  const handleChangePassword = async () => {
    if (newPassword !== confirmPassword) {
      changePasswordError = 'Les mots de passe ne correspondent pas';
      return;
    }

    if (newPassword.length < 8) {
      changePasswordError = 'Le mot de passe doit faire au moins 8 caractères';
      return;
    }

    isChangingPassword = true;
    changePasswordError = '';

    try {
      const response = await fetch('/api/admin/change-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          current_password: currentPassword,
          new_password: newPassword
        }),
        credentials: 'include'
      });

      if (response.ok) {
        // Mot de passe changé avec succès
        needsPasswordChange = false;
        isAdminAuthenticated = true;
        loadMembers();
      } else {
        changePasswordError = 'Mot de passe actuel incorrect';
      }
    } catch (err) {
      changePasswordError = 'Erreur de connexion';
    } finally {
      isChangingPassword = false;
    }
  };
</script>

<svelte:head>
  <title>Admin — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-start md:justify-center p-4 md:p-6 relative overflow-hidden">
  <div class="max-w-md md:max-w-2xl w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-6 md:p-10 transition-all duration-1000 animate-fade-in mt-10 md:mt-0">
    
    <!-- En-tête commune -->
    <div class="flex items-center gap-3 md:gap-4 mb-6 md:mb-8">
      <div class="text-4xl md:text-5xl">
        {#if $currentTheme === 'jardin-secret'}
          🔑
        {:else if $currentTheme === 'space-hub'}
          🔒
        {:else}
          🛡️
        {/if}
      </div>
      <h1 class="text-2xl md:text-3xl font-extrabold text-[var(--text-primary)]">
        {#if needsPasswordChange}
          🔐 Changement de mot de passe obligatoire
        {:else if isAdminAuthenticated}
          Administration Nook
        {:else}
          Connexion Admin
        {/if}
      </h1>
    </div>

    <!-- 1. FORMULAIRE DE CHANGEMENT DE MOT DE PASSE (première connexion) -->
    {#if needsPasswordChange}
      <div class="space-y-6">
        <div class="p-4 bg-yellow-500/20 border border-yellow-500/40 rounded-xl">
          <p class="text-yellow-700 dark:text-yellow-300 font-medium">
            ⚠️ Pour des raisons de sécurité, vous devez changer le mot de passe par défaut.
          </p>
          <p class="text-sm mt-1 text-yellow-600 dark:text-yellow-400">
            Le mot de passe par défaut est <strong>admin123</strong>
          </p>
        </div>

        {#if changePasswordError}
          <div class="p-3 bg-red-500/20 border border-red-500/40 text-red-600 dark:text-red-400 rounded-xl">
            {changePasswordError}
          </div>
        {/if}

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Mot de passe actuel</label>
            <input
              type="password"
              bind:value={currentPassword}
              placeholder="admin123 (par défaut)"
              class="w-full p-3 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)]"
            />
          </div>

          <div>
            <label class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Nouveau mot de passe</label>
            <input
              type="password"
              bind:value={newPassword}
              placeholder="Au moins 8 caractères"
              class="w-full p-3 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)]"
            />
          </div>

          <div>
            <label class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Confirmer le nouveau mot de passe</label>
            <input
              type="password"
              bind:value={confirmPassword}
              placeholder="Retapez le nouveau mot de passe"
              class="w-full p-3 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)]"
              onkeydown={(e) => e.key === 'Enter' && handleChangePassword()}
            />
          </div>

          <button
            onclick={handleChangePassword}
            disabled={isChangingPassword}
            class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50"
          >
            {isChangingPassword ? 'Changement en cours...' : 'Changer le mot de passe'}
          </button>
        </div>
      </div>

    <!-- 2. FORMULAIRE DE CONNEXION ADMIN -->
    {:else if !isAdminAuthenticated}
      <div>
        {#if loginError}
          <div class="mb-4 p-3 bg-red-500/20 border border-red-500/40 text-red-600 dark:text-red-400 rounded-xl">
            {loginError}
          </div>
        {/if}
        <input
          type="text"
          bind:value={username}
          placeholder="Nom d'utilisateur admin"
          class="w-full p-3 mb-3 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)]"
        />
        <input
          type="password"
          bind:value={password}
          placeholder="Mot de passe"
          onkeydown={(e) => e.key === 'Enter' && handleAdminLogin()}
          class="w-full p-3 mb-6 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)]"
        />
        <button
          onclick={handleAdminLogin}
          disabled={isLoggingIn}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl disabled:opacity-60"
        >
          {isLoggingIn ? 'Connexion...' : 'Se connecter'}
        </button>
        <p class="mt-6 text-xs text-center text-[var(--text-secondary)]">
          Identifiants par défaut après installation : <strong>admin</strong> / <strong>admin123</strong>
        </p>
      </div>

    <!-- 3. PANNEAU ADMIN (Affiché après connexion réussie) -->
    {:else}
      <!-- Bouton Déconnexion -->
      <div class="text-right mb-4">
        <button onclick={handleLogout} class="text-sm text-[var(--text-secondary)] hover:text-[var(--accent)]">
          Déconnexion
        </button>
      </div>

      {#if adminError}
        <div class="mb-4 p-3 bg-red-500/20 border border-red-500/40 text-red-600 dark:text-red-400 rounded-xl flex justify-between">
          <span>{adminError}</span>
          <button onclick={() => adminError = null} class="font-bold">×</button>
        </div>
      {/if}

      <!-- Bouton inviter -->
      <div class="mb-6">
        <button
          onclick={createInvite}
          disabled={isLoading}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl disabled:opacity-60"
        >
          {isLoading ? 'Création...' : 'Générer un lien d\'invitation'}
        </button>
      </div>

      <!-- Lien d'invitation généré -->
      {#if inviteLink}
        <div class="mb-8 p-4 bg-white/20 dark:bg-black/20 rounded-xl border border-white/30">
          <p class="font-medium mb-2 text-[var(--text-primary)]">Lien d'invitation :</p>
          <div class="flex gap-2">
            <input type="text" value={inviteLink} readonly class="flex-1 p-2 rounded bg-white/30 dark:bg-black/30 text-sm text-[var(--text-primary)]" />
            <button onclick={copyLink} class="px-4 bg-[var(--accent)/80] text-white rounded">
              {copyFeedback ? '✓' : 'Copier'}
            </button>
          </div>
        </div>
      {/if}

      <!-- Liste des membres -->
      <h2 class="text-xl font-bold mb-4 text-[var(--text-primary)]">Membres</h2>
      {#if isLoading && members.length === 0}
        <p class="text-center py-4 text-[var(--text-secondary)]">Chargement...</p>
      {:else if members.length === 0}
        <p class="text-center py-4 italic text-[var(--text-secondary)]">Aucun membre.</p>
      {:else}
        <div class="space-y-3">
          {#each members as member (member.id)}
            <div class="flex items-center justify-between p-4 bg-white/20 dark:bg-black/20 rounded-xl border border-white/30">
              <div>
                <div class="font-semibold text-[var(--text-primary)]">{member.name}</div>
                <div class="text-sm opacity-80 text-[var(--text-secondary)]">ID: {member.id}</div>
              </div>
              <div class="flex items-center gap-3">
                <span class={`px-3 py-1 rounded-full text-sm ${member.approved ? 'bg-green-500/30 text-green-700 dark:text-green-300' : 'bg-yellow-500/30 text-yellow-700 dark:text-yellow-300'}`}>
                  {member.approved ? '✓ Approuvé' : 'En attente'}
                </span>
                {#if !member.approved}
                  <button onclick={() => approveMember(member.id)} class="px-3 py-1 bg-green-500 text-white rounded text-sm hover:bg-green-600">
                    Approuver
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  @keyframes fade-in { from { opacity: 0; transform: translateY(30px); } to { opacity: 1; transform: translateY(0); } }
  .animate-fade-in { animation: fade-in 1s ease-out; }
</style>