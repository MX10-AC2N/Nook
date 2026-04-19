<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';

  interface UserPresence {
    user_id: string;
    username: string;
    online: boolean;
    last_seen: number;
  }

  let presences = $state<UserPresence[]>([]);
  let loading = $state(true);
  let heartbeatInterval: ReturnType<typeof setInterval> | null = null;

  // Charger les présences
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

  // Envoyer un heartbeat
  async function sendHeartbeat() {
    if (!browser) return;
    
    try {
      await fetch('/api/presence/heartbeat', { credentials: 'include' });
    } catch (e) {
      console.error('Erreur heartbeat:', e);
    }
  }

  // Formater le last_seen
  function formatLastSeen(timestamp: number): string {
    if (timestamp === 0) return 'Hors ligne';
    
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;
    
    if (diff < 60) return 'En ligne';
    if (diff < 3600) return `Il y a ${Math.floor(diff / 60)} min`;
    if (diff < 86400) return `Il y a ${Math.floor(diff / 3600)} h`;
    return `Il y a ${Math.floor(diff / 86400)} j`;
  }

  // Obtenir le statut d'un utilisateur
  export function getUserStatus(userId: string): { online: boolean; lastSeen: string } {
    const presence = presences.find(p => p.user_id === userId);
    if (!presence) {
      return { online: false, lastSeen: 'Hors ligne' };
    }
    return {
      online: presence.online,
      lastSeen: formatLastSeen(presence.last_seen)
    };
  }

  // Obtenir le nombre d'utilisateurs en ligne
  export function getOnlineCount(): number {
    return presences.filter(p => p.online).length;
  }

  onMount(() => {
    loadPresences();
    
    // Heartbeat toutes les 60 secondes
    heartbeatInterval = setInterval(sendHeartbeat, 60000);
    
    // Rafraîchir les présences toutes les 30 secondes
    const refreshInterval = setInterval(loadPresences, 30000);
    
    return () => {
      if (heartbeatInterval) clearInterval(heartbeatInterval);
      clearInterval(refreshInterval);
    };
  });
</script>

<!-- Ce composant ne rend rien visuellement, il fournit juste les données -->
<!-- Utilisez getUserStatus() et getOnlineCount() dans d'autres composants -->
