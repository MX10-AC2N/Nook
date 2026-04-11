<script lang="ts">
  let { 
    username = '', 
    name = null as string | null, 
    size = 32, 
    className = '',
    userId = ''
  }: { 
    username: string; 
    name?: string | null; 
    size?: number; 
    className?: string;
    userId?: string;
  } = $props();

  // Generate initials from name or username
  const source = $derived(name || username || '?');
  const parts = $derived(source.trim().split(/\s+/));
  const initials = $derived(parts.length >= 2 
    ? (parts[0][0] + parts[1][0]).toUpperCase() 
    : source.substring(0, 2).toUpperCase());

  // Generate consistent color based on userId or username
  const seed = $derived(userId || username || 'x');
  const hash = $derived(() => {
    let h = 0;
    for (let i = 0; i < seed.length; i++) {
      h = seed.charCodeAt(i) + ((h << 5) - h);
    }
    return h;
  });
  const hue = $derived(Math.abs(hash()) % 360);
  const color = $derived(`hsl(${hue}, 55%, 55%)`);
</script>

<div 
  class="avatar {className}" 
  style="width: {size}px; height: {size}px; background-color: {color}; font-size: {size * 0.4}px;"
  title="{name || username}"
  aria-label="Avatar de {name || username}"
>
  {initials}
</div>

<style>
  .avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    color: white;
    font-weight: 600;
    text-transform: uppercase;
    user-select: none;
    flex-shrink: 0;
  }
</style>
