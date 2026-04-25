<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import Avatar from './Avatar.svelte';

  interface MissedCall {
    id: string;
    conversation_id: string;
    caller_id: string;
    caller_name: string;
    callee_id: string;
    callee_name: string;
    call_type: string;
    status: string;
    created_at: number;
  }

  let missedCalls = $state<MissedCall[]>([]);
  let loading = $state(true);
  let showAll = $state(false);

  // Formater le temps relatif
  function formatRelativeTime(timestamp: number): string {
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;
    
    if (diff < 60) return 'À l\'instant';
    if (diff < 3600) return `${Math.floor(diff / 60)} min`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} h`;
    return `${Math.floor(diff / 86400)} j`;
  }

  // Charger les appels manqués
  async function loadMissedCalls() {
    if (!browser) return;
    
    try {
      const res = await fetch('/api/missed-calls', { credentials: 'include' });
      if (res.ok) {
        missedCalls = await res.json();
      }
    } catch (e) {
      console.error('Erreur chargement appels manqués:', e);
    } finally {
      loading = false;
    }
  }

  // Naviguer vers la conversation
  function goToConversation(convId: string) {
    goto(`/chat?conv=${convId}`);
  }

  // Appels affichés (limités ou tous)
  let displayedCalls = $derived(showAll ? missedCalls : missedCalls.slice(0, 3));

  onMount(() => {
    loadMissedCalls();
    
    // Rafraîchir toutes les 30 secondes
    const interval = setInterval(loadMissedCalls, 30000);
    return () => clearInterval(interval);
  });
</script>

{#if missedCalls.length > 0}
  <div class="missed-calls" role="alert" aria-live="polite">
    <div class="missed-header">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/>
        <line x1="1" y1="1" x2="23" y2="23" stroke-width="2"/>
      </svg>
      <span class="missed-title">Appels manqués ({missedCalls.length})</span>
    </div>
    
    <div class="missed-list">
      {#each displayedCalls as call (call.id)}
        <button 
          class="missed-item" 
          onclick={() => goToConversation(call.conversation_id)}
          aria-label="Appel manqué de {call.caller_name}"
        >
          <div class="call-icon" class:video={call.call_type === 'video'}>
            {#if call.call_type === 'video'}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="23 7 16 12 23 17 23 7"/>
                <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
              </svg>
            {:else}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/>
              </svg>
            {/if}
          </div>
          
          <div class="call-info">
            <span class="caller-name">{call.caller_name}</span>
            <span class="call-time">{formatRelativeTime(call.created_at)}</span>
          </div>
          
          <div class="call-status" class:declined={call.status === 'declined'}>
            {call.status === 'declined' ? 'Refusé' : 'Manqué'}
          </div>
        </button>
      {/each}
    </div>
    
    {#if missedCalls.length > 3}
      <button 
        class="show-all-btn" 
        onclick={() => showAll = !showAll}
      >
        {showAll ? 'Voir moins' : `Voir tous (${missedCalls.length})`}
      </button>
    {/if}
  </div>
{/if}

<style>
  .missed-calls {
    background: var(--bg-secondary, #f8fafc);
    border-radius: 0.5rem;
    padding: 0.75rem;
    margin-bottom: 1rem;
    border: 1px solid var(--border, #e2e8f0);
  }
  
  .missed-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
    color: var(--text-secondary, #64748b);
    font-size: 0.85rem;
    font-weight: 500;
  }
  
  .missed-header svg {
    width: 1rem;
    height: 1rem;
    color: var(--error, #ef4444);
  }
  
  .missed-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .missed-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem;
    background: transparent;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition: background 0.15s ease;
  }
  
  .missed-item:hover {
    background: var(--bg-hover, #f1f5f9);
  }
  
  .call-icon {
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--error-light, #fef2f2);
    color: var(--error, #ef4444);
    flex-shrink: 0;
  }
  
  .call-icon.video {
    background: var(--info-light, #eff6ff);
    color: var(--info, #3b82f6);
  }
  
  .call-icon svg {
    width: 1rem;
    height: 1rem;
  }
  
  .call-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }
  
  .caller-name {
    font-weight: 500;
    color: var(--text-primary, #1e293b);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .call-time {
    font-size: 0.75rem;
    color: var(--text-muted, #94a3b8);
  }
  
  .call-status {
    font-size: 0.75rem;
    font-weight: 500;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    background: var(--warning-light, #fef3c7);
    color: var(--warning, #d97706);
  }
  
  .call-status.declined {
    background: var(--error-light, #fef2f2);
    color: var(--error, #ef4444);
  }
  
  .show-all-btn {
    width: 100%;
    padding: 0.5rem;
    margin-top: 0.5rem;
    background: transparent;
    border: 1px dashed var(--border, #e2e8f0);
    border-radius: 0.375rem;
    color: var(--text-secondary, #64748b);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  
  .show-all-btn:hover {
    background: var(--bg-hover, #f1f5f9);
    border-color: var(--primary, #6366f1);
    color: var(--primary, #6366f1);
  }
</style>
