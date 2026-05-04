<!-- frontend/src/lib/components/NotificationToast.svelte -->
<!-- Bulles de notification in-app — fonctionne sur HTTP/LAN -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { notifications, dismissNotification, markAllRead } from '$lib/notificationStore.svelte';
  import type { AppNotification } from '$lib/notificationStore.svelte';

  // Afficher seulement les non-lues (max 5 visibles)
  let visible = $derived(notifications.items.filter(n => !n.read).slice(0, 5));
  let totalCount = $derived(notifications.items.filter(n => !n.read).length);
  let showHistory = $state(false);

  function handleClick(n: AppNotification) {
    dismissNotification(n.id);
    if (n.href) goto(n.href);
  }

  function getBgColor(type: string): string {
    const colors: Record<string, string> = {
      message:  'var(--accent, #4ade80)',
      chess:    '#8b5cf6',
      poll:     '#f59e0b',
      calendar: '#3b82f6',
      admin:    '#ef4444',
      call:     '#10b981',
      info:     '#64748b',
    };
    return colors[type] ?? colors.info;
  }
</script>

<!-- Bouton compteur (toujours visible) -->
{#if totalCount > 0}
  <button
    class="notif-badge"
    onclick={() => showHistory = !showHistory}
    aria-label="{totalCount} notification{totalCount > 1 ? 's' : ''}"
  >
    🔔 {totalCount}
  </button>
{/if}

<!-- Historique -->
{#if showHistory && notifications.items.length > 0}
  <div class="notif-backdrop" onclick={() => showHistory = false} role="presentation"></div>
  <div class="notif-history">
    <div class="notif-history-header">
      <strong>Notifications</strong>
      <button class="notif-clear" onclick={markAllRead}>Tout marquer lu</button>
    </div>
    {#each notifications.items as n (n.id)}
      <div
        class="notif-history-item"
        class:unread={!n.read}
        onclick={() => handleClick(n)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleClick(n); }}
        role="button"
        tabindex="0"
      >
        <span class="notif-icon">{n.icon}</span>
        <div class="notif-text">
          <strong>{n.title}</strong>
          <span>{n.body}</span>
        </div>
        <button class="notif-dismiss" onclick={(e) => { e.stopPropagation(); dismissNotification(n.id); }}>✕</button>
      </div>
    {/each}
  </div>
{/if}

<!-- Toasts actifs (slide-in) -->
<div class="toast-container">
  {#each visible as n (n.id)}
    <div
      class="toast"
      style="border-left: 4px solid {getBgColor(n.type)}"
      onclick={() => handleClick(n)}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleClick(n); }}
      role="button"
      tabindex="0"
    >
      <span class="toast-icon">{n.icon}</span>
      <div class="toast-body">
        <strong>{n.title}</strong>
        <p>{n.body}</p>
      </div>
      <button class="toast-close" onclick={(e) => { e.stopPropagation(); dismissNotification(n.id); }}>✕</button>
    </div>
  {/each}
</div>

<style>
  /* Badge compteur */
  .notif-badge {
    position: fixed;
    top: .75rem;
    right: .75rem;
    z-index: 9999;
    background: var(--accent, #4ade80);
    color: #fff;
    border: none;
    border-radius: 9999px;
    padding: .4rem .8rem;
    font-size: .85rem;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 2px 12px rgba(0,0,0,.2);
    animation: pop-in .3s ease-out;
  }
  .notif-badge:hover { transform: scale(1.1); }

  /* Backdrop */
  .notif-backdrop {
    position: fixed; inset: 0; z-index: 9998;
    background: rgba(0,0,0,.3);
  }

  /* Historique */
  .notif-history {
    position: fixed;
    top: 3.5rem;
    right: .75rem;
    z-index: 9999;
    width: min(360px, 90vw);
    max-height: 60vh;
    overflow-y: auto;
    background: var(--bg-primary, #fff);
    border-radius: .75rem;
    box-shadow: 0 8px 30px rgba(0,0,0,.2);
    border: 1px solid var(--border, #e2e8f0);
  }
  .notif-history-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: .75rem 1rem;
    border-bottom: 1px solid var(--border, #e2e8f0);
  }
  .notif-clear {
    background: none; border: none; color: var(--accent, #4ade80);
    cursor: pointer; font-size: .8rem; font-weight: 600;
  }
  .notif-history-item {
    display: flex; align-items: flex-start; gap: .6rem;
    padding: .6rem 1rem;
    cursor: pointer;
    border-bottom: 1px solid var(--border, #f1f5f9);
    transition: background .15s;
  }
  .notif-history-item:hover { background: var(--bg-secondary, #f8fafc); }
  .notif-history-item.unread { background: color-mix(in srgb, var(--accent, #4ade80) 8%, transparent); }
  .notif-icon { font-size: 1.3rem; flex-shrink: 0; margin-top: .1rem; }
  .notif-text { flex: 1; min-width: 0; }
  .notif-text strong { display: block; font-size: .85rem; }
  .notif-text span { font-size: .78rem; color: var(--text-secondary, #64748b); }
  .notif-dismiss {
    background: none; border: none; color: var(--text-secondary);
    cursor: pointer; font-size: .9rem; padding: .2rem;
  }

  /* Toasts */
  .toast-container {
    position: fixed;
    bottom: 1.5rem;
    right: 1rem;
    z-index: 10000;
    display: flex;
    flex-direction: column-reverse;
    gap: .5rem;
    pointer-events: none;
  }
  .toast {
    display: flex; align-items: flex-start; gap: .6rem;
    padding: .75rem 1rem;
    background: var(--bg-primary, #fff);
    border-radius: .6rem;
    box-shadow: 0 4px 20px rgba(0,0,0,.15);
    min-width: 280px;
    max-width: 380px;
    cursor: pointer;
    pointer-events: auto;
    animation: slide-in .3s ease-out;
    border: 1px solid var(--border, #e2e8f0);
  }
  .toast:hover { transform: translateX(-4px); }
  .toast-icon { font-size: 1.5rem; flex-shrink: 0; }
  .toast-body { flex: 1; min-width: 0; }
  .toast-body strong { display: block; font-size: .85rem; margin-bottom: .15rem; }
  .toast-body p {
    margin: 0; font-size: .78rem; color: var(--text-secondary, #64748b);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .toast-close {
    background: none; border: none; color: var(--text-secondary);
    cursor: pointer; font-size: .9rem; padding: .2rem; flex-shrink: 0;
  }

  @keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to   { transform: translateX(0); opacity: 1; }
  }
  @keyframes pop-in {
    from { transform: scale(0); }
    to   { transform: scale(1); }
  }

  @media (max-width: 480px) {
    .toast-container { left: .5rem; right: .5rem; }
    .toast { min-width: auto; max-width: none; }
    .notif-history { left: .5rem; right: .5rem; width: auto; }
  }
</style>
