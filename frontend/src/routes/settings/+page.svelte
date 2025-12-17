<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { exportBackup, importBackup } from '$lib/backup';
  import { sendEmergencyAlert } from '$lib/emergency';

  let backupFileInput;
  let emergencyMessage = $state('');
  let exportFeedback = $state(false);
  let importFeedback = $state({ success: false, message: '' });

  const handleExport = async () => {
    // Note : Dans une version réelle, tu récupères les données depuis IndexedDB ou ton storage
    const messages = []; // Placeholder – à remplir avec tes vraies données
    const privateKeyJson = localStorage.getItem('nook-keys');
    if (!privateKeyJson) {
      importFeedback = { success: false, message: 'Clé privée manquante' };
      return;
    }
    const privateKey = JSON.parse(privateKeyJson).privateKey;

    try {
      exportBackup(messages, privateKey);
      exportFeedback = true;
      setTimeout(() => exportFeedback = false, 3000);
    } catch (err) {
      importFeedback = { success: false, message: 'Échec de l’export' };
    }
  };

  const handleImport = async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const privateKeyJson = localStorage.getItem('nook-keys');
    if (!privateKeyJson) {
      importFeedback = { success: false, message: 'Clé privée manquante' };
      return;
    }
    const privateKey = JSON.parse(privateKeyJson).privateKey;

    try {
      const messages = await importBackup(file, privateKey);
      importFeedback = { success: true, message: `${messages.length} messages restaurés !` };
      console.log('Restauration réussie :', messages);
    } catch (err) {
      importFeedback = { success: false, message: 'Fichier invalide ou clé incorrecte' };
      console.error(err);
    }

    // Reset input pour permettre réimport du même fichier
    e.target.value = '';
    setTimeout(() => importFeedback = { success: false, message: '' }, 4000);
  };

  const triggerEmergency = async () => {
    if (!emergencyMessage.trim()) return;

    try {
      await sendEmergencyAlert(emergencyMessage.trim());
      emergencyMessage = '';
      importFeedback = { success: true, message: 'Alerte envoyée à tous les membres !' };
      setTimeout(() => importFeedback = { success: false, message: '' }, 4000);
    } catch (err) {
      importFeedback = { success: false, message: 'Échec de l’envoi de l’alerte' };
    }
  };
</script>

<svelte:head>
  <title>Sécurité & Sauvegarde — Nook</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center p-6 relative">
  <!-- Carte principale glassmorphism -->
  <div class="max-w-2xl w-full bg-white/15 dark:bg-black/15 backdrop-blur-2xl border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-10 animate-fade-in">

    <!-- Header thématique -->
    <div class="flex items-center gap-5 mb-12">
      <div class="text-6xl animate-float">
        {#if $currentTheme === 'jardin-secret'}
          🔒
        {:else if $currentTheme === 'space-hub'}
          🛡️
        {:else}
          🛡️
        {/if}
      </div>
      <h1 class="text-3xl font-extrabold text-[var(--text-primary)]">Sécurité & Sauvegarde</h1>
    </div>

    <!-- Section Sauvegarde -->
    <div class="mb-12 p-6 bg-white/20 dark:bg-black/20 rounded-2xl border border-white/30 backdrop-blur-md">
      <h2 class="text-2xl font-bold mb-6 text-[var(--text-primary)]">Sauvegarde des données</h2>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <p class="text-[var(--text-secondary)] mb-4">Exportez toutes vos conversations chiffrées</p>
          <button
            onclick={handleExport}
            class="w-full py-4 bg-green-500/80 hover:bg-green-500 text-white font-semibold rounded-2xl shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-300"
          >
            📥 Exporter la sauvegarde
          </button>
          {#if exportFeedback}
            <p class="mt-3 text-center text-green-400 font-medium animate-pulse">✓ Sauvegarde téléchargée !</p>
          {/if}
        </div>

        <div>
          <p class="text-[var(--text-secondary)] mb-4">Importez une sauvegarde existante</p>
          <label class="block w-full py-4 bg-blue-500/80 hover:bg-blue-500 text-white font-semibold rounded-2xl shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-300 cursor-pointer text-center">
            📤 Importer une sauvegarde
            <input
              type="file"
              accept=".bin"
              on:change={handleImport}
              class="hidden"
              bind:this={backupFileInput}
            />
          </label>
        </div>
      </div>
    </div>

    <!-- Section Alerte d'urgence -->
    <div class="p-6 bg-red-500/10 dark:bg-red-900/10 rounded-2xl border border-red-500/30 backdrop-blur-md">
      <h2 class="text-2xl font-bold mb-6 text-red-600 dark:text-red-400">Alerte d'urgence</h2>
      
      <p class="text-[var(--text-secondary)] mb-6">
        Envoyez instantanément un message à tous les membres de la famille<br />
        (utilisé uniquement en cas de réel besoin)
      </p>

      <textarea
        bind:value={emergencyMessage}
        placeholder="Décrivez brièvement la situation..."
        rows="4"
        class="w-full p-4 mb-6 rounded-xl bg-white/20 dark:bg-black/30 border border-red-500/40 text-[var(--text-primary)] placeholder-red-400/70 focus:outline-none focus:ring-4 focus:ring-red-500/50 resize-none"
      ></textarea>

      <button
        onclick={triggerEmergency}
        disabled={!emergencyMessage.trim()}
        class="w-full py-5 bg-red-600 hover:bg-red-700 disabled:bg-red-600/50 disabled:cursor-not-allowed text-white font-bold text-xl rounded-2xl shadow-xl hover:shadow-2xl hover:scale-105 active:scale-95 transition-all duration-300 flex items-center justify-center gap-3"
      >
        <span class="text-3xl animate-pulse">⚠️</span>
        Envoyer l'alerte d'urgence
      </button>
    </div>

    <!-- Feedback global -->
    {#if importFeedback.message}
      <div class={`mt-8 p-5 rounded-2xl text-center font-medium text-lg backdrop-blur-md border ${
        importFeedback.success 
          ? 'bg-green-500/20 border-green-500/50 text-green-400' 
          : 'bg-red-500/20 border-red-500/50 text-red-400'
      } animate-pulse`}>
        {importFeedback.message}
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(40px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-15px); }
  }

  .animate-fade-in { animation: fade-in 1s ease-out forwards; }
  .animate-float { animation: float 6s infinite ease-in-out; }

  @media (prefers-reduced-motion: reduce) {
    * { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  }
</style>