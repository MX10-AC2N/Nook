<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '$lib/ui/ThemeSwitcher.svelte';

  let name = $state('');
  let error = $state('');

  onMount(() => {
    const storedName = localStorage.getItem('nook-name');
    if (storedName) {
      window.location.href = '/chat';
    }
  });

  const join = () => {
    if (name.trim()) {
      localStorage.setItem('nook-name', name.trim());
      window.location.href = '/chat';
    } else {
      error = 'Veuillez entrer votre prénom.';
    }
  };

  const handleKeyUp = (e) => {
    if (e.key === 'Enter') {
      join();
    }
  };
</script>

<div class="flex flex-col items-center justify-center min-h-screen bg-white p-4">
  <div class="max-w-md w-full">
    <h1 class="text-3xl font-bold text-center mb-6 text-gray-800">Nook</h1>
    <p class="text-center mb-8 text-gray-600">Rejoignez votre espace familial privé</p>

    {#if error}
      <div class="mb-4 p-3 bg-red-100 text-red-700 rounded">{error}</div>
    {/if}

    <input
      type="text"
      bind:value={name}
      onkeyup={handleKeyUp}
      placeholder="Votre prénom"
      class="w-full p-3 border border-gray-300 rounded-lg mb-4 focus:outline-none focus:ring-2 focus:ring-green-500"
    />

    <button
      onclick={join}
      class="w-full bg-green-500 hover:bg-green-600 text-white font-medium py-3 rounded-lg transition-colors"
    >
      Rejoindre
    </button>

    <div class="mt-8 text-center text-sm text-gray-500">
      <p>Protégé par le chiffrement de bout en bout</p>
      <p>Respectueux de votre vie privée</p>
    </div>
  </div>

  <!-- Sélecteur de thème -->
  <ThemeSwitcher />
</div>

<style>
  body {
    margin: 0;
    font-family: system-ui, -apple-system, sans-serif;
  }
</style>