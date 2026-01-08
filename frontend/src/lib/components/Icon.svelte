<script lang="ts">
  // Props que le composant accepte
  export let name: string;               // nom de l’icône (ex. "home", "logo")
  export let size: string = '1em';       // taille CSS (ex. "24px", "2rem")
  export let color: string = 'currentColor'; // couleur héritée du texte
  export let className: string = '';    // classe CSS supplémentaire

  // Chemin de base où se trouvent les SVG
  const basePath = '/icons';

  // Construction du chemin complet du fichier SVG
  // Si le nom est "logo", on utilise le SVG animé spécial,
  // sinon on charge le SVG standard depuis le répertoire `basePath`.
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