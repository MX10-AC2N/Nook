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

  function getInitials(): string {
    const source = name || username || '?';
    const parts = source.trim().split(/\s+/);
    if (parts.length >= 2) {
      return (parts[0][0] + parts[1][0]).toUpperCase();
    }
    return source.substring(0, 2).toUpperCase();
  }

  function getColor(): string {
    const seed = userId || username || 'x';
    let hash = 0;
    for (let i = 0; i < seed.length; i++) {
      hash = seed.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = Math.abs(hash) % 360;
    return `hsl(${hue}, 55%, 55%)`;
  }
</script>

<div 
  class="avatar {className}" 
  style="width: {size}px; height: {size}px; background-color: {getColor()}; font-size: {size * 0.4}px;"
  title="{name || username}"
  aria-label="Avatar de {name || username}"
>
  {getInitials()}
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
