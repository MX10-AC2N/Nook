<script>
  // Charger les thèmes CSS ici (correctement bundlés par Vite)
  import '../lib/ui/themes/jardin.css';
  import '../lib/ui/themes/space.css';
  import '../lib/ui/themes/maison.css';

  // Initialiser le thème au chargement
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  onMount(() => {
    // Appliquer le thème sauvegardé
    currentTheme.subscribe(theme => {
      document.body.className = `theme-${theme}`;
    });
  });
</script>

<slot />