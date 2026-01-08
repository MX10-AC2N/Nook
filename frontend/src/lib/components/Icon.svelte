<script lang="ts">
  export let name: string;
  export let size: string = '1em';
  export let color: string = 'currentColor';
  export let className: string = '';

  const basePath = '/icons';

  // Chemin du SVG : logo animé externe ou icône standard
  \( : src = name === 'logo' ? '/logo-animated.svg' : ` \){basePath}/${name}.svg`;
</script>

<object
  type="image/svg+xml"
  data={src}
  width={size}
  height={size}
  style="pointer-events: none; color: {color};"
  class={className}
  aria-label={name}
>
  <img src={src} alt={name} width={size} height={size} />
</object>

<style>
  object,
  img {
    display: inline-block;
    vertical-align: middle;
  }
</style>