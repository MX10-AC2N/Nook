<script>
  import { currentTheme } from './ThemeStore';

  const themes = [
    { id: 'jardin', name: 'Jardin Secret', icon: '🌿' },
    { id: 'space', name: 'Space Hub', icon: '🚀' },
    { id: 'maison', name: 'Maison Chaleureuse', icon: '🏠' }
  ];

  let selected = $state('jardin');

  $effect(() => {
    currentTheme.subscribe(theme => selected = theme)();
  });

  const selectTheme = (id) => {
    currentTheme.set(id);
  };
</script>

<div class="theme-switcher fixed bottom-4 right-4 z-50 bg-white rounded-xl shadow-lg p-3 border border-[var(--border)] transition-all duration-300">
  <div class="flex gap-1 mb-2">
    {#each themes as theme}
      <button
        class="w-10 h-10 rounded-full flex items-center justify-center text-xl border-2 transition-all {selected === theme.id ? 'border-[var(--accent)] scale-110' : 'border-gray-300'}"
        onclick={() => selectTheme(theme.id)}
        title={theme.name}
      >
        {theme.icon}
      </button>
    {/each}
  </div>
</div>

<style>
  .theme-switcher {
    background: var(--bg-primary);
    --shadow: var(--depth);
    box-shadow: var(--shadow);
  }
</style>