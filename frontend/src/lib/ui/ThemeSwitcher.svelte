<!-- frontend/src/lib/ui/ThemeSwitcher.svelte -->
<script>
  import { currentTheme, type Theme } from '$lib/ui/ThemeStore';

  const themes: { id: Theme; name: string; icon: string }[] = [
    { id: 'jardin', name: 'Jardin Secret', icon: '🌿' },
    { id: 'space', name: 'Space Hub', icon: '🚀' },
    { id: 'maison', name: 'Maison Chaleureuse', icon: '🏠' }
  ];

  let selected = $state('jardin');

  $effect(() => {
    currentTheme.subscribe(theme => selected = theme)();
  });

  const selectTheme = (id: Theme) => {
    currentTheme.set(id);
  };
</script>

<div class="theme-switcher p-4 bg-white rounded shadow">
  <h3 class="font-bold mb-2">Choisir un thème</h3>
  <div class="flex gap-2">
    {#each themes as theme}
      <button
        class="p-2 rounded border {selected === theme.id ? 'border-green-500' : 'border-gray-300'}"
        onclick={() => selectTheme(theme.id)}
      >
        {theme.icon}
      </button>
    {/each}
  </div>
  <div class="mt-2 text-sm">
    {#each themes as theme}
      {#if selected === theme.id}
        <span class="font-medium">{theme.name}</span>
      {/if}
    {/each}
  </div>
</div>

<style>
  .theme-switcher {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 1000;
    background: var(--bg-primary);
    border: 1px solid var(--border);
  }
</style>