<script>
  import { onMount } from 'svelte';
  import ThemeSwitcher from '../lib/ui/ThemeSwitcher.svelte';

  let name = $state('');
  let error = $state('');

  const join = () => {
    if (name.trim()) {
      localStorage.setItem('nook-name', name.trim());
      window.location.href = '/chat';
    } else {
      error = 'Veuillez entrer votre prénom.';
    }
  };

  onMount(() => {
    const storedName = localStorage.getItem('nook-name');
    if (storedName) window.location.href = '/chat';
  });
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-4 bg-[var(--bg-primary)] text-[var(--text-primary)]">
  <div class="max-w-md w-full text-center">
    <div class="text-5xl mb-4 animate-fade-in">🌿</div>
    <h1 class="text-4xl font-bold mb-2">Nook</h1>
    <p class="text-[var(--text-secondary)] mb-8">Votre espace familial privé</p>

    {#if error}
      <div class="mb-4 p-3 bg-red-100 text-red-700 rounded-lg">{error}</div>
    {/if}

    <input
      type="text"
      bind:value={name}
      placeholder="Votre prénom"
      class="w-full p-4 rounded-xl border border-[var(--border)] bg-[var(--input-bg)] text-[var(--text-primary)] mb-4 focus:outline-none focus:ring-2 focus:ring-[var(--accent)] transition-all"
      onkeydown={(e) => e.key === 'Enter' && join()}
    />

    <button
      onclick={join}
      class="w-full py-4 bg-[var(--accent)] text-white font-medium rounded-xl shadow-lg hover:shadow-xl transition-all duration-300"
    >
      Rejoindre
    </button>

    <p class="mt-8 text-sm text-[var(--text-secondary)]">
      ✅ Zéro tracking • ✅ Chiffrement E2EE • ✅ Open-source
    </p>
  </div>

  <ThemeSwitcher />
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .animate-fade-in {
    animation: fade-in 0.6s var(--animation) forwards;
  }
</style>