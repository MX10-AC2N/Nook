<script>
  import { onMount } from 'svelte';
  import { generateDeviceLink, registerDevice } from '$lib/device';
  import { saveMessages, loadMessages } from '$lib/storage';

  let deviceLink = '';
  let exportData = '';

  const createDeviceLink = () => {
    deviceLink = generateDeviceLink();
  };

  const exportMessages = async () => {
    // À implémenter avec la clé privée
    exportData = 'JSON chiffré exporté (à copier)';
  };
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Paramètres</h1>
  
  <div class="mb-6">
    <h2 class="text-xl font-semibold mb-2">Appareils</h2>
    <button onclick={createDeviceLink} class="bg-blue-500 text-white p-2 rounded">
      Ajouter un appareil
    </button>
    {#if deviceLink}
      <div class="mt-2 p-2 bg-gray-100 rounded">
        <p>Lien d’appareil :</p>
        <input type="text" value={deviceLink} class="w-full mt-1 p-1" readonly />
      </div>
    {/if}
  </div>

  <div>
    <h2 class="text-xl font-semibold mb-2">Export</h2>
    <button onclick={exportMessages} class="bg-green-500 text-white p-2 rounded">
      Exporter les messages
    </button>
    {#if exportData}
      <textarea class="w-full mt-2 p-2 border rounded" rows="4" readonly>{exportData}</textarea>
    {/if}
  </div>
</div>