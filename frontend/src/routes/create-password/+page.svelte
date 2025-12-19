<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  let memberId = $state('');
  let username = $state('');
  let password = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);
  let memberName = $state('');
  let hasPassword = $state(false);

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    memberId = urlParams.get('member_id') || '';
    
    if (!memberId) {
      error = 'Lien invalide. Aucun ID membre fourni.';
      return;
    }

    checkMemberStatus();
  });

  const checkMemberStatus = async () => {
    try {
      const response = await fetch(`/api/member/check-password?member_id=${memberId}`);
      if (response.ok) {
        const data = await response.json();
        memberName = data.name;
        hasPassword = data.has_password;
        
        if (hasPassword) {
          success = 'Vous avez déjà un mot de passe. <a href="/login" class="underline text-[var(--accent)]">Connectez-vous ici</a>';
        }
      } else if (response.status === 404) {
        error = 'Membre non trouvé ou non encore approuvé par l\'administrateur.';
      }
    } catch (err) {
      error = 'Impossible de vérifier le statut du membre.';
    }
  };

  const createPassword = async () => {
    if (!password.trim() || !confirmPassword.trim()) {
      error = 'Veuillez remplir tous les champs';
      return;
    }

    if (password !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas';
      return;
    }

    if (password.length < 6) {
      error = 'Le mot de passe doit faire au moins 6 caractères';
      return;
    }

    if (username.trim() && username.length < 3) {
      error = 'Le nom d\'utilisateur doit faire au moins 3 caractères';
      return;
    }

    isLoading = true;
    error = '';

    try {
      const response = await fetch('/api/member/create-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          member_id: memberId,
          username: username.trim(),
          password: password
        })
      });

      const data = await response.json();

      if (response.ok) {
        if (data.success) {
          success = '✅ Mot de passe créé avec succès ! ';
          success += username.trim() 
            ? `Vous pouvez maintenant vous connecter avec "<strong>${username.trim()}</strong>"`
            : `Vous pouvez maintenant vous connecter avec votre ID "<strong>${memberId}</strong>"`;
          
          setTimeout(() => {
            goto('/login');
          }, 3000);
        } else {
          error = data.message;
        }
      } else {
        error = data.message || 'Erreur lors de la création du mot de passe';
      }
    } catch (err) {
      error = 'Impossible de contacter le serveur';
    } finally {
      isLoading = false;
    }
  };
</script>

<svelte:head>
  <title>Créer votre mot de passe — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4">
  {#if hasPassword}
    <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">
      <div class="text-6xl mb-6">✅</div>
      <h1 class="text-2xl font-bold mb-4 text-[var(--text-primary)]">Mot de passe déjà défini</h1>
      <p class="mb-6 text-[var(--text-secondary)]">Vous avez déjà créé un mot de passe pour votre compte.</p>
      <a href="/login" class="inline-block py-3 px-6 bg-[var(--accent)] text-white rounded-xl font-semibold hover:opacity-90">
        Se connecter
      </a>
    </div>
  {:else if memberName}
    <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8">
      <div class="text-6xl mb-6 text-center">🔐</div>
      
      <h1 class="text-2xl font-bold mb-2 text-center text-[var(--text-primary)]">Créer votre mot de passe</h1>
      <p class="text-center text-[var(--text-secondary)] mb-6">
        Bonjour <strong>{memberName}</strong> ! Créez un mot de passe pour votre compte.
      </p>

      {#if error}
        <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30">
          {error}
        </div>
      {/if}

      {#if success}
        <div class="mb-4 p-3 bg-green-500/20 text-green-600 dark:text-green-400 rounded-xl border border-green-500/30">
          {@html success}
        </div>
      {:else}
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-2 text-[var(--text-primary)]">
              Nom d'utilisateur (optionnel)
            </label>
            <input
              type="text"
              bind:value={username}
              placeholder="Choisissez un nom facile à retenir"
              class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            />
            <p class="text-xs text-[var(--text-secondary)] mt-1">
              Si laissé vide, vous utiliserez votre ID pour vous connecter
            </p>
          </div>

          <div>
            <label class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Mot de passe</label>
            <input
              type="password"
              bind:value={password}
              placeholder="Au moins 6 caractères"
              class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            />
          </div>

          <div>
            <label class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Confirmer le mot de passe</label>
            <input
              type="password"
              bind:value={confirmPassword}
              placeholder="Retapez votre mot de passe"
              class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
              onkeydown={(e) => e.key === 'Enter' && createPassword()}
            />
          </div>

          <button
            onclick={createPassword}
            disabled={isLoading}
            class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 transition"
          >
            {isLoading ? 'Création en cours...' : 'Créer mon mot de passe'}
          </button>
        </div>

        <div class="mt-6 p-4 bg-blue-500/10 border border-blue-500/20 rounded-xl">
          <p class="text-sm text-[var(--text-primary)]">
            <strong>Votre ID membre :</strong>
            <code class="block mt-1 p-2 bg-black/20 dark:bg-white/10 rounded text-sm break-all">{memberId}</code>
            <span class="text-xs block mt-2 text-[var(--text-secondary)]">
              Gardez cet ID au cas où, mais vous pourrez utiliser votre nom d'utilisateur pour vous connecter.
            </span>
          </p>
        </div>
      {/if}

      <div class="mt-6 text-center">
        <a href="/login" class="text-[var(--accent)] hover:underline">
          Déjà un mot de passe ? Connectez-vous
        </a>
      </div>
    </div>
  {:else if error}
    <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">
      <div class="text-6xl mb-6">❌</div>
      <h1 class="text-2xl font-bold mb-4 text-[var(--text-primary)]">Lien invalide</h1>
      <p class="mb-6 text-[var(--text-secondary)]">{error}</p>
      <a href="/" class="inline-block py-3 px-6 bg-gray-200 dark:bg-gray-700 rounded-xl text-[var(--text-primary)]">
        Retour à l'accueil
      </a>
    </div>
  {:else}
    <div class="text-center">
      <div class="inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-[var(--accent)] border-r-transparent"></div>
      <p class="mt-4 text-[var(--text-secondary)]">Vérification de votre compte...</p>
    </div>
  {/if}
</div>

<style>
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  .animate-spin { animation: spin 1s linear infinite; }
</style>