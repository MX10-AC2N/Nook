<script>
  import { exportBackup, importBackup } from '$lib/backup';
  import { sendEmergencyAlert } from '$lib/emergency';

  let backupFile;
  let emergencyMessage = '';

  const handleExport = async () => {
    const privateKey = localStorage.getItem('nook-keys')?.privateKey;
    if (privateKey) {
      // Charger les messages depuis IndexedDB
      const messages = []; // à récupérer via storage.ts
      exportBackup(messages, privateKey);
    }
  };

  const handleImport = async (e) => {
    backupFile = e.target.files[0];
    const privateKey = localStorage.getItem('nook-keys')?.privateKey;
    if (privateKey && backupFile) {
      const messages = await importBackup(backupFile, privateKey);
      console.log('Messages restaurés:', messages);
    }
  };

  const triggerEmergency = async () => {
    if (emergencyMessage) {
      await sendEmergencyAlert(emergencyMessage);
      emergencyMessage = '';
    }
  };
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Sécurité & Sauvegarde</h1>

  <div class="mb-6">
    <h2 class="text-xl font-semibold mb-2">Sauvegarde</h2>
    <button onclick={handleExport} class="bg-green-500 text-white p-2 rounded mr-2">Exporter</button>
    <label class="bg-blue-500 text-white p-2 rounded cursor-pointer">
      Importer
      <input type="file" accept=".bin" on:change={handleImport} class="hidden" />
    </label>
  </div>

  <div>
    <h2 class="text-xl font-semibold mb-2">Alerte d’urgence</h2>
    <textarea
      bind:value={emergencyMessage}
      placeholder="Message d’urgence (envoyé à tous les membres)"
      class="w-full p-2 border rounded mb-2"
      rows="3"
    ></textarea>
    <button onclick={triggerEmergency} class="bg-red-500 text-white p-2 rounded">
      ⚠️ Envoyer une alerte d’urgence
    </button>
  </div>
</div>