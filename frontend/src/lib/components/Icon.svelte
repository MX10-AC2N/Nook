<script lang="ts">
  export let name: string;
  export let size = '1em';
  export let color = 'currentColor';
  export let class: string = '';
  
  // Pour le logo animé, on utilise un import différent
  import logoAnimated from '$lib/assets/logo-animated.svg?url';
  
  const icons: Record<string, string> = {
    'logo': logoAnimated,
    'call': '/icons/call.svg',
    'upload': '/icons/upload.svg',
    // ... autres icônes
  };
</script>

{#if name === 'logo'}
  <!-- Pour le logo animé, on inclut le SVG directement -->
  <svg 
    xmlns="http://www.w3.org/2000/svg" 
    viewBox="0 0 120 120"
    style="width: {size}; height: {size}; color: {color};"
    class={class}
  >
    <!-- Contenu du SVG animé ici -->
    <!-- Vous pouvez copier le contenu du SVG animé ci-dessus -->
  </svg>
{:else}
  <img 
    src={icons[name] || `/icons/${name}.svg`} 
    alt={name}
    style="width: {size}; height: {size}; color: {color}; fill: {color};"
    class={class}
  />
{/if}