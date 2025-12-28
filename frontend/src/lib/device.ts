import { browser } from '$app/environment';

// Génère un lien d'ajout d'appareil
export function generateDeviceLink(): string {
  if (!browser) return '';
  
  const token = crypto.randomUUID();
  localStorage.setItem('device-token', token);
  return `${window.location.origin}/join-device?token=${token}`;
}

// Enregistre un nouvel appareil (via WebSocket)
export async function registerDevice(name: string): Promise<void> {
  if (!browser) return;
  
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`ws://${window.location.host}/ws`);
    
    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'register-device', name }));
      resolve();
    };
    
    ws.onerror = () => {
      reject(new Error('WebSocket connection failed'));
    };
    
    ws.onclose = () => {
      if (ws.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket closed before connection established'));
      }
    };
  });
}
