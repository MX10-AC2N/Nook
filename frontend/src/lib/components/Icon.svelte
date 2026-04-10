<script lang="ts">
  import { onMount } from 'svelte';

  let { name, size = 20, className = '' }: { name: string; size?: number; className?: string } = $props();

  const basePath = '/icons';

  let src = $derived(
    name === 'logo'
      ? '/logo-animated.svg'
      : `${basePath}/${name}.svg`
  );

  let svgContent = $state('');
  let loaded = $state(false);

  onMount(async () => {
    try {
      const res = await fetch(src);
      if (!res.ok) { loaded = true; return; }
      const text = await res.text();
      if (text.includes('<svg')) {
        // Strip XML declaration and doctype if present
        svgContent = text
          .replace(/<\?xml[^?]*\?>\s*/g, '')
          .replace(/<!DOCTYPE[^>]*>\s*/g, '');
      }
    } catch {
      // Fall through to img fallback
    }
    loaded = true;
  });
</script>

{#if svgContent}
  <span
    class="icon icon-inline {className}"
    style="width:{size}px;height:{size}px;display:inline-flex;align-items:center;justify-content:center;"
  >{@html svgContent}</span>
{:else}
  <img {src} alt={name} width={size} height={size} class="icon {className}" />
{/if}

<style>
  .icon-inline :global(svg) {
    width: 100%;
    height: 100%;
    fill: currentColor;
    display: block;
  }
</style>
