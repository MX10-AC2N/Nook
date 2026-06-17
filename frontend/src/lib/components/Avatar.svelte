<script lang="ts">
  const DICEBEAR_STYLES = [
    { id: 'adventurer',  label: 'Aventurier',  icon: '🎮' },
    { id: 'avataaars',   label: 'Cartoon',     icon: '😊' },
    { id: 'open-peeps',  label: 'Illustré',    icon: '✏️' },
    { id: 'notionists',  label: 'Minimaliste', icon: '🎨' },
    { id: 'fun-emoji',   label: 'Emoji',       icon: '😄' },
    { id: 'big-smile',   label: 'Sourire',     icon: '😁' },
    { id: 'lorelei',     label: 'Portrait',    icon: '🧑' },
    { id: 'personas',    label: 'Personas',    icon: '💼' },
    { id: 'bottts',      label: 'Robot',       icon: '🤖' },
    { id: 'initials',    label: 'Initiales',   icon: '🔤' },
  ];

  let { username = '', name = null, size = 32, userId = '', style = '', seed = '' }: {
    username?: string;
    name?: string | null;
    size?: number;
    userId?: string;
    style?: string;
    seed?: string;
  } = $props();

  let imgFailed = $state(false);
  let imgLoading = $state(true);

  const CDN_BASE = '/api/avatar';

  // Timeout fallback: si l'image ne charge pas en 3s, on passe au fallback
  $effect(() => {
    if (imgLoading) {
      const timeout = setTimeout(() => {
        imgFailed = true;
        imgLoading = false;
      }, 3000);
      return () => clearTimeout(timeout);
    }
  });

  function getAvatarStyle(): string {
    // Use explicit style prop, or fallback to default
    return style || 'adventurer';
  }

  function getSeed(): string {
    return seed || username || userId || 'nook';
  }

  function getAvatarUrl(): string {
    const s = getAvatarStyle();
    const params = new URLSearchParams();
    params.set('seed', getSeed());
    params.set('size', String(size));
    return `${CDN_BASE}/${s}/svg?${params.toString()}`;
  }

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
    return `hsl(${Math.abs(hash) % 360}, 55%, 55%)`;
  }

  function handleError() {
    imgFailed = true;
    imgLoading = false;
  }

  function handleLoad() {
    imgLoading = false;
  }
</script>

{#if !imgFailed && !imgLoading && getAvatarStyle() !== 'initials'}
  <img
    class="avatar-img"
    src={getAvatarUrl()}
    alt={name || username}
    width={size}
    height={size}
    title={name || username}
    loading="lazy"
    onerror={handleError}
    onload={handleLoad}
  />
{:else if imgLoading}
  <div
    class="avatar-fallback"
    style="width: {size}px; height: {size}px; background-color: {getColor()}; font-size: {size * 0.4}px;"
    title={name || username}
  >
    {getInitials()}
  </div>
{:else}
  <div
    class="avatar-fallback"
    style="width: {size}px; height: {size}px; background-color: {getColor()}; font-size: {size * 0.4}px;"
    title={name || username}
  >
    {getInitials()}
  </div>
{/if}

<style>
  .avatar-img {
    border-radius: 50%;
    flex-shrink: 0;
    object-fit: cover;
    display: block;
  }
  .avatar-fallback {
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
