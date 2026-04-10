<script lang="ts">
  let { name, size = 20, className = '' }: { name: string; size?: number; className?: string } = $props();

  const basePath = '/icons';

  let src = $derived(
    name === 'logo'
      ? '/logo-animated.svg'
      : `${basePath}/${name}.svg`
  );
</script>

<img {src} alt={name} width={size} height={size} class="icon {className}" />
