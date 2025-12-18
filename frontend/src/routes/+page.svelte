<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { goto } from '$app/navigation';
  import { checkAuth } from '$lib/auth'; // <-- À créer (voir plus bas)

  let userStatus = $state('loading'); // 'loading' | 'invited' | 'approved' | 'admin' | 'guest'
  let memberId = $state(null);
  let error = $state('');

  // Vérifie le statut de l'utilisateur au chargement
  onMount(async () => {
    try {
      const status = await checkAuth();
      userStatus = status.status;
      memberId = status.memberId;

      // Redirige si déjà pleinement authentifié
      if (userStatus === 'approved') {
        goto('/chat');
      } else if (userStatus === 'admin') {
        goto('/admin');
      }
    } catch (err) {
      console.error('Erreur vérification auth:', err);
      userStatus = 'guest'; // Par défaut, montre l'interface invitée
    }
  });

  // Pour les invités : demande à rejoindre
  const requestToJoin = async () => {
    try {
      const response = await fetch('/api/join', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: name.trim(),
          // NOTE: Dans la vraie version, il faudra générer une clé publique ici
          public_key: 'temp_key_' + Date.now()
        })
      });

      if (response.ok) {
        const data = await response.json();
        userStatus = 'invited';
        memberId = data.memberId; // Le backend doit renvoyer l'ID
        error = '';
      } else {
        error = 'Erreur lors de la demande. Lien invalide ?';
      }
    } catch (err) {
      error = 'Impossible de contacter le serveur.';
    }
  };

  // Pour les membres invités (en attente) : tenter de se connecter
  const tryLogin = async () => {
    if (!memberId) return;
    try {
      const response = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ member_id: memberId })
      });

      if (response.ok) {
        // Le cookie 'nook_session' est défini par le serveur
        userStatus = 'approved';
        goto('/chat');
      } else {
        error = 'Votre compte n\'est pas encore approuvé par l\'admin.';
      }
    } catch (err) {
      error = 'Erreur de connexion.';
    }
  };
</script>

<svelte:head>
  <title>Nook – Votre espace familial privé</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-start md:justify-center p-4 md:p-6 relative overflow-hidden">
  <!-- Carte glassmorphism centrale -->
  <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-6 md:p-10 text-center transition-all duration-1000 animate-fade-in mt-10 md:mt-0">
    
    <!-- Emoji et titre -->
    <div class="text-6xl md:text-8xl mb-6 md:mb-8 animate-float">
      {#if $currentTheme === 'jardin-secret'}
        🌿
      {:else if $currentTheme === 'space-hub'}
        🚀
      {:else if $currentTheme === 'maison-chaleureuse'}
        🏠
      {:else}
        🌿
      {/if}
    </div>

    <h1 class="text-4xl md:text-5xl font-extrabold mb-3 md:mb-4 bg-gradient-to-r from-[var(--accent)] to-[var(--accent-light, var(--accent))] bg-clip-text text-transparent">
      Nook
    </h1>
    
    <p class="text-base md:text-lg text-[var(--text-secondary)] mb-8 md:mb-10 opacity-90">
      Votre espace familial privé et sécurisé
    </p>

    <!-- ÉTAT : Chargement -->
    {#if userStatus === 'loading'}
      <div class="py-10">
        <div class="inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-[var(--accent)] border-r-transparent"></div>
        <p class="mt-4 text-[var(--text-secondary)]">Vérification de votre accès...</p>
      </div>

    <!-- ÉTAT : Invité (arrivé via un lien d'invitation) -->
    {:else if userStatus === 'guest'}
      <p class="text-sm text-[var(--text-secondary)] mb-6">
        Vous avez été invité à rejoindre ce Nook privé.
      </p>
      {#if error}
        <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-2xl border border-red-500/30">
          {error}
        </div>
      {/if}
      <input
        type="text"
        bind:value={name}
        placeholder="Votre prénom"
        class="w-full p-4 rounded-2xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] placeholder-[var(--text-secondary)/70] focus:outline-none focus:ring-4 focus:ring-[var(--accent)/40] transition-all duration-300 mb-4"
        onkeydown={(e) => e.key === 'Enter' && requestToJoin()}
      />
      <button
        on:click={requestToJoin}
        class="w-full py-4 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-xl hover:shadow-2xl hover:scale-105 transition-all duration-300"
      >
        Demander à rejoindre
      </button>

    <!-- ÉTAT : Invité (demande envoyée, en attente d'approbation) -->
    {:else if userStatus === 'invited'}
      <div class="py-6">
        <div class="text-5xl mb-4">⏳</div>
        <h3 class="text-xl font-bold mb-2">Demande envoyée !</h3>
        <p class="text-[var(--text-secondary)] mb-6">
          L'administrateur du groupe doit approuver votre demande. Vous pouvez vérifier manuellement si c'est fait.
        </p>
        {#if error}
          <div class="mb-4 p-3 bg-yellow-500/20 text-yellow-600 dark:text-yellow-400 rounded-2xl border border-yellow-500/30">
            {error}
          </div>
        {/if}
        <div class="flex gap-3">
          <button
            on:click={tryLogin}
            class="flex-1 py-3 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-xl hover:shadow-2xl transition-all duration-300"
          >
            Me connecter
          </button>
          <a
            href="/"
            class="flex-1 py-3 bg-white/20 dark:bg-black/20 text-[var(--text-primary)] font-semibold rounded-2xl border border-white/30 text-center hover:bg-white/30 transition"
          >
            Rafraîchir
          </a>
        </div>
      </div>

    <!-- ÉTAT : Erreur ou inconnu -->
    {:else}
      <div class="py-6">
        <p class="text-[var(--text-secondary)] mb-6">
          Impossible de déterminer votre statut. Essayez de rafraîchir la page.
        </p>
        <a
          href="/"
          class="inline-block py-3 px-6 bg-[var(--accent)] text-white font-semibold rounded-2xl shadow-xl hover:shadow-2xl transition"
        >
          Retour à l'accueil
        </a>
      </div>
    {/if}

    <p class="mt-6 md:mt-10 text-xs md:text-sm text-[var(--text-secondary)/80]">
      ✅ Zéro tracking • ✅ Chiffrement E2EE • ✅ Open-source
    </p>
  </div>

  <!-- ThemeSwitcher en bas à droite -->
  <div class="absolute bottom-8 right-8">
    <ThemeSwitcher />
  </div>
</div>

<style>
  /* (Conserve les styles d'animation existants du fichier original) */
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(30px); }
    to { opacity: 1; transform: translateY(0); }
  }
  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }
  .animate-fade-in { animation: fade-in 1.2s ease-out forwards; }
  .animate-float { animation: float 5s infinite ease-in-out; }
</style>