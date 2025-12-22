<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { goto } from '@roxi/routify';  // ← Routify au lieu de $app/navigation (SvelteKit)
  import { generateKeyPair, storeKeys } from '$lib/crypto';

  let token = $state('');
  let name = $state('');
  let error = $state('');
  let success = $state('');
  let isLoading = $state(false);
  let memberId = $state('');

  onMount(() => {
    // Récupère le token depuis l'URL
    const urlParams = new URLSearchParams(window.location.search);
    token = urlParams.get('token') || '';
    
    if (!token) {
      error = 'Lien d\'invitation invalide. Demandez un nouveau lien à l\'administrateur.';
    }
  });

  const submitRequest = async () => {
    if (!name.trim()) {
      error = 'Veuillez entrer votre prénom';
      return;
    }

    if (name.trim().length < 2) {
      error = 'Le prénom doit contenir au moins 2 caractères';
      return;
    }

    isLoading = true;
    error = '';

    try {
      // Générer une paire de clés cryptographiques avec libsodium
      console.log('Génération des clés cryptographiques...');
      const keyPair = await generateKeyPair();
      console.log('Clés générées:', keyPair.publicKey.substring(0, 50) + '...');

      const response = await fetch(`/api/join?token=${encodeURIComponent(token)}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: name.trim(),
          public_key: keyPair.publicKey
        })
      });

      if (response.ok) {
        const data = await response.json();
        success = data.message;
        
        // Extraire l'ID du membre du message de succès
        const match = data.message.match(/ID: (\S+)/);
        if (match && match[1]) {
          memberId = match[1];
          
          // Stocker les clés localement avec l'ID du membre
          storeKeys(memberId, {
            publicKey: keyPair.publicKey,
            privateKey: keyPair.privateKey,
            memberId: memberId
          });
          
          console.log('Clés stockées pour le membre:', memberId);
        }
        
        error = '';
      } else if (response.status === 400) {
        error = 'Lien d\'invitation invalide ou expiré.';
      } else if (response.status === 500) {
        error = 'Erreur serveur. Veuillez réessayer plus tard.';
      } else {
        error = 'Erreur inattendue. Code: ' + response.status;
      }
    } catch (err) {
      console.error('Erreur:', err);
      error = 'Impossible de contacter le serveur. Vérifiez votre connexion.';
    } finally {
      isLoading = false;
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && !isLoading) {
      submitRequest();
    }
  };
</script>

<svelte:head>
  <title>Rejoindre Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4 relative overflow-hidden">
  <div class="max-w-md w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-8 text-center">

    <div class="text-6xl mb-6">
      {#if $currentTheme === 'jardin-secret'}
        🌿
      {:else if $currentTheme === 'space-hub'}
        🚀
      {:else}
        🏠
      {/if}
    </div>

    <h1 class="text-3xl font-bold mb-2 text-[var(--text-primary)]">Rejoindre Nook</h1>

    <p class="text-[var(--text-secondary)] mb-6">
      Vous avez été invité à rejoindre un espace familial privé et sécurisé
    </p>

    <!-- Indicateur de sécurité -->
    <div class="mb-6 p-3 bg-green-500/10 border border-green-500/30 rounded-xl">
      <div class="flex items-center justify-center gap-2 text-green-400">
        <span class="text-xl">🔒</span>
        <span class="text-sm font-medium">Connexion chiffrée de bout en bout</span>
      </div>
    </div>

    {#if error}
      <div class="mb-4 p-3 bg-red-500/20 text-red-600 dark:text-red-400 rounded-xl border border-red-500/30">
        <div class="flex items-center gap-2">
          <span>⚠️</span>
          <span>{error}</span>
        </div>
      </div>
    {/if}

    {#if success}
      <div class="mb-4 p-3 bg-green-500/20 text-green-600 dark:text-green-400 rounded-xl border border-green-500/30">
        <div class="text-4xl mb-2">✅</div>
        <p class="font-semibold text-lg">{success}</p>
        <p class="text-sm mt-2">L'administrateur vous approuvera bientôt.</p>

        {#if memberId}
          <div class="mt-4 p-2 bg-white/10 dark:bg-black/10 rounded">
            <p class="text-xs font-mono break-all">Votre ID: <span class="font-bold">{memberId}</span></p>
            <p class="text-xs mt-1">Conservez cet ID pour votre première connexion !</p>
          </div>
        {/if}

        <div class="mt-4">
          <a href="/" class="inline-block px-4 py-2 bg-[var(--accent)] text-white rounded-lg hover:opacity-90 transition">
            Retour à l'accueil
          </a>
        </div>
      </div>
    {:else if token}
      <div class="space-y-4">
        <div>
          <label for="member-name" class="block text-sm font-medium mb-2 text-left text-[var(--text-primary)]">
            Votre prénom
          </label>
          <input
            id="member-name"
            type="text"
            bind:value={name}
            placeholder="Ex: Jean, Marie, Pierre..."
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
            onkeydown={handleKeyPress}
            disabled={isLoading}
            maxlength="50"
          />
          <p class="text-xs text-[var(--text-secondary)] mt-1 text-left">
            Ce nom sera visible par les autres membres de la famille
          </p>
        </div>

        <div class="p-3 bg-blue-500/10 border border-blue-500/30 rounded-xl">
          <p class="text-xs text-blue-400">
            <span class="font-semibold">⚠️ Important :</span> 
            Des clés cryptographiques seront générées pour sécuriser vos communications.
            Ces clés seront stockées localement dans votre navigateur.
          </p>
        </div>

        <button
          onclick={submitRequest}
          disabled={isLoading || !name.trim()}
          class="w-full py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition flex items-center justify-center gap-2"
        >
          {#if isLoading}
            <span class="animate-spin">⟳</span>
            <span>Génération des clés de sécurité...</span>
          {:else}
            <span>🔐</span>
            <span>Demander à rejoindre</span>
          {/if}
        </button>
      </div>
    {:else}
      <div class="p-4 bg-yellow-500/20 text-yellow-600 dark:text-yellow-400 rounded-xl">
        <div class="text-3xl mb-2">🔗</div>
        <p class="font-medium">Vous avez besoin d'un lien d'invitation</p>
        <p class="text-sm mt-2">Contactez l'administrateur de votre espace familial pour obtenir un lien valide.</p>
      </div>
    {/if}

    <div class="mt-6 pt-4 border-t border-white/20">
      <p class="text-sm text-[var(--text-secondary)]">
        <a href="/" class="text-[var(--accent)] hover:underline flex items-center justify-center gap-1">
          <span>←</span>
          <span>Retour à l'accueil</span>
        </a>
      </p>
    </div>
  </div>

  <div class="absolute bottom-8 right-8">
    <ThemeSwitcher />
  </div>
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>