<script>
  import '$lib/ui/themes/jardin.css';
  import '$lib/ui/themes/space.css';
  import '$lib/ui/themes/maison.css';
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  onMount(() => {
    currentTheme.subscribe(theme => {
      document.body.className = `theme-${theme}`;
    });
  });
</script>
<slot />