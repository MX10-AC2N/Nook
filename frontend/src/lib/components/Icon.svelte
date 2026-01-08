<script lang="ts">
  export let name: string;
  export let size: string = '1em';
  export let color: string = 'currentColor';
  export let className: string = '';

  // Chemin de base pour les icônes statiques
  const basePath = '/icons';

  // Cas spécial pour le logo animé (inline SVG pour animation SMIL ou CSS)
  $: isLogo = name === 'logo';
  \( : iconSrc = isLogo ? '' : ` \){basePath}/${name}.svg`;
</script>

{#if isLogo}
  <!-- Logo animé inline (copie le contenu complet de ton logo-animated.svg ici) -->
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 120 120"
    width={size}
    height={size}
    fill={color}
    class={className}
    aria-label="Logo Nook"
  >
    <!-- Colle ici le <path> ou groupes du SVG animé -->
    <!-- Exemple placeholder (remplace par ton vrai SVG animé) -->
    <circle cx="60" cy="60" r="50" fill="none" stroke={color} stroke-width="8">
      <animate attributeName="stroke-dasharray" values="0,314;314,0" dur="2s" repeatCount="indefinite" />
    </circle>
    <text x="60" y="70" text-anchor="middle" font-size="24" fill={color}>Nook</text>
  </svg>
{:else}
  <!-- Icônes standards : utilisation de <object> pour héritage currentColor (si SVG utilise currentColor) -->
  <object
    type="image/svg+xml"
    data={iconSrc}
    width={size}
    height={size}
    style="color: {color}; pointer-events: none;"
    class={className}
    aria-label={name}
  >
    <!-- Fallback img si object échoue (rare) -->
    <img src={iconSrc} alt={name} width={size} height={size} />
  </object>
{/if}

<style>
  /* Assure héritage color pour SVG chargés via object (si SVG utilise currentColor) */
  object {
    display: inline-block;
    vertical-align: middle;
  }
</style>