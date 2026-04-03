<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { browser } from '$app/environment';
  import { callStore, callManager } from '$lib/webrtc-calls.svelte.ts';
  import { getCurrentTheme } from '$lib/ui/ThemeStore.svelte.ts';

  interface PendingCall {
    from_user_id: string;
    from_user_name: string;
    conversation_id: string;
    callType: 'audio' | 'video';
  }

  let pendingCall = $state<PendingCall | null>(null);
  let isVisible = $state(false);

  const isOnCallPage = $derived(
    browser && ($page.url?.pathname?.startsWith('/call') ?? false)
  );

  const callIcon = $derived(pendingCall?.callType === 'video' ? '\ud83d\udcf9' : '\ud83c\udfa4');
  const callLabel = $derived(
    pendingCall
      ? (pendingCall.callType === 'video' ? 'Appel vid\u00e9o' : 'Appel audio')
      : ''
  );

  $effect(() => {
    if (pendingCall && !isOnCallPage) {
      const t = setTimeout(() => { isVisible = true; }, 50);
      callManager.startRingtone();
      return () => clearTimeout(t);
    } else {
      isVisible = false;
    }
  });

  $effect(() => {
    if (!browser) return;

    function handleIncomingCall(event: Event) {
      const detail = (event as CustomEvent).detail;
      if (detail && !callStore.isInCall) {
        pendingCall = {
          from_user_id: detail.from_user_id,
          from_user_name: detail.from_user_name ?? detail.from_user_id,
          conversation_id: detail.conversation_id ?? '',
          callType: detail.callType ?? 'audio',
        };
      }
    }

    window.addEventListener('incoming-call', handleIncomingCall as EventListener);
    return () => {
      window.removeEventListener('incoming-call', handleIncomingCall as EventListener);
    };
  });

  async function handleAnswer() {
    if (!pendingCall) return;
    callManager.stopRingtone();
    callStore.isAnswering = true;
    callStore.currentConversationId = pendingCall.conversation_id;
    callStore.callType = pendingCall.callType;

    const localPc = pendingCall;
    pendingCall = null;
    isVisible = false;

    try {
      await callManager.startCall(
        localPc.conversation_id,
        [localPc.from_user_id],
        localPc.callType
      );
      goto('/call/' + localPc.conversation_id + '?type=' + localPc.callType);
    } catch (err) {
      callStore.error = err instanceof Error ? err.message : 'Erreur lors de la r\u00e9ponse';
      callStore.isAnswering = false;
    }
  }

  async function handleReject() {
    callManager.stopRingtone();
    if (pendingCall) {
      callManager.sendReject(pendingCall.from_user_id);
      pendingCall = null;
    }
    isVisible = false;
  }
</script>

{#if pendingCall && !isOnCallPage}
  <div class="call-banner" class:visible={isVisible} data-theme={getCurrentTheme()}>
    <div class="call-banner-ring">
      <span class="ring-icon">{callIcon}</span>
    </div>

    <div class="call-banner-info">
      <span class="call-banner-label">{callLabel}</span>
      <span class="call-banner-caller">{pendingCall.from_user_name}</span>
    </div>

    <div class="call-banner-actions">
      <button class="btn btn-answer" onclick={handleAnswer} aria-label="Decrocher">
        Decrocher
      </button>
      <button class="btn btn-reject" onclick={handleReject} aria-label="Refuser">
        Refuser
      </button>
    </div>
  </div>
{/if}

<style>
  .call-banner {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 10000;
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1.25rem;
    background: var(--bg-secondary, #1e1e2e);
    border-bottom: 2px solid var(--accent, #4ade80);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    transform: translateY(-100%);
    transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .call-banner.visible { transform: translateY(0); }
  .call-banner-ring {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--accent, #4ade80);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    animation: pulse-ring 1.5s ease-in-out infinite;
  }
  .ring-icon { font-size: 2rem; }
  @keyframes pulse-ring {
    0%, 100% { transform: scale(1); box-shadow: 0 0 0 0 rgba(74, 222, 128, 0.7); }
    50% { transform: scale(1.05); box-shadow: 0 0 0 10px rgba(74, 222, 128, 0); }
  }
  .call-banner-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }
  .call-banner-label {
    font-size: 0.75rem;
    color: var(--text-muted, #666);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .call-banner-caller {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .call-banner-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
  }
  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .btn:hover { transform: translateY(-1px); }
  .btn-answer { background: var(--accent, #4ade80); color: #000; }
  .btn-answer:hover { background: #3cc66e; }
  .btn-reject { background: var(--accent-danger, #ef4444); color: white; }
  .btn-reject:hover { background: #dc2626; }
  @media (max-width: 640px) {
    .call-banner { flex-direction: column; gap: 0.5rem; padding: 0.75rem; }
    .call-banner-ring { width: 40px; height: 40px; }
    .call-banner-info { text-align: center; }
    .call-banner-actions { width: 100%; }
    .call-banner-actions .btn { flex: 1; }
  }
</style>
