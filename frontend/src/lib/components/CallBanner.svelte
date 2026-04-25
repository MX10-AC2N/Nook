<script lang="ts">
  /**
   * CallBanner.svelte — Global notification banner for incoming WebRTC calls.
   *
   * Visible from any page (chat, settings, chess, etc.) when a call_request
   * signal arrives.  Automatically hides when the user navigates to /call/*.
   */

  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { callStore, callManager } from '$lib/webrtc-calls.svelte.ts';

  interface PendingIncomingCall {
    fromUserId: string;
    fromUserName: string;
    conversationId: string;
    callType: 'audio' | 'video';
  }

  // ── Local state ──────────────────────────────────────────────────────
  let incomingCall = $state<PendingIncomingCall | null>(null);

  const _hasPendingCall = $derived(incomingCall !== null);
  const _onCallPage   = $derived(
    $page.url.pathname.startsWith('/call/')
  );
  const _shouldShow   = $derived(
    _hasPendingCall && !_onCallPage
  );

  // SVG icons based on call type
  const _callIcon = $derived(incomingCall?.callType === 'video' 
    ? 'video' 
    : 'phone');
  const _callLabel = $derived(incomingCall?.callType === 'video' 
    ? 'Appel vidéo' 
    : 'Appel audio');

  // ── Listen for the incoming-call custom event ─────────────────────────
  if (browser) {
    $effect(() => {
      function onIncoming(e: Event) {
        const ev = e as CustomEvent;
        incomingCall = {
          fromUserId:   ev.detail.from_user_id   ?? '',
          fromUserName: ev.detail.from_user_name  ?? ev.detail.from_user_id ?? 'Inconnu',
          conversationId: ev.detail.conversationId ?? '',
          callType:     ev.detail.callType         ?? 'audio',
        };
        // Start ringtone so the user hears it immediately
        callManager.startRingtone();
      }
      window.addEventListener('incoming-call', onIncoming);
      return () => window.removeEventListener('incoming-call', onIncoming);
    });
  }

  // ── Actions ───────────────────────────────────────────────────────────
  function answerCall() {
    if (!incomingCall) return;
    const { conversationId, callType, fromUserId } = incomingCall;

    // Stop ringtone — the call page will take over
    callManager.stopRingtone();

    // Set store state so downstream code knows we answered
    callStore.isCalling = true;
    callStore.callType  = callType;
    callStore.currentConversationId = conversationId;

    // Navigate to the call page (handles WebRTC negotiation)
    goto(`/call/${conversationId}?type=${callType}`);

    // Clear local state — banner hides because _onCallPage becomes true
    incomingCall = null;
  }

  function declineCall() {
    if (!incomingCall) return;
    const { conversationId, fromUserId } = incomingCall;

    // Stop ringtone
    callManager.stopRingtone();

    // Send rejection signal to caller
    callManager.sendReject(conversationId, fromUserId);

    // Clear local state -> banner disappears
    incomingCall = null;
  }

  // If the user is already in an active call, clear any stale banner data
  $effect(() => {
    if (callStore.isInCall) {
      incomingCall = null;
    }
  });
</script>

{#if _shouldShow}
  <div class="call-banner" role="alert" aria-live="polite">
    <div class="banner-content">
      <div class="banner-left">
        <span class="ring-icon pulse">
          {#if _callIcon === 'video'}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="23 7 16 12 23 17 23 7"/>
              <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/>
            </svg>
          {/if}
        </span>
        <div class="caller-info">
          <span class="caller-name">{incomingCall?.fromUserName}</span>
          <span class="call-type">{_callLabel}</span>
        </div>
      </div>
      <div class="banner-actions">
        <button
          class="btn-decline"
          onclick={declineCall}
          aria-label="Refuser l'appel"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
          Refuser
        </button>
        <button
          class="btn-answer"
          onclick={answerCall}
          aria-label="Décrocher"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/>
          </svg>
          Décrocher
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Banner shell ──────────────────────────────────────────────── */
  .call-banner {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 9999;
    background: var(--bg-secondary, #ffffff);
    border-bottom: 2px solid var(--accent, #4ade80);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    animation: slideDown 0.35s ease-out;
  }

  @keyframes slideDown {
    from { transform: translateY(-100%); opacity: 0; }
    to   { transform: translateY(0);        opacity: 1; }
  }

  .banner-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.85rem 1.5rem;
    max-width: 900px;
    margin: 0 auto;
  }

  /* ── Left side: caller info ────────────────────────────────────── */
  .banner-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
  }

  .ring-icon {
    width: 2rem;
    height: 2rem;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .ring-icon svg {
    width: 100%;
    height: 100%;
    color: var(--accent, #4ade80);
  }

  .ring-icon.pulse {
    animation: pulse 1.2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50%      { opacity: 0.6; transform: scale(1.15); }
  }

  .caller-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .caller-name {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary, #1e293b);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .call-type {
    font-size: 0.8rem;
    color: var(--text-secondary, #64748b);
  }

  /* ── Right side: action buttons ────────────────────────────────── */
  .banner-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .btn-answer,
  .btn-decline {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.55rem 1rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }
  
  .btn-answer svg,
  .btn-decline svg {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
  }

  .btn-answer {
    background: var(--accent, #4ade80);
    color: #166534;
  }
  .btn-answer:hover {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .btn-decline {
    background: var(--accent-danger, #ef4444);
    color: #ffffff;
  }
  .btn-decline:hover {
    filter: brightness(1.15);
    transform: translateY(-1px);
  }

  /* ── Responsive ────────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .banner-content {
      flex-direction: column;
      align-items: stretch;
      padding: 0.75rem 1rem;
      gap: 0.65rem;
    }

    .banner-left {
      justify-content: center;
    }

    .banner-actions {
      justify-content: center;
    }

    .btn-answer,
    .btn-decline {
      flex: 1;
      text-align: center;
    }
  }
</style>
