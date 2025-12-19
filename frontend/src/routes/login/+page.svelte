<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  let identifier = $state(''); // Peut être username ou member_id
  let password = $state('');
  let error = $state('');
  let isLoading = $state(false);
  let loginType = $state('auto'); // 'auto', 'member', 'admin'
  let showAdminHint = $state(false);

  onMount(() => {
    // Vérifie si déjà connecté
    checkExistingSession();
  });

  const checkExistingSession = async () => {
    try {
      // Vérifie session utilisateur
      const userRes = await fetch('/api/validate-session', {
        credentials: 'include'
      });
      if (userRes.ok) {
        goto('/chat');
        return;
      }

      // Vérifie session admin
      const adminRes = await fetch('/api/admin/validate', {
        credentials: 'include'
      });
      if (adminRes.ok) {
        goto('/admin');
        return;
      }
    } catch (err) {
      // Pas de session, reste sur la page
    }
  };

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

    isLoading = true;
    error = '';

    try {
      let response;
      
      if (loginType === 'admin' || identifier.toLowerCase() === 'admin') {
        // Tentative de connexion admin
        response = await fetch('/api/admin/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ 
            username: identifier.trim(), 
            password: password.trim() 
          }),
          credentials: 'include'
        });
        
        if (response.ok) {
          // Vérifie si besoin de changer le mot de passe (première connexion)
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
        }
      }
      
      // Si pas admin OU échec admin, essaie connexion membre
      response = await fetch('/api/member/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          identifier: identifier.trim(), 
          password: password.trim() 
        }),
        credentials: 'include'
      });

      if (response.ok) {
        goto('/chat');
      } else if (response.status === 401) {
        error = 'Identifiants incorrects ou compte non approuvé';
      } else {
        error = 'Erreur de connexion';
      }
    } catch (err) {
      console.error('Login error:', err);
      error = 'Impossible de contacter le serveur';
    } finally {
      isLoading = false;
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter') {
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
      Accédez à votre espace familial
    </p>

    {#if showAdminHint}
      <div class="mb-4 p-3 bg-blue-500/20 border border-blue-500/40 text-blue-600 dark:text-blue-400 rounded-xl">
        <p class="text-sm">
          <strong>Connexion admin détectée</strong><br>
          Utilisez vos identifiants administrateur
        </p>
      </div>
    {/if}

    {#if error}
      <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30">
        {error}
      </div>
    {/if}

    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
          Identifiant
        </label>
        <input
          type="text"
          bind:value={identifier}
          placeholder="Nom d'utilisateur, ID membre ou 'admin'"
          oninput={detectLoginType}
          class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          onkeydown={handleKeyPress}
        />
        <p class="text-xs text-[var(--text-secondary)] mt-2 text-left">
          Pour les admins : utilisez "admin"<br>
          Pour les membres : votre nom d'utilisateur ou ID
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
          Mot de passe
        </label>
        <input
          type="password"
          bind:value={password}
          placeholder="Votre mot de passe"
          class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          onkeydown={handleKeyPress}
        />
      </div>

      <button
        onclick={handleLogin}
        disabled={isLoading}
        class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 transition"
      >
        {isLoading ? 'Connexion...' : 'Se connecter'}
      </button>
    </div>

    <div class="mt-8 space-y-4">
      <div class="p-4 bg-white/10 dark:bg-black/10 rounded-xl">
        <p class="text-sm text-[var(--text-secondary)] mb-2">
          <strong>Première connexion ?</strong>
        </p>
        <p class="text-xs text-[var(--text-secondary)]">
          • <strong>Admin</strong> : Identifiants par défaut : "admin" / "admin123"<br>
          • <strong>Membre</strong> : Utilisez l'ID reçu après inscription
        </p>
      </div>

      <div class="flex flex-col gap-2">
        <a href="/join" class="text-sm text-[var(--accent)] hover:underline">
          🎁 J'ai un lien d'invitation
        </a>
        <a href="/create-password" class="text-sm text-[var(--accent)] hover:underline">
          🔐 Créer un mot de passe (après approbation)
        </a>
        <a href="/" class="text-sm text-[var(--text-secondary)] hover:underline mt-2">
          ← Retour à l'accueil
        </a>
      </div>
    </div>
  </div>
</div>