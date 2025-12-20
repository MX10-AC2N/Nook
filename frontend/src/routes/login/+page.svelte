<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  let username = $state('');
  let password = $state('');
  let error = $state('');
  let isLoading = $state(false);

  onMount(async () => {
    // Vérifier si déjà connecté
    try {
      const res = await fetch('/api/validate-session', {
        credentials: 'include'
      });
      
      if (res.ok) {
        const data = await res.json();
        if (data.role === 'admin') {
          goto('/admin');
        } else {
          goto('/chat');
        }
      }
    } catch (err) {
      // Pas connecté, rester sur la page
    }
  });

  const handleLogin = async () => {
    if (!username.trim() || !password) {
      error = 'Veuillez remplir tous les champs';
      return;
    }

    isLoading = true;
    error = '';

    try {
      const response = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          username: username.trim(),
          password: password
        }),
        credentials: 'include'
      });

      const data = await response.json();
      
      if (data.success) {
        // Récupérer les infos de session
        const sessionRes = await fetch('/api/validate-session', {
          credentials: 'include'
        });
        
        if (sessionRes.ok) {
          const sessionData = await sessionRes.json();
          
          if (data.message.includes('changer votre mot de passe')) {
            // Premier login, rediriger vers changement de mot de passe
            goto('/change-password');
          } else if (sessionData.role === 'admin') {
            goto('/admin');
          } else {
            goto('/chat');
          }
        } else {
          goto('/chat'); // Fallback
        }
      } else {
        error = data.message || 'Identifiants incorrects';
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
      Accédez à votre espace familial
    </p>

    {#if error}
      <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30">
        {error}
      </div>
    {/if}

    <div class="space-y-4">
      <div>
        <label for="username" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
          Identifiant
        </label>
        <input
          id="username"
          type="text"
          bind:value={username}
          placeholder="Votre identifiant"
          class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          onkeydown={handleKeyPress}
          disabled={isLoading}
        />
        <p class="text-xs text-[var(--text-secondary)] mt-2 text-left">
          • Pour l'<strong>admin</strong> : utilisez "admin"<br>
          • Pour les <strong>membres</strong> : votre identifiant personnel
        </p>
      </div>

      <div>
        <label for="password" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
          Mot de passe
        </label>
        <input
          id="password"
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
        class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition"
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
          • <strong>Membre</strong> : Utilisez l'identifiant fourni lors de l'inscription
        </p>
      </div>

      <div class="flex flex-col gap-2">
        <a href="/register" class="text-sm text-[var(--accent)] hover:underline">
          📝 Pas encore de compte ? S'inscrire
        </a>
        <a href="/" class="text-sm text-[var(--text-secondary)] hover:underline mt-2">
          ← Retour à l'accueil
        </a>
      </div>
    </div>
  </div>
</div>