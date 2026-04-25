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

  const CDN_BASE = 'https://api.dicebear.com/9.x';

  function getAvatarStyle(): string {
    // Use explicit style prop, or fallback to default
    return style || 'adventurer';
  }

  function getSeed(): string {
    return seed || username || userId || 'nook';
  }

  function getDicebearUrl(): string {
    const s = getAvatarStyle();
    if (s === 'initials') {
      return `${CDN_BASE}/initials/svg?seed=${encodeURIComponent(getSeed())}&size=${size}`;
    }
    return `${CDN_BASE}/${s}/svg?seed=${encodeURIComponent(getSeed())}&size=${size}`;
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
  }
</script>

{#if !imgFailed && getAvatarStyle() !== 'initials'}
  <img
    class="avatar-img"
    src={getDicebearUrl()}
    alt={name || username}
    width={size}
    height={size}
    title={name || username}
    loading="lazy"
    onerror={handleError}
  />
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
