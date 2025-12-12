<script>
  let name = $state('');
  const join = () => {
    if (name.trim()) {
      localStorage.setItem('nook-name', name);
      window.location.href = '/chat';
    }
  };
</script>

<main class="p-4 max-w-md mx-auto">
  <h1 class="text-2xl font-bold text-center mb-6">Nook</h1>
  <p class="text-center mb-4">Rejoignez votre espace familial privé</p>
  <input
    bind:value={name}
    placeholder="Votre prénom"
    class="w-full p-2 border rounded mb-4"
    onkeyup={(e) => e.key === 'Enter' && join()}
  />
  <button onclick={join} class="w-full bg-green-500 text-white p-2 rounded">
    Rejoindre
  </button>
</main>

<style>
  main { margin-top: 20vh; }
</style>