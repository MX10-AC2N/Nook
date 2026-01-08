<script lang="ts">
  export let name: string;
  export let size: string = '1em';
  export let color: string = 'currentColor';
  export let className: string = '';

  // Chemin de base pour les icônes standards
  const iconsBase = '/icons';

  // Cas spécial logo animé (fichier externe dans static/)
  $: isLogo = name === 'logo';
  \( : src = isLogo ? '/logo-animated.svg' : ` \){iconsBase}/${name}.svg`;
</script>

<!-- Utilisation de <object> pour charger les SVG externes 
     → Hérite currentColor si le SVG utilise fill/stroke="currentColor" -->
<object
  type="image/svg+xml"
  data={src}
  width={size}
  height={size}
  style="pointer-events: none; color: {color};"
  class={className}
  aria-label={name}
>
  <!-- Fallback img (rare, mais sécurisé pour vieux navigateurs) -->
  <img src={src} alt={name} width={size} height={size} />
</object>

<style>
  object,
  img {
    display: inline-block;
    vertical-align: middle;
  }
</style>