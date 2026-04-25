// frontend/src/lib/presenceStore.svelte.ts
// Store for user presence data

import { browser } from '$app/environment';

interface UserPresence {
  user_id: string;
  username: string;
  online: boolean;
  last_seen: number;
}

// Reactive state
let presences = $state<UserPresence[]>([]);
let loading = $state(true);
let heartbeatInterval: ReturnType<typeof setInterval> | null = null;

// Load presences from API
async function loadPresences() {
  if (!browser) return;
  
  try {
    const res = await fetch('/api/presence', { credentials: 'include' });
    if (res.ok) {
      presences = await res.json();
    }
  } catch (e) {
    console.error('Erreur chargement présences:', e);
  } finally {
    loading = false;
  }
}

// Send heartbeat to maintain online status
async function sendHeartbeat() {
  if (!browser) return;
  
  try {
    await fetch('/api/presence/heartbeat', { credentials: 'include' });
  } catch (e) {
    console.error('Erreur heartbeat:', e);
  }
}

// Format last_seen timestamp
function formatLastSeen(timestamp: number): string {
  if (timestamp === 0) return 'Hors ligne';
  
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;
  
  if (diff < 60) return 'En ligne';
  if (diff < 3600) return `Il y a ${Math.floor(diff / 60)} min`;
  if (diff < 86400) return `Il y a ${Math.floor(diff / 3600)} h`;
  return `Il y a ${Math.floor(diff / 86400)} j`;
}

// Exported functions
export function getUserStatus(userId: string): { online: boolean; lastSeen: string } {
  const presence = getPresences().find(p => p.user_id === userId);
  if (!presence) {
    return { online: false, lastSeen: 'Hors ligne' };
  }
  return {
    online: presence.online,
    lastSeen: formatLastSeen(presence.last_seen)
  };
}

export function getOnlineCount(): number {
  return getPresences().filter(p => p.online).length;
}

// Get presences array
export function getPresences(): UserPresence[] {
  return presences;
}

// Get loading state
export function isLoading(): boolean {
  return loading;
}

// Initialize presence tracking
export function initPresence() {
  if (!browser) return;
  
  loadPresences();
  
  // Heartbeat every 60 seconds
  heartbeatInterval = setInterval(sendHeartbeat, 60000);
  
  // Refresh presences every 30 seconds
  const refreshInterval = setInterval(loadPresences, 30000);
  
  // Cleanup on page unload
  if (typeof window !== 'undefined') {
    window.addEventListener('beforeunload', () => {
      if (heartbeatInterval) clearInterval(heartbeatInterval);
      clearInterval(refreshInterval);
    });
  }
}
