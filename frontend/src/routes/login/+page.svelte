<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { loginMember, loginAdmin, checkAuth } from '$lib/authStore';

  let identifier = $state(''); // Peut être username ou member_id
  let password = $state('');
  let error = $state('');
  let isLoading = $state(false);
  let loginType = $state('auto'); // 'auto', 'member', 'admin'
  let showAdminHint = $state(false);

  onMount(async () => {
    // Vérifie si déjà connecté
    const isAuthenticated = await checkAuth();
    if (isAuthenticated) {
      goto('/chat');
    }
  });

  const detectLoginType = () => {
    // Si l'identifiant est "admin", c'est probablement un admin
    if (identifier.toLowerCase() === 'admin') {
      loginType = 'admin';
      showAdminHint = true;
    } else if (identifier.includes('@')) {
      // Si ça contient @, c'est peut-être un email (futur fonctionnalité)
      loginType = 'member';
    } else {
      // Par défaut, essaye les deux
      loginType = 'auto';
    }
  };

  const handleLogin = async () => {
    if (!identifier.trim() || !password.trim()) {
      error = 'Veuillez remplir tous les champs';
      return;
    }

    if (password.length < 1) {
      error = 'Le mot de passe est requis';
      return;
    }

    isLoading = true;
    error = '';

    try {
      let result;
      
      if (loginType === 'admin' || identifier.toLowerCase() === 'admin') {
        // Tentative de connexion admin
        result = await loginAdmin(identifier.trim(), password.trim());
        
        if (result.success) {
          // Vérifier si l'admin doit changer son mot de passe
          const firstLoginCheck = await fetch('/api/admin/check-first-login', {
            credentials: 'include'
          });
          
          if (firstLoginCheck.ok) {
            const data = await firstLoginCheck.json();
            if (data.needs_password_change) {
              // Premier login admin, doit changer le mot de passe
              goto('/admin');
              return;
            }
          }
          
          goto('/admin');
          return;
        } else {
          // Si échec admin, essaie connexion membre
          result = await loginMember(identifier.trim(), password.trim());
        }
      } else {
        // Tentative de connexion membre
        result = await loginMember(identifier.trim(), password.trim());
      }

      if (result.success) {
        // Vérifier si le membre doit changer son mot de passe temporaire
        const changeCheck = await fetch('/api/member/check-password-change', {
          credentials: 'include'
        });
        
        if (changeCheck.ok) {
          const data = await changeCheck.json();
          if (data.needs_password_change) {
            // Premier login, doit changer le mot de passe temporaire
            goto('/change-password');
            return;
          }
        }
        
        goto('/chat');
      } else {
        error = result.error || 'Identifiants incorrects';
      }
      
    } catch (err) {
      console.error('Login error:', err);
      error = 'Impossible de contacter le serveur';
    } finally {
      isLoading = false;
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && !isLoading) {
      handleLogin();
    }
  };
</script>

<svelte:head>
  <title>Connexion — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4">
  <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">
    
    <div class="text-6xl mb-6">
      🔐
    </div>

    <h1 class="text-3xl font-bold mb-2 text-[var(--text-primary)]">Connexion à Nook</h1>
    
    <p class="text-[var(--text-secondary)] mb-8">
      Accédez à votre espace familial sécurisé
    </p>

    {#if showAdminHint}
      <div class="mb-4 p-3 bg-blue-500/20 border border-blue-500/40 text-blue-600 dark:text-blue-400 rounded-xl animate-pulse">
        <p class="text-sm">
          <strong>Connexion admin détectée</strong><br>
          Utilisez vos identifiants administrateur
        </p>
      </div>
    {/if}

    {#if error}
      <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30 animate-fade-in">
        <div class="flex items-center gap-2">
          <span>⚠️</span>
          <span>{error}</span>
        </div>
      </div>
    {/if}

    <div class="space-y-4">
      <div>
        <label for="login-identifier" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
          Identifiant
        </label>
        <input
          id="login-identifier"
          type="text"
          bind:value={identifier}
          placeholder="Nom d'utilisateur, ID membre ou 'admin'"
          oninput={detectLoginType}
          class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          onkeydown={handleKeyPress}
          disabled={isLoading}
        />
        <p class="text-xs text-[var(--text-secondary)] mt-2 text-left">
          • Pour les <strong>admins</strong> : utilisez "admin"<br>
          • Pour les <strong>membres</strong> : votre nom d'utilisateur
        </p>
      </div>

      <div>
        <label for="login-password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
          Mot de passe
        </label>
        <input
          id="login-password"
          type="password"
          bind:value={password}
          placeholder="Votre mot de passe"
          class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          onkeydown={handleKeyPress}
          disabled={isLoading}
        />
      </div>

      <button
        onclick={handleLogin}
        disabled={isLoading}
        class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center justify-center gap-2"
      >
        {#if isLoading}
          <span class="animate-spin">⟳</span>
          <span>Connexion en cours...</span>
        {:else}
          <span>→</span>
          <span>Se connecter</span>
        {/if}
      </button>
    </div>

    <div class="mt-8 space-y-4">
      <div class="p-4 bg-white/10 dark:bg-black/10 rounded-xl">
        <p class="text-sm text-[var(--text-secondary)] mb-2">
          <strong>Première connexion ?</strong>
        </p>
        <p class="text-xs text-[var(--text-secondary)]">
          • <strong>Admin</strong> : Identifiants par défaut : "admin" / "admin123"<br>
          • <strong>Membre</strong> : Utilisez le nom d'utilisateur fourni par l'administrateur
        </p>
      </div>

      <div class="flex flex-col gap-2">
        <a href="/join" class="text-sm text-[var(--accent)] hover:underline flex items-center gap-1">
          <span>🎁</span>
          <span>J'ai un lien d'invitation (ancien système)</span>
        </a>
        <a href="/create-password" class="text-sm text-[var(--accent)] hover:underline flex items-center gap-1">
          <span>🔐</span>
          <span>Créer un mot de passe (ancien système)</span>
        </a>
        <a href="/" class="text-sm text-[var(--text-secondary)] hover:underline mt-2 flex items-center gap-1">
          <span>←</span>
          <span>Retour à l'accueil</span>
        </a>
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  .animate-fade-in {
    animation: fade-in 0.3s ease-out;
  }
</style>