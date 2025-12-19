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
  let needsPasswordChange = $state(false);
  let passwordLinkFeedback = $state(null); // Pour les feedbacks de liens

  // Vérifie au chargement si une session admin existe déjà
  onMount(async () => {
    try {
      const checkResponse = await fetch('/api/admin/validate', {
        credentials: 'include'
      });
      
      if (checkResponse.ok) {
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
    
    console.log('Status:', res.status, res.statusText); // <-- Ajoute ce log
    
    if (!res.ok) {
      const errorText = await res.text();
      console.error('Erreur détaillée:', errorText);
      throw new Error(`Erreur ${res.status}: ${errorText}`);
    }
    
    const data = await res.json();
    members = data.members || [];
  } catch (err) {
    console.error('Erreur loadMembers:', err);
    adminError = `Erreur : ${err.message}`;
    if (err.message.includes('401') || err.message.includes('500')) {
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

  // --- NOUVEAU : Générer le lien de création de mot de passe ---
  const getPasswordCreationLink = (memberId) => {
    // Utilise l'URL actuelle comme base
    const baseUrl = window.location.origin;
    return `${baseUrl}/create-password?member_id=${memberId}`;
  };

  const copyPasswordLink = async (memberId, memberName) => {
    try {
      const link = getPasswordCreationLink(memberId);
      await navigator.clipboard.writeText(link);
      
      passwordLinkFeedback = {
        memberId,
        message: `Lien copié pour ${memberName}`
      };
      
      setTimeout(() => {
        passwordLinkFeedback = null;
      }, 3000);
    } catch (err) {
      adminError = "Échec de la copie du lien.";
    }
  };

  const sendPasswordLinkEmail = async (memberId, memberName) => {
    // Placeholder - à implémenter si tu veux envoyer par email
    alert(`Fonctionnalité email à implémenter pour ${memberName}`);
  };

  const handleLogout = async () => {
    await fetch('/api/admin/logout', { method: 'POST', credentials: 'include' });
    isAdminAuthenticated = false;
    username = '';
    password = '';
    members = [];
    inviteLink = '';
    needsPasswordChange = false;
    passwordLinkFeedback = null;
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
  <div class="max-w-4xl w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-6 md:p-10 transition-all duration-1000 animate-fade-in mt-10 md:mt-0">
    
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

    <!-- Feedback pour lien de mot de passe -->
    {#if passwordLinkFeedback}
      <div class="mb-4 p-3 bg-green-500/20 border border-green-500/40 text-green-600 dark:text-green-400 rounded-xl animate-pulse">
        ✅ {passwordLinkFeedback.message}
      </div>
    {/if}

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
            <label for="current-password" class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Mot de passe actuel</label>
            <input
              id="current-password"
              type="password"
              bind:value={currentPassword}
              placeholder="admin123 (par défaut)"
              class="w-full p-3 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)]"
            />
          </div>

          <div>
            <label for="new-password-admin" class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Nouveau mot de passe</label>
            <input
              id="new-password-admin"
              type="password"
              bind:value={newPassword}
              placeholder="Au moins 8 caractères"
              class="w-full p-3 rounded-xl bg-white/30 dark:bg-black/30 border border-white/40 text-[var(--text-primary)]"
            />
          </div>

          <div>
            <label for="confirm-new-password" class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Confirmer le nouveau mot de passe</label>
            <input
              id="confirm-new-password"
              type="password" bind:value={confirmPassword}
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
      <!-- En-tête avec boutons -->
      <div class="flex flex-col md:flex-row justify-between items-start md:items-center mb-6 gap-4">
        <div>
          <h2 class="text-xl font-bold text-[var(--text-primary)]">Gestion des membres</h2>
          <p class="text-sm text-[var(--text-secondary)]">
            {members.length} membre{members.length !== 1 ? 's' : ''} dans la famille
          </p>
        </div>
        
        <div class="flex gap-3">
          <button
            onclick={createInvite}
            disabled={isLoading}
            class="px-4 py-2 bg-[var(--accent)] text-white font-semibold rounded-xl disabled:opacity-60 hover:opacity-90"
          >
            {isLoading ? 'Création...' : '➕ Nouvelle invitation'}
          </button>
          <button
            onclick={handleLogout}
            class="px-4 py-2 bg-gray-500/20 text-[var(--text-secondary)] font-semibold rounded-xl hover:bg-gray-500/30"
          >
            Déconnexion
          </button>
        </div>
      </div>

      <!-- Lien d'invitation généré -->
      {#if inviteLink}
        <div class="mb-8 p-4 bg-white/20 dark:bg-black/20 rounded-xl border border-white/30">
          <div class="flex justify-between items-center mb-2">
            <p class="font-medium text-[var(--text-primary)]">✨ Nouveau lien d'invitation généré</p>
            <button onclick={() => inviteLink = ''} class="text-sm text-[var(--text-secondary)] hover:text-[var(--accent)]">
              × Fermer
            </button>
          </div>
          <div class="flex flex-col md:flex-row gap-2">
            <input 
              type="text" 
              value={inviteLink} 
              readonly 
              class="flex-1 p-3 rounded bg-white/30 dark:bg-black/30 text-sm text-[var(--text-primary)]" 
            />
            <button
              onclick={copyLink}
              class="px-6 py-3 bg-[var(--accent)]/80 hover:bg-[var(--accent)] text-white rounded font-medium transition flex items-center justify-center gap-2"
            >
              {#if copyFeedback}
                <span>✅ Copié !</span>
              {:else}
                <span>📋 Copier le lien</span>
              {/if}
            </button>
          </div>
        </div>
      {/if}

      <!-- Messages d'erreur -->
      {#if adminError}
        <div class="mb-4 p-3 bg-red-500/20 border border-red-500/40 text-red-600 dark:text-red-400 rounded-xl flex justify-between">
          <span>{adminError}</span>
          <button onclick={() => adminError = null} class="font-bold hover:text-red-700">×</button>
        </div>
      {/if}

      <!-- Liste des membres -->
      {#if isLoading && members.length === 0}
        <div class="text-center py-12">
          <div class="inline-block h-8 w-8 animate-spin rounded-full border-3 border-solid border-[var(--accent)] border-r-transparent"></div>
          <p class="mt-4 text-[var(--text-secondary)]">Chargement des membres...</p>
        </div>
      {:else if members.length === 0}
        <div class="text-center py-12">
          <div class="text-5xl mb-4">👋</div>
          <h3 class="text-xl font-bold text-[var(--text-primary)] mb-2">Aucun membre pour le moment</h3>
          <p class="text-[var(--text-secondary)]">
            Générez votre premier lien d'invitation pour commencer !
          </p>
        </div>
      {:else}
        <div class="space-y-4">
          {#each members as member (member.id)}
            <div class="bg-white/10 dark:bg-black/10 rounded-xl border border-white/20 p-4 hover:bg-white/15 transition">
              <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <!-- Informations du membre -->
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-2">
                    <span class="font-semibold text-lg text-[var(--text-primary)]">{member.name}</span>
                    <span class={`px-2 py-1 rounded-full text-xs font-medium ${member.approved ? 'bg-green-500/20 text-green-700 dark:text-green-300' : 'bg-yellow-500/20 text-yellow-700 dark:text-yellow-300 animate-pulse'}`}>
                      {member.approved ? '✅ Approuvé' : '⏳ En attente'}
                    </span>
                  </div>
                  
                  <div class="flex flex-wrap gap-4 text-sm text-[var(--text-secondary)]">
                    <div class="flex items-center gap-1">
                      <span class="opacity-70">ID :</span>
                      <code class="bg-black/20 px-2 py-1 rounded text-xs">{member.id.slice(0, 8)}...</code>
                    </div>
                    <div class="flex items-center gap-1">
                      <span class="opacity-70">Inscrit le :</span>
                      <span>{new Date(member.joined_at).toLocaleDateString()}</span>
                    </div>
                  </div>

                  <!-- Indicateur mot de passe -->
                  {#if member.approved && member.has_password !== undefined && !member.has_password}
                    <div class="mt-3 p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                      <div class="flex items-center gap-2 text-sm">
                        <span class="text-blue-500">🔓</span>
                        <span class="text-blue-600 dark:text-blue-400">
                          <strong>Mot de passe non défini</strong> - Envoyez le lien ci-dessous
                        </span>
                      </div>
                    </div>
                  {/if}
                </div>

                <!-- Actions -->
                <div class="flex flex-col sm:flex-row gap-2">
                  {#if !member.approved}
                    <button
                      onclick={() => approveMember(member.id)}
                      class="px-4 py-2 bg-green-500 hover:bg-green-600 text-white font-medium rounded-lg transition flex items-center justify-center gap-2"
                    >
                      <span>✅</span>
                      <span>Approuver</span>
                    </button>
                  {:else}
                    <!-- Actions pour membres approuvés -->
                    <div class="flex flex-col gap-2">
                      <button
                        onclick={() => copyPasswordLink(member.id, member.name)}
                        class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white font-medium rounded-lg transition flex items-center justify-center gap-2"
                        title="Copier le lien de création de mot de passe"
                      >
                        <span>🔐</span>
                        <span>Lien mot de passe</span>
                      </button>
                      
                      {#if member.has_password === undefined || member.has_password === false}
                        <div class="text-xs text-center text-blue-500">
                          À envoyer au membre
                        </div>
                      {:else}
                        <div class="text-xs text-center text-green-500">
                          ✅ Mot de passe défini
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Lien généré (affiché temporairement) -->
              {#if passwordLinkFeedback && passwordLinkFeedback.memberId === member.id}
                <div class="mt-3 p-3 bg-green-500/10 border border-green-500/20 rounded-lg">
                  <div class="flex flex-col md:flex-row md:items-center justify-between gap-2">
                    <div>
                      <p class="text-sm font-medium text-green-600 dark:text-green-400 mb-1">
                        Lien de création de mot de passe pour {member.name} :
                      </p>
                      <input
                        type="text"
                        value={getPasswordCreationLink(member.id)}
                        readonly
                        class="w-full p-2 text-sm bg-black/20 rounded border border-green-500/30 text-green-700 dark:text-green-300"
                      />
                    </div>
                    <button
                      onclick={() => navigator.clipboard.writeText(getPasswordCreationLink(member.id))}
                      class="px-3 py-2 bg-green-500 hover:bg-green-600 text-white rounded text-sm font-medium"
                    >
                      Recopier
                    </button>
                  </div>
                  <p class="text-xs text-green-600/70 dark:text-green-400/70 mt-2">
                    Partagez ce lien avec {member.name} pour qu'il crée son mot de passe.
                  </p>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <!-- Guide d'utilisation -->
      <div class="mt-8 p-4 bg-white/10 dark:bg-black/10 rounded-xl border border-white/20">
        <h3 class="font-bold text-[var(--text-primary)] mb-2">📋 Guide rapide</h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
          <div class="flex items-start gap-2">
            <span class="text-green-500">1.</span>
            <span class="text-[var(--text-secondary)]">
              <strong>Générez un lien d'invitation</strong> pour ajouter un nouveau membre
            </span>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-green-500">2.</span>
            <span class="text-[var(--text-secondary)]">
              <strong>Approuvez</strong> les demandes en attente
            </span>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-green-500">3.</span>
            <span class="text-[var(--text-secondary)]">
              <strong>Envoyez le lien "Lien mot de passe"</strong> aux membres approuvés
            </span>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-green-500">4.</span>
            <span class="text-[var(--text-secondary)]">
              Les membres créent leur mot de passe sur <code>/create-password</code>
            </span>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes fade-in { from { opacity: 0; transform: translateY(30px); } to { opacity: 1; transform: translateY(0); } }
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
  .animate-fade-in { animation: fade-in 1s ease-out; }
  .animate-spin { animation: spin 1s linear infinite; }
  .animate-pulse { animation: pulse 2s infinite; }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }
</style>