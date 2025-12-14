<!-- frontend/src/routes/+layout.svelte -->
<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  onMount(() => {
    // Applique le thème sauvegardé
    currentTheme.subscribe(theme => {
      document.body.className = `theme-${theme}`;
      // Charge la police si nécessaire (Space Hub)
      if (theme === 'space') {
        const link = document.createElement('link');
        link.href = 'https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700&display=swap';
        link.rel = 'stylesheet';
        document.head.appendChild(link);
      }
      // Police Maison (Caveat)
      if (theme === 'maison') {
        const link = document.createElement('link');
        link.href = 'https://fonts.googleapis.com/css2?family=Caveat:wght@400;700&display=swap';
        link.rel = 'stylesheet';
        document.head.appendChild(link);
      }
    });
  });
</script>

<slot />