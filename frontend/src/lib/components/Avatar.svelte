<script lang="ts">
  interface Props {
    username?: string;
    name?: string | null;
    size?: number;
    className?: string;
    userId?: string;
    avatarUrl?: string | null;
  }

  let { username = '', name = null, size = 32, className = '', userId = '', avatarUrl = null }: Props = $props();

  function getInitials(): string {
    const source = name || username || '?';
    const parts = source.trim().split(/\s+/);
    if (parts.length >= 2 && parts[0][0] && parts[1][0]) {
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

  const hasImage = $derived(avatarUrl && avatarUrl.length > 0);
</script>

{#if hasImage}
  <img
    src={avatarUrl}
    alt="Avatar de {name || username}"
    class="avatar avatar-img {className}"
    style="width: {size}px; height: {size}px;"
    title={name || username}
  />
{:else}
  <div
    class="avatar avatar-fallback {className}"
    style="width: {size}px; height: {size}px; background-color: {getColor()}; font-size: {size * 0.4}px;"
    title={name || username}
    aria-label="Avatar de {name || username}"
  >
    {getInitials()}
  </div>
{/if}

<style>
  .avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    user-select: none;
    flex-shrink: 0;
  }
  .avatar-fallback {
    color: white;
    font-weight: 600;
    text-transform: uppercase;
  }
  .avatar-img {
    object-fit: cover;
  }
</style>
