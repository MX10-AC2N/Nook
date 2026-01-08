<script lang="ts">
  /* ------------------------------------------------------------------
   * Récupération des propriétés en « runes mode ».
   * $props() renvoie un objet contenant toutes les props passées au
   * composant. Nous utilisons la déstructuration avec des valeurs par
   * défaut pour reproduire le comportement habituel d’« export let ».
   * ------------------------------------------------------------------ */
  const {
    name,
    size = '1em',
    color = 'currentColor',
    className = ''
  } = $props<{                     // typage générique (facultatif)
    name: string;
    size?: string;
    color?: string;
    className?: string;
  }>();

  // Chemin de base où se trouvent les SVG
  const basePath = '/icons';

  // Construction du chemin complet du fichier SVG
  const src =
    name === 'logo'
      ? '/logo-animated.svg'
      : `${basePath}/${name}.svg`;
</script>

<!--
  <object> charge le SVG externe et hérite de `currentColor`
  si le SVG utilise cette valeur.
-->
<object
  type="image/svg+xml"
  {src}
  width={size}
  height={size}
  style="pointer-events: none; color: {color};"
  class={className}
  aria-label={name}
>
  <!-- Fallback image pour les navigateurs qui ne supportent pas <object> -->
  <img src={src} alt={name} width={size} height={size} />
</object>

<style>
  object,
  img {
    display: inline-block;
    vertical-align: middle;
  }
</style>