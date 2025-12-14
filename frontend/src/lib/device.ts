// Génère un lien d’ajout d’appareil
export function generateDeviceLink() {
  const token = crypto.randomUUID();
  localStorage.setItem('device-token', token);
  return `${window.location.origin}/join-device?token=${token}`;
}

// Enregistre un nouvel appareil (via WebSocket)
export async function registerDevice(name: string) {
  const ws = new WebSocket(`ws://${window.location.host}/ws`);
  ws.onopen = () => {
    ws.send(JSON.stringify({ type: 'register-device', name }));
  };
}