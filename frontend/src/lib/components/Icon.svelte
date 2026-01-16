<script lang="ts">
  // -----------------------------------------------------------------
  // 1️⃣ Récupération des props (Svelte 5 – $props())
  // -----------------------------------------------------------------
  // $props() renvoie un objet contenant toutes les props passées au
  // composant. On le déstructure avec des valeurs par défaut.
  const {
    name,
    size = '1em',
    color = 'currentColor',
    className = ''
  } = $props<{
    name: string;
    size?: string;
    color?: string;
    className?: string;
  }>();

  // -----------------------------------------------------------------
  // 2️⃣ Chemin de base où se trouvent les SVG
  // -----------------------------------------------------------------
  const basePath = '/icons';

  // -----------------------------------------------------------------
  // 3️⃣ src est **réactif** – il se met à jour dès que `name` change
  // -----------------------------------------------------------------
  let src = $derived(
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